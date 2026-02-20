use std::sync::Arc;

use esphomeapi::{
  Client,
  model::{EntityState, LightInfo, LightState},
};
use tokio::sync::watch;

pub use esphomeapi::model::ColorMode;

use super::{BaseEntity, StateError, StateResult};

/// Builder for constructing light commands with a fluent API.
///
/// Created via [`Light::command()`]. Set properties by chaining methods,
/// then call [`send()`](LightCommandBuilder::send) to execute.
///
/// # Example
/// ```ignore
/// light.command()
///     .state(true)
///     .brightness(0.8)
///     .rgb(1.0, 0.5, 0.0)
///     .send()
///     .await?;
/// ```
pub struct LightCommandBuilder<'a> {
  light: &'a Light,
  state: Option<bool>,
  brightness: Option<f32>,
  color_mode: Option<ColorMode>,
  color_brightness: Option<f32>,
  rgb: Option<(f32, f32, f32)>,
  white: Option<f32>,
  color_temperature: Option<f32>,
  cold_white: Option<f32>,
  warm_white: Option<f32>,
  transition_length: Option<f32>,
  flash_length: Option<f32>,
  effect: Option<String>,
}

impl<'a> LightCommandBuilder<'a> {
  fn new(light: &'a Light) -> Self {
    Self {
      light,
      state: None,
      brightness: None,
      color_mode: None,
      color_brightness: None,
      rgb: None,
      white: None,
      color_temperature: None,
      cold_white: None,
      warm_white: None,
      transition_length: None,
      flash_length: None,
      effect: None,
    }
  }

  pub fn state(mut self, state: bool) -> Self {
    self.state = Some(state);
    self
  }

  pub fn brightness(mut self, brightness: f32) -> Self {
    self.brightness = Some(brightness);
    self
  }

  pub fn color_mode(mut self, color_mode: ColorMode) -> Self {
    self.color_mode = Some(color_mode);
    self
  }

  pub fn color_brightness(mut self, color_brightness: f32) -> Self {
    self.color_brightness = Some(color_brightness);
    self
  }

  pub fn rgb(mut self, r: f32, g: f32, b: f32) -> Self {
    self.rgb = Some((r, g, b));
    self
  }

  pub fn white(mut self, white: f32) -> Self {
    self.white = Some(white);
    self
  }

  pub fn color_temperature(mut self, color_temperature: f32) -> Self {
    self.color_temperature = Some(color_temperature);
    self
  }

  pub fn cold_white(mut self, cold_white: f32) -> Self {
    self.cold_white = Some(cold_white);
    self
  }

  pub fn warm_white(mut self, warm_white: f32) -> Self {
    self.warm_white = Some(warm_white);
    self
  }

  pub fn transition_length(mut self, transition_length: f32) -> Self {
    self.transition_length = Some(transition_length);
    self
  }

  pub fn flash_length(mut self, flash_length: f32) -> Self {
    self.flash_length = Some(flash_length);
    self
  }

  pub fn effect(mut self, effect: impl Into<String>) -> Self {
    self.effect = Some(effect.into());
    self
  }

  pub async fn send(self) -> esphomeapi::Result<()> {
    self
      .light
      .client
      .light_command(
        self.light.info.entity_info.key,
        self.state,
        self.brightness,
        self.color_mode,
        self.color_brightness,
        self.rgb,
        self.white,
        self.color_temperature,
        self.cold_white,
        self.warm_white,
        self.transition_length,
        self.flash_length,
        self.effect,
      )
      .await
  }
}

#[derive(Clone)]
pub struct Light {
  client: Arc<Client>,
  info: LightInfo,
  state: watch::Receiver<Option<EntityState>>,
}

impl Light {
  pub fn new(
    client: Arc<Client>,
    info: LightInfo,
    state: watch::Receiver<Option<EntityState>>,
  ) -> Self {
    Light {
      client,
      info,
      state,
    }
  }

  pub fn command(&self) -> LightCommandBuilder<'_> {
    LightCommandBuilder::new(self)
  }

  pub fn get_state(&self) -> StateResult<LightState> {
    match self.state.borrow().as_ref() {
      Some(EntityState::Light(state)) => Ok(state.clone()),
      Some(_) => Err(StateError::NotValidState),
      None => Err(StateError::EntityKeyNotFound(self.info.entity_info.key)),
    }
  }

  /// Returns a cloned receiver for watching state changes from an external context.
  pub fn state_receiver(&self) -> watch::Receiver<Option<EntityState>> {
    self.state.clone()
  }

  /// Wait for the next state change and return the updated state.
  pub async fn state_changed(&mut self) -> StateResult<LightState> {
    self
      .state
      .changed()
      .await
      .map_err(|_| StateError::EntityKeyNotFound(self.info.entity_info.key))?;
    self.get_state()
  }

  pub fn is_on(&self) -> esphomeapi::Result<bool> {
    let state = self.get_state()?;

    Ok(state.state)
  }

  pub async fn turn_on(&self) -> esphomeapi::Result<()> {
    self.command().state(true).send().await
  }

  pub async fn turn_off(&self) -> esphomeapi::Result<()> {
    self.command().state(false).send().await
  }

  pub async fn toggle(&self) -> esphomeapi::Result<()> {
    match self.is_on()? {
      true => self.turn_off().await,
      false => self.turn_on().await,
    }
  }

  pub fn brightness(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.brightness)
  }

  pub fn color_mode(&self) -> esphomeapi::Result<ColorMode> {
    let state = self.get_state()?;

    Ok(state.color_mode)
  }

  pub fn color_brightness(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.color_brightness)
  }

  pub fn rgb(&self) -> esphomeapi::Result<(f32, f32, f32)> {
    let state = self.get_state()?;

    Ok((state.red, state.green, state.blue))
  }

  pub fn white(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.white)
  }

  pub fn color_temperature(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.color_temperature)
  }

  pub fn cold_white(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.cold_white)
  }

  pub fn warm_white(&self) -> esphomeapi::Result<f32> {
    let state = self.get_state()?;

    Ok(state.warm_white)
  }

  pub fn effect(&self) -> esphomeapi::Result<String> {
    let state = self.get_state()?;

    Ok(state.effect)
  }
}

impl BaseEntity for Light {
  fn key(&self) -> u32 {
    self.info.entity_info.key
  }

  fn name(&self) -> String {
    self.info.entity_info.name.clone()
  }
}
