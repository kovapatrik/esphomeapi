use std::{collections::HashMap, sync::Arc};

pub mod entity;

use entity::Entity;
pub use esphomeapi::model::EntityState;
use esphomeapi::{
  Client,
  model::{DeviceInfo, EntityInfo, UserService},
};

pub use esphomeapi::discovery::{ServiceInfo, discover};
use tokio::sync::{broadcast::Receiver, watch};
use tracing::info;

pub struct Manager {
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

    let client = Arc::new(client);

    // Create a watch channel per entity and build entity map
    let mut state_senders: HashMap<u32, watch::Sender<Option<EntityState>>> = HashMap::new();
    let mut entities = HashMap::new();

    for entity in entities_response {
      match entity {
        EntityInfo::Light(info) => {
          let (tx, rx) = watch::channel(None);
          state_senders.insert(info.entity_info.key, tx);
          let entity = entity::Light::new(client.clone(), info.clone(), rx);
          entities.insert(info.entity_info.key, Entity::Light(entity));
        }
        EntityInfo::Switch(info) => {
          let (tx, rx) = watch::channel(None);
          state_senders.insert(info.entity_info.key, tx);
          let entity = entity::Switch::new(client.clone(), info.clone(), rx);
          entities.insert(info.entity_info.key, Entity::Switch(entity));
        }
        _ => {}
      }
    }

    let mut services = HashMap::new();
    for service in services_response {
      services.insert(service.key, service);
    }

    let state_subscriber = client.subscribe_states().await.unwrap();
    Self::spawn_state_update_task(state_senders, state_subscriber);

    Self {
      device_info,
      entities,
      services,
    }
  }

  pub fn get_entities(&self) -> HashMap<u32, Entity> {
    self.entities.clone()
  }

  fn spawn_state_update_task(
    state_senders: HashMap<u32, watch::Sender<Option<EntityState>>>,
    mut subscriber: Receiver<EntityState>,
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
