use esphomeapi_manager::Manager as RustManager;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

use napi::tokio::sync::broadcast::error::RecvError;
use napi_derive::napi;
use tracing::warn;

use crate::entity;
use crate::model::{HomeAssistantEvent, HomeassistantActionRequest, LogEvent, LogLevel};

#[napi(object)]
pub struct ConnectionOptions {
  pub address: String,
  pub port: u32,
  pub password: Option<String>,
  pub expected_name: Option<String>,
  pub psk: Option<String>,
  pub client_info: Option<String>,
  pub keep_alive_duration: Option<u32>,
}

#[napi]
pub struct Manager {
  inner: RustManager,
}

#[napi]
impl Manager {
  #[napi(factory)]
  pub async fn connect(options: ConnectionOptions) -> Result<Manager> {
    let manager = RustManager::new(
      options.address,
      options.port,
      options.password,
      options.expected_name,
      options.psk,
      options.client_info,
      options.keep_alive_duration,
    )
    .await;

    Ok(Manager { inner: manager })
  }

  #[napi]
  pub fn get_device_name(&self) -> String {
    self.inner.device_info.name.clone()
  }

  #[napi]
  pub fn get_device_mac(&self) -> String {
    self.inner.device_info.mac_address.clone()
  }

  #[napi]
  pub fn get_entities(&self) -> Vec<Either<entity::Light, entity::Switch>> {
    self
      .inner
      .get_entities()
      .values()
      .filter_map(|e| match e {
        esphomeapi_manager::entity::Entity::Light(l) => Some(Either::A(entity::Light::new(l))),
        esphomeapi_manager::entity::Entity::Switch(s) => Some(Either::B(entity::Switch::new(s))),
        _ => None,
      })
      .collect()
  }

  /// Subscribe to Home Assistant state events.
  ///
  /// Sends the subscription request to the device, then calls `callback` for every
  /// subsequent event. The callback runs on the JS thread via a threadsafe function.
  ///
  /// Call this once per subscription; the callback keeps firing until the connection
  /// closes. If you need a second independent listener, call
  /// `onHomeAssistantState` instead (no extra request is sent).
  #[napi]
  pub async fn subscribe_home_assistant_states(
    &self,
    callback: ThreadsafeFunction<HomeAssistantEvent, (), HomeAssistantEvent, Status, false, true>,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .subscribe_home_assistant_states()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(event) => {
            callback.call(event.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!(
              "home_assistant_states receiver lagged, missed {} messages",
              n
            );
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Register an additional listener for Home Assistant state events without
  /// sending a new subscription request to the device.
  ///
  /// Use this when `subscribe_home_assistant_states` has already been called
  /// and you need a second independent callback.
  #[napi]
  pub fn on_home_assistant_state(
    &self,
    callback: ThreadsafeFunction<HomeAssistantEvent, (), HomeAssistantEvent, Status, false, true>,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .home_assistant_states_receiver()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(event) => {
            callback.call(event.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!(
              "home_assistant_states receiver lagged, missed {} messages",
              n
            );
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Subscribe to Home Assistant action request events.
  ///
  /// Sends the subscription request to the device, then calls `callback` for every
  /// subsequent action request. Call this once; use `onHomeAssistantActionRequest`
  /// for additional listeners without re-sending the request.
  #[napi]
  pub async fn subscribe_home_assistant_action_requests(
    &self,
    callback: ThreadsafeFunction<
      HomeassistantActionRequest,
      (),
      HomeassistantActionRequest,
      Status,
      false,
      true,
    >,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .subscribe_home_assistant_action_requests()
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(request) => {
            callback.call(request.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!(
              "home_assistant_action_requests receiver lagged, missed {} messages",
              n
            );
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Register an additional listener for Home Assistant action requests without
  /// sending a new subscription request to the device.
  #[napi]
  pub fn on_home_assistant_action_request(
    &self,
    callback: ThreadsafeFunction<
      HomeassistantActionRequest,
      (),
      HomeassistantActionRequest,
      Status,
      false,
      true,
    >,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .home_assistant_action_requests_receiver()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(request) => {
            callback.call(request.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!(
              "home_assistant_action_requests receiver lagged, missed {} messages",
              n
            );
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Subscribe to ESPHome logs.
  ///
  /// Sends the subscription request to the device, then calls `callback` for every
  /// subsequent action request. Call this once; use `onLogs`
  /// for additional listeners without re-sending the request.
  #[napi]
  pub async fn subscribe_logs(
    &self,
    level: LogLevel,
    dump_config: bool,
    callback: ThreadsafeFunction<LogEvent, (), LogEvent, Status, false, true>,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .subscribe_logs(level.into(), dump_config)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(request) => {
            callback.call(request.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!("logs receiver lagged, missed {} messages", n);
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Register an additional listener for ESPHome logs without
  /// sending a new subscription request to the device.
  #[napi]
  pub fn on_logs(
    &self,
    callback: ThreadsafeFunction<LogEvent, (), LogEvent, Status, false, true>,
  ) -> Result<()> {
    let mut rx = self
      .inner
      .logs_receiver()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    napi::bindgen_prelude::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(request) => {
            callback.call(request.into(), ThreadsafeFunctionCallMode::NonBlocking);
          }
          Err(RecvError::Lagged(n)) => {
            warn!("logs receiver lagged, missed {} messages", n);
          }
          Err(RecvError::Closed) => break,
        }
      }
    });

    Ok(())
  }

  /// Send the current state of a Home Assistant entity to the device.
  ///
  /// Used to respond to a `HomeAssistantEvent` of type `"StateSubscription"` or
  /// `"StateRequest"`, or to push an update when a subscribed entity changes.
  #[napi]
  pub async fn send_home_assistant_state(
    &self,
    entity_id: String,
    state: String,
    attribute: Option<String>,
  ) -> Result<()> {
    self
      .inner
      .send_home_assistant_state(entity_id, state, attribute)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
  }
}
