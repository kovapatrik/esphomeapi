use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use protobuf::{EnumOrUnknown, Message as _};
use tokio::sync::{broadcast, oneshot};
use tokio::time::timeout;
use tracing::{info, warn};

use crate::connection::{
  Connected, Connection, ConnectionConfig, ProtobufMessage, RouterHandle, SharedChannels,
};
use crate::model::{
  parse_user_service, CameraImage, ColorMode, DeviceInfo, EntityInfo, EntityState,
  HomeAssistantEvent, HomeassistantActionRequest, LogEvent, LogLevel, UserService,
  LIST_ENTITIES_SERVICES_RESPONSE_TYPES,
};
use crate::utils::Options as _;
use crate::{proto, CommandHandle, Result};

/// A self-reconnecting ESPHome client.
///
/// `Client` is cheap to clone — all clones share the same connection, broadcast
/// channels, and reconnect task.  The reconnect task runs automatically whenever
/// the device drops the connection abruptly (e.g. `BrokenPipe`).  A graceful
/// `DisconnectRequest` from the device stops the reconnect loop and fires
/// `on_device_disconnect()`.
#[derive(Clone)]
pub struct Client {
  /// Long-lived broadcast channels shared across reconnects.
  channels: Arc<SharedChannels>,
  /// Swappable router handle — updated atomically on each reconnect.
  router: Arc<RwLock<RouterHandle>>,
  /// Fires when the connection drops (both graceful and abrupt).
  disconnect_tx: broadcast::Sender<()>,
  /// Fires after each successful automatic reconnect.
  reconnect_tx: broadcast::Sender<()>,
  /// Set to `true` by `disconnect()` to prevent reconnect after a deliberate disconnect.
  cancelled: Arc<AtomicBool>,
}

impl Client {
  /// Connect to an ESPHome device and start the automatic reconnect loop.
  pub async fn connect(
    host: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<u32>,
  ) -> Result<Self> {
    let config = ConnectionConfig {
      host,
      port,
      password,
      expected_name,
      psk,
      client_info: client_info.unwrap_or_else(|| "esphome-rs".to_string()),
      keep_alive_duration: Duration::from_secs(keep_alive_duration.unwrap_or(20) as u64),
    };

    let channels = Arc::new(SharedChannels::new());

    let mut conn = Connection::new_from_config(config.clone())
      .connect_with_channels(true, Arc::clone(&channels))
      .await?;

    let router_handle = conn.router_handle().clone();
    let router = Arc::new(RwLock::new(router_handle));
    let disconnect_rx = conn.take_device_disconnect_rx().unwrap();

    let (disconnect_tx, _) = broadcast::channel(1);
    let (reconnect_tx, _) = broadcast::channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));

    Self::spawn_reconnect_task(
      config,
      Arc::clone(&channels),
      Arc::clone(&router),
      conn,
      disconnect_rx,
      disconnect_tx.clone(),
      reconnect_tx.clone(),
      Arc::clone(&cancelled),
    );

    Ok(Self {
      channels,
      router,
      disconnect_tx,
      reconnect_tx,
      cancelled,
    })
  }

  // ── Event subscriptions ────────────────────────────────────────────────────

  /// Subscribe to connection drop events (both abrupt and graceful).
  pub fn on_device_disconnect(&self) -> broadcast::Receiver<()> {
    self.disconnect_tx.subscribe()
  }

  /// Subscribe to successful automatic reconnect events.
  pub fn on_reconnect(&self) -> broadcast::Receiver<()> {
    self.reconnect_tx.subscribe()
  }

  /// Get a receiver for entity state updates.
  pub fn states_receiver(&self) -> broadcast::Receiver<EntityState> {
    self.channels.subscribe_states()
  }

  /// Get a receiver for Home Assistant state events.
  pub fn home_assistant_states_receiver(&self) -> broadcast::Receiver<HomeAssistantEvent> {
    self.channels.subscribe_home_assistant_events()
  }

  /// Get a receiver for log events.
  pub fn logs_receiver(&self) -> broadcast::Receiver<LogEvent> {
    self.channels.subscribe_logs()
  }

  /// Get a receiver for Home Assistant action request events.
  pub fn home_assistant_action_requests_receiver(
    &self,
  ) -> broadcast::Receiver<HomeassistantActionRequest> {
    self.channels.subscribe_action_requests()
  }

  /// Get a receiver for camera image frames.
  pub fn camera_receiver(&self) -> broadcast::Receiver<CameraImage> {
    self.channels.subscribe_camera()
  }

  // ── Command handle ─────────────────────────────────────────────────────────

  /// Create a cloneable handle for sending device commands.
  ///
  /// All handles share the same underlying router reference and automatically
  /// point to the new connection after a reconnect.
  pub fn command_handle(&self) -> CommandHandle {
    CommandHandle::from_shared(Arc::clone(&self.router))
  }

  // ── Device requests ────────────────────────────────────────────────────────

  /// Fetch device info from the device.
  pub async fn device_info(&self) -> Result<DeviceInfo> {
    let response = self
      .send_await_response(
        proto::api::DeviceInfoRequest::default(),
        proto::api::DeviceInfoResponse::get_option_id(),
        Duration::from_secs(10),
      )
      .await?;
    Ok(proto::api::DeviceInfoResponse::parse_from_bytes(&response.protobuf_data)?.into())
  }

  /// Fetch all entities and user services from the device.
  pub async fn list_entities_services(&self) -> Result<(Vec<EntityInfo>, Vec<UserService>)> {
    let entity_service_map = LIST_ENTITIES_SERVICES_RESPONSE_TYPES.clone();
    let mut response_types: Vec<u32> = entity_service_map.keys().cloned().collect();
    response_types.push(proto::api::ListEntitiesServicesResponse::get_option_id());

    let responses = self
      .send_await_multiple(
        proto::api::ListEntitiesRequest::new(),
        response_types,
        proto::api::ListEntitiesDoneResponse::get_option_id(),
        Duration::from_secs(60),
      )
      .await?;

    let mut entities = Vec::new();
    let mut services = Vec::new();
    for message in responses {
      if message.protobuf_type == proto::api::ListEntitiesServicesResponse::get_option_id() {
        services.push(parse_user_service(&message.protobuf_data)?);
      } else {
        let parser = entity_service_map
          .get(&message.protobuf_type)
          .ok_or_else(|| format!("Unknown message type: {}", message.protobuf_type))?;
        entities.push(parser(&message.protobuf_data)?);
      }
    }
    Ok((entities, services))
  }

  /// Ask the device to start sending entity state updates.
  pub async fn request_states(&self) -> Result<()> {
    self.send(proto::api::SubscribeStatesRequest::new()).await
  }

  /// Ask the device to start sending Home Assistant state events.
  pub async fn request_home_assistant_states(&self) -> Result<()> {
    self
      .send(proto::api::SubscribeHomeAssistantStatesRequest::new())
      .await
  }

  /// Ask the device to start sending log events.
  pub async fn request_logs(&self, level: LogLevel, dump_config: bool) -> Result<()> {
    self
      .send(proto::api::SubscribeLogsRequest {
        level: EnumOrUnknown::new(level.into()),
        dump_config,
        ..Default::default()
      })
      .await
  }

  /// Ask the device to start sending Home Assistant action requests.
  pub async fn request_home_assistant_action_requests(&self) -> Result<()> {
    self
      .send(proto::api::SubscribeHomeassistantServicesRequest::new())
      .await
  }

  /// Send the current state of a Home Assistant entity to the device.
  pub async fn send_home_assistant_state(
    &self,
    entity_id: String,
    state: String,
    attribute: Option<String>,
  ) -> Result<()> {
    self
      .send(proto::api::HomeAssistantStateResponse {
        entity_id,
        state,
        attribute: attribute.unwrap_or_default(),
        ..Default::default()
      })
      .await
  }

  /// Initiate a graceful client-side disconnect.
  ///
  /// Sets the cancelled flag (preventing automatic reconnect), sends
  /// `DisconnectRequest`, and waits up to 5 s for `DisconnectResponse`.
  pub async fn disconnect(&self) -> Result<()> {
    self.cancelled.store(true, Ordering::Relaxed);

    let router = self.get_router();
    let msg = ProtobufMessage {
      protobuf_type: proto::api::DisconnectRequest::get_option_id(),
      protobuf_data: proto::api::DisconnectRequest::default().write_to_bytes()?,
    };
    let _ = timeout(
      Duration::from_secs(5),
      router.send_await_response(msg, proto::api::DisconnectResponse::get_option_id()),
    )
    .await;

    Ok(())
  }

  pub async fn switch_command(&self, key: u32, state: bool) -> Result<()> {
    self.command_handle().switch_command(key, state).await
  }

  pub async fn light_command(
    &self,
    key: u32,
    state: Option<bool>,
    brightness: Option<f32>,
    color_mode: Option<ColorMode>,
    color_brightness: Option<f32>,
    rgb: Option<(f32, f32, f32)>,
    white: Option<f32>,
    color_temperature: Option<f32>,
    cold_white: Option<f32>,
    warm_white: Option<f32>,
    transition_length: Option<f32>,
    flash_length: Option<f32>,
    effect: Option<String>,
  ) -> Result<()> {
    self
      .command_handle()
      .light_command(
        key,
        state,
        brightness,
        color_mode,
        color_brightness,
        rgb,
        white,
        color_temperature,
        cold_white,
        warm_white,
        transition_length,
        flash_length,
        effect,
      )
      .await
  }

  fn get_router(&self) -> RouterHandle {
    self.router.read().unwrap().clone()
  }

  async fn send<M: protobuf::MessageFull>(&self, message: M) -> Result<()> {
    let router = self.get_router();
    router
      .send(ProtobufMessage {
        protobuf_type: M::get_option_id(),
        protobuf_data: message.write_to_bytes()?,
      })
      .await
  }

  async fn send_await_response<M: protobuf::MessageFull>(
    &self,
    message: M,
    response_type: u32,
    duration: Duration,
  ) -> Result<ProtobufMessage> {
    let router = self.get_router();
    timeout(
      duration,
      router.send_await_response(
        ProtobufMessage {
          protobuf_type: M::get_option_id(),
          protobuf_data: message.write_to_bytes()?,
        },
        response_type,
      ),
    )
    .await
    .map_err(|_| "Timeout waiting for response")?
  }

  async fn send_await_multiple<M: protobuf::MessageFull>(
    &self,
    message: M,
    response_types: Vec<u32>,
    until_type: u32,
    duration: Duration,
  ) -> Result<Vec<ProtobufMessage>> {
    let router = self.get_router();
    let mut rx = router
      .send_await_multiple(
        ProtobufMessage {
          protobuf_type: M::get_option_id(),
          protobuf_data: message.write_to_bytes()?,
        },
        response_types,
        until_type,
      )
      .await?;

    let mut responses = Vec::new();
    while let Ok(Some(msg)) = timeout(duration, rx.recv()).await {
      responses.push(msg);
    }
    Ok(responses)
  }

  // ── Reconnect task ─────────────────────────────────────────────────────────

  fn spawn_reconnect_task(
    config: ConnectionConfig,
    channels: Arc<SharedChannels>,
    router: Arc<RwLock<RouterHandle>>,
    initial_conn: Connection<Connected>,
    initial_disconnect_rx: oneshot::Receiver<bool>,
    disconnect_tx: broadcast::Sender<()>,
    reconnect_tx: broadcast::Sender<()>,
    cancelled: Arc<AtomicBool>,
  ) {
    tokio::spawn(async move {
      // Keep the live connection alive here. Replacing it drops the old one,
      // aborting its reader / router / keep-alive tasks.
      let mut _live_conn = initial_conn;
      let mut disconnect_rx = initial_disconnect_rx;

      loop {
        // true = abrupt (reconnect), false = graceful DisconnectRequest (no reconnect)
        let should_reconnect = disconnect_rx.await.unwrap_or(true);
        let _ = disconnect_tx.send(());

        if !should_reconnect || cancelled.load(Ordering::Relaxed) {
          info!("Connection closed — stopping reconnect loop.");
          break;
        }

        info!("Connection lost, attempting to reconnect…");

        let mut delay = Duration::from_secs(5);
        let mut new_conn: Connection<Connected> = loop {
          tokio::time::sleep(delay).await;

          match Connection::new_from_config(config.clone())
            .connect_with_channels(true, Arc::clone(&channels))
            .await
          {
            Ok(conn) => break conn,
            Err(e) => {
              warn!("Reconnect failed: {e}, retrying in {delay:?}");
              delay = (delay * 2).min(Duration::from_secs(60));
            }
          }
        };

        // Swap the router handle — all CommandHandle clones see the new connection.
        *router.write().unwrap() = new_conn.router_handle().clone();

        let Some(rx) = new_conn.take_device_disconnect_rx() else {
          warn!("No disconnect receiver after reconnect — stopping reconnect loop.");
          break;
        };

        _live_conn = new_conn; // drops old tasks, keeps new ones alive
        disconnect_rx = rx;

        let _ = reconnect_tx.send(());
        info!("Reconnected successfully.");
      }
    });
  }
}
