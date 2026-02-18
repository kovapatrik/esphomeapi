use protobuf::Message as _;

use super::services;
use crate::{api, Result};

#[derive(Debug)]
pub enum EntityInfo {
  AlarmControlPanel(services::AlarmControlPanelInfo),
  BinarySensor(services::BinarySensorInfo),
  Button(services::ButtonInfo),
  Camera(services::CameraInfo),
  Climate(services::ClimateInfo),
  Cover(services::CoverInfo),
  Date(services::DateInfo),
  DateTime(services::DateTimeInfo),
  Event(services::EventInfo),
  Fan(services::FanInfo),
  Light(services::LightInfo),
  Lock(services::LockInfo),
  MediaPlayer(services::MediaPlayerInfo),
  Number(services::NumberInfo),
  Select(services::SelectInfo),
  Sensor(services::SensorInfo),
  Switch(services::SwitchInfo),
  Text(services::TextInfo),
  TextSensor(services::TextSensorInfo),
  Time(services::TimeInfo),
  Update(services::UpdateInfo),
  Valve(services::ValveInfo),
}

impl EntityInfo {
  pub fn key(&self) -> u32 {
    match self {
      EntityInfo::AlarmControlPanel(info) => info.entity_info.key,
      EntityInfo::BinarySensor(info) => info.entity_info.key,
      EntityInfo::Button(info) => info.entity_info.key,
      EntityInfo::Camera(info) => info.entity_info.key,
      EntityInfo::Climate(info) => info.entity_info.key,
      EntityInfo::Cover(info) => info.entity_info.key,
      EntityInfo::Date(info) => info.entity_info.key,
      EntityInfo::DateTime(info) => info.entity_info.key,
      EntityInfo::Event(info) => info.entity_info.key,
      EntityInfo::Fan(info) => info.entity_info.key,
      EntityInfo::Light(info) => info.entity_info.key,
      EntityInfo::Lock(info) => info.entity_info.key,
      EntityInfo::MediaPlayer(info) => info.entity_info.key,
      EntityInfo::Number(info) => info.entity_info.key,
      EntityInfo::Select(info) => info.entity_info.key,
      EntityInfo::Sensor(info) => info.entity_info.key,
      EntityInfo::Switch(info) => info.entity_info.key,
      EntityInfo::Text(info) => info.entity_info.key,
      EntityInfo::TextSensor(info) => info.entity_info.key,
      EntityInfo::Time(info) => info.entity_info.key,
      EntityInfo::Update(info) => info.entity_info.key,
      EntityInfo::Valve(info) => info.entity_info.key,
    }
  }

  pub fn parse_alarm_control_panel(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesAlarmControlPanelResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::AlarmControlPanel(
      services::AlarmControlPanelInfo {
        entity_info,
        supported_features: data.supported_features,
        requires_code: data.requires_code,
        requires_code_to_arm: data.requires_code_to_arm,
      },
    ))
  }

  pub fn parse_binary_sensor(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesBinarySensorResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::BinarySensor(services::BinarySensorInfo {
      entity_info,
      device_class: data.device_class,
      is_status_binary_sensor: data.is_status_binary_sensor,
    }))
  }

  pub fn parse_button(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesButtonResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Button(services::ButtonInfo {
      entity_info,
      device_class: data.device_class,
    }))
  }

  pub fn parse_camera(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesCameraResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Camera(services::CameraInfo { entity_info }))
  }

  pub fn parse_climate(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesClimateResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Climate(services::ClimateInfo {
      entity_info,
      legacy_supports_away: data.legacy_supports_away,
      supported_custom_fan_modes: data.supported_custom_fan_modes,
      supported_custom_presets: data.supported_custom_presets,
      supported_fan_modes: data
        .supported_fan_modes
        .iter()
        .map(|v| v.enum_value_or_default().into())
        .collect(),
      supported_modes: data
        .supported_modes
        .iter()
        .map(|v| v.enum_value_or_default().into())
        .collect(),
      supported_presets: data
        .supported_presets
        .iter()
        .map(|v| v.enum_value_or_default().into())
        .collect(),
      supported_swing_modes: data
        .supported_swing_modes
        .iter()
        .map(|v| v.enum_value_or_default().into())
        .collect(),
      supports_action: data.supports_action,
      supports_current_humidity: data.supports_current_humidity,
      supports_current_temperature: data.supports_current_temperature,
      supports_target_humidity: data.supports_target_humidity,
      supports_two_point_target_temperature: data.supports_two_point_target_temperature,
      visual_current_temperature_step: data.visual_current_temperature_step,
      visual_max_humidity: data.visual_max_humidity,
      visual_max_temperature: data.visual_max_temperature,
      visual_min_humidity: data.visual_min_humidity,
      visual_min_temperature: data.visual_min_temperature,
      visual_target_temperature_step: data.visual_target_temperature_step,
    }))
  }

  pub fn parse_cover(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesCoverResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Cover(services::CoverInfo {
      entity_info,
      assumed_state: data.assumed_state,
      device_class: data.device_class,
      supports_position: data.supports_position,
      supports_stop: data.supports_stop,
      supports_tilt: data.supports_tilt,
    }))
  }

  pub fn parse_date(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesDateResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Date(services::DateInfo { entity_info }))
  }

  pub fn parse_date_time(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesDateTimeResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::DateTime(services::DateTimeInfo { entity_info }))
  }

  pub fn parse_event(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesEventResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Event(services::EventInfo {
      entity_info,
      device_class: data.device_class,
      event_types: data.event_types,
    }))
  }

  pub fn parse_fan(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesFanResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Fan(services::FanInfo {
      entity_info,
      supported_preset_modes: data.supported_preset_modes,
      supported_speed_count: data.supported_speed_count,
      supports_oscillation: data.supports_oscillation,
      supports_speed: data.supports_speed,
      supports_direction: data.supports_direction,
    }))
  }

  pub fn parse_light(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesLightResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Light(services::LightInfo {
      entity_info,
      effects: data.effects,
      legacy_supports_brightness: data.legacy_supports_brightness,
      legacy_supports_color_temperature: data.legacy_supports_color_temperature,
      legacy_supports_rgb: data.legacy_supports_rgb,
      legacy_supports_white_value: data.legacy_supports_white_value,
      max_mireds: data.max_mireds,
      min_mireds: data.min_mireds,
      supported_color_modes: data
        .supported_color_modes
        .iter()
        .map(|v| v.enum_value_or_default().into())
        .collect(),
    }))
  }

  pub fn parse_lock(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesLockResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Lock(services::LockInfo {
      entity_info,
      assumed_state: data.assumed_state,
      code_format: data.code_format,
      requires_code: data.requires_code,
      supports_open: data.supports_open,
    }))
  }

  pub fn parse_media_player(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesMediaPlayerResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::MediaPlayer(services::MediaPlayerInfo {
      entity_info,
      supports_pause: data.supports_pause,
      supported_formats: data
        .supported_formats
        .iter()
        .map(|v| v.clone().into())
        .collect(),
    }))
  }

  pub fn parse_number(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesNumberResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Number(services::NumberInfo {
      entity_info,
      device_class: data.device_class,
      max_value: data.max_value,
      min_value: data.min_value,
      mode: data.mode.enum_value_or_default().into(),
      step: data.step,
      unit_of_measurement: data.unit_of_measurement,
    }))
  }

  pub fn parse_select(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesSelectResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Select(services::SelectInfo {
      entity_info,
      options: data.options,
    }))
  }

  pub fn parse_sensor(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesSensorResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Sensor(services::SensorInfo {
      entity_info,
      accuracy_decimals: data.accuracy_decimals,
      device_class: data.device_class,
      force_update: data.force_update,
      legacy_last_reset_type: data.legacy_last_reset_type.enum_value_or_default().into(),
      state_class: data.state_class.enum_value_or_default().into(),
      unit_of_measurement: data.unit_of_measurement,
    }))
  }

  pub fn parse_switch(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesSwitchResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Switch(services::SwitchInfo {
      entity_info,
      assumed_state: data.assumed_state,
      device_class: data.device_class,
    }))
  }

  pub fn parse_text(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesTextResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Text(services::TextInfo {
      entity_info,
      max_length: data.max_length,
      min_length: data.min_length,
      mode: data.mode.enum_value_or_default().into(),
      pattern: data.pattern,
    }))
  }

  pub fn parse_text_sensor(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesTextSensorResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::TextSensor(services::TextSensorInfo {
      entity_info,
      device_class: data.device_class,
    }))
  }

  pub fn parse_time(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesTimeResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Time(services::TimeInfo { entity_info }))
  }

  pub fn parse_update(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesUpdateResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Update(services::UpdateInfo {
      entity_info,
      device_class: data.device_class.clone(),
    }))
  }

  pub fn parse_valve(data: &[u8]) -> Result<Self> {
    let data = api::ListEntitiesValveResponse::parse_from_bytes(data)?;

    let entity_info = services::BaseEntityInfo {
      disabled_by_default: data.disabled_by_default,
      entity_category: data.entity_category.enum_value_or_default().into(),
      object_id: data.object_id,
      key: data.key,
      name: data.name,
      icon: data.icon,
    };

    Ok(EntityInfo::Valve(services::ValveInfo {
      entity_info,
      assumed_state: data.assumed_state,
      device_class: data.device_class,
      supports_position: data.supports_position,
      supports_stop: data.supports_stop,
    }))
  }
}

pub fn parse_user_service(data: &[u8]) -> Result<services::UserService> {
  let data = api::ListEntitiesServicesResponse::parse_from_bytes(data)?;

  let args = data
    .args
    .iter()
    .map(|arg| services::UserServiceArg {
      name: arg.name.clone(),
      arg_type: arg.type_.enum_value_or_default().into(),
    })
    .collect();

  Ok(services::UserService {
    key: data.key,
    name: data.name,
    args,
  })
}
