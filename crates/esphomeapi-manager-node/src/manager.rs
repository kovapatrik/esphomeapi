use esphomeapi_manager::Manager as RustManager;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::entity;

#[napi(object)]
pub struct ConnectionOptions {
  pub address: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: Option<String>,
  pub keep_alive_duration: Option<u32>,
}

#[napi]
pub struct Manager {
  inner: RustManager,
}

#[napi]
impl Manager {
  #[napi(factory)]
  pub async fn connect(options: ConnectionOptions) -> Result<Manager> {
    let manager = RustManager::new(
      options.address,
      options.port,
      options.password,
      options.expected_name,
      options.psk,
      options.client_info,
      options.keep_alive_duration,
    )
    .await;

    Ok(Manager { inner: manager })
  }

  #[napi]
  pub fn get_device_name(&self) -> String {
    self.inner.device_info.name.clone()
  }

  #[napi]
  pub fn get_device_mac(&self) -> String {
    self.inner.device_info.mac_address.clone()
  }

  #[napi]
  pub fn get_entities(&self) -> Vec<Either<entity::Light, entity::Switch>> {
    self
      .inner
      .get_entities()
      .values()
      // .into_values()
      .filter_map(|e| match e {
        esphomeapi_manager::entity::Entity::Light(l) => Some(Either::A(entity::Light::new(l))),
        esphomeapi_manager::entity::Entity::Switch(s) => Some(Either::B(entity::Switch::new(s))),
        _ => None,
      })
      .collect()
  }
}
