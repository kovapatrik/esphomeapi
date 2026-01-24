use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

pub mod entity;

use entity::Entity;
use esphomeapi::{
  Client, Options as _, api,
  model::{DeviceInfo, EntityInfo, EntityState, SUBCRIBE_STATES_RESPONSE_TYPES, UserService},
};

pub use esphomeapi::{Error, Result};

pub use esphomeapi::discovery::{ServiceInfo, discover};

pub struct Manager {
  pub device_info: DeviceInfo,
  entities: HashMap<u32, Entity>,
  states: Arc<RwLock<HashMap<u32, EntityState>>>,
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

    let states = Arc::new(RwLock::new(HashMap::new()));

    let mut state_msg_types = SUBCRIBE_STATES_RESPONSE_TYPES
      .keys()
      .cloned()
      .collect::<Vec<u32>>();

    state_msg_types.push(api::CameraImageResponse::get_option_id());

    for msg_type in state_msg_types {
      let states = states.clone();
      client.add_message_handler(
        msg_type,
        Box::new(move |_, msg| {
          if msg.protobuf_type == api::CameraImageResponse::get_option_id() {
            return Ok(());
          }

          if let Some(parser) = SUBCRIBE_STATES_RESPONSE_TYPES.get(&msg.protobuf_type) {
            let state = parser(&msg.protobuf_data).unwrap();
            states.write().unwrap().insert(state.key(), state);
          }
          Ok(())
        }),
        false,
      );
    }

    client.subscribe_states().await.unwrap();

    let mut entities = HashMap::new();

    let client = Arc::new(client);

    for entity in entities_response {
      match entity {
        EntityInfo::Light(info) => {
          let entity = entity::Light::new(client.clone(), info.clone(), states.clone());
          entities.insert(info.entity_info.key, Entity::Light(entity));
        }
        EntityInfo::Switch(info) => {
          let entity = entity::Switch::new(client.clone(), info.clone(), states.clone());
          entities.insert(info.entity_info.key, Entity::Switch(entity));
        }
        _ => {}
      }
    }

    let mut services = HashMap::new();

    for service in services_response {
      services.insert(service.key, service);
    }

    Self {
      device_info,
      entities,
      services,
      states,
    }
  }

  pub fn get_entities(&self) -> HashMap<u32, Entity> {
    self.entities.clone()
  }
}
