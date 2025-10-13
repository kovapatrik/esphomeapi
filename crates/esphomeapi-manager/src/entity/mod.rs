mod light;
mod switch;

use std::fmt;

pub use light::{ColorMode, Light};
pub use switch::Switch;

type StateResult<T> = std::result::Result<T, StateError>;

#[derive(Debug, Clone)]
pub enum StateError {
  EntityKeyNotFound(u32),
  NotValidState,
}

impl fmt::Display for StateError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Self::EntityKeyNotFound(key) => write!(f, "entity key {} not found", key),
      Self::NotValidState => write!(f, "invalid state"),
    }
  }
}

impl std::error::Error for StateError {}

#[derive(Clone)]
pub enum Entity {
  Switch(Switch),
  Light(Light),
  Sensor(),
}

pub trait BaseEntity {
  fn key(&self) -> u32;
  fn name(&self) -> String;
}
