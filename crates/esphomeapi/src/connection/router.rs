use std::sync::Arc;
use std::time::SystemTime;

use futures::SinkExt as _;
use protobuf::{Message as _, MessageDyn};
use tokio::io::BufWriter;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::codec::FramedWrite;

use crate::model::{
  CameraImage, EntityState, HomeAssistantEvent, HomeassistantActionRequest, LogEvent,
  SUBCRIBE_STATES_RESPONSE_TYPES,
};
use crate::proto;
use crate::utils::Options as _;

use super::codec::{EspHomeEncoder, ProtobufMessage};

/// Long-lived broadcast channels shared across reconnects.
///
/// Created once per `Client` and passed into each new `MessageRouter`.
/// Subscribers hold `broadcast::Receiver`s that keep working across reconnects
/// because the senders never change.
pub(crate) struct SharedChannels {
  pub state_tx: broadcast::Sender<EntityState>,
  pub ha_event_tx: broadcast::Sender<HomeAssistantEvent>,
  pub log_tx: broadcast::Sender<LogEvent>,
  pub action_request_tx: broadcast::Sender<HomeassistantActionRequest>,
  pub camera_tx: broadcast::Sender<CameraImage>,
}

impl SharedChannels {
  pub fn new() -> Self {
    Self {
      state_tx: broadcast::channel(64).0,
      ha_event_tx: broadcast::channel(32).0,
      log_tx: broadcast::channel(128).0,
      action_request_tx: broadcast::channel(32).0,
      camera_tx: broadcast::channel(8).0,
    }
  }

  pub fn subscribe_states(&self) -> broadcast::Receiver<EntityState> {
    self.state_tx.subscribe()
  }

  pub fn subscribe_home_assistant_events(&self) -> broadcast::Receiver<HomeAssistantEvent> {
    self.ha_event_tx.subscribe()
  }

  pub fn subscribe_logs(&self) -> broadcast::Receiver<LogEvent> {
    self.log_tx.subscribe()
  }

  pub fn subscribe_action_requests(&self) -> broadcast::Receiver<HomeassistantActionRequest> {
    self.action_request_tx.subscribe()
  }

  pub fn subscribe_camera(&self) -> broadcast::Receiver<CameraImage> {
    self.camera_tx.subscribe()
  }
}

/// Tracks a pending request awaiting a response
struct PendingRequest {
  response_type: u32,
  tx: oneshot::Sender<ProtobufMessage>,
}

/// Tracks a pending request awaiting multiple responses until a terminator
struct PendingMultiRequest {
  response_types: Vec<u32>,
  until_type: u32,
  tx: mpsc::Sender<ProtobufMessage>,
}

/// Internal message type for communicating with the router task
pub enum RouterCommand {
  /// Send a message without waiting for a response
  Send { message: ProtobufMessage },
  /// Send a message and wait for a specific response type
  SendAwaitResponse {
    message: ProtobufMessage,
    response_type: u32,
    tx: oneshot::Sender<ProtobufMessage>,
  },
  /// Send a message and collect responses until a terminator type
  SendAwaitMultiple {
    message: ProtobufMessage,
    response_types: Vec<u32>,
    until_type: u32,
    tx: mpsc::Sender<ProtobufMessage>,
  },
}

/// Handle for sending commands to the router
#[derive(Clone)]
pub struct RouterHandle {
  command_tx: mpsc::Sender<RouterCommand>,
}

impl RouterHandle {
  pub async fn send(&self, message: ProtobufMessage) -> crate::Result<()> {
    self
      .command_tx
      .send(RouterCommand::Send { message })
      .await
      .map_err(|_| "Router channel closed")?;
    Ok(())
  }

  pub async fn send_await_response(
    &self,
    message: ProtobufMessage,
    response_type: u32,
  ) -> crate::Result<ProtobufMessage> {
    let (tx, rx) = oneshot::channel();
    self
      .command_tx
      .send(RouterCommand::SendAwaitResponse {
        message,
        response_type,
        tx,
      })
      .await
      .map_err(|_| "Router channel closed")?;

    rx.await.map_err(|_| "Response channel closed".into())
  }

  pub async fn send_await_multiple(
    &self,
    message: ProtobufMessage,
    response_types: Vec<u32>,
    until_type: u32,
  ) -> crate::Result<mpsc::Receiver<ProtobufMessage>> {
    let (tx, rx) = mpsc::channel(32);
    self
      .command_tx
      .send(RouterCommand::SendAwaitMultiple {
        message,
        response_types,
        until_type,
        tx,
      })
      .await
      .map_err(|_| "Router channel closed")?;

    Ok(rx)
  }
}

/// The message router handles all message routing between the connection and subscribers
pub struct MessageRouter {
  /// Receives decoded messages from the TCP reader
  message_rx: mpsc::Receiver<ProtobufMessage>,
  /// Receives commands from the RouterHandle
  command_rx: mpsc::Receiver<RouterCommand>,
  /// Writer for sending messages to the device
  writer: FramedWrite<BufWriter<OwnedWriteHalf>, EspHomeEncoder>,

  /// Shared broadcast channels (outlive this router instance)
  channels: Arc<SharedChannels>,

  // Pending request tracking
  pending_single: Option<PendingRequest>,
  pending_multi: Option<PendingMultiRequest>,

  // Signals when the connection drops.
  // Sends `true` for abrupt disconnect (reconnect), `false` for graceful DisconnectRequest.
  device_disconnect_tx: Option<oneshot::Sender<bool>>,
}

impl MessageRouter {
  pub fn new(
    message_rx: mpsc::Receiver<ProtobufMessage>,
    writer: FramedWrite<BufWriter<OwnedWriteHalf>, EspHomeEncoder>,
    channels: Arc<SharedChannels>,
  ) -> (Self, RouterHandle, oneshot::Receiver<bool>) {
    let (command_tx, command_rx) = mpsc::channel(32);
    let (device_disconnect_tx, device_disconnect_rx) = oneshot::channel::<bool>();

    let router = Self {
      message_rx,
      command_rx,
      writer,
      channels,
      pending_single: None,
      pending_multi: None,
      device_disconnect_tx: Some(device_disconnect_tx),
    };

    let handle = RouterHandle { command_tx };

    (router, handle, device_disconnect_rx)
  }

  /// Run the message router event loop
  pub async fn run(mut self) {
    loop {
      tokio::select! {
        // Handle incoming messages from the device
        message = self.message_rx.recv() => {
          match message {
            Some(msg) => {
              if !self.handle_incoming_message(msg).await {
                break;
              }
            }
            // Reader task exited — TCP connection lost (abrupt)
            None => {
              if let Some(tx) = self.device_disconnect_tx.take() {
                let _ = tx.send(true);
              }
              break;
            }
          }
        }

        // Handle commands from the RouterHandle
        Some(command) = self.command_rx.recv() => {
          if !self.handle_command(command).await {
            break;
          }
        }

        // Exit when all channels are closed
        else => {
          break;
        }
      }
    }
  }

  /// Returns `false` when the router loop should exit (device-initiated disconnect).
  async fn handle_incoming_message(&mut self, message: ProtobufMessage) -> bool {
    let msg_type = message.protobuf_type;

    // Check if this is a request from the device (PingRequest, GetTimeRequest, DisconnectRequest)
    if msg_type == proto::api::PingRequest::get_option_id()
      || msg_type == proto::api::GetTimeRequest::get_option_id()
      || msg_type == proto::api::DisconnectRequest::get_option_id()
    {
      self.handle_device_request(message).await
    } else {
      // Otherwise treat it as a response/push
      self.route_response(message).await;
      true
    }
  }

  async fn route_response(&mut self, message: ProtobufMessage) {
    // First check if this matches a pending single request
    if let Some(pending) = &self.pending_single {
      if message.protobuf_type == pending.response_type {
        if let Some(pending) = self.pending_single.take() {
          let _ = pending.tx.send(message);
          return;
        }
      }
    }

    // Check if this matches a pending multi request
    if let Some(pending) = &self.pending_multi {
      if message.protobuf_type == pending.until_type {
        // Terminator received, complete the multi request
        self.pending_multi = None;
        return;
      }
      if pending.response_types.contains(&message.protobuf_type) {
        if let Some(pending) = &self.pending_multi {
          let _ = pending.tx.send(message).await;
          return;
        }
      }
    }

    // Route to appropriate broadcast channel based on message type
    self.broadcast_message(message).await;
  }

  async fn broadcast_message(&self, message: ProtobufMessage) {
    let msg_type = message.protobuf_type;

    // Entity state updates
    if let Some(parser) = SUBCRIBE_STATES_RESPONSE_TYPES.get(&msg_type) {
      if let Ok(state) = parser(&message.protobuf_data) {
        let _ = self.channels.state_tx.send(state);
        return;
      }
    }

    // Home Assistant state events
    if msg_type == proto::api::SubscribeHomeAssistantStateResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::SubscribeHomeAssistantStateResponse::parse_from_bytes(&message.protobuf_data)
      {
        let event: HomeAssistantEvent = proto_msg.into();
        let _ = self.channels.ha_event_tx.send(event);
        return;
      }
    }

    // Home Assistant action requests
    if msg_type == proto::api::HomeassistantActionRequest::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::HomeassistantActionRequest::parse_from_bytes(&message.protobuf_data)
      {
        let request: HomeassistantActionRequest = proto_msg.into();
        let _ = self.channels.action_request_tx.send(request);
        return;
      }
    }

    // Log events
    if msg_type == proto::api::SubscribeLogsResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::SubscribeLogsResponse::parse_from_bytes(&message.protobuf_data)
      {
        let event: LogEvent = proto_msg.into();
        let _ = self.channels.log_tx.send(event);
        return;
      }
    }

    // Camera image frames
    if msg_type == proto::api::CameraImageResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::CameraImageResponse::parse_from_bytes(&message.protobuf_data)
      {
        let image: CameraImage = proto_msg.into();
        let _ = self.channels.camera_tx.send(image);
        return;
      }
    }
  }

  /// Returns `false` when the router loop should exit (device-initiated disconnect or write error).
  async fn handle_device_request(&mut self, message: ProtobufMessage) -> bool {
    let msg_type = message.protobuf_type;

    // Handle PingRequest
    if msg_type == proto::api::PingRequest::get_option_id() {
      let response = proto::api::PingResponse::default();
      return self.send_proto_message(&response).await;
    }

    // Handle GetTimeRequest
    if msg_type == proto::api::GetTimeRequest::get_option_id() {
      let mut response = proto::api::GetTimeResponse::new();
      response.epoch_seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
      return self.send_proto_message(&response).await;
    }

    // Handle DisconnectRequest (device-initiated graceful disconnect)
    if msg_type == proto::api::DisconnectRequest::get_option_id() {
      let response = proto::api::DisconnectResponse::default();
      self.send_proto_message(&response).await;
      if let Some(tx) = self.device_disconnect_tx.take() {
        let _ = tx.send(false); // graceful — do not reconnect
      }
      return false; // exit the router loop
    }

    true
  }

  /// Returns `false` when the router loop should exit (write error = connection lost).
  async fn handle_command(&mut self, command: RouterCommand) -> bool {
    match command {
      RouterCommand::Send { message } => self.send_message(message).await,
      RouterCommand::SendAwaitResponse {
        message,
        response_type,
        tx,
      } => {
        self.pending_single = Some(PendingRequest { response_type, tx });
        self.send_message(message).await
      }
      RouterCommand::SendAwaitMultiple {
        message,
        response_types,
        until_type,
        tx,
      } => {
        self.pending_multi = Some(PendingMultiRequest {
          response_types,
          until_type,
          tx,
        });
        self.send_message(message).await
      }
    }
  }

  /// Returns `false` when the write fails (connection lost).
  async fn send_message(&mut self, message: ProtobufMessage) -> bool {
    if let Err(e) = self.writer.send(message).await {
      eprintln!("Error sending message: {:?}", e);
      if let Some(tx) = self.device_disconnect_tx.take() {
        let _ = tx.send(true); // abrupt write failure
      }
      return false;
    }
    true
  }

  /// Returns `false` when the write fails (connection lost).
  async fn send_proto_message<M: protobuf::Message + MessageDyn>(&mut self, message: &M) -> bool {
    let protobuf_type = message
      .descriptor_dyn()
      .proto()
      .options
      .as_ref()
      .and_then(|options| proto::api_options::exts::id.get(options))
      .unwrap();
    let protobuf_data = message.write_to_bytes().unwrap();
    self
      .send_message(ProtobufMessage {
        protobuf_type,
        protobuf_data,
      })
      .await
  }
}
