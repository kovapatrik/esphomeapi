mod proto {
  include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

pub use proto::api;

mod client;
mod command_handle;
mod connection;
pub mod discovery;
pub mod model;
mod utils;

pub use client::Client;
pub use command_handle::CommandHandle;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
