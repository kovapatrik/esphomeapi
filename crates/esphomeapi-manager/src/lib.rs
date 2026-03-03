use std::{collections::HashMap, sync::Arc};

pub mod entity;

use entity::Entity;
pub use esphomeapi::model::{DeviceInfo, EntityState};
use esphomeapi::{
  Client, CommandHandle,
  model::{EntityInfo, UserService},
};

pub use esphomeapi::{Error, Result};

pub use esphomeapi::discovery::{ServiceInfo, discover};
pub use esphomeapi::model::{HomeAssistantEvent, HomeassistantActionRequest, LogEvent, LogLevel};
use tokio::sync::{broadcast, watch};
use tracing::info;

pub struct Manager {
  client: Client,
  command_handle: Arc<CommandHandle>,
  pub device_info: DeviceInfo,
  entities: HashMap<u32, Entity>,
  services: HashMap<u32, UserService>,
  disconnect_tx: broadcast::Sender<()>,
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
    let mut client = Client::new(
      address,
      port,
      password,
      expected_name,
      psk,
      client_info,
      keep_alive_duration,
    );

    client.connect(true).await.unwrap();
    let device_info = client.device_info().await.unwrap();
    let (entities_response, services_response) = client.list_entities_services().await.unwrap();

    let command_handle = Arc::new(client.command_handle().unwrap());

    // Create a watch channel per entity and build entity map
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

    // Send the state subscription request once, then use the receiver for internal routing.
    client.request_states().await.unwrap();
    let state_subscriber = client.states_receiver().unwrap();
    Self::spawn_state_update_task(state_senders, state_subscriber);

    // Internally handle device-initiated disconnects. Takes the one-shot receiver
    // so no &mut Client reference is needed inside the task.
    let (disconnect_tx, _) = broadcast::channel(1);
    if let Some(rx) = client.take_device_disconnect_receiver() {
      let tx = disconnect_tx.clone();
      tokio::spawn(async move {
        let _ = rx.await;
        let _ = tx.send(());
      });
    }

    Self {
      client,
      command_handle,
      device_info,
      entities,
      services,
      disconnect_tx,
    }
  }

  pub fn get_entities(&self) -> &HashMap<u32, Entity> {
    &self.entities
  }

  /// Subscribe to device-initiated disconnect events.
  ///
  /// The receiver resolves with `Ok(())` when the device sends a `DisconnectRequest`.
  /// The manager does not automatically reconnect; construct a new `Manager` to reconnect.
  pub fn on_device_disconnect(&self) -> broadcast::Receiver<()> {
    self.disconnect_tx.subscribe()
  }

  /// Get a new receiver for all entity state updates.
  ///
  /// Each call returns an independent receiver without sending a new request to the device.
  /// The subscription request was already sent during `new()`.
  pub fn states_receiver(&self) -> broadcast::Receiver<EntityState> {
    self.client.states_receiver().unwrap()
  }

  /// Subscribe to Home Assistant state events.
  ///
  /// Sends the subscription request to the device and returns a receiver. Call this
  /// once; call `home_assistant_states_receiver()` for additional independent receivers
  /// without re-sending the request.
  pub async fn subscribe_home_assistant_states(
    &self,
  ) -> Result<broadcast::Receiver<HomeAssistantEvent>> {
    self.client.request_home_assistant_states().await?;
    Ok(self.client.home_assistant_states_receiver()?)
  }

  /// Get a new receiver for Home Assistant state events without re-sending the request.
  pub fn home_assistant_states_receiver(&self) -> Result<broadcast::Receiver<HomeAssistantEvent>> {
    self.client.home_assistant_states_receiver()
  }

  /// Subscribe to Home Assistant action request events.
  ///
  /// Sends the subscription request to the device and returns a receiver. Call this
  /// once; call `home_assistant_action_requests_receiver()` for additional independent
  /// receivers without re-sending the request.
  pub async fn subscribe_home_assistant_action_requests(
    &self,
  ) -> Result<broadcast::Receiver<HomeassistantActionRequest>> {
    self.client.request_home_assistant_action_requests().await?;
    Ok(self.client.home_assistant_action_requests_receiver()?)
  }

  /// Get a new receiver for Home Assistant action request events without re-sending the request.
  pub fn home_assistant_action_requests_receiver(
    &self,
  ) -> Result<broadcast::Receiver<HomeassistantActionRequest>> {
    self.client.home_assistant_action_requests_receiver()
  }

  // Subscribe to ESPHome logs.
  ///
  /// Sends the subscription request to the device and returns a receiver. Call this
  /// once; call `logs_receiver()` for additional independent receivers without re-sending the request.
  pub async fn subscribe_logs(
    &self,
    log_level: LogLevel,
    dump_config: bool,
  ) -> Result<broadcast::Receiver<LogEvent>> {
    self.client.request_logs(log_level, dump_config).await?;
    Ok(self.client.logs_receiver()?)
  }

  /// Get a new receiver for ESPHome logs without re-sending the request.
  pub fn logs_receiver(&self) -> Result<broadcast::Receiver<LogEvent>> {
    self.client.logs_receiver()
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
    self
      .client
      .send_home_assistant_state(entity_id, state, attribute)
      .await
  }

  /// Disconnect from the device.
  pub async fn disconnect(&mut self) -> Result<()> {
    self.client.disconnect().await
  }

  fn spawn_state_update_task(
    state_senders: HashMap<u32, watch::Sender<Option<EntityState>>>,
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
