use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi_derive::napi;
use std::sync::OnceLock;
use tracing::{Level, Subscriber};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub type LogFn = Box<dyn Fn(String) + Send + Sync>;

struct ConsoleLogger {
  trace: LogFn,
  debug: LogFn,
  info: LogFn,
  warn: LogFn,
  error: LogFn,
}

static CONSOLE: OnceLock<ConsoleLogger> = OnceLock::new();

pub struct NodeTracingLayer;

impl<S> Layer<S> for NodeTracingLayer
where
  S: Subscriber,
{
  fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
    let Some(console) = CONSOLE.get() else {
      return;
    };

    let mut visitor = MessageVisitor::default();
    event.record(&mut visitor);

    let target = event.metadata().target();
    let message = format!("[{}] {}", target, visitor.message);

    let func = match *event.metadata().level() {
      Level::TRACE => &console.trace,
      Level::DEBUG => &console.debug,
      Level::INFO => &console.info,
      Level::WARN => &console.warn,
      Level::ERROR => &console.error,
    };

    func(message);
  }
}

#[derive(Default)]
struct MessageVisitor {
  message: String,
}

impl tracing::field::Visit for MessageVisitor {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if field.name() == "message" {
      self.message = format!("{:?}", value);
    } else if self.message.is_empty() {
      self.message = format!("{} = {:?}", field.name(), value);
    } else {
      self
        .message
        .push_str(&format!(", {} = {:?}", field.name(), value));
    }
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if field.name() == "message" {
      self.message = value.to_string();
    } else if self.message.is_empty() {
      self.message = format!("{} = {}", field.name(), value);
    } else {
      self
        .message
        .push_str(&format!(", {} = {}", field.name(), value));
    }
  }
}

fn get_method(obj: &Object, name: &str) -> Result<LogFn> {
  let func: Function<'_, String, ()> = obj
    .get_named_property(name)
    .map_err(|_| Error::from_reason(format!("Missing method: {}", name)))?;

  let tsfn = func
    .build_threadsafe_function()
    .callee_handled::<false>()
    .weak::<true>()
    .build()?;

  Ok(Box::new(move |msg: String| {
    tsfn.call(msg, ThreadsafeFunctionCallMode::NonBlocking);
  }))
}

/// Initialize the logger with a console-like object.
/// The object must have `debug`, `info`, `warn`, and `error` methods.
///
/// Example:
/// ```javascript
/// initLogger(console);
/// ```
#[napi(
  ts_args_type = "console: Pick<Console, 'log' | 'warn' | 'error' | 'info' | 'debug' | 'trace'>"
)]
pub fn init_logger(console: Object) -> Result<()> {
  let trace = get_method(&console, "trace")?;
  let debug = get_method(&console, "debug")?;
  let info = get_method(&console, "info")?;
  let warn = get_method(&console, "warn")?;
  let error = get_method(&console, "error")?;

  CONSOLE
    .set(ConsoleLogger {
      trace,
      debug,
      info,
      warn,
      error,
    })
    .map_err(|_| Error::from_reason("Logger already initialized"))?;

  tracing_subscriber::registry()
    .with(NodeTracingLayer)
    .try_init()
    .map_err(|e| Error::from_reason(format!("Failed to initialize tracing: {}", e)))?;

  Ok(())
}
