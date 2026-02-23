use esphomeapi_manager::DeviceInfo as RustDeviceInfo;
use napi_derive::napi;

#[derive(Clone)]
#[napi(object)]
pub struct DeviceInfo {
  pub uses_password: bool,
  pub name: String,
  pub friendly_name: String,
  pub mac_address: String,
  pub compilation_time: String,
  pub model: String,
  pub manufacturer: String,
  pub has_deep_sleep: bool,
  pub esphome_version: String,
  pub project_name: String,
  pub project_version: String,
  pub webserver_port: u32,
  pub legacy_voice_assistant_version: u32,
  pub voice_assistant_feature_flags: u32,
  pub legacy_bluetooth_proxy_version: u32,
  pub bluetooth_proxy_feature_flags: u32,
  pub suggested_area: String,
}

impl From<RustDeviceInfo> for DeviceInfo {
  fn from(info: RustDeviceInfo) -> Self {
    DeviceInfo {
      uses_password: info.uses_password,
      name: info.name,
      friendly_name: info.friendly_name,
      mac_address: info.mac_address,
      compilation_time: info.compilation_time,
      model: info.model,
      manufacturer: info.manufacturer,
      has_deep_sleep: info.has_deep_sleep,
      esphome_version: info.esphome_version,
      project_name: info.project_name,
      project_version: info.project_version,
      webserver_port: info.webserver_port,
      legacy_voice_assistant_version: info.legacy_voice_assistant_version,
      voice_assistant_feature_flags: info.voice_assistant_feature_flags,
      legacy_bluetooth_proxy_version: info.legacy_bluetooth_proxy_version,
      bluetooth_proxy_feature_flags: info.bluetooth_proxy_feature_flags,
      suggested_area: info.suggested_area,
    }
  }
}
