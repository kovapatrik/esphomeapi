use esphomeapi_manager::HomeAssistantEvent as RustHomeAssistantEvent;
use napi_derive::napi;

#[napi(string_enum)]
pub enum HomeAssistantEventKind {
  StateSubscription,
  StateRequest,
}

/// A Home Assistant state event received from the ESPHome device.
///
/// `eventType` is either `"StateSubscription"` (device wants ongoing updates)
/// or `"StateRequest"` (device wants the current value once).
#[napi(object)]
pub struct HomeAssistantEvent {
  pub event_type: HomeAssistantEventKind,
  pub entity_id: String,
  pub attribute: Option<String>,
}

impl From<RustHomeAssistantEvent> for HomeAssistantEvent {
  fn from(event: RustHomeAssistantEvent) -> Self {
    match event {
      RustHomeAssistantEvent::StateSubscription {
        entity_id,
        attribute,
      } => Self {
        event_type: HomeAssistantEventKind::StateSubscription,
        entity_id,
        attribute,
      },
      RustHomeAssistantEvent::StateRequest {
        entity_id,
        attribute,
      } => Self {
        event_type: HomeAssistantEventKind::StateRequest,
        entity_id,
        attribute,
      },
    }
  }
}
