use esphomeapi_manager::{LogEvent as RustLogEvent, LogLevel as RustLogLevel};
use napi_derive::napi;

#[napi]
pub enum LogLevel {
  None = 0,
  Error,
  Warn,
  Info,
  Config,
  Debug,
  Verbose,
  VeryVerbose,
}

impl From<RustLogLevel> for LogLevel {
  fn from(value: RustLogLevel) -> Self {
    match value {
      RustLogLevel::None => LogLevel::None,
      RustLogLevel::Error => LogLevel::Error,
      RustLogLevel::Warn => LogLevel::Warn,
      RustLogLevel::Info => LogLevel::Info,
      RustLogLevel::Config => LogLevel::Config,
      RustLogLevel::Debug => LogLevel::Debug,
      RustLogLevel::Verbose => LogLevel::Verbose,
      RustLogLevel::VeryVerbose => LogLevel::VeryVerbose,
    }
  }
}

impl From<LogLevel> for RustLogLevel {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::None => RustLogLevel::None,
      LogLevel::Error => RustLogLevel::Error,
      LogLevel::Warn => RustLogLevel::Warn,
      LogLevel::Info => RustLogLevel::Info,
      LogLevel::Config => RustLogLevel::Config,
      LogLevel::Debug => RustLogLevel::Debug,
      LogLevel::Verbose => RustLogLevel::Verbose,
      LogLevel::VeryVerbose => RustLogLevel::VeryVerbose,
    }
  }
}

#[napi(object)]
pub struct LogEvent {
  pub level: LogLevel,
  pub message: String,
}

impl From<RustLogEvent> for LogEvent {
  fn from(value: RustLogEvent) -> Self {
    LogEvent {
      level: value.level.into(),
      message: String::from_utf8_lossy(&value.message).to_string(),
    }
  }
}
