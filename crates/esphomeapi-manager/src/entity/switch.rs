use std::sync::Arc;

use esphomeapi::{
  Client,
  model::{EntityState, SwitchInfo, SwitchState},
};
use tokio::sync::watch;

use super::{BaseEntity, StateError, StateResult};

#[derive(Clone)]
pub struct Switch {
  client: Arc<Client>,
  info: SwitchInfo,
  state: watch::Receiver<Option<EntityState>>,
}

impl Switch {
  pub fn new(
    client: Arc<Client>,
    info: SwitchInfo,
    state: watch::Receiver<Option<EntityState>>,
  ) -> Self {
    Switch {
      client,
      info,
      state,
    }
  }

  pub fn get_state(&self) -> StateResult<SwitchState> {
    match self.state.borrow().as_ref() {
      Some(EntityState::Switch(state)) => Ok(state.clone()),
      Some(_) => Err(StateError::NotValidState),
      None => Err(StateError::EntityKeyNotFound(self.info.entity_info.key)),
    }
  }

  /// Returns a cloned receiver for watching state changes from an external context.
  pub fn state_receiver(&self) -> watch::Receiver<Option<EntityState>> {
    self.state.clone()
  }

  /// Wait for the next state change and return the updated state.
  pub async fn state_changed(&mut self) -> StateResult<SwitchState> {
    self
      .state
      .changed()
      .await
      .map_err(|_| StateError::EntityKeyNotFound(self.info.entity_info.key))?;
    self.get_state()
  }

  pub fn is_on(&self) -> StateResult<bool> {
    Ok(self.get_state()?.state)
  }

  pub async fn turn_on(&self) -> esphomeapi::Result<()> {
    self
      .client
      .switch_command(self.info.entity_info.key, true)
      .await
  }

  pub async fn turn_off(&self) -> esphomeapi::Result<()> {
    self
      .client
      .switch_command(self.info.entity_info.key, false)
      .await
  }

  pub async fn toggle(&self) -> esphomeapi::Result<()> {
    match self.is_on()? {
      true => self.turn_off().await,
      false => self.turn_on().await,
    }
  }

  pub async fn set_state(&self, state: bool) -> esphomeapi::Result<()> {
    match state {
      true => self.turn_on().await,
      false => self.turn_off().await,
    }
  }
}

impl BaseEntity for Switch {
  fn key(&self) -> u32 {
    self.info.entity_info.key
  }

  fn name(&self) -> String {
    self.info.entity_info.name.clone()
  }
}
