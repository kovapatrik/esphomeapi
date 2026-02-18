use std::collections::HashMap;

use enumflags2::{bitflags, BitFlags};

use crate::{
  api::{self, DeviceInfoResponse},
  proto,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct APIVersion {
  pub major: u8,
  pub minor: u8,
}

impl APIVersion {
  pub fn new(major: u8, minor: u8) -> APIVersion {
    APIVersion { major, minor }
  }
}

#[bitflags]
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BluetoothProxyFeature {
  PassiveScan = 1 << 0,
  ActiveConnections = 1 << 1,
  RemoteCaching = 1 << 2,
  Pairing = 1 << 3,
  CacheClearing = 1 << 4,
  RawAdvertisements = 1 << 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothProxySubscriptionFlag {
  RawAdvertisements = 1 << 0,
}

#[bitflags]
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum VoiceAssistantFeature {
  VoiceAssistant = 1 << 0,
  Speaker = 1 << 1,
  APIAudio = 1 << 2,
  Timers = 1 << 3,
  Announce = 1 << 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceAssistantSubscriptionFlag {
  APIAudio = 1 << 0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<DeviceInfoResponse> for DeviceInfo {
  fn from(proto: DeviceInfoResponse) -> Self {
    DeviceInfo {
      uses_password: proto.uses_password,
      name: proto.name,
      friendly_name: proto.friendly_name,
      mac_address: proto.mac_address,
      compilation_time: proto.compilation_time,
      model: proto.model,
      manufacturer: proto.manufacturer,
      has_deep_sleep: proto.has_deep_sleep,
      esphome_version: proto.esphome_version,
      project_name: proto.project_name,
      project_version: proto.project_version,
      webserver_port: proto.webserver_port,
      legacy_voice_assistant_version: proto.legacy_voice_assistant_version,
      voice_assistant_feature_flags: proto.voice_assistant_feature_flags,
      legacy_bluetooth_proxy_version: proto.legacy_bluetooth_proxy_version,
      bluetooth_proxy_feature_flags: proto.bluetooth_proxy_feature_flags,
      suggested_area: proto.suggested_area,
    }
  }
}

impl DeviceInfo {
  pub fn default() -> DeviceInfo {
    DeviceInfo {
      uses_password: false,
      name: String::new(),
      friendly_name: String::new(),
      mac_address: String::new(),
      compilation_time: String::new(),
      model: String::new(),
      manufacturer: String::new(),
      has_deep_sleep: false,
      esphome_version: String::new(),
      project_name: String::new(),
      project_version: String::new(),
      webserver_port: 0,
      legacy_voice_assistant_version: 0,
      voice_assistant_feature_flags: 0,
      legacy_bluetooth_proxy_version: 0,
      bluetooth_proxy_feature_flags: 0,
      suggested_area: String::new(),
    }
  }

  pub fn bluetooth_proxy_feature_flags_compat(&self, api_version: APIVersion) -> u32 {
    if api_version < APIVersion::new(1, 9) {
      let mut flags = BitFlags::empty();
      if self.legacy_bluetooth_proxy_version >= 1 {
        flags |= BluetoothProxyFeature::PassiveScan;
      }
      if self.legacy_bluetooth_proxy_version >= 2 {
        flags |= BluetoothProxyFeature::ActiveConnections;
      }
      if self.legacy_bluetooth_proxy_version >= 3 {
        flags |= BluetoothProxyFeature::RemoteCaching;
      }
      if self.legacy_bluetooth_proxy_version >= 4 {
        flags |= BluetoothProxyFeature::Pairing;
      }
      if self.legacy_bluetooth_proxy_version >= 5 {
        flags |= BluetoothProxyFeature::CacheClearing;
      }
      return flags.bits();
    }
    return self.bluetooth_proxy_feature_flags;
  }

  pub fn voice_assistant_feature_flags_compat(&self, api_version: APIVersion) -> u32 {
    if api_version < APIVersion::new(1, 10) {
      let mut flags = BitFlags::empty();
      if self.legacy_voice_assistant_version >= 1 {
        flags |= VoiceAssistantFeature::VoiceAssistant;
      }
      if self.legacy_voice_assistant_version >= 2 {
        flags |= VoiceAssistantFeature::Speaker;
      }
      return flags.bits();
    }
    return self.voice_assistant_feature_flags;
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityCategory {
  None = 0,
  Config,
  Diagnostic,
}

impl From<proto::api::EntityCategory> for EntityCategory {
  fn from(value: proto::api::EntityCategory) -> Self {
    match value {
      proto::api::EntityCategory::ENTITY_CATEGORY_NONE => EntityCategory::None,
      proto::api::EntityCategory::ENTITY_CATEGORY_CONFIG => EntityCategory::Config,
      proto::api::EntityCategory::ENTITY_CATEGORY_DIAGNOSTIC => EntityCategory::Diagnostic,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseEntityInfo {
  pub object_id: String,
  pub key: u32,
  pub name: String,
  pub disabled_by_default: bool,
  pub icon: String,
  pub entity_category: EntityCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseEntityState {
  pub key: u32,
}

// ==================== BINARY SENSOR ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySensorInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
  pub is_status_binary_sensor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySensorState {
  pub entity_state: BaseEntityState,
  pub state: bool,
  pub missing_state: bool,
}

// ==================== COVER ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverInfo {
  pub entity_info: BaseEntityInfo,
  pub assumed_state: bool,
  pub supports_stop: bool,
  pub supports_position: bool,
  pub supports_tilt: bool,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyCoverState {
  Open = 0,
  Closed,
}

impl From<proto::api::LegacyCoverState> for LegacyCoverState {
  fn from(value: proto::api::LegacyCoverState) -> Self {
    match value {
      proto::api::LegacyCoverState::LEGACY_COVER_STATE_OPEN => LegacyCoverState::Open,
      proto::api::LegacyCoverState::LEGACY_COVER_STATE_CLOSED => LegacyCoverState::Closed,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyCoverCommand {
  Open = 0,
  Close,
  Stop,
}

impl From<proto::api::LegacyCoverCommand> for LegacyCoverCommand {
  fn from(value: proto::api::LegacyCoverCommand) -> Self {
    match value {
      proto::api::LegacyCoverCommand::LEGACY_COVER_COMMAND_OPEN => LegacyCoverCommand::Open,
      proto::api::LegacyCoverCommand::LEGACY_COVER_COMMAND_CLOSE => LegacyCoverCommand::Close,
      proto::api::LegacyCoverCommand::LEGACY_COVER_COMMAND_STOP => LegacyCoverCommand::Stop,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverOperation {
  Idle = 0,
  Opening,
  Closing,
}

impl From<proto::api::CoverOperation> for CoverOperation {
  fn from(value: proto::api::CoverOperation) -> Self {
    match value {
      proto::api::CoverOperation::COVER_OPERATION_IDLE => CoverOperation::Idle,
      proto::api::CoverOperation::COVER_OPERATION_IS_OPENING => CoverOperation::Opening,
      proto::api::CoverOperation::COVER_OPERATION_IS_CLOSING => CoverOperation::Closing,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverState {
  pub entity_state: BaseEntityState,
  pub legacy_state: LegacyCoverState,
  pub position: f32,
  pub tilt: f32,
  pub current_operation: CoverOperation,
}

impl CoverState {
  pub fn is_closed(&self, api_version: APIVersion) -> bool {
    if api_version < APIVersion::new(1, 1) {
      return self.legacy_state == LegacyCoverState::Closed;
    }
    return self.position == 0.0;
  }
}

// ==================== EVENT ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
  pub event_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
  pub entity_state: BaseEntityState,
  pub event_type: String,
}

// ==================== FAN ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanInfo {
  pub entity_info: BaseEntityInfo,
  pub supports_oscillation: bool,
  pub supports_speed: bool,
  pub supports_direction: bool,
  pub supported_speed_count: i32,
  pub supported_preset_modes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanSpeed {
  Low = 0,
  Medium,
  High,
}

impl From<proto::api::FanSpeed> for FanSpeed {
  fn from(value: proto::api::FanSpeed) -> Self {
    match value {
      proto::api::FanSpeed::FAN_SPEED_LOW => FanSpeed::Low,
      proto::api::FanSpeed::FAN_SPEED_MEDIUM => FanSpeed::Medium,
      proto::api::FanSpeed::FAN_SPEED_HIGH => FanSpeed::High,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanDirection {
  Forward = 0,
  Reverse,
}

impl From<proto::api::FanDirection> for FanDirection {
  fn from(value: proto::api::FanDirection) -> Self {
    match value {
      proto::api::FanDirection::FAN_DIRECTION_FORWARD => FanDirection::Forward,
      proto::api::FanDirection::FAN_DIRECTION_REVERSE => FanDirection::Reverse,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanState {
  pub entity_state: BaseEntityState,
  pub oscillating: bool,
  pub speed: FanSpeed,
  pub speed_level: i32,
  pub direction: FanDirection,
  pub preset_mode: String,
}

// ==================== LIGHT ====================
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum LightColorCapability {
  OnOff = 1 << 0,
  Brightness = 1 << 1,
  White = 1 << 2,
  ColorTemperature = 1 << 3,
  ColdWarmWhite = 1 << 4,
  RGB = 1 << 5,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub enum ColorMode {
  #[default]
  Unknown = 0,
  OnOff = 1,
  LegacyBrightness = 2,
  Brightness = 3,
  White = 7,
  ColorTemperature = 11,
  ColdWarmWhite = 19,
  RGB = 35,
  RGBWhite = 39,
  RGBColorTemperature = 47,
  RGBColdWarmWhite = 51,
}

impl From<proto::api::ColorMode> for ColorMode {
  fn from(value: proto::api::ColorMode) -> Self {
    match value {
      proto::api::ColorMode::COLOR_MODE_UNKNOWN => ColorMode::Unknown,
      proto::api::ColorMode::COLOR_MODE_ON_OFF => ColorMode::OnOff,
      proto::api::ColorMode::COLOR_MODE_LEGACY_BRIGHTNESS => ColorMode::LegacyBrightness,
      proto::api::ColorMode::COLOR_MODE_BRIGHTNESS => ColorMode::Brightness,
      proto::api::ColorMode::COLOR_MODE_WHITE => ColorMode::White,
      proto::api::ColorMode::COLOR_MODE_COLOR_TEMPERATURE => ColorMode::ColorTemperature,
      proto::api::ColorMode::COLOR_MODE_COLD_WARM_WHITE => ColorMode::ColdWarmWhite,
      proto::api::ColorMode::COLOR_MODE_RGB => ColorMode::RGB,
      proto::api::ColorMode::COLOR_MODE_RGB_WHITE => ColorMode::RGBWhite,
      proto::api::ColorMode::COLOR_MODE_RGB_COLOR_TEMPERATURE => ColorMode::RGBColorTemperature,
      proto::api::ColorMode::COLOR_MODE_RGB_COLD_WARM_WHITE => ColorMode::RGBColdWarmWhite,
    }
  }
}

impl Into<proto::api::ColorMode> for ColorMode {
  fn into(self) -> proto::api::ColorMode {
    match self {
      ColorMode::Unknown => proto::api::ColorMode::COLOR_MODE_UNKNOWN,
      ColorMode::OnOff => proto::api::ColorMode::COLOR_MODE_ON_OFF,
      ColorMode::LegacyBrightness => proto::api::ColorMode::COLOR_MODE_LEGACY_BRIGHTNESS,
      ColorMode::Brightness => proto::api::ColorMode::COLOR_MODE_BRIGHTNESS,
      ColorMode::White => proto::api::ColorMode::COLOR_MODE_WHITE,
      ColorMode::ColorTemperature => proto::api::ColorMode::COLOR_MODE_COLOR_TEMPERATURE,
      ColorMode::ColdWarmWhite => proto::api::ColorMode::COLOR_MODE_COLD_WARM_WHITE,
      ColorMode::RGB => proto::api::ColorMode::COLOR_MODE_RGB,
      ColorMode::RGBWhite => proto::api::ColorMode::COLOR_MODE_RGB_WHITE,
      ColorMode::RGBColorTemperature => proto::api::ColorMode::COLOR_MODE_RGB_COLOR_TEMPERATURE,
      ColorMode::RGBColdWarmWhite => proto::api::ColorMode::COLOR_MODE_RGB_COLD_WARM_WHITE,
    }
  }
}

impl Into<u8> for ColorMode {
  fn into(self) -> u8 {
    self as u8
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightInfo {
  pub entity_info: BaseEntityInfo,
  pub supported_color_modes: Vec<ColorMode>,
  pub min_mireds: f32,
  pub max_mireds: f32,
  pub effects: Vec<String>,

  // deprecated, do not use
  pub legacy_supports_brightness: bool,
  pub legacy_supports_rgb: bool,
  pub legacy_supports_white_value: bool,
  pub legacy_supports_color_temperature: bool,
}

impl LightInfo {
  pub fn supported_color_modes_compat(&self, api_version: APIVersion) -> Vec<u8> {
    if api_version < APIVersion::new(1, 6) {
      let key = (
        self.legacy_supports_brightness,
        self.legacy_supports_rgb,
        self.legacy_supports_white_value,
        self.legacy_supports_color_temperature,
      );

      let legacy_mode = match key {
        (false, false, false, false) => LightColorCapability::OnOff as u8,
        (true, false, false, false) => {
          (LightColorCapability::OnOff | LightColorCapability::Brightness).bits()
        }
        (true, false, false, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::ColorTemperature)
          .bits(),
        (true, true, false, false) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB)
          .bits(),
        (true, true, true, false) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::White)
          .bits(),
        (true, true, false, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::ColorTemperature)
          .bits(),
        (true, true, true, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::White
          | LightColorCapability::ColorTemperature)
          .bits(),
        _ => LightColorCapability::OnOff as u8,
      };

      return vec![legacy_mode];
    }
    return self
      .supported_color_modes
      .clone()
      .iter()
      .map(|x| (*x).into())
      .collect();
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightState {
  pub entity_state: BaseEntityState,
  pub state: bool,
  pub brightness: f32,
  pub color_mode: ColorMode,
  pub color_brightness: f32,
  pub red: f32,
  pub green: f32,
  pub blue: f32,
  pub white: f32,
  pub color_temperature: f32,
  pub cold_white: f32,
  pub warm_white: f32,
  pub effect: String,
}

// ==================== SENSOR ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStateClass {
  None = 0,
  Measurement,
  TotalIncreasing,
  Total,
  MeasurementAngle,
}

impl From<proto::api::SensorStateClass> for SensorStateClass {
  fn from(value: proto::api::SensorStateClass) -> Self {
    match value {
      proto::api::SensorStateClass::STATE_CLASS_NONE => SensorStateClass::None,
      proto::api::SensorStateClass::STATE_CLASS_MEASUREMENT => SensorStateClass::Measurement,
      proto::api::SensorStateClass::STATE_CLASS_TOTAL_INCREASING => {
        SensorStateClass::TotalIncreasing
      }
      proto::api::SensorStateClass::STATE_CLASS_TOTAL => SensorStateClass::Total,
      proto::api::SensorStateClass::STATE_CLASS_MEASUREMENT_ANGLE => {
        SensorStateClass::MeasurementAngle
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastResetType {
  None = 0,
  Never,
  Auto,
}

impl From<proto::api::SensorLastResetType> for LastResetType {
  fn from(value: proto::api::SensorLastResetType) -> Self {
    match value {
      proto::api::SensorLastResetType::LAST_RESET_NONE => LastResetType::None,
      proto::api::SensorLastResetType::LAST_RESET_NEVER => LastResetType::Never,
      proto::api::SensorLastResetType::LAST_RESET_AUTO => LastResetType::Auto,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
  pub unit_of_measurement: String,
  pub accuracy_decimals: i32,
  pub force_update: bool,
  pub state_class: SensorStateClass,
  pub legacy_last_reset_type: LastResetType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorState {
  pub entity_state: BaseEntityState,
  pub state: f32,
  pub missing_state: bool,
}

// ==================== SWITCH ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchInfo {
  pub entity_info: BaseEntityInfo,
  pub assumed_state: bool,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchState {
  pub entity_state: BaseEntityState,
  pub state: bool,
}

// ==================== TEXT SENSOR ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSensorInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSensorState {
  pub entity_state: BaseEntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== CAMERA ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraInfo {
  pub entity_info: BaseEntityInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraState {
  pub entity_state: BaseEntityState,
  pub data: Vec<u8>,
}

// ==================== CLIMATE ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClimateMode {
  Off = 0,
  HeatCool,
  Cool,
  Heat,
  FanOnly,
  Dry,
  Auto,
}

impl From<proto::api::ClimateMode> for ClimateMode {
  fn from(value: proto::api::ClimateMode) -> Self {
    match value {
      proto::api::ClimateMode::CLIMATE_MODE_OFF => ClimateMode::Off,
      proto::api::ClimateMode::CLIMATE_MODE_HEAT_COOL => ClimateMode::HeatCool,
      proto::api::ClimateMode::CLIMATE_MODE_COOL => ClimateMode::Cool,
      proto::api::ClimateMode::CLIMATE_MODE_HEAT => ClimateMode::Heat,
      proto::api::ClimateMode::CLIMATE_MODE_FAN_ONLY => ClimateMode::FanOnly,
      proto::api::ClimateMode::CLIMATE_MODE_DRY => ClimateMode::Dry,
      proto::api::ClimateMode::CLIMATE_MODE_AUTO => ClimateMode::Auto,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClimateFanMode {
  On = 0,
  Off,
  Auto,
  Low,
  Medium,
  High,
  Middle,
  Focus,
  Diffuse,
  Quiet,
}

impl From<proto::api::ClimateFanMode> for ClimateFanMode {
  fn from(value: proto::api::ClimateFanMode) -> Self {
    match value {
      proto::api::ClimateFanMode::CLIMATE_FAN_ON => ClimateFanMode::On,
      proto::api::ClimateFanMode::CLIMATE_FAN_OFF => ClimateFanMode::Off,
      proto::api::ClimateFanMode::CLIMATE_FAN_AUTO => ClimateFanMode::Auto,
      proto::api::ClimateFanMode::CLIMATE_FAN_LOW => ClimateFanMode::Low,
      proto::api::ClimateFanMode::CLIMATE_FAN_MEDIUM => ClimateFanMode::Medium,
      proto::api::ClimateFanMode::CLIMATE_FAN_HIGH => ClimateFanMode::High,
      proto::api::ClimateFanMode::CLIMATE_FAN_MIDDLE => ClimateFanMode::Middle,
      proto::api::ClimateFanMode::CLIMATE_FAN_FOCUS => ClimateFanMode::Focus,
      proto::api::ClimateFanMode::CLIMATE_FAN_DIFFUSE => ClimateFanMode::Diffuse,
      proto::api::ClimateFanMode::CLIMATE_FAN_QUIET => ClimateFanMode::Quiet,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClimateSwingMode {
  Off = 0,
  Both,
  Vertical,
  Horizontal,
}

impl From<proto::api::ClimateSwingMode> for ClimateSwingMode {
  fn from(value: proto::api::ClimateSwingMode) -> Self {
    match value {
      proto::api::ClimateSwingMode::CLIMATE_SWING_OFF => ClimateSwingMode::Off,
      proto::api::ClimateSwingMode::CLIMATE_SWING_BOTH => ClimateSwingMode::Both,
      proto::api::ClimateSwingMode::CLIMATE_SWING_VERTICAL => ClimateSwingMode::Vertical,
      proto::api::ClimateSwingMode::CLIMATE_SWING_HORIZONTAL => ClimateSwingMode::Horizontal,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClimateAction {
  Off = 0,
  Cooling,
  Heating,
  Idle,
  Drying,
  Fan,
}

impl From<proto::api::ClimateAction> for ClimateAction {
  fn from(value: proto::api::ClimateAction) -> Self {
    match value {
      proto::api::ClimateAction::CLIMATE_ACTION_OFF => ClimateAction::Off,
      proto::api::ClimateAction::CLIMATE_ACTION_COOLING => ClimateAction::Cooling,
      proto::api::ClimateAction::CLIMATE_ACTION_HEATING => ClimateAction::Heating,
      proto::api::ClimateAction::CLIMATE_ACTION_IDLE => ClimateAction::Idle,
      proto::api::ClimateAction::CLIMATE_ACTION_DRYING => ClimateAction::Drying,
      proto::api::ClimateAction::CLIMATE_ACTION_FAN => ClimateAction::Fan,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ClimatePreset {
  None = 0,
  Home,
  Away,
  Boost,
  Comfort,
  Eco,
  Sleep,
  Activity,
}

impl From<proto::api::ClimatePreset> for ClimatePreset {
  fn from(value: proto::api::ClimatePreset) -> Self {
    match value {
      proto::api::ClimatePreset::CLIMATE_PRESET_NONE => ClimatePreset::None,
      proto::api::ClimatePreset::CLIMATE_PRESET_HOME => ClimatePreset::Home,
      proto::api::ClimatePreset::CLIMATE_PRESET_AWAY => ClimatePreset::Away,
      proto::api::ClimatePreset::CLIMATE_PRESET_BOOST => ClimatePreset::Boost,
      proto::api::ClimatePreset::CLIMATE_PRESET_COMFORT => ClimatePreset::Comfort,
      proto::api::ClimatePreset::CLIMATE_PRESET_ECO => ClimatePreset::Eco,
      proto::api::ClimatePreset::CLIMATE_PRESET_SLEEP => ClimatePreset::Sleep,
      proto::api::ClimatePreset::CLIMATE_PRESET_ACTIVITY => ClimatePreset::Activity,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClimateInfo {
  pub entity_info: BaseEntityInfo,
  pub supports_current_temperature: bool,
  pub supports_two_point_target_temperature: bool,
  pub supported_modes: Vec<ClimateMode>,
  pub visual_min_temperature: f32,
  pub visual_max_temperature: f32,
  pub visual_target_temperature_step: f32,
  pub visual_current_temperature_step: f32,
  pub legacy_supports_away: bool,
  pub supports_action: bool,
  pub supported_fan_modes: Vec<ClimateFanMode>,
  pub supported_swing_modes: Vec<ClimateSwingMode>,
  pub supported_custom_fan_modes: Vec<String>,
  pub supported_presets: Vec<ClimatePreset>,
  pub supported_custom_presets: Vec<String>,
  pub supports_current_humidity: bool,
  pub supports_target_humidity: bool,
  pub visual_min_humidity: f32,
  pub visual_max_humidity: f32,
}

impl ClimateInfo {
  pub fn supported_presets_compat(&self, api_version: APIVersion) -> Vec<ClimatePreset> {
    if api_version < APIVersion::new(1, 5) {
      if self.legacy_supports_away {
        return vec![ClimatePreset::Home, ClimatePreset::Away];
      }
      return vec![];
    }
    return self.supported_presets.clone();
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClimateState {
  pub entity_state: BaseEntityState,
  pub mode: ClimateMode,
  pub action: ClimateAction,
  pub current_temperature: f32,
  pub target_temperature: f32,
  pub target_temperature_low: f32,
  pub target_temperature_high: f32,
  pub legacy_away: bool,
  pub fan_mode: ClimateFanMode,
  pub swing_mode: ClimateSwingMode,
  pub custom_fan_mode: String,
  pub preset: ClimatePreset,
  pub custom_preset: String,
  pub current_humidity: f32,
  pub target_humidity: f32,
}

impl ClimateState {
  pub fn preset_compat(&self, api_version: APIVersion) -> ClimatePreset {
    if api_version < APIVersion::new(1, 5) {
      if self.legacy_away {
        return ClimatePreset::Away;
      }
      return ClimatePreset::Home;
    }
    return self.preset;
  }
}

// ==================== NUMBER ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberMode {
  Auto = 0,
  Box,
  Slider,
}

impl From<proto::api::NumberMode> for NumberMode {
  fn from(value: proto::api::NumberMode) -> Self {
    match value {
      proto::api::NumberMode::NUMBER_MODE_AUTO => NumberMode::Auto,
      proto::api::NumberMode::NUMBER_MODE_BOX => NumberMode::Box,
      proto::api::NumberMode::NUMBER_MODE_SLIDER => NumberMode::Slider,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberInfo {
  pub entity_info: BaseEntityInfo,
  pub min_value: f32,
  pub max_value: f32,
  pub step: f32,
  pub unit_of_measurement: String,
  pub mode: NumberMode,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberState {
  pub entity_state: BaseEntityState,
  pub state: f32,
  pub missing_state: bool,
}

// ==================== DATETIME DATE ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateInfo {
  pub entity_info: BaseEntityInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateState {
  pub entity_state: BaseEntityState,
  pub missing_state: bool,
  pub year: u32,
  pub month: u32,
  pub day: u32,
}

// ==================== DATETIME TIME ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeInfo {
  pub entity_info: BaseEntityInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeState {
  pub entity_state: BaseEntityState,
  pub missing_state: bool,
  pub hour: u32,
  pub minute: u32,
  pub second: u32,
}

// ==================== DATETIME DATETIME ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeInfo {
  pub entity_info: BaseEntityInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeState {
  pub entity_state: BaseEntityState,
  pub missing_state: bool,
  pub epoch_seconds: u32,
}

// ==================== SELECT ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectInfo {
  pub entity_info: BaseEntityInfo,
  pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectState {
  pub entity_state: BaseEntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== SIREN ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SirenInfo {
  pub entity_info: BaseEntityInfo,
  pub tones: Vec<String>,
  pub supports_volume: bool,
  pub supports_duration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SirenState {
  pub entity_state: BaseEntityState,
  pub state: bool,
}

// ==================== BUTTON ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
}

// ==================== LOCK ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockState {
  None = 0,
  Locked,
  Unlocked,
  Jammed,
  Locking,
  Unlocking,
}

impl From<proto::api::LockState> for LockState {
  fn from(value: proto::api::LockState) -> Self {
    match value {
      proto::api::LockState::LOCK_STATE_NONE => LockState::None,
      proto::api::LockState::LOCK_STATE_LOCKED => LockState::Locked,
      proto::api::LockState::LOCK_STATE_UNLOCKED => LockState::Unlocked,
      proto::api::LockState::LOCK_STATE_JAMMED => LockState::Jammed,
      proto::api::LockState::LOCK_STATE_LOCKING => LockState::Locking,
      proto::api::LockState::LOCK_STATE_UNLOCKING => LockState::Unlocking,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockCommand {
  Unlock = 0,
  Lock,
  Open,
}

impl From<proto::api::LockCommand> for LockCommand {
  fn from(value: proto::api::LockCommand) -> Self {
    match value {
      proto::api::LockCommand::LOCK_UNLOCK => LockCommand::Unlock,
      proto::api::LockCommand::LOCK_LOCK => LockCommand::Lock,
      proto::api::LockCommand::LOCK_OPEN => LockCommand::Open,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockInfo {
  pub entity_info: BaseEntityInfo,
  pub supports_open: bool,
  pub assumed_state: bool,

  pub requires_code: bool,
  pub code_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntityState {
  pub entity_state: BaseEntityState,
  pub state: LockState,
}

// ==================== VALVE ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValveInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
  pub assumed_state: bool,
  pub supports_stop: bool,
  pub supports_position: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValveOperation {
  Idle = 0,
  Opening,
  Closing,
}

impl From<proto::api::ValveOperation> for ValveOperation {
  fn from(value: proto::api::ValveOperation) -> Self {
    match value {
      proto::api::ValveOperation::VALVE_OPERATION_IDLE => ValveOperation::Idle,
      proto::api::ValveOperation::VALVE_OPERATION_IS_OPENING => ValveOperation::Opening,
      proto::api::ValveOperation::VALVE_OPERATION_IS_CLOSING => ValveOperation::Closing,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValveState {
  pub entity_state: BaseEntityState,
  pub position: f32,
  pub current_operation: ValveOperation,
}

// ==================== MEDIA PLAYER ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaPlayerState {
  None = 0,
  Idle,
  Playing,
  Paused,
  Announcing,
  Off,
  On,
}

impl From<proto::api::MediaPlayerState> for MediaPlayerState {
  fn from(value: proto::api::MediaPlayerState) -> Self {
    match value {
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_NONE => MediaPlayerState::None,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_IDLE => MediaPlayerState::Idle,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_PLAYING => MediaPlayerState::Playing,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_PAUSED => MediaPlayerState::Paused,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_ANNOUNCING => MediaPlayerState::Announcing,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_OFF => MediaPlayerState::Off,
      proto::api::MediaPlayerState::MEDIA_PLAYER_STATE_ON => MediaPlayerState::On,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaPlayerCommand {
  Play = 0,
  Pause,
  Stop,
  Mute,
  Unmute,
  Toggle,
  VolumeUp,
  VolumeDown,
  Enqueue,
  RepeatOne,
  RepeatOff,
  ClearPlaylist,
  TurnOn,
  TurnOff,
}

impl From<proto::api::MediaPlayerCommand> for MediaPlayerCommand {
  fn from(value: proto::api::MediaPlayerCommand) -> Self {
    match value {
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_PLAY => MediaPlayerCommand::Play,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_PAUSE => MediaPlayerCommand::Pause,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_STOP => MediaPlayerCommand::Stop,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_MUTE => MediaPlayerCommand::Mute,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_UNMUTE => MediaPlayerCommand::Unmute,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_TOGGLE => MediaPlayerCommand::Toggle,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_VOLUME_UP => {
        MediaPlayerCommand::VolumeUp
      }
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_VOLUME_DOWN => {
        MediaPlayerCommand::VolumeDown
      }
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_ENQUEUE => MediaPlayerCommand::Enqueue,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_REPEAT_ONE => {
        MediaPlayerCommand::RepeatOne
      }
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_REPEAT_OFF => {
        MediaPlayerCommand::RepeatOff
      }
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_CLEAR_PLAYLIST => {
        MediaPlayerCommand::ClearPlaylist
      }
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_TURN_ON => MediaPlayerCommand::TurnOn,
      proto::api::MediaPlayerCommand::MEDIA_PLAYER_COMMAND_TURN_OFF => MediaPlayerCommand::TurnOff,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaPlayerFormatPurpose {
  Default = 0,
  Announcement,
}

impl From<proto::api::MediaPlayerFormatPurpose> for MediaPlayerFormatPurpose {
  fn from(value: proto::api::MediaPlayerFormatPurpose) -> Self {
    match value {
      api::MediaPlayerFormatPurpose::MEDIA_PLAYER_FORMAT_PURPOSE_DEFAULT => {
        MediaPlayerFormatPurpose::Default
      }
      api::MediaPlayerFormatPurpose::MEDIA_PLAYER_FORMAT_PURPOSE_ANNOUNCEMENT => {
        MediaPlayerFormatPurpose::Announcement
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPlayerSupportedFormat {
  format: String,
  sample_rate: u32,
  num_channels: u32,
  purpose: MediaPlayerFormatPurpose,
  sample_bytes: u32,
}

impl From<proto::api::MediaPlayerSupportedFormat> for MediaPlayerSupportedFormat {
  fn from(value: proto::api::MediaPlayerSupportedFormat) -> Self {
    MediaPlayerSupportedFormat {
      format: value.format,
      sample_rate: value.sample_rate,
      num_channels: value.num_channels,
      purpose: value.purpose.enum_value_or_default().into(),
      sample_bytes: value.sample_bytes,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPlayerInfo {
  pub entity_info: BaseEntityInfo,
  pub supports_pause: bool,
  pub supported_formats: Vec<MediaPlayerSupportedFormat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaPlayerEntityState {
  pub entity_state: BaseEntityState,
  pub state: MediaPlayerState,
  pub volume: f32,
  pub muted: bool,
}

// ==================== ALARM CONTROL PANEL ====================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmControlPanelState {
  Disarmed = 0,
  ArmedHome,
  ArmedAway,
  ArmedNight,
  ArmedVacation,
  ArmedCustomBypass,
  Pending,
  Arming,
  Disarming,
  Triggered,
}

impl From<proto::api::AlarmControlPanelState> for AlarmControlPanelState {
  fn from(value: proto::api::AlarmControlPanelState) -> Self {
    match value {
      crate::api::AlarmControlPanelState::ALARM_STATE_DISARMED => AlarmControlPanelState::Disarmed,
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMED_HOME => {
        AlarmControlPanelState::ArmedHome
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMED_AWAY => {
        AlarmControlPanelState::ArmedAway
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMED_NIGHT => {
        AlarmControlPanelState::ArmedNight
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMED_VACATION => {
        AlarmControlPanelState::ArmedVacation
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMED_CUSTOM_BYPASS => {
        AlarmControlPanelState::ArmedCustomBypass
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_PENDING => AlarmControlPanelState::Pending,
      crate::api::AlarmControlPanelState::ALARM_STATE_ARMING => AlarmControlPanelState::Arming,
      crate::api::AlarmControlPanelState::ALARM_STATE_DISARMING => {
        AlarmControlPanelState::Disarming
      }
      crate::api::AlarmControlPanelState::ALARM_STATE_TRIGGERED => {
        AlarmControlPanelState::Triggered
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmControlPanelCommand {
  Disarm = 0,
  ArmHome,
  ArmAway,
  ArmNight,
  ArmVacation,
  ArmCustomBypass,
  Trigger,
}

impl From<proto::api::AlarmControlPanelStateCommand> for AlarmControlPanelCommand {
  fn from(value: proto::api::AlarmControlPanelStateCommand) -> Self {
    match value {
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_DISARM => {
        AlarmControlPanelCommand::Disarm
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_ARM_HOME => {
        AlarmControlPanelCommand::ArmHome
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_ARM_AWAY => {
        AlarmControlPanelCommand::ArmAway
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_ARM_NIGHT => {
        AlarmControlPanelCommand::ArmNight
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_ARM_VACATION => {
        AlarmControlPanelCommand::ArmVacation
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_ARM_CUSTOM_BYPASS => {
        AlarmControlPanelCommand::ArmCustomBypass
      }
      crate::api::AlarmControlPanelStateCommand::ALARM_CONTROL_PANEL_TRIGGER => {
        AlarmControlPanelCommand::Trigger
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlarmControlPanelInfo {
  pub entity_info: BaseEntityInfo,
  pub supported_features: u32,
  pub requires_code: bool,
  pub requires_code_to_arm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlarmControlPanelEntityState {
  pub entity_state: BaseEntityState,
  pub state: AlarmControlPanelState,
}

// ==================== TEXT ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextMode {
  Text = 0,
  Password,
}

impl From<proto::api::TextMode> for TextMode {
  fn from(value: proto::api::TextMode) -> Self {
    match value {
      proto::api::TextMode::TEXT_MODE_TEXT => TextMode::Text,
      proto::api::TextMode::TEXT_MODE_PASSWORD => TextMode::Password,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInfo {
  pub entity_info: BaseEntityInfo,
  pub min_length: u32,
  pub max_length: u32,
  pub pattern: String,
  pub mode: TextMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
  pub entity_state: BaseEntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== UPDATE ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCommand {
  None = 0,
  Install,
  Check,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
  pub entity_info: BaseEntityInfo,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateState {
  pub entity_state: BaseEntityState,
  pub missing_state: bool,
  pub in_progress: bool,
  pub has_progress: bool,
  pub progress: f32,
  pub current_version: String,
  pub latest_version: String,
  pub title: String,
  pub release_summary: String,
  pub release_url: String,
}

// ==================== USER-DEFINED SERVICES ====================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeassistantActionRequest {
  pub service: String,
  pub is_event: bool,
  pub data: HashMap<String, String>,
  pub data_template: HashMap<String, String>,
  pub variables: HashMap<String, String>,
  pub call_id: u32,
  pub wants_response: bool,
  pub response_template: String,
}

impl From<proto::api::HomeassistantActionRequest> for HomeassistantActionRequest {
  fn from(value: proto::api::HomeassistantActionRequest) -> Self {
    let data = value
      .data
      .into_iter()
      .map(|item| (item.key, item.value))
      .collect();
    let data_template = value
      .data_template
      .into_iter()
      .map(|item| (item.key, item.value))
      .collect();
    let variables = value
      .variables
      .into_iter()
      .map(|item| (item.key, item.value))
      .collect();

    Self {
      service: value.service,
      is_event: value.is_event,
      data,
      data_template,
      variables,
      call_id: value.call_id,
      wants_response: value.wants_response,
      response_template: value.response_template,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserServiceArgType {
  Bool = 0,
  Int,
  Float,
  String,
  BoolArray,
  IntArray,
  FloatArray,
  StringArray,
}

impl From<proto::api::ServiceArgType> for UserServiceArgType {
  fn from(value: proto::api::ServiceArgType) -> Self {
    match value {
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_BOOL => UserServiceArgType::Bool,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_INT => UserServiceArgType::Int,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_FLOAT => UserServiceArgType::Float,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_STRING => UserServiceArgType::String,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_BOOL_ARRAY => UserServiceArgType::BoolArray,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_INT_ARRAY => UserServiceArgType::IntArray,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_FLOAT_ARRAY => UserServiceArgType::FloatArray,
      proto::api::ServiceArgType::SERVICE_ARG_TYPE_STRING_ARRAY => UserServiceArgType::StringArray,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserServiceArg {
  pub name: String,
  pub arg_type: UserServiceArgType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserService {
  pub name: String,
  pub key: u32,
  pub args: Vec<UserServiceArg>,
}

// ==================== BLUETOOTH ====================

pub fn uuid_convert(uuid: String) -> String {
  let mut uuid = uuid.to_lowercase();
  if uuid.len() < 8 {
    uuid = format!("0000{}-0000-1000-8000-00805f9b34fb", uuid[2..].to_string());
  }
  return uuid;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothLEAdvertisement {
  pub address: u64,
  pub rssi: i32,
  pub address_type: u32,
  pub name: String,
  pub service_uuids: Vec<String>,
  pub service_data: HashMap<String, Vec<u8>>,
  pub manufacturer_data: HashMap<u16, Vec<u8>>,
}

impl BluetoothLEAdvertisement {
  pub fn from_pb(data: proto::api::BluetoothLEAdvertisementResponse) -> Self {
    let mut manufacturer_data: HashMap<u16, Vec<u8>> = HashMap::new();
    let mut service_data: HashMap<String, Vec<u8>> = HashMap::new();
    let mut service_uuids: Vec<String> = Vec::new();

    let raw_manufacturer_data = data.manufacturer_data;
    if !raw_manufacturer_data.is_empty() {
      if !raw_manufacturer_data[0].data.is_empty() {
        raw_manufacturer_data.iter().for_each(|item| {
          manufacturer_data.insert(item.uuid.parse().unwrap(), item.data.clone());
        });
      } else {
        // legacy data
        raw_manufacturer_data.iter().for_each(|item| {
          manufacturer_data.insert(
            item.uuid.parse().unwrap(),
            item
              .legacy_data
              .iter()
              .flat_map(|&num| num.to_le_bytes().to_vec())
              .collect(),
          );
        });
      }
    }

    let raw_service_data = data.service_data;
    if !raw_service_data.is_empty() {
      if !raw_service_data[0].data.is_empty() {
        raw_service_data.iter().for_each(|item| {
          service_data.insert(uuid_convert(item.uuid.clone()), item.data.clone());
        });
      } else {
        // legacy data
        raw_service_data.iter().for_each(|item| {
          service_data.insert(
            uuid_convert(item.uuid.clone()),
            item
              .legacy_data
              .iter()
              .flat_map(|&num| num.to_le_bytes().to_vec())
              .collect(),
          );
        });
      }
    }

    let raw_service_uuids = data.service_uuids;
    if !raw_service_uuids.is_empty() {
      service_uuids.extend(
        raw_service_uuids
          .iter()
          .map(|uuid| uuid_convert(uuid.clone())),
      );
    }

    Self {
      address: data.address,
      rssi: data.rssi,
      address_type: data.address_type,
      name: String::from_utf8_lossy(&data.name).into(),
      service_uuids,
      service_data,
      manufacturer_data,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDeviceConnection {
  pub address: u64,
  pub connected: bool,
  pub mtu: u16,
  pub error: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDevicePairing {
  pub address: u64,
  pub paired: bool,
  pub error: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDeviceUnpairing {
  pub address: u64,
  pub success: bool,
  pub error: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDeviceClearCache {
  pub address: u64,
  pub success: bool,
  pub error: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTRead {
  pub address: u64,
  pub handle: u16,
  pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTDescriptor {
  pub uuid: String,
  pub handle: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTCharacteristic {
  pub uuid: String,
  pub handle: u16,
  pub properties: u8,
  pub descriptors: Vec<BluetoothGATTDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTService {
  pub uuid: String,
  pub handle: u16,
  pub characteristics: Vec<BluetoothGATTCharacteristic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTServices {
  pub address: u64,
  pub services: Vec<BluetoothGATTService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ESPHomeBluetoothGATTServices {
  pub address: u64,
  pub services: Vec<BluetoothGATTService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothConnectionsFree {
  pub free: u8,
  pub limit: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothGATTError {
  pub address: u64,
  pub handle: u16,
  pub error: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothDeviceRequestType {
  Connect = 0,
  Disconnect,
  Pair,
  Unpair,
  ConnectV3WithCache,
  ConnectV3WithoutCache,
  ClearCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceAssistantCommandFlag {
  UseVAD = 1 << 0,
  UseWakeWord = 1 << 1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceAssistantAudioSettings {
  pub noise_suppression_level: u8,
  pub auto_gain: u8,
  pub volume_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceAssistantCommand {
  pub start: bool,
  pub conversation_id: String,
  pub flags: u8,
  pub audio_settings: Vec<VoiceAssistantAudioSettings>,
  pub wake_word_phrase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantAudioData {
  pub data: Vec<u8>,
  pub end: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantAnnounceFinished {
  pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantWakeWord {
  pub id: String,
  pub wake_word: String,
  pub trained_languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantConfigurationResponse {
  pub available_wake_words: Vec<VoiceAssistantWakeWord>,
  pub active_wake_words: Vec<String>,
  pub max_active_wake_words: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantConfigurationRequest {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAssistantSetConfiguration {
  pub active_wake_words: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
  None = 0,
  Error,
  Warn,
  Info,
  Config,
  Debug,
  Verbose,
  VeryVerbose,
}

impl From<proto::api::LogLevel> for LogLevel {
  fn from(value: proto::api::LogLevel) -> Self {
    match value {
      proto::api::LogLevel::LOG_LEVEL_NONE => LogLevel::None,
      proto::api::LogLevel::LOG_LEVEL_ERROR => LogLevel::Error,
      proto::api::LogLevel::LOG_LEVEL_WARN => LogLevel::Warn,
      proto::api::LogLevel::LOG_LEVEL_INFO => LogLevel::Info,
      proto::api::LogLevel::LOG_LEVEL_CONFIG => LogLevel::Config,
      proto::api::LogLevel::LOG_LEVEL_DEBUG => LogLevel::Debug,
      proto::api::LogLevel::LOG_LEVEL_VERBOSE => LogLevel::Verbose,
      proto::api::LogLevel::LOG_LEVEL_VERY_VERBOSE => LogLevel::VeryVerbose,
    }
  }
}

// ==================== LOG EVENT ====================

/// Event received when subscribed to device logs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEvent {
  pub level: LogLevel,
  pub message: Vec<u8>,
}

impl From<proto::api::SubscribeLogsResponse> for LogEvent {
  fn from(value: proto::api::SubscribeLogsResponse) -> Self {
    LogEvent {
      level: value.level.enum_value_or_default().into(),
      message: value.message,
    }
  }
}

// ==================== CAMERA ====================

/// A camera image frame received from the device
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraImage {
  pub key: u32,
  pub data: Vec<u8>,
  /// Whether this is the last frame in the current image
  pub done: bool,
}

impl From<proto::api::CameraImageResponse> for CameraImage {
  fn from(value: proto::api::CameraImageResponse) -> Self {
    CameraImage {
      key: value.key,
      data: value.data,
      done: value.done,
    }
  }
}

// ==================== HOME ASSISTANT EVENTS ====================

/// Event received when subscribed to Home Assistant states.
/// The ESPHome device sends these to tell the client which HA entities it wants to monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeAssistantEvent {
  /// Device wants to subscribe to ongoing state changes for this entity
  StateSubscription {
    entity_id: String,
    attribute: Option<String>,
  },
  /// Device wants the current state once (one-time request)
  StateRequest {
    entity_id: String,
    attribute: Option<String>,
  },
}

impl From<proto::api::SubscribeHomeAssistantStateResponse> for HomeAssistantEvent {
  fn from(value: proto::api::SubscribeHomeAssistantStateResponse) -> Self {
    let attribute = if value.attribute.is_empty() {
      None
    } else {
      Some(value.attribute)
    };

    if value.once {
      HomeAssistantEvent::StateRequest {
        entity_id: value.entity_id,
        attribute,
      }
    } else {
      HomeAssistantEvent::StateSubscription {
        entity_id: value.entity_id,
        attribute,
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceAssistantEventType {
  Error = 0,
  RunStart,
  RunEnd,
  STTStart,
  STTEnd,
  IntentStart,
  IntentEnd,
  TTSStart,
  TTSEnd,
  WakeWordStart,
  WakeWordEnd,
  SSTVADStart,
  SSTVADEnd,
  TTSStreamStart = 98,
  TTSStreamEnd = 99,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceAssistantTimerEventType {
  TimerStarted = 0,
  TimerUpdated,
  TimerCancelled,
  TimerFinished,
}
