use std::collections::HashMap;

use enumflags2::bitflags;

use crate::proto;

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

pub enum BluetoothProxyFeature {
  PassiveScan = 1 << 0,
  ActiveConnections = 1 << 1,
  RemoteCaching = 1 << 2,
  Pairing = 1 << 3,
  CacheClearing = 1 << 4,
  RawAdvertisements = 1 << 5,
}

pub enum BluetoothProxySubscriptionFlag {
  RawAdvertisements = 1 << 0,
}

pub enum VoiceAssistantFeature {
  VoiceAssistant = 1 << 0,
  Speaker = 1 << 1,
  APIAudio = 1 << 2,
  Timers = 1 << 3,
  Announce = 1 << 4,
}

pub enum VoiceAssistantSubscriptionFlag {
  APIAudio = 1 << 0,
}

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
  pub webserver_port: u16,
  pub legacy_voice_assistant_version: u8,
  pub voice_assistant_feature_flags: u8,
  pub legacy_bluetooth_proxy_version: u8,
  pub bluetooth_proxy_feature_flags: u8,
  pub suggested_area: String,
}

impl DeviceInfo {
  fn new() -> DeviceInfo {
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

  pub fn bluetooth_proxy_feature_flags_compat(&self, api_version: APIVersion) -> u8 {
    if api_version < APIVersion::new(1, 9) {
      let mut flags = 0;
      if self.legacy_bluetooth_proxy_version >= 1 {
        flags |= BluetoothProxyFeature::PassiveScan as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 2 {
        flags |= BluetoothProxyFeature::ActiveConnections as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 3 {
        flags |= BluetoothProxyFeature::RemoteCaching as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 4 {
        flags |= BluetoothProxyFeature::Pairing as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 5 {
        flags |= BluetoothProxyFeature::CacheClearing as u8;
      }
      return flags;
    }
    return self.bluetooth_proxy_feature_flags;
  }

  pub fn voice_assistant_feature_flags_compat(&self, api_version: APIVersion) -> u8 {
    if api_version < APIVersion::new(1, 10) {
      let mut flags = 0;
      if self.legacy_voice_assistant_version >= 1 {
        flags |= VoiceAssistantFeature::VoiceAssistant as u8;
      }
      if self.legacy_voice_assistant_version >= 2 {
        flags |= VoiceAssistantFeature::Speaker as u8;
      }
      return flags;
    }
    return self.voice_assistant_feature_flags;
  }
}

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

pub struct EntityInfo {
  pub object_id: String,
  pub key: u32,
  pub name: String,
  pub unique_id: String,
  pub disabled_by_default: bool,
  pub icon: String,
  pub enitity_category: EntityCategory,
}

pub struct EntityState {
  pub key: u8,
}

// ==================== BINARY SENSOR ====================
pub struct BinarySensorInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
  pub is_status_binary_sensor: bool,
}

pub struct BinarySensorState {
  pub entity_state: EntityState,
  pub state: bool,
  pub missing_state: bool,
}

// ==================== COVER ====================

pub struct CoverInfo {
  pub entity_info: EntityInfo,
  pub assumed_state: bool,
  pub supports_stop: bool,
  pub supports_position: bool,
  pub supports_tilt: bool,
  pub device_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum LegacyCoverState {
  Open = 0,
  Closed,
}

pub enum LegacyCoverCommand {
  Open = 0,
  Close,
  Stop,
}

pub enum CoverOperation {
  Idle = 0,
  Opening,
  Closing,
}

pub struct CoverState {
  pub entity_state: EntityState,
  pub legacy_state: Option<LegacyCoverState>,
  pub position: f32,
  pub tilt: f32,
  pub current_operation: Option<CoverOperation>,
}

impl CoverState {
  pub fn is_closed(&self, api_version: APIVersion) -> bool {
    if api_version < APIVersion::new(1, 1) {
      if let Some(legacy_state) = self.legacy_state {
        return legacy_state == LegacyCoverState::Closed;
      }
    }
    return self.position == 0.0;
  }
}

// ==================== EVENT ====================
pub struct EventInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
  pub event_types: Vec<String>,
}

pub struct Event {
  pub entity_state: EntityState,
  pub event_type: String,
}

// ==================== FAN ====================
pub struct FanInfo {
  pub entity_info: EntityInfo,
  pub supports_oscillation: bool,
  pub supports_speed: bool,
  pub supports_direction: bool,
  pub supported_speed_count: i32,
  pub supported_preset_modes: Vec<String>,
}

pub enum FanSpeed {
  Low = 0,
  Medium,
  High,
}

pub enum FanDirection {
  Forward = 0,
  Reverse,
}

pub struct FanState {
  pub entity_state: EntityState,
  pub oscillating: bool,
  pub speed: Option<FanSpeed>,
  pub speed_level: u8,
  pub direction: Option<FanDirection>,
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ColorMode {
  Unknown = 0,
  OnOff = 1,
  Brightness = 2,
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

impl Into<u8> for ColorMode {
  fn into(self) -> u8 {
    self as u8
  }
}

pub struct LightInfo {
  pub entity_info: EntityInfo,
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

pub struct LightState {
  pub entity_state: EntityState,
  pub state: bool,
  pub brightness: f32,
  pub color_mode: u8,
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

pub enum SensorStateClass {
  None = 0,
  Measurement,
  TotalIncreasing,
  Total,
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
    }
  }
}

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

pub struct SensorInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
  pub unit_of_measurement: String,
  pub accuracy_decimals: i32,
  pub force_update: bool,
  pub state_class: SensorStateClass,
  pub legacy_last_reset_type: LastResetType,
}

pub struct SensorState {
  pub entity_state: EntityState,
  pub state: f32,
  pub missing_state: bool,
}

// ==================== SWITCH ====================
pub struct SwitchInfo {
  pub entity_info: EntityInfo,
  pub assumed_state: bool,
  pub device_class: String,
}

pub struct SwitchState {
  pub entity_state: EntityState,
  pub state: bool,
}

// ==================== TEXT SENSOR ====================
pub struct TextSensorInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
}

pub struct TextSensorState {
  pub entity_state: EntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== CAMERA ====================
pub struct CameraInfo {
  pub entity_info: EntityInfo,
}

pub struct CameraState {
  pub entity_state: EntityState,
  pub data: Vec<u8>,
}

// ==================== CLIMATE ====================

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

pub struct ClimateInfo {
  pub entity_info: EntityInfo,
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

pub struct ClimateState {
  pub entity_state: EntityState,
  pub mode: Option<ClimateMode>,
  pub action: Option<ClimateAction>,
  pub current_temperature: f32,
  pub target_temperature: f32,
  pub target_temperature_low: f32,
  pub target_temperature_high: f32,
  pub legacy_away: bool,
  pub fan_mode: Option<ClimateFanMode>,
  pub swing_mode: Option<ClimateSwingMode>,
  pub custom_fan_mode: String,
  pub preset: Option<ClimatePreset>,
  pub custom_preset: String,
  pub current_humidity: f32,
  pub target_humidity: f32,
}

impl ClimateState {
  pub fn preset_compat(&self, api_version: APIVersion) -> Option<ClimatePreset> {
    if api_version < APIVersion::new(1, 5) {
      if self.legacy_away {
        return Some(ClimatePreset::Away);
      }
      return Some(ClimatePreset::Home);
    }
    return self.preset;
  }
}

// ==================== NUMBER ====================

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

pub struct NumberInfo {
  pub entity_info: EntityInfo,
  pub min_value: f32,
  pub max_value: f32,
  pub step: f32,
  pub unit_of_measurement: String,
  pub mode: NumberMode,
  pub device_class: String,
}

pub struct NumberState {
  pub entity_state: EntityState,
  pub state: f32,
  pub missing_state: bool,
}

// ==================== DATETIME DATE ====================

pub struct DateInfo {
  pub entity_info: EntityInfo,
}

pub struct DateState {
  pub entity_state: EntityState,
  pub missing_state: bool,
  pub year: u16,
  pub month: u8,
  pub day: u8,
}

// ==================== DATETIME TIME ====================
pub struct TimeInfo {
  pub entity_info: EntityInfo,
}

pub struct TimeState {
  pub entity_state: EntityState,
  pub missing_state: bool,
  pub hour: u8,
  pub minute: u8,
  pub second: u8,
}

// ==================== DATETIME DATETIME ====================

pub struct DateTimeInfo {
  pub entity_info: EntityInfo,
}

pub struct DateTimeState {
  pub entity_state: EntityState,
  pub missing_state: bool,
  pub epoch_seconds: u32,
}

// ==================== SELECT ====================

pub struct SelectInfo {
  pub entity_info: EntityInfo,
  pub options: Vec<String>,
}

pub struct SelectState {
  pub entity_state: EntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== SIREN ====================

pub struct SirenInfo {
  pub entity_info: EntityInfo,
  pub tones: Vec<String>,
  pub supports_volume: bool,
  pub supports_duration: bool,
}

pub struct SirenState {
  pub entity_state: EntityState,
  pub state: bool,
}

// ==================== BUTTON ====================

pub struct ButtonInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
}

// ==================== LOCK ====================

pub enum LockState {
  None = 0,
  Locked,
  Unlocked,
  Jammed,
  Locking,
  Unlocking,
}

pub enum LockCommand {
  Unlock = 0,
  Lock,
  Open,
}

pub struct LockInfo {
  pub entity_info: EntityInfo,
  pub supports_open: bool,
  pub assumed_state: bool,

  pub requires_code: bool,
  pub code_format: String,
}

pub struct LockEntityState {
  pub entity_state: EntityState,
  pub state: Option<LockState>,
}

// ==================== VALVE ====================

pub struct ValveInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
  pub assumed_state: bool,
  pub supports_stop: bool,
  pub supports_position: bool,
}

pub enum ValveOperation {
  Idle = 0,
  Opening,
  Closing,
}

pub struct ValveState {
  pub entity_state: EntityState,
  pub position: f32,
  pub current_operation: Option<ValveOperation>,
}

// ==================== MEDIA PLAYER ====================

pub enum MediaPlayerState {
  None = 0,
  Idle,
  Playing,
  Paused,
}

pub enum MediaPlayerCommand {
  Play = 0,
  Pause,
  Stop,
  Mute,
  Unmute,
}

pub enum MediaPlayerFormatPurpose {
  Default = 0,
  Announcement,
}

// struct MediaPlayerSupportedFormat {
//   format: String,
//   sample_rate: u32,
//   num_channels: u8,
//   purpose: Option<MediaPlayerFormatPurpose>,
//   sample_bytes: u8,
// }

pub struct MediaPlayerInfo {
  pub entity_info: EntityInfo,
  pub supports_pause: bool,
}

pub struct MediaPlayerEntityState {
  pub entity_state: EntityState,
  pub state: Option<MediaPlayerState>,
  pub volume: f32,
  pub muted: bool,
}

// ==================== ALARM CONTROL PANEL ====================

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

pub enum AlarmControlPanelCommand {
  Disarm = 0,
  ArmHome,
  ArmAway,
  ArmNight,
  ArmVacation,
  ArmCustomBypass,
  Trigger,
}

pub struct AlarmControlPanelInfo {
  pub entity_info: EntityInfo,
  pub supported_features: u32,
  pub requires_code: bool,
  pub requires_code_to_arm: bool,
}

pub struct AlarmControlPanelEntityState {
  pub entity_state: EntityState,
  pub state: Option<AlarmControlPanelState>,
}

// ==================== TEXT ====================
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

pub struct TextInfo {
  pub entity_info: EntityInfo,
  pub min_length: u32,
  pub max_length: u32,
  pub pattern: String,
  pub mode: TextMode,
}

pub struct TextState {
  pub entity_state: EntityState,
  pub state: String,
  pub missing_state: bool,
}

// ==================== UPDATE ====================
pub enum UpdateCommand {
  None = 0,
  Install,
  Check,
}

pub struct UpdateInfo {
  pub entity_info: EntityInfo,
  pub device_class: String,
}

pub struct UpdateState {
  pub entity_state: EntityState,
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
pub struct HomeassistantServiceCall {
  pub service: String,
  pub is_event: bool,
  pub data: HashMap<String, String>,
  pub data_template: HashMap<String, String>,
  pub variables: HashMap<String, String>,
}

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

pub struct UserServiceArg {
  pub name: String,
  pub arg_type: UserServiceArgType,
}

pub struct UserService {
  pub name: String,
  pub key: u8,
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
      name: data.name, // TODO: check if correct, UTF-8 conversion might be needed
      service_uuids,
      service_data,
      manufacturer_data,
    }
  }
}

pub struct BluetoothDeviceConnection {
  pub address: u64,
  pub connected: bool,
  pub mtu: u16,
  pub error: u8,
}

pub struct BluetoothDevicePairing {
  pub address: u64,
  pub paired: bool,
  pub error: u8,
}

pub struct BluetoothDeviceUnpairing {
  pub address: u64,
  pub success: bool,
  pub error: u8,
}

pub struct BluetoothDeviceClearCache {
  pub address: u64,
  pub success: bool,
  pub error: u8,
}

pub struct BluetoothGATTRead {
  pub address: u64,
  pub handle: u16,
  pub data: Vec<u8>,
}

pub struct BluetoothGATTDescriptor {
  pub uuid: String,
  pub handle: u16,
}

pub struct BluetoothGATTCharacteristic {
  pub uuid: String,
  pub handle: u16,
  pub properties: u8,
  pub descriptors: Vec<BluetoothGATTDescriptor>,
}

pub struct BluetoothGATTService {
  pub uuid: String,
  pub handle: u16,
  pub characteristics: Vec<BluetoothGATTCharacteristic>,
}

pub struct BluetoothGATTServices {
  pub address: u64,
  pub services: Vec<BluetoothGATTService>,
}

pub struct ESPHomeBluetoothGATTServices {
  pub address: u64,
  pub services: Vec<BluetoothGATTService>,
}

pub struct BluetoothConnectionsFree {
  pub free: u8,
  pub limit: u8,
}

pub struct BluetoothGATTError {
  pub address: u64,
  pub handle: u16,
  pub error: u8,
}

pub enum BluetoothDeviceRequestType {
  Connect = 0,
  Disconnect,
  Pair,
  Unpair,
  ConnectV3WithCache,
  ConnectV3WithoutCache,
  ClearCache,
}

pub enum VoiceAssistantCommandFlag {
  UseVAD = 1 << 0,
  UseWakeWord = 1 << 1,
}

pub struct VoiceAssistantAudioSettings {
  pub noise_suppression_level: u8,
  pub auto_gain: u8,
  pub volume_multiplier: f32,
}

pub struct VoiceAssistantCommand {
  pub start: bool,
  pub conversation_id: String,
  pub flags: u8,
  pub audio_settings: Vec<VoiceAssistantAudioSettings>,
  pub wake_word_phrase: String,
}

pub struct VoiceAssistantAudioData {
  pub data: Vec<u8>,
  pub end: bool,
}

pub struct VoiceAssistantAnnounceFinished {
  pub success: bool,
}

pub struct VoiceAssistantWakeWord {
  pub id: String,
  pub wake_word: String,
  pub trained_languages: Vec<String>,
}

pub struct VoiceAssistantConfigurationResponse {
  pub available_wake_words: Vec<VoiceAssistantWakeWord>,
  pub active_wake_words: Vec<String>,
  pub max_active_wake_words: u8,
}

pub struct VoiceAssistantConfigurationRequest {}

pub struct VoiceAssistantSetConfiguration {
  pub active_wake_words: Vec<u8>,
}

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

pub enum VoiceAssistantTimerEventType {
  TimerStarted = 0,
  TimerUpdated,
  TimerCancelled,
  TimerFinished,
}