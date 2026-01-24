pub mod discovery;
mod entity;
pub mod logger;
mod manager;
mod model;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]

pub enum Error {
  #[error("esphomeapi error: {0}")]
  EsphomeapiError(#[from] esphomeapi_manager::Error),
}

impl Into<napi::Error> for Error {
  fn into(self) -> napi::Error {
    napi::Error::new(napi::Status::GenericFailure, self.to_string())
  }
}
