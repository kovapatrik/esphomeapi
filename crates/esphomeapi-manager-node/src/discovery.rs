use esphomeapi_manager::ServiceInfo as RustServiceInfo;
use napi_derive::napi;

use crate::Result;

#[napi(object)]
pub struct ServiceInfo {
  pub ty_domain: String,
  pub sub_domain: Option<String>,
  pub fullname: String,
  pub server: String,
  pub addresses: Vec<String>,
  pub port: u16,
  pub host_ttl: u32,
  pub other_ttl: u32,
  pub priority: u16,
  pub weight: u16,
}

impl From<RustServiceInfo> for ServiceInfo {
  fn from(value: RustServiceInfo) -> Self {
    Self {
      ty_domain: value.ty_domain,
      sub_domain: value.sub_domain,
      fullname: value.fullname,
      server: value.server,
      addresses: value.addresses.iter().map(|a| a.to_string()).collect(),
      port: value.port,
      host_ttl: value.host_ttl,
      other_ttl: value.other_ttl,
      priority: value.priority,
      weight: value.weight,
    }
  }
}

#[napi]
pub async fn discover(seconds: u32) -> Result<Vec<ServiceInfo>> {
  let result = esphomeapi_manager::discover(seconds).await?;
  Ok(
    result
      .iter()
      .map(|service_info| service_info.clone().into())
      .collect(),
  )
}
