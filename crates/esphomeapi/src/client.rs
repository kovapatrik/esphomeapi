use protobuf::{EnumOrUnknown, Message as _};
use tokio::sync::{broadcast, oneshot};

use crate::{
  connection::{Connected, Connection, Disconnected},
  model::{
    parse_user_service, CameraImage, ColorMode, DeviceInfo, EntityInfo, EntityState,
    HomeAssistantEvent, HomeassistantActionRequest, LogEvent, LogLevel, UserService,
    LIST_ENTITIES_SERVICES_RESPONSE_TYPES,
  },
  utils::Options as _,
};
use std::time::Duration;

use crate::{proto, CommandHandle, Result};

/// Internal enum to wrap both connection states
enum ConnectionState {
  Disconnected(Connection<Disconnected>),
  Connected(Connection<Connected>),
}

pub struct Client {
  /// The wrapped connection, using Option to allow state transitions
  connection: Option<ConnectionState>,
}

impl Client {
  pub fn new(
    address: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<u32>,
  ) -> Self {
    Self {
      connection: Some(ConnectionState::Disconnected(Connection::new(
        address,
        port,
        password,
        expected_name,
        psk,
        client_info,
        keep_alive_duration,
      ))),
    }
  }

  /// Connect to the ESPHome device
  pub async fn connect(&mut self, login: bool) -> Result<()> {
    let conn = self
      .connection
      .take()
      .ok_or("Connection state is invalid")?;

    match conn {
      ConnectionState::Disconnected(c) => {
        let connected = c.connect(login).await?;
        self.connection = Some(ConnectionState::Connected(connected));
        Ok(())
      }
      ConnectionState::Connected(c) => {
        // Already connected, put it back
        self.connection = Some(ConnectionState::Connected(c));
        Err("Already connected".into())
      }
    }
  }

  /// Wait for the ESPHome device to initiate a disconnect.
  ///
  /// Blocks until the device sends `DisconnectRequest`.  At that point the
  /// router has already replied with `DisconnectResponse` and all tasks are
  /// torn down.  The client transitions to disconnected state so it can be
  /// reconnected later.
  pub async fn wait_for_device_disconnect(&mut self) -> Result<()> {
    let conn = self
      .connection
      .take()
      .ok_or("Connection state is invalid")?;

    match conn {
      ConnectionState::Connected(c) => {
        let disconnected = c.wait_for_device_disconnect().await?;
        self.connection = Some(ConnectionState::Disconnected(disconnected));
        Ok(())
      }
      ConnectionState::Disconnected(c) => {
        self.connection = Some(ConnectionState::Disconnected(c));
        Err("Not connected".into())
      }
    }
  }

  /// Disconnect from the ESPHome device
  pub async fn disconnect(&mut self) -> Result<()> {
    let conn = self
      .connection
      .take()
      .ok_or("Connection state is invalid")?;

    match conn {
      ConnectionState::Connected(c) => {
        let disconnected = c.disconnect().await?;
        self.connection = Some(ConnectionState::Disconnected(disconnected));
        Ok(())
      }
      ConnectionState::Disconnected(c) => {
        // Already disconnected, put it back
        self.connection = Some(ConnectionState::Disconnected(c));
        Ok(())
      }
    }
  }

  /// Check if connected
  pub fn is_connected(&self) -> bool {
    matches!(self.connection, Some(ConnectionState::Connected(_)))
  }

  /// Return a cloneable handle for sending device commands.
  ///
  /// The handle is cheap to clone and holds only a reference to the
  /// message router, so it does not prevent `disconnect()` from being called.
  pub fn command_handle(&self) -> Result<CommandHandle> {
    Ok(self.connected()?.command_handle())
  }

  /// Take the one-shot receiver that resolves when the device initiates a disconnect.
  ///
  /// Returns `None` if not connected or already taken.  Pass the receiver into a
  /// background task to react to device-initiated disconnects without holding a
  /// `&mut Client` reference inside the task.
  pub fn take_device_disconnect_receiver(&mut self) -> Option<oneshot::Receiver<()>> {
    match &mut self.connection {
      Some(ConnectionState::Connected(c)) => c.take_device_disconnect_rx(),
      _ => None,
    }
  }

  /// Get a reference to the connected connection, or return an error
  fn connected(&self) -> Result<&Connection<Connected>> {
    match &self.connection {
      Some(ConnectionState::Connected(c)) => Ok(c),
      Some(ConnectionState::Disconnected(_)) => Err("Not connected".into()),
      None => Err("Connection state is invalid".into()),
    }
  }

  pub async fn device_info(&self) -> Result<DeviceInfo> {
    let conn = self.connected()?;
    let message = proto::api::DeviceInfoRequest::default();

    let response = conn
      .send_message_await_response(
        Box::new(message),
        proto::api::DeviceInfoResponse::get_option_id(),
      )
      .await?;

    let response = proto::api::DeviceInfoResponse::parse_from_bytes(&response.protobuf_data)?;

    Ok(response.into())
  }

  pub async fn list_entities_services(&self) -> Result<(Vec<EntityInfo>, Vec<UserService>)> {
    let conn = self.connected()?;
    let message = proto::api::ListEntitiesRequest::new();

    let entity_service_map = LIST_ENTITIES_SERVICES_RESPONSE_TYPES.clone();
    let mut response_protobuf_types: Vec<u32> = entity_service_map.keys().cloned().collect();
    // Add user defined services to the list of expected responses
    response_protobuf_types.push(proto::api::ListEntitiesServicesResponse::get_option_id());

    let response = conn
      .send_message_await_until(
        Box::new(message),
        response_protobuf_types,
        proto::api::ListEntitiesDoneResponse::get_option_id(),
        Duration::from_secs(60),
      )
      .await?;

    let mut entities = Vec::new();
    let mut services = Vec::new();
    for message in response {
      if message.protobuf_type == proto::api::ListEntitiesServicesResponse::get_option_id() {
        let parsed_service = parse_user_service(&message.protobuf_data)?;
        services.push(parsed_service);
      } else {
        let parser = entity_service_map
          .get(&message.protobuf_type)
          .ok_or_else(|| format!("Unknown message type: {}", message.protobuf_type))?;
        let parsed_message = parser(&message.protobuf_data)?;
        entities.push(parsed_message);
      }
    }

    Ok((entities, services))
  }

  /// Send a SubscribeStatesRequest to the device.
  ///
  /// Call this once to start receiving entity state updates. After calling this,
  /// use `states_receiver()` to get one or more independent receivers for the stream.
  pub async fn request_states(&self) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::SubscribeStatesRequest::new();
    conn.send_message(Box::new(message)).await
  }

  /// Get a new receiver for entity state updates.
  ///
  /// Each call returns an independent receiver that gets all future state messages.
  /// Call `request_states()` once before subscribing; this method can be called
  /// multiple times without sending additional requests to the device.
  pub fn states_receiver(&self) -> Result<broadcast::Receiver<EntityState>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_states())
  }

  /// Send a SubscribeHomeAssistantStatesRequest to the device.
  ///
  /// Call this once to start receiving HA state events. After calling this,
  /// use `home_assistant_states_receiver()` to get one or more independent receivers.
  pub async fn request_home_assistant_states(&self) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::SubscribeHomeAssistantStatesRequest::new();
    conn.send_message(Box::new(message)).await
  }

  /// Get a new receiver for Home Assistant state events.
  ///
  /// Each call returns an independent receiver. Call `request_home_assistant_states()`
  /// once before subscribing; this method can be called multiple times without
  /// sending additional requests to the device.
  pub fn home_assistant_states_receiver(&self) -> Result<broadcast::Receiver<HomeAssistantEvent>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_home_assistant_events())
  }

  /// Send a SubscribeLogsRequest to the device.
  ///
  /// Call this once to start receiving log events. After calling this,
  /// use `logs_receiver()` to get one or more independent receivers.
  pub async fn request_logs(&self, level: LogLevel, dump_config: bool) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::SubscribeLogsRequest {
      level: EnumOrUnknown::new(level.into()),
      dump_config,
      ..Default::default()
    };
    conn.send_message(Box::new(message)).await
  }

  /// Get a new receiver for log events.
  ///
  /// Each call returns an independent receiver. Call `request_logs()` once before
  /// subscribing; this method can be called multiple times without sending additional
  /// requests to the device.
  pub fn logs_receiver(&self) -> Result<broadcast::Receiver<LogEvent>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_logs())
  }

  /// Send a SubscribeHomeassistantServicesRequest to the device.
  ///
  /// Call this once to start receiving action request events. After calling this,
  /// use `home_assistant_action_requests_receiver()` to get one or more independent receivers.
  pub async fn request_home_assistant_action_requests(&self) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::SubscribeHomeassistantServicesRequest::new();
    conn.send_message(Box::new(message)).await
  }

  /// Get a new receiver for Home Assistant action request events.
  ///
  /// Each call returns an independent receiver. Call
  /// `request_home_assistant_action_requests()` once before subscribing; this method
  /// can be called multiple times without sending additional requests to the device.
  pub fn home_assistant_action_requests_receiver(
    &self,
  ) -> Result<broadcast::Receiver<HomeassistantActionRequest>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_action_requests())
  }

  /// Get a new receiver for camera image frames.
  ///
  /// Camera frames are sent automatically by the device once states are subscribed
  /// via `request_states()`. Each call returns an independent receiver; call this
  /// before `request_states()` to avoid missing early frames.
  pub fn camera_receiver(&self) -> Result<broadcast::Receiver<CameraImage>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_camera())
  }

  pub async fn switch_command(&self, key: u32, state: bool) -> Result<()> {
    self
      .connected()?
      .command_handle()
      .switch_command(key, state)
      .await
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
      .connected()?
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

  /// Send the current state of a Home Assistant entity to the device.
  ///
  /// This is used to respond to HomeAssistantEvent::StateRequest or
  /// to update the device when a subscribed entity changes.
  pub async fn send_home_assistant_state(
    &self,
    entity_id: String,
    state: String,
    attribute: Option<String>,
  ) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::HomeAssistantStateResponse {
      entity_id,
      state,
      attribute: attribute.unwrap_or_default(),
      ..Default::default()
    };

    conn.send_message(Box::new(message)).await?;
    Ok(())
  }
}
