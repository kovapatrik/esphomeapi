use esphomeapi_manager::entity::{BaseEntity as _, Switch as RustSwitch};
use esphomeapi_manager::EntityState;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

#[napi]
pub struct Switch {
  inner: RustSwitch,
}

impl Switch {
  pub fn new(rust_switch: RustSwitch) -> Self {
    Switch { inner: rust_switch }
  }
}

#[napi]
impl Switch {
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
    match self.inner.get_state() {
      Ok(state) => Ok(state.state),
      Err(e) => Err(Error::new(Status::GenericFailure, e.to_string())),
    }
  }

  /// Register a callback that is called whenever the switch state changes.
  ///
  /// The callback receives a single boolean argument indicating whether the switch is on.
  #[napi(
    ts_args_type = "callback: (isOn: boolean) => void",
    ts_return_type = "void"
  )]
  pub fn on_state_change(
    &self,
    callback: ThreadsafeFunction<bool, (), bool, Status, false, true>,
  ) -> Result<()> {
    let mut receiver = self.inner.state_receiver();

    napi::bindgen_prelude::spawn(async move {
      while receiver.changed().await.is_ok() {
        if let Some(EntityState::Switch(s)) = receiver.borrow().clone() {
          callback.call(s.state, ThreadsafeFunctionCallMode::NonBlocking);
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
  pub async fn set_state(&self, state: bool) -> Result<()> {
    self
      .inner
      .set_state(state)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }
}
