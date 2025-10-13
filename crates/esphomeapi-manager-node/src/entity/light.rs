use esphomeapi_manager::entity::{BaseEntity as _, Light as RustLight};
use napi::bindgen_prelude::*;
use napi_derive::napi;

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

#[napi]
pub struct Light {
  inner: RustLight,
}

impl Light {
  pub fn new(rust_light: RustLight) -> Self {
    Light { inner: rust_light }
  }
}

#[napi]
impl Light {
  #[napi(getter)]
  pub fn key(&self) -> u32 {
    self.inner.key()
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.inner.name().to_string()
  }

  #[napi(getter)]
  pub fn is_on(&self) -> Result<bool> {
    self
      .inner
      .is_on()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
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

  #[napi(getter)]
  pub fn brightness(&self) -> Result<f32> {
    self
      .inner
      .brightness()
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
  pub fn color_brightness(&self) -> Result<f32> {
    self
      .inner
      .color_brightness()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn rgb(&self) -> Result<(f32, f32, f32)> {
    self
      .inner
      .rgb()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn white(&self) -> Result<f32> {
    self
      .inner
      .white()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn color_temperature(&self) -> Result<f32> {
    self
      .inner
      .color_temperature()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn cold_white(&self) -> Result<f32> {
    self
      .inner
      .cold_white()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn warm_white(&self) -> Result<f32> {
    self
      .inner
      .warm_white()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }

  #[napi(getter)]
  pub fn effect(&self) -> Result<String> {
    self
      .inner
      .effect()
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
