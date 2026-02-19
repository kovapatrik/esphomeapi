use protobuf::{EnumOrUnknown, Message};
use tokio::sync::broadcast;

use crate::{
  connection::{Connected, Connection, Disconnected},
  model::{
    parse_user_service, CameraImage, ColorMode, DeviceInfo, EntityInfo, EntityState,
    HomeAssistantEvent, HomeassistantActionRequest, LogEvent, UserService,
    LIST_ENTITIES_SERVICES_RESPONSE_TYPES,
  },
  utils::Options as _,
};
use std::time::Duration;

use crate::{proto, Result};

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

  /// Subscribe to camera image frames.
  ///
  /// Camera frames are sent automatically by the device once states are subscribed.
  /// Call this before `subscribe_states` to avoid missing early frames.
  pub fn subscribe_camera(&self) -> Result<broadcast::Receiver<CameraImage>> {
    let conn = self.connected()?;
    Ok(conn.subscribe_camera())
  }

  /// Subscribe to entity state updates.
  ///
  /// This sends a SubscribeStatesRequest to the device and returns a receiver
  /// for state update events. The device will continuously send state updates
  /// for all entities.
  pub async fn subscribe_states(&self) -> Result<broadcast::Receiver<EntityState>> {
    let conn = self.connected()?;
    let subscription = conn.subscribe_states();
    let message = proto::api::SubscribeStatesRequest::new();
    conn.send_message(Box::new(message)).await?;
    Ok(subscription)
  }

  /// Subscribe to Home Assistant state events.
  ///
  /// This sends a SubscribeHomeAssistantStatesRequest to the device and returns
  /// a receiver for HA state events. The device will send events indicating which
  /// Home Assistant entities it wants to monitor.
  pub async fn subscribe_home_assistant_states(
    &self,
  ) -> Result<broadcast::Receiver<HomeAssistantEvent>> {
    let conn = self.connected()?;
    let subscription = conn.subscribe_home_assistant_events();
    let message = proto::api::SubscribeHomeAssistantStatesRequest::new();
    conn.send_message(Box::new(message)).await?;
    Ok(subscription)
  }

  /// Subscribe to log events.
  ///
  /// This sends a SubscribeLogsRequest to the device and returns a receiver
  /// for log events. You can specify the minimum log level to receive.
  pub async fn subscribe_logs(
    &self,
    level: proto::api::LogLevel,
    dump_config: bool,
  ) -> Result<broadcast::Receiver<LogEvent>> {
    let conn = self.connected()?;
    let subscription = conn.subscribe_logs();
    let message = proto::api::SubscribeLogsRequest {
      level: EnumOrUnknown::new(level),
      dump_config,
      ..Default::default()
    };
    conn.send_message(Box::new(message)).await?;
    Ok(subscription)
  }

  /// Subscribe to Home Assistant action request events.
  ///
  /// This sends a SubscribeHomeassistantServicesRequest to the device and returns
  /// a receiver for action request events. The device will send events when it wants
  /// to trigger a Home Assistant service.
  pub async fn subscribe_home_assistant_action_requests(
    &self,
  ) -> Result<broadcast::Receiver<HomeassistantActionRequest>> {
    let conn = self.connected()?;
    let subscription = conn.subscribe_action_requests();
    let message = proto::api::SubscribeHomeassistantServicesRequest::new();
    conn.send_message(Box::new(message)).await?;
    Ok(subscription)
  }

  pub async fn switch_command(&self, key: u32, state: bool) -> Result<()> {
    let conn = self.connected()?;
    let message = proto::api::SwitchCommandRequest {
      key,
      state,
      ..Default::default()
    };

    conn.send_message(Box::new(message)).await?;
    Ok(())
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
    let conn = self.connected()?;
    let message = proto::api::LightCommandRequest {
      key,
      has_state: state.is_some(),
      state: state.unwrap_or_default(),
      has_brightness: brightness.is_some(),
      brightness: brightness.unwrap_or_default(),
      has_color_mode: color_mode.is_some(),
      color_mode: EnumOrUnknown::new(color_mode.unwrap_or_default().into()),
      has_color_brightness: color_brightness.is_some(),
      color_brightness: color_brightness.unwrap_or_default(),
      has_rgb: rgb.is_some(),
      red: rgb.unwrap_or_default().0,
      green: rgb.unwrap_or_default().1,
      blue: rgb.unwrap_or_default().2,
      has_white: white.is_some(),
      white: white.unwrap_or_default(),
      has_color_temperature: color_temperature.is_some(),
      color_temperature: color_temperature.unwrap_or_default(),
      has_cold_white: cold_white.is_some(),
      cold_white: cold_white.unwrap_or_default(),
      has_warm_white: warm_white.is_some(),
      warm_white: warm_white.unwrap_or_default(),
      has_transition_length: transition_length.is_some(),
      transition_length: (transition_length.unwrap_or_default() * 1000.0).round() as u32,
      has_flash_length: flash_length.is_some(),
      flash_length: (flash_length.unwrap_or_default() * 1000.0).round() as u32,
      has_effect: effect.is_some(),
      effect: effect.unwrap_or_default(),
      ..Default::default()
    };

    conn.send_message(Box::new(message)).await?;
    Ok(())
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
