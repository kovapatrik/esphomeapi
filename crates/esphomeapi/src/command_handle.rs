use std::sync::{Arc, RwLock};

use protobuf::EnumOrUnknown;

use crate::connection::{ProtobufMessage, RouterHandle};
use crate::model::ColorMode;
use crate::utils::Options as _;
use crate::{proto, Result};

/// A cloneable handle for sending commands to a connected ESPHome device.
///
/// Obtained via [`Client::command_handle()`] after a successful connection.
/// All clones share the same underlying router reference, so calling
/// [`update_from`](CommandHandle::update_from) on any clone (e.g. after a
/// reconnect) immediately affects every entity that holds a clone.
#[derive(Clone)]
pub struct CommandHandle {
  router: Arc<RwLock<RouterHandle>>,
}

impl CommandHandle {
  /// Create a handle that shares an existing `Arc<RwLock<RouterHandle>>`.
  ///
  /// All clones point to the same arc — swapping the router on reconnect is
  /// immediately visible to every entity that holds a clone.
  pub(crate) fn from_shared(router: Arc<RwLock<RouterHandle>>) -> Self {
    Self { router }
  }

  /// Acquire a read lock long enough to clone the `RouterHandle`, then drop
  /// the lock before awaiting — avoids holding a sync lock across `.await`.
  async fn send_proto<M>(&self, message: M) -> Result<()>
  where
    M: protobuf::MessageFull,
  {
    let router = self.router.read().unwrap().clone();
    let protobuf_type = M::get_option_id();
    let protobuf_data = message.write_to_bytes()?;
    router
      .send(ProtobufMessage {
        protobuf_type,
        protobuf_data,
      })
      .await
  }

  pub async fn switch_command(&self, key: u32, state: bool) -> Result<()> {
    let message = proto::api::SwitchCommandRequest {
      key,
      state,
      ..Default::default()
    };
    self.send_proto(message).await
  }

  pub async fn light_command(
    &self,
    key: u32,
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
  ) -> Result<()> {
    let message = proto::api::LightCommandRequest {
      key,
      has_state: state.is_some(),
      state: state.unwrap_or_default(),
      has_brightness: brightness.is_some(),
      brightness: brightness.unwrap_or_default(),
      has_color_mode: color_mode.is_some(),
      color_mode: EnumOrUnknown::new(color_mode.unwrap_or_default().into()),
      has_color_brightness: color_brightness.is_some(),
      color_brightness: color_brightness.unwrap_or_default(),
      has_rgb: rgb.is_some(),
      red: rgb.unwrap_or_default().0,
      green: rgb.unwrap_or_default().1,
      blue: rgb.unwrap_or_default().2,
      has_white: white.is_some(),
      white: white.unwrap_or_default(),
      has_color_temperature: color_temperature.is_some(),
      color_temperature: color_temperature.unwrap_or_default(),
      has_cold_white: cold_white.is_some(),
      cold_white: cold_white.unwrap_or_default(),
      has_warm_white: warm_white.is_some(),
      warm_white: warm_white.unwrap_or_default(),
      has_transition_length: transition_length.is_some(),
      transition_length: (transition_length.unwrap_or_default() * 1000.0).round() as u32,
      has_flash_length: flash_length.is_some(),
      flash_length: (flash_length.unwrap_or_default() * 1000.0).round() as u32,
      has_effect: effect.is_some(),
      effect: effect.unwrap_or_default(),
      ..Default::default()
    };
    self.send_proto(message).await
  }
}
