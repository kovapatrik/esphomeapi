use protobuf::Message as _;

use super::services;
use crate::{api, Result};

#[derive(Debug, Clone)]
pub enum EntityState {
  AlarmControlPanel(services::AlarmControlPanelEntityState),
  BinarySensor(services::BinarySensorState),
  Climate(services::ClimateState),
  Cover(services::CoverState),
  Date(services::DateState),
  DateTime(services::DateTimeState),
  Event(services::Event),
  Fan(services::FanState),
  Light(services::LightState),
  Lock(services::LockEntityState),
  MediaPlayer(services::MediaPlayerEntityState),
  Number(services::NumberState),
  Select(services::SelectState),
  Sensor(services::SensorState),
  Switch(services::SwitchState),
  Text(services::TextState),
  TextSensor(services::TextSensorState),
  Time(services::TimeState),
  Update(services::UpdateState),
  Valve(services::ValveState),
}

impl EntityState {
  pub fn key(&self) -> u32 {
    match self {
      EntityState::AlarmControlPanel(state) => state.entity_state.key,
      EntityState::BinarySensor(state) => state.entity_state.key,
      EntityState::Climate(state) => state.entity_state.key,
      EntityState::Cover(state) => state.entity_state.key,
      EntityState::Date(state) => state.entity_state.key,
      EntityState::DateTime(state) => state.entity_state.key,
      EntityState::Event(state) => state.entity_state.key,
      EntityState::Fan(state) => state.entity_state.key,
      EntityState::Light(state) => state.entity_state.key,
      EntityState::Lock(state) => state.entity_state.key,
      EntityState::MediaPlayer(state) => state.entity_state.key,
      EntityState::Number(state) => state.entity_state.key,
      EntityState::Select(state) => state.entity_state.key,
      EntityState::Sensor(state) => state.entity_state.key,
      EntityState::Switch(state) => state.entity_state.key,
      EntityState::Text(state) => state.entity_state.key,
      EntityState::TextSensor(state) => state.entity_state.key,
      EntityState::Time(state) => state.entity_state.key,
      EntityState::Update(state) => state.entity_state.key,
      EntityState::Valve(state) => state.entity_state.key,
    }
  }

  pub fn parse_alarm_control_panel(data: &[u8]) -> Result<Self> {
    let data = api::AlarmControlPanelStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::AlarmControlPanel(
      services::AlarmControlPanelEntityState {
        entity_state,
        state: data.state.enum_value_or_default().into(),
      },
    ))
  }

  pub fn parse_binary_sensor(data: &[u8]) -> Result<Self> {
    let data = api::BinarySensorStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::BinarySensor(services::BinarySensorState {
      entity_state,
      state: data.state,
      missing_state: data.missing_state,
    }))
  }

  pub fn parse_climate(data: &[u8]) -> Result<Self> {
    let data = api::ClimateStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Climate(services::ClimateState {
      entity_state,
      action: data.action.enum_value_or_default().into(),
      current_temperature: data.current_temperature,
      current_humidity: data.current_humidity,
      custom_fan_mode: data.custom_fan_mode,
      custom_preset: data.custom_preset,
      fan_mode: data.fan_mode.enum_value_or_default().into(),
      legacy_away: data.unused_legacy_away,
      mode: data.mode.enum_value_or_default().into(),
      preset: data.preset.enum_value_or_default().into(),
      swing_mode: data.swing_mode.enum_value_or_default().into(),
      target_humidity: data.target_humidity,
      target_temperature: data.target_temperature,
      target_temperature_high: data.target_temperature_high,
      target_temperature_low: data.target_temperature_low,
    }))
  }

  pub fn parse_cover(data: &[u8]) -> Result<Self> {
    let data = api::CoverStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Cover(services::CoverState {
      entity_state,
      current_operation: data.current_operation.enum_value_or_default().into(),
      legacy_state: data.legacy_state.enum_value_or_default().into(),
      position: data.position,
      tilt: data.tilt,
    }))
  }

  pub fn parse_date(data: &[u8]) -> Result<Self> {
    let data = api::DateStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Date(services::DateState {
      entity_state,
      day: data.day,
      missing_state: data.missing_state,
      month: data.month,
      year: data.year,
    }))
  }

  pub fn parse_date_time(data: &[u8]) -> Result<Self> {
    let data = api::DateTimeStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::DateTime(services::DateTimeState {
      entity_state,
      epoch_seconds: data.epoch_seconds,
      missing_state: data.missing_state,
    }))
  }

  pub fn parse_event(data: &[u8]) -> Result<Self> {
    let data = api::EventResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Event(services::Event {
      entity_state,
      event_type: data.event_type.clone(),
    }))
  }

  pub fn parse_fan(data: &[u8]) -> Result<Self> {
    let data = api::FanStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Fan(services::FanState {
      entity_state,
      direction: data.direction.enum_value_or_default().into(),
      oscillating: data.oscillating,
      preset_mode: data.preset_mode.clone(),
      speed: data.speed.enum_value_or_default().into(),
      speed_level: data.speed_level,
    }))
  }

  pub fn parse_light(data: &[u8]) -> Result<Self> {
    let data = api::LightStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Light(services::LightState {
      entity_state,
      blue: data.blue,
      brightness: data.brightness,
      cold_white: data.cold_white,
      color_brightness: data.color_brightness,
      color_mode: data.color_mode.enum_value_or_default().into(),
      color_temperature: data.color_temperature,
      effect: data.effect.clone(),
      green: data.green,
      red: data.red,
      state: data.state,
      warm_white: data.warm_white,
      white: data.white,
    }))
  }

  pub fn parse_lock(data: &[u8]) -> Result<Self> {
    let data = api::LockStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Lock(services::LockEntityState {
      entity_state,
      state: data.state.enum_value_or_default().into(),
    }))
  }

  pub fn parse_media_player(data: &[u8]) -> Result<Self> {
    let data = api::MediaPlayerStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::MediaPlayer(services::MediaPlayerEntityState {
      entity_state,
      muted: data.muted,
      state: data.state.enum_value_or_default().into(),
      volume: data.volume,
    }))
  }

  pub fn parse_number(data: &[u8]) -> Result<Self> {
    let data = api::NumberStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Number(services::NumberState {
      entity_state,
      missing_state: data.missing_state,
      state: data.state,
    }))
  }

  pub fn parse_select(data: &[u8]) -> Result<Self> {
    let data = api::SelectStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Select(services::SelectState {
      entity_state,
      missing_state: data.missing_state,
      state: data.state,
    }))
  }

  pub fn parse_sensor(data: &[u8]) -> Result<Self> {
    let data = api::SensorStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Sensor(services::SensorState {
      entity_state,
      missing_state: data.missing_state,
      state: data.state,
    }))
  }

  pub fn parse_switch(data: &[u8]) -> Result<Self> {
    let data = api::SwitchStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Switch(services::SwitchState {
      entity_state,
      state: data.state,
    }))
  }

  pub fn parse_text(data: &[u8]) -> Result<Self> {
    let data = api::TextStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Text(services::TextState {
      entity_state,
      missing_state: data.missing_state,
      state: data.state.clone(),
    }))
  }

  pub fn parse_text_sensor(data: &[u8]) -> Result<Self> {
    let data = api::TextSensorStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::TextSensor(services::TextSensorState {
      entity_state,
      missing_state: data.missing_state,
      state: data.state.clone(),
    }))
  }

  pub fn parse_time(data: &[u8]) -> Result<Self> {
    let data = api::TimeStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Time(services::TimeState {
      entity_state,
      hour: data.hour,
      minute: data.minute,
      second: data.second,
      missing_state: data.missing_state,
    }))
  }

  pub fn parse_update(data: &[u8]) -> Result<Self> {
    let data = api::UpdateStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Update(services::UpdateState {
      entity_state,
      current_version: data.current_version.clone(),
      has_progress: data.has_progress,
      in_progress: data.in_progress,
      latest_version: data.latest_version.clone(),
      missing_state: data.missing_state,
      progress: data.progress,
      release_summary: data.release_summary.clone(),
      release_url: data.release_url.clone(),
      title: data.title.clone(),
    }))
  }

  pub fn parse_valve(data: &[u8]) -> Result<Self> {
    let data = api::ValveStateResponse::parse_from_bytes(data)?;

    let entity_state = services::BaseEntityState {
      key: data.key.clone(),
    };

    Ok(EntityState::Valve(services::ValveState {
      entity_state,
      current_operation: data.current_operation.enum_value_or_default().into(),
      position: data.position,
    }))
  }
}
