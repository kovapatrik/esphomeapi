use protobuf::{Message, MessageDyn, MessageFull};

use crate::proto::api_options::exts::id;

pub trait Options {
  fn get_option_id() -> u32;
}

impl<T> Options for T
where
  T: MessageFull + MessageDyn + Message,
{
  fn get_option_id() -> u32 {
    let msg = T::descriptor();
    let options = msg.proto().options.as_ref().unwrap();
    id.get(options).unwrap()
  }
}
