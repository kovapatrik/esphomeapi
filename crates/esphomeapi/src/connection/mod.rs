mod codec;
mod router;

use std::time::Duration;

use bytes::BytesMut;
use codec::{EspHomeDecoder, EspHomeEncoder, EspHomeHandshake, HandshakeResult};
use protobuf::Message as _;
use router::{MessageRouter, RouterConfig, RouterHandle};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::model::{
  CameraImage, EntityState, HomeAssistantEvent, HomeassistantActionRequest, LogEvent,
};
use crate::{proto, Result};

use crate::utils::Options as _;
pub use codec::ProtobufMessage;

/// State marker for a disconnected connection
pub struct Disconnected;

/// State marker for a connected connection - holds all connection resources
pub struct Connected {
  router_handle: RouterHandle,
  router_task: JoinHandle<()>,
  reader_task: JoinHandle<()>,
  keep_alive_task: JoinHandle<()>,
}

impl Drop for Connected {
  fn drop(&mut self) {
    self.keep_alive_task.abort();
    self.reader_task.abort();
    self.router_task.abort();
  }
}

/// Configuration for creating a connection
#[derive(Clone, Debug)]
pub struct ConnectionConfig {
  pub host: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: String,
  pub keep_alive_duration: Duration,
}

/// Connection to an ESPHome device
///
/// Uses typestate pattern to encode connection state at compile time:
/// - `Connection<Disconnected>` - can call `connect()`
/// - `Connection<Connected>` - can call messaging and subscription methods
pub struct Connection<S> {
  config: ConnectionConfig,
  state: S,
}

impl Connection<Disconnected> {
  /// Create a new disconnected connection
  pub fn new(
    host: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<u32>,
  ) -> Self {
    let config = ConnectionConfig {
      host,
      port,
      password,
      expected_name,
      psk,
      client_info: client_info.unwrap_or_else(|| "esphome-rs".to_string()),
      keep_alive_duration: Duration::from_secs(keep_alive_duration.unwrap_or(20) as u64),
    };

    Connection {
      config,
      state: Disconnected,
    }
  }

  /// Connect to the ESPHome device
  ///
  /// Consumes the disconnected connection and returns a connected one on success.
  pub async fn connect(self, login: bool) -> Result<Connection<Connected>> {
    // Establish TCP connection
    let stream = TcpStream::connect(format!("{}:{}", self.config.host, self.config.port)).await?;
    let (reader, writer) = stream.into_split();

    // Create the codec and perform handshake
    let codec = EspHomeHandshake::new(self.config.psk.clone(), self.config.expected_name.clone())?;

    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    // Perform handshake (codec-specific logic is self-contained)
    let (decoder, encoder) = Self::perform_handshake(codec, &mut reader, &mut writer).await?;

    // Create channels for message passing
    let (message_tx, message_rx) = tokio::sync::mpsc::channel(32);

    // Create framed reader with the decoder
    let framed_reader = FramedRead::new(reader, decoder);

    // Spawn reader task
    let reader_task = Self::spawn_reader_task(framed_reader, message_tx);

    // Create the message router
    let framed_writer = FramedWrite::new(writer, encoder);
    let (router, router_handle) =
      MessageRouter::new(message_rx, framed_writer, RouterConfig::default());

    // Spawn router task
    let router_task = tokio::spawn(async move {
      router.run().await;
    });

    // Perform hello/login handshake
    Self::perform_hello(&router_handle, &self.config, login).await?;

    // Start keep-alive
    let keep_alive_task =
      Self::spawn_keep_alive_task(router_handle.clone(), self.config.keep_alive_duration);

    Ok(Connection {
      config: self.config,
      state: Connected {
        router_handle,
        router_task,
        reader_task,
        keep_alive_task,
      },
    })
  }

  /// Perform the protocol handshake
  async fn perform_handshake(
    mut codec: EspHomeHandshake,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: &mut BufWriter<tokio::net::tcp::OwnedWriteHalf>,
  ) -> Result<(EspHomeDecoder, EspHomeEncoder)> {
    let mut buffer = BytesMut::with_capacity(1024);

    loop {
      match codec.process(&mut buffer)? {
        HandshakeResult::NeedMoreData => {
          // Need more data - read from socket
          let n = reader.read_buf(&mut buffer).await?;
          if n == 0 {
            return Err("Connection closed during handshake".into());
          }
        }
        HandshakeResult::SendFrame(frame) => {
          // Send the frame to the server
          writer.write_all(&frame).await?;
          writer.flush().await?;
        }
        HandshakeResult::Complete(decoder, encoder) => {
          return Ok((decoder, encoder));
        }
      }
    }
  }

  /// Spawn the reader task that forwards messages to the router
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
          Some(Err(_)) => {
            break;
          }
          None => {
            break;
          }
        }
      }
    })
  }

  /// Perform the Hello and optional Connect handshake
  async fn perform_hello(
    router: &RouterHandle,
    config: &ConnectionConfig,
    login: bool,
  ) -> Result<()> {
    // Send HelloRequest
    let mut hello = proto::api::HelloRequest::default();
    hello.client_info = config.client_info.clone();
    hello.api_version_major = 1;
    hello.api_version_minor = 10;

    let hello_msg = ProtobufMessage {
      protobuf_type: proto::api::HelloRequest::get_option_id(),
      protobuf_data: hello.write_to_bytes()?,
    };

    let response = router
      .send_await_response(hello_msg, proto::api::HelloResponse::get_option_id())
      .await?;

    let hello_response = proto::api::HelloResponse::parse_from_bytes(&response.protobuf_data)?;

    // Verify device name if expected
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

    // Send AuthenticationRequest for legacy authentication (deprecated in ESPHome 2026.1.0)
    // We send this without waiting for response - newer devices ignore it,
    // older devices will process it asynchronously
    if login {
      let mut auth = proto::api::AuthenticationRequest::default();
      if let Some(password) = &config.password {
        auth.password = password.clone();
      }

      let auth_msg = ProtobufMessage {
        protobuf_type: proto::api::AuthenticationRequest::get_option_id(),
        protobuf_data: auth.write_to_bytes()?,
      };

      // Fire and forget - don't wait for response
      router.send(auth_msg).await?;
    }

    Ok(())
  }

  /// Spawn the keep-alive ping task
  fn spawn_keep_alive_task(router: RouterHandle, duration: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(duration);

      loop {
        interval.tick().await;

        let ping = proto::api::PingRequest::default();
        let ping_msg = ProtobufMessage {
          protobuf_type: proto::api::PingRequest::get_option_id(),
          protobuf_data: ping.write_to_bytes().unwrap(),
        };

        if router.send(ping_msg).await.is_err() {
          break;
        }
      }
    })
  }
}

impl Connection<Connected> {
  /// Get a reference to the router handle
  fn router(&self) -> &RouterHandle {
    &self.state.router_handle
  }

  /// Send a message without waiting for a response
  pub async fn send_message(&self, message: Box<dyn protobuf::MessageDyn>) -> Result<()> {
    let protobuf_type = message
      .descriptor_dyn()
      .proto()
      .options
      .as_ref()
      .and_then(|options| proto::api_options::exts::id.get(options))
      .ok_or("Message has no ID option")?;

    let protobuf_data = message.write_to_bytes_dyn()?;

    self
      .router()
      .send(ProtobufMessage {
        protobuf_type,
        protobuf_data,
      })
      .await
  }

  /// Send a message and wait for a specific response type
  pub async fn send_message_await_response(
    &self,
    message: Box<dyn protobuf::MessageDyn>,
    response_type: u32,
  ) -> Result<ProtobufMessage> {
    let protobuf_type = message
      .descriptor_dyn()
      .proto()
      .options
      .as_ref()
      .and_then(|options| proto::api_options::exts::id.get(options))
      .ok_or("Message has no ID option")?;

    let protobuf_data = message.write_to_bytes_dyn()?;

    let response = timeout(
      Duration::from_secs(10),
      self.router().send_await_response(
        ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
        response_type,
      ),
    )
    .await
    .map_err(|_| "Timeout waiting for response")??;

    Ok(response)
  }

  /// Send a message and collect responses until a terminator type is received
  pub async fn send_message_await_until(
    &self,
    message: Box<dyn protobuf::MessageDyn>,
    response_types: Vec<u32>,
    until_type: u32,
    timeout_duration: Duration,
  ) -> Result<Vec<ProtobufMessage>> {
    let protobuf_type = message
      .descriptor_dyn()
      .proto()
      .options
      .as_ref()
      .and_then(|options| proto::api_options::exts::id.get(options))
      .ok_or("Message has no ID option")?;

    let protobuf_data = message.write_to_bytes_dyn()?;

    let mut rx = self
      .router()
      .send_await_multiple(
        ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
        response_types,
        until_type,
      )
      .await?;

    let mut responses = Vec::new();

    while let Ok(Some(msg)) = timeout(timeout_duration, rx.recv()).await {
      responses.push(msg);
    }

    Ok(responses)
  }

  /// Subscribe to entity state updates
  pub fn subscribe_states(&self) -> broadcast::Receiver<EntityState> {
    self.router().subscriptions().subscribe_states()
  }

  /// Subscribe to Home Assistant events
  pub fn subscribe_home_assistant_events(&self) -> broadcast::Receiver<HomeAssistantEvent> {
    self
      .router()
      .subscriptions()
      .subscribe_home_assistant_events()
  }

  /// Subscribe to log events
  pub fn subscribe_logs(&self) -> broadcast::Receiver<LogEvent> {
    self.router().subscriptions().subscribe_logs()
  }

  /// Subscribe to Home Assistant action requests
  pub fn subscribe_action_requests(&self) -> broadcast::Receiver<HomeassistantActionRequest> {
    self.router().subscriptions().subscribe_action_requests()
  }

  /// Subscribe to camera image frames
  ///
  /// Camera frames are sent automatically by the device after subscribing to states.
  /// No separate request is needed.
  pub fn subscribe_camera(&self) -> broadcast::Receiver<CameraImage> {
    self.router().subscriptions().subscribe_camera()
  }

  /// Disconnect from the device
  ///
  /// Consumes the connected connection and returns a disconnected one.
  pub async fn disconnect(self) -> Result<Connection<Disconnected>> {
    // Send disconnect request (best effort)
    let disconnect = proto::api::DisconnectRequest::default();
    let msg = ProtobufMessage {
      protobuf_type: proto::api::DisconnectRequest::get_option_id(),
      protobuf_data: disconnect.write_to_bytes()?,
    };
    let _ = self.router().send(msg).await;

    // Tasks are aborted when `self.state` (Connected) is dropped
    Ok(Connection {
      config: self.config,
      state: Disconnected,
    })
  }
}
