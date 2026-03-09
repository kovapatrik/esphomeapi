mod codec;
mod router;

use std::sync::Arc;
use std::time::Duration;

use bytes::BytesMut;
use codec::{EspHomeDecoder, EspHomeEncoder, EspHomeHandshake, HandshakeResult};
use protobuf::Message as _;
use router::MessageRouter;
pub(crate) use router::RouterHandle;
pub(crate) use router::SharedChannels;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::utils::Options as _;
use crate::{proto, Result};
pub use codec::ProtobufMessage;

pub(crate) struct Disconnected;

pub(crate) struct Connected {
  router_handle: RouterHandle,
  router_task: JoinHandle<()>,
  reader_task: JoinHandle<()>,
  keep_alive_task: JoinHandle<()>,
  device_disconnect_rx: Option<oneshot::Receiver<bool>>,
}

impl Drop for Connected {
  fn drop(&mut self) {
    self.keep_alive_task.abort();
    self.reader_task.abort();
    self.router_task.abort();
  }
}

#[derive(Clone, Debug)]
pub(crate) struct ConnectionConfig {
  pub host: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: String,
  pub keep_alive_duration: Duration,
}

pub(crate) struct Connection<S> {
  config: ConnectionConfig,
  state: S,
}

impl Connection<Disconnected> {
  pub(crate) fn new_from_config(config: ConnectionConfig) -> Self {
    Connection {
      config,
      state: Disconnected,
    }
  }

  /// Connect using pre-existing shared channels so subscribers survive reconnects.
  pub(crate) async fn connect_with_channels(
    self,
    login: bool,
    channels: Arc<SharedChannels>,
  ) -> Result<Connection<Connected>> {
    let stream = TcpStream::connect(format!("{}:{}", self.config.host, self.config.port)).await?;
    let (reader, writer) = stream.into_split();

    let codec = EspHomeHandshake::new(self.config.psk.clone(), self.config.expected_name.clone())?;

    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let (decoder, encoder) = Self::perform_handshake(codec, &mut reader, &mut writer).await?;

    let (message_tx, message_rx) = tokio::sync::mpsc::channel(32);
    let framed_reader = FramedRead::new(reader, decoder);
    let reader_task = Self::spawn_reader_task(framed_reader, message_tx);

    let framed_writer = FramedWrite::new(writer, encoder);
    let (router, router_handle, device_disconnect_rx) =
      MessageRouter::new(message_rx, framed_writer, channels);

    let router_task = tokio::spawn(async move {
      router.run().await;
    });

    Self::perform_hello(&router_handle, &self.config, login).await?;

    let keep_alive_task =
      Self::spawn_keep_alive_task(router_handle.clone(), self.config.keep_alive_duration);

    Ok(Connection {
      config: self.config,
      state: Connected {
        router_handle,
        router_task,
        reader_task,
        keep_alive_task,
        device_disconnect_rx: Some(device_disconnect_rx),
      },
    })
  }

  async fn perform_handshake(
    mut codec: EspHomeHandshake,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: &mut BufWriter<tokio::net::tcp::OwnedWriteHalf>,
  ) -> Result<(EspHomeDecoder, EspHomeEncoder)> {
    let mut buffer = BytesMut::with_capacity(1024);

    loop {
      match codec.process(&mut buffer)? {
        HandshakeResult::NeedMoreData => {
          let n = reader.read_buf(&mut buffer).await?;
          if n == 0 {
            return Err("Connection closed during handshake".into());
          }
        }
        HandshakeResult::SendFrame(frame) => {
          writer.write_all(&frame).await?;
          writer.flush().await?;
        }
        HandshakeResult::Complete(decoder, encoder) => {
          return Ok((decoder, encoder));
        }
      }
    }
  }

  fn spawn_reader_task(
    mut reader: FramedRead<BufReader<tokio::net::tcp::OwnedReadHalf>, EspHomeDecoder>,
    tx: tokio::sync::mpsc::Sender<ProtobufMessage>,
  ) -> JoinHandle<()> {
    tokio::spawn(async move {
      loop {
        match reader.next().await {
          Some(Ok(message)) => {
            if tx.send(message).await.is_err() {
              break;
            }
          }
          Some(Err(_)) | None => break,
        }
      }
    })
  }

  async fn perform_hello(
    router: &RouterHandle,
    config: &ConnectionConfig,
    login: bool,
  ) -> Result<()> {
    let mut hello = proto::api::HelloRequest::default();
    hello.client_info = config.client_info.clone();
    hello.api_version_major = 1;
    hello.api_version_minor = 10;

    let response = router
      .send_await_response(
        ProtobufMessage {
          protobuf_type: proto::api::HelloRequest::get_option_id(),
          protobuf_data: hello.write_to_bytes()?,
        },
        proto::api::HelloResponse::get_option_id(),
      )
      .await?;

    let hello_response = proto::api::HelloResponse::parse_from_bytes(&response.protobuf_data)?;

    if let Some(expected_name) = &config.expected_name {
      if &hello_response.name != expected_name {
        return Err(
          format!(
            "Device name mismatch: expected '{}', got '{}'",
            expected_name, hello_response.name
          )
          .into(),
        );
      }
    }

    if login {
      let mut auth = proto::api::AuthenticationRequest::default();
      if let Some(password) = &config.password {
        auth.password = password.clone();
      }
      router
        .send(ProtobufMessage {
          protobuf_type: proto::api::AuthenticationRequest::get_option_id(),
          protobuf_data: auth.write_to_bytes()?,
        })
        .await?;
    }

    Ok(())
  }

  fn spawn_keep_alive_task(router: RouterHandle, duration: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(duration);
      loop {
        interval.tick().await;
        let ping_msg = ProtobufMessage {
          protobuf_type: proto::api::PingRequest::get_option_id(),
          protobuf_data: proto::api::PingRequest::default().write_to_bytes().unwrap(),
        };
        if router.send(ping_msg).await.is_err() {
          break;
        }
      }
    })
  }
}

impl Connection<Connected> {
  pub(crate) fn router_handle(&self) -> &RouterHandle {
    &self.state.router_handle
  }

  pub(crate) fn take_device_disconnect_rx(&mut self) -> Option<oneshot::Receiver<bool>> {
    self.state.device_disconnect_rx.take()
  }
}
