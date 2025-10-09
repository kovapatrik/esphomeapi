use bytes::{BufMut, Bytes, BytesMut};
use bytes_varint::*;
use tokio_util::codec::{Decoder, Encoder};

use crate::connection::codec::FrameCodec;

use super::EspHomeMessage;

#[derive(Clone)]
pub struct Plain {}

impl Plain {
  pub fn new() -> Self {
    Plain {}
  }
}

impl FrameCodec for Plain {
  fn parse_frame(
    &self,
    src: &mut bytes::BytesMut,
  ) -> Result<Option<(BytesMut, usize)>, std::io::Error> {
    if src.is_empty() {
      return Ok(None);
    }

    let preamble = src.try_get_usize_varint().unwrap();
    if preamble != 0x00 {
      return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid preamble",
      ));
    }
    let length = src.try_get_usize_varint().unwrap();
    let msg_type = src.try_get_usize_varint().unwrap();

    if src.len() < length as usize {
      return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid message length",
      ));
    }

    let msg = src.split_to(length as usize);

    Ok(Some((msg, msg_type)))
  }

  fn get_handshake_frame(&mut self) -> Option<Bytes> {
    None
  }

  fn close(&mut self) {}
}

impl Decoder for Plain {
  type Item = EspHomeMessage;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    let (msg, msg_type) = match self.parse_frame(src) {
      Ok(Some((frame, msg_type))) => (frame, msg_type),
      Ok(None) => return Ok(None),
      Err(err) => return Err(err),
    };

    Ok(Some(EspHomeMessage::new_response(
      msg_type as u32,
      msg.to_vec(),
    )))
  }
}

impl Encoder<EspHomeMessage> for Plain {
  type Error = std::io::Error;

  fn encode(&mut self, item: EspHomeMessage, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    let message = item.get_protobuf_message();
    dst.put_u8(0);
    dst.put_usize_varint(message.protobuf_data.len());
    dst.put_u32_varint(message.protobuf_type);
    dst.extend_from_slice(&message.protobuf_data);
    Ok(())
  }
}
