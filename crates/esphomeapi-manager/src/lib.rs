use std::{collections::HashMap, sync::Arc};

pub mod entity;

use entity::Entity;
pub use esphomeapi::model::{DeviceInfo, EntityState};
use esphomeapi::{
  Client,
  model::{EntityInfo, UserService},
};
use tokio::sync::{broadcast, watch};
use tracing::info;

pub use esphomeapi::discovery::{ServiceInfo, discover};
pub use esphomeapi::model::{HomeAssistantEvent, HomeassistantActionRequest, LogEvent, LogLevel};
pub use esphomeapi::{Error, Result};

pub struct Manager {
  pub client: Client,
  pub device_info: DeviceInfo,
  entities: HashMap<u32, Entity>,
  services: HashMap<u32, UserService>,
}

impl Manager {
  pub async fn new(
    address: String,
    port: u32,
    password: Option<String>,
    expected_name: Option<String>,
    psk: Option<String>,
    client_info: Option<String>,
    keep_alive_duration: Option<u32>,
  ) -> Manager {
    let client = Client::connect(
      address,
      port,
      password,
      expected_name,
      psk,
      client_info,
      keep_alive_duration,
    )
    .await
    .unwrap();

    let device_info = client.device_info().await.unwrap();
    let (entities_response, services_response) = client.list_entities_services().await.unwrap();

    let command_handle = Arc::new(client.command_handle());

    // Per-entity watch channels.
    let mut state_senders: HashMap<u32, watch::Sender<Option<EntityState>>> = HashMap::new();
    let mut entities = HashMap::new();

    for entity in entities_response {
      match entity {
        EntityInfo::Light(info) => {
          let (tx, rx) = watch::channel(None);
          state_senders.insert(info.entity_info.key, tx);
          let entity = entity::Light::new(Arc::clone(&command_handle), info.clone(), rx);
          entities.insert(info.entity_info.key, Entity::Light(entity));
        }
        EntityInfo::Switch(info) => {
          let (tx, rx) = watch::channel(None);
          state_senders.insert(info.entity_info.key, tx);
          let entity = entity::Switch::new(Arc::clone(&command_handle), info.clone(), rx);
          entities.insert(info.entity_info.key, Entity::Switch(entity));
        }
        _ => {}
      }
    }

    let mut services = HashMap::new();
    for service in services_response {
      services.insert(service.key, service);
    }

    client.request_states().await.unwrap();

    // The state receiver is tied to the long-lived SharedChannels — it keeps
    // working across reconnects without needing to be replaced.
    let state_subscriber = client.states_receiver();
    let state_senders = Arc::new(state_senders);
    Self::spawn_state_update_task(Arc::clone(&state_senders), state_subscriber);

    // After each reconnect, re-request entity states on the new connection.
    let reconnect_rx = client.on_reconnect();
    let client_for_task = client.clone();
    tokio::spawn(async move {
      let mut rx = reconnect_rx;
      loop {
        match rx.recv().await {
          Ok(()) => {
            let _ = client_for_task.request_states().await;
          }
          Err(broadcast::error::RecvError::Closed) => break,
          Err(broadcast::error::RecvError::Lagged(_)) => continue,
        }
      }
    });

    Manager {
      client,
      device_info,
      entities,
      services,
    }
  }

  pub fn get_entities(&self) -> &HashMap<u32, Entity> {
    &self.entities
  }

  pub fn get_services(&self) -> &HashMap<u32, UserService> {
    &self.services
  }

  /// Subscribe to device-initiated disconnect events.
  pub fn on_device_disconnect(&self) -> broadcast::Receiver<()> {
    self.client.on_device_disconnect()
  }

  /// Subscribe to successful reconnect events.
  pub fn on_reconnect(&self) -> broadcast::Receiver<()> {
    self.client.on_reconnect()
  }

  /// Get a receiver for all entity state updates.
  pub fn states_receiver(&self) -> broadcast::Receiver<EntityState> {
    self.client.states_receiver()
  }

  /// Subscribe to Home Assistant state events.
  pub async fn subscribe_home_assistant_states(
    &self,
  ) -> Result<broadcast::Receiver<HomeAssistantEvent>> {
    self.client.request_home_assistant_states().await?;
    Ok(self.client.home_assistant_states_receiver())
  }

  /// Get a receiver for Home Assistant state events without re-sending the request.
  pub fn home_assistant_states_receiver(&self) -> broadcast::Receiver<HomeAssistantEvent> {
    self.client.home_assistant_states_receiver()
  }

  /// Subscribe to Home Assistant action request events.
  pub async fn subscribe_home_assistant_action_requests(
    &self,
  ) -> Result<broadcast::Receiver<HomeassistantActionRequest>> {
    self.client.request_home_assistant_action_requests().await?;
    Ok(self.client.home_assistant_action_requests_receiver())
  }

  /// Get a receiver for Home Assistant action request events without re-sending the request.
  pub fn home_assistant_action_requests_receiver(
    &self,
  ) -> broadcast::Receiver<HomeassistantActionRequest> {
    self.client.home_assistant_action_requests_receiver()
  }

  /// Subscribe to ESPHome logs.
  pub async fn subscribe_logs(
    &self,
    log_level: LogLevel,
    dump_config: bool,
  ) -> Result<broadcast::Receiver<LogEvent>> {
    self.client.request_logs(log_level, dump_config).await?;
    Ok(self.client.logs_receiver())
  }

  /// Get a receiver for log events without re-sending the request.
  pub fn logs_receiver(&self) -> broadcast::Receiver<LogEvent> {
    self.client.logs_receiver()
  }

  /// Send the current state of a Home Assistant entity to the device.
  pub async fn send_home_assistant_state(
    &self,
    entity_id: String,
    state: String,
    attribute: Option<String>,
  ) -> Result<()> {
    self
      .client
      .send_home_assistant_state(entity_id, state, attribute)
      .await
  }

  /// Initiate a client-side disconnect.
  pub async fn disconnect(&self) -> Result<()> {
    self.client.disconnect().await
  }

  fn spawn_state_update_task(
    state_senders: Arc<HashMap<u32, watch::Sender<Option<EntityState>>>>,
    mut subscriber: broadcast::Receiver<EntityState>,
  ) {
    tokio::spawn(async move {
      while let Ok(state) = subscriber.recv().await {
        info!(state = ?state, "got state");
        if let Some(tx) = state_senders.get(&state.key()) {
          let _ = tx.send(Some(state));
        }
      }
    });
  }
}
