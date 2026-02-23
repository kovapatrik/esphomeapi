use esphomeapi_manager::entity::{BaseEntity as _, Light as RustLight};
use esphomeapi_manager::EntityState;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::entity::EntityKind;
use crate::model::ColorMode;

#[napi(object)]
pub struct LightCommandOptions {
  pub state: Option<bool>,
  pub brightness: Option<f64>,
  pub color_mode: Option<ColorMode>,
  pub color_brightness: Option<f64>,
  pub rgb: Option<(f64, f64, f64)>,
  pub white: Option<f64>,
  pub color_temperature: Option<f64>,
  pub cold_white: Option<f64>,
  pub warm_white: Option<f64>,
  pub transition_length: Option<f64>,
  pub flash_length: Option<f64>,
  pub effect: Option<String>,
}

#[napi(object)]
pub struct LightState {
  pub is_on: bool,
  pub brightness: f64,
  pub color_mode: ColorMode,
  pub color_brightness: f64,
  pub red: f64,
  pub green: f64,
  pub blue: f64,
  pub white: f64,
  pub color_temperature: f64,
  pub cold_white: f64,
  pub warm_white: f64,
  pub effect: String,
}

#[napi]
#[derive(Clone)]
pub struct Light {
  inner: RustLight,
  pub key: u32,
  pub name: String,
  #[napi(ts_type = "EntityKind.Light")]
  pub kind: EntityKind,
}

impl Light {
  pub fn new(rust_light: &RustLight) -> Self {
    Light {
      inner: rust_light.clone(),
      key: rust_light.key(),
      name: rust_light.name().to_string(),
      kind: EntityKind::Light,
    }
  }
}

#[napi]
impl Light {
  #[napi(getter)]
  pub fn is_on(&self) -> Result<bool> {
    self
      .inner
      .is_on()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn brightness(&self) -> Result<f64> {
    self
      .inner
      .brightness()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn color_mode(&self) -> Result<ColorMode> {
    self
      .inner
      .color_mode()
      .map(|cm| cm.into())
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn color_brightness(&self) -> Result<f64> {
    self
      .inner
      .color_brightness()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn rgb(&self) -> Result<(f64, f64, f64)> {
    self
      .inner
      .rgb()
      .map(|(r, g, b)| (r as f64, g as f64, b as f64))
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn white(&self) -> Result<f64> {
    self
      .inner
      .white()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn color_temperature(&self) -> Result<f64> {
    self
      .inner
      .color_temperature()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn cold_white(&self) -> Result<f64> {
    self
      .inner
      .cold_white()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn warm_white(&self) -> Result<f64> {
    self
      .inner
      .warm_white()
      .map(|v| v as f64)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn effect(&self) -> Result<String> {
    self
      .inner
      .effect()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  /// Register a callback that is called whenever the light state changes.
  ///
  /// The callback receives a `LightState` object with all current state values.
  #[napi]
  pub fn on_state_change(
    &self,
    callback: ThreadsafeFunction<LightState, (), LightState, Status, false, true>,
  ) -> Result<()> {
    let mut receiver = self.inner.state_receiver();

    napi::bindgen_prelude::spawn(async move {
      while receiver.changed().await.is_ok() {
        if let Some(EntityState::Light(s)) = receiver.borrow().clone() {
          let state = LightState {
            is_on: s.state,
            brightness: s.brightness as f64,
            color_mode: s.color_mode.into(),
            color_brightness: s.color_brightness as f64,
            red: s.red as f64,
            green: s.green as f64,
            blue: s.blue as f64,
            white: s.white as f64,
            color_temperature: s.color_temperature as f64,
            cold_white: s.cold_white as f64,
            warm_white: s.warm_white as f64,
            effect: s.effect,
          };
          callback.call(state, ThreadsafeFunctionCallMode::NonBlocking);
        }
      }
    });

    Ok(())
  }

  #[napi]
  pub async fn turn_on(&self) -> Result<()> {
    self
      .inner
      .turn_on()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn turn_off(&self) -> Result<()> {
    self
      .inner
      .turn_off()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn toggle(&self) -> Result<()> {
    self
      .inner
      .toggle()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi]
  pub async fn send_command(&self, options: LightCommandOptions) -> Result<()> {
    let mut builder = self.inner.command();

    if let Some(state) = options.state {
      builder = builder.state(state);
    }
    if let Some(brightness) = options.brightness {
      builder = builder.brightness(brightness as f32);
    }
    if let Some(color_mode) = options.color_mode {
      builder = builder.color_mode(color_mode.into());
    }
    if let Some(color_brightness) = options.color_brightness {
      builder = builder.color_brightness(color_brightness as f32);
    }
    if let Some((r, g, b)) = options.rgb {
      builder = builder.rgb(r as f32, g as f32, b as f32);
    }
    if let Some(white) = options.white {
      builder = builder.white(white as f32);
    }
    if let Some(color_temperature) = options.color_temperature {
      builder = builder.color_temperature(color_temperature as f32);
    }
    if let Some(cold_white) = options.cold_white {
      builder = builder.cold_white(cold_white as f32);
    }
    if let Some(warm_white) = options.warm_white {
      builder = builder.warm_white(warm_white as f32);
    }
    if let Some(transition_length) = options.transition_length {
      builder = builder.transition_length(transition_length as f32);
    }
    if let Some(flash_length) = options.flash_length {
      builder = builder.flash_length(flash_length as f32);
    }
    if let Some(effect) = options.effect {
      builder = builder.effect(effect);
    }

    builder
      .send()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }
}
