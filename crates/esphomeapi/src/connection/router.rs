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

/// Configuration for broadcast channel capacities
#[derive(Clone)]
pub struct RouterConfig {
  pub state_channel_capacity: usize,
  pub ha_event_channel_capacity: usize,
  pub log_channel_capacity: usize,
  pub service_channel_capacity: usize,
  pub camera_channel_capacity: usize,
}

impl Default for RouterConfig {
  fn default() -> Self {
    Self {
      state_channel_capacity: 64,
      ha_event_channel_capacity: 32,
      log_channel_capacity: 128,
      service_channel_capacity: 32,
      camera_channel_capacity: 8,
    }
  }
}

/// Handles for subscribing to different event streams
#[derive(Clone)]
pub struct SubscriptionHandles {
  state_tx: broadcast::Sender<EntityState>,
  ha_event_tx: broadcast::Sender<HomeAssistantEvent>,
  log_tx: broadcast::Sender<LogEvent>,
  action_request_tx: broadcast::Sender<HomeassistantActionRequest>,
  camera_tx: broadcast::Sender<CameraImage>,
}

impl SubscriptionHandles {
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
  subscriptions: SubscriptionHandles,
}

impl RouterHandle {
  pub fn subscriptions(&self) -> &SubscriptionHandles {
    &self.subscriptions
  }

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

  // Broadcast channels for different event types
  state_tx: broadcast::Sender<EntityState>,
  ha_event_tx: broadcast::Sender<HomeAssistantEvent>,
  log_tx: broadcast::Sender<LogEvent>,
  action_request_tx: broadcast::Sender<HomeassistantActionRequest>,
  camera_tx: broadcast::Sender<CameraImage>,

  // Pending request tracking
  pending_single: Option<PendingRequest>,
  pending_multi: Option<PendingMultiRequest>,
}

impl MessageRouter {
  pub fn new(
    message_rx: mpsc::Receiver<ProtobufMessage>,
    writer: FramedWrite<BufWriter<OwnedWriteHalf>, EspHomeEncoder>,
    config: RouterConfig,
  ) -> (Self, RouterHandle) {
    let (state_tx, _) = broadcast::channel(config.state_channel_capacity);
    let (ha_event_tx, _) = broadcast::channel(config.ha_event_channel_capacity);
    let (log_tx, _) = broadcast::channel(config.log_channel_capacity);
    let (action_request_tx, _) = broadcast::channel(config.service_channel_capacity);
    let (camera_tx, _) = broadcast::channel(config.camera_channel_capacity);

    let (command_tx, command_rx) = mpsc::channel(32);

    let subscriptions = SubscriptionHandles {
      state_tx: state_tx.clone(),
      ha_event_tx: ha_event_tx.clone(),
      log_tx: log_tx.clone(),
      action_request_tx: action_request_tx.clone(),
      camera_tx: camera_tx.clone(),
    };

    let router = Self {
      message_rx,
      command_rx,
      writer,
      state_tx,
      ha_event_tx,
      log_tx,
      action_request_tx,
      camera_tx,
      pending_single: None,
      pending_multi: None,
    };

    let handle = RouterHandle {
      command_tx,
      subscriptions,
    };

    (router, handle)
  }

  /// Run the message router event loop
  pub async fn run(mut self) {
    loop {
      tokio::select! {
        // Handle incoming messages from the device
        Some(message) = self.message_rx.recv() => {
          self.handle_incoming_message(message).await;
        }

        // Handle commands from the RouterHandle
        Some(command) = self.command_rx.recv() => {
          self.handle_command(command).await;
        }

        // Exit when all channels are closed
        else => {
          break;
        }
      }
    }
  }

  async fn handle_incoming_message(&mut self, message: ProtobufMessage) {
    let msg_type = message.protobuf_type;

    // Check if this is a request from the device (PingRequest, GetTimeRequest, DisconnectRequest)
    if msg_type == proto::api::PingRequest::get_option_id()
      || msg_type == proto::api::GetTimeRequest::get_option_id()
      || msg_type == proto::api::DisconnectRequest::get_option_id()
    {
      self.handle_device_request(message).await;
    } else {
      // Otherwise treat it as a response/push
      self.route_response(message).await;
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
        let _ = self.state_tx.send(state);
        return;
      }
    }

    // Home Assistant state events
    if msg_type == proto::api::SubscribeHomeAssistantStateResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::SubscribeHomeAssistantStateResponse::parse_from_bytes(&message.protobuf_data)
      {
        let event: HomeAssistantEvent = proto_msg.into();
        let _ = self.ha_event_tx.send(event);
        return;
      }
    }

    // Home Assistant action requests
    if msg_type == proto::api::HomeassistantActionRequest::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::HomeassistantActionRequest::parse_from_bytes(&message.protobuf_data)
      {
        let request: HomeassistantActionRequest = proto_msg.into();
        let _ = self.action_request_tx.send(request);
        return;
      }
    }

    // Log events
    if msg_type == proto::api::SubscribeLogsResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::SubscribeLogsResponse::parse_from_bytes(&message.protobuf_data)
      {
        let event: LogEvent = proto_msg.into();
        let _ = self.log_tx.send(event);
        return;
      }
    }

    // Camera image frames
    if msg_type == proto::api::CameraImageResponse::get_option_id() {
      if let Ok(proto_msg) =
        proto::api::CameraImageResponse::parse_from_bytes(&message.protobuf_data)
      {
        let image: CameraImage = proto_msg.into();
        let _ = self.camera_tx.send(image);
        return;
      }
    }
  }

  async fn handle_device_request(&mut self, message: ProtobufMessage) {
    let msg_type = message.protobuf_type;

    // Handle PingRequest
    if msg_type == proto::api::PingRequest::get_option_id() {
      let response = proto::api::PingResponse::default();
      self.send_proto_message(&response).await;
      return;
    }

    // Handle GetTimeRequest
    if msg_type == proto::api::GetTimeRequest::get_option_id() {
      let mut response = proto::api::GetTimeResponse::new();
      response.epoch_seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
      self.send_proto_message(&response).await;
      return;
    }

    // Handle DisconnectRequest
    if msg_type == proto::api::DisconnectRequest::get_option_id() {
      let response = proto::api::DisconnectResponse::default();
      self.send_proto_message(&response).await;
      // TODO: Signal connection to close
      return;
    }
  }

  async fn handle_command(&mut self, command: RouterCommand) {
    match command {
      RouterCommand::Send { message } => {
        self.send_message(message).await;
      }
      RouterCommand::SendAwaitResponse {
        message,
        response_type,
        tx,
      } => {
        self.pending_single = Some(PendingRequest { response_type, tx });
        self.send_message(message).await;
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
        self.send_message(message).await;
      }
    }
  }

  async fn send_message(&mut self, message: ProtobufMessage) {
    if let Err(e) = self.writer.send(message).await {
      eprintln!("Error sending message: {:?}", e);
      return;
    }
  }

  async fn send_proto_message<M: protobuf::Message + MessageDyn>(&mut self, message: &M) {
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
      .await;
  }
}
