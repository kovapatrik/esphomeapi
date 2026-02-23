mod light;
mod switch;

use napi::Either;
use napi_derive::napi;

pub use light::Light;
pub use switch::Switch;

#[derive(Debug, Clone)]
#[napi(string_enum)]
pub enum EntityKind {
  Light,
  Switch,
}

#[napi]
pub type Entity = Either<Light, Switch>;
