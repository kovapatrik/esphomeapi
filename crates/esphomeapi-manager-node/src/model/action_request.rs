use std::collections::HashMap;

use esphomeapi_manager::HomeassistantActionRequest as RustActionRequest;
use napi_derive::napi;

#[napi(object)]
pub struct HomeassistantActionRequest {
  pub service: String,
  pub is_event: bool,
  pub data: HashMap<String, String>,
  pub data_template: HashMap<String, String>,
  pub variables: HashMap<String, String>,
  pub call_id: u32,
  pub wants_response: bool,
  pub response_template: String,
}

impl From<RustActionRequest> for HomeassistantActionRequest {
  fn from(r: RustActionRequest) -> Self {
    Self {
      service: r.service,
      is_event: r.is_event,
      data: r.data,
      data_template: r.data_template,
      variables: r.variables,
      call_id: r.call_id,
      wants_response: r.wants_response,
      response_template: r.response_template,
    }
  }
}
