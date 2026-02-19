use bytes::{BufMut, BytesMut};
use bytes_varint::VarIntSupportMut;
use tokio_util::codec::{Decoder, Encoder};

use super::{CodecError, ProtobufMessage};

/// Preamble byte for plain (unencrypted) protocol
const PLAIN_PREAMBLE: u8 = 0x00;

/// Read a varint from the buffer without consuming it
///
/// Returns `Ok(Some((value, bytes_consumed)))` if a complete varint was read,
/// `Ok(None)` if more bytes are needed, or an error if the varint is too large.
fn peek_varint(src: &[u8]) -> Result<Option<(usize, usize)>, CodecError> {
  if src.is_empty() {
    return Ok(None);
  }

  let mut value: usize = 0;
  let mut shift: u32 = 0;

  for (i, &byte) in src.iter().enumerate() {
    value |= ((byte & 0x7F) as usize) << shift;

    if byte & 0x80 == 0 {
      return Ok(Some((value, i + 1)));
    }

    shift += 7;
    if shift >= 64 {
      return Err(CodecError::VarintTooLarge);
    }
  }

  // Need more bytes
  Ok(None)
}

/// Parse a complete frame from the buffer
///
/// Returns the message data and type if a complete frame is available.
fn parse_frame(src: &mut BytesMut) -> Result<Option<ProtobufMessage>, CodecError> {
  if src.is_empty() {
    return Ok(None);
  }

  // Peek at header without consuming
  let mut offset = 0;

  // Read preamble
  let (preamble, preamble_len) = match peek_varint(&src[offset..])? {
    Some(v) => v,
    None => return Ok(None),
  };

  if preamble != PLAIN_PREAMBLE as usize {
    return Err(CodecError::InvalidPreamble {
      expected: PLAIN_PREAMBLE,
      actual: preamble as u8,
    });
  }
  offset += preamble_len;

  // Read message length
  let (length, length_len) = match peek_varint(&src[offset..])? {
    Some(v) => v,
    None => return Ok(None),
  };
  offset += length_len;

  // Read message type
  let (msg_type, msg_type_len) = match peek_varint(&src[offset..])? {
    Some(v) => v,
    None => return Ok(None),
  };
  offset += msg_type_len;

  // Check if we have enough data for the message body
  if src.len() < offset + length {
    return Ok(None);
  }

  // Now consume the header bytes
  let _ = src.split_to(offset);

  // Extract the message data
  let data = src.split_to(length);

  Ok(Some(ProtobufMessage {
    protobuf_type: msg_type as u32,
    protobuf_data: data.to_vec(),
  }))
}

/// Encode a message into the buffer
fn encode_message(message: ProtobufMessage, dst: &mut BytesMut) {
  // Write preamble
  dst.put_u8(PLAIN_PREAMBLE);

  // Write message length as varint
  dst.put_usize_varint(message.protobuf_data.len());

  // Write message type as varint
  dst.put_u32_varint(message.protobuf_type);

  // Write message data
  dst.extend_from_slice(&message.protobuf_data);
}

/// Decoder for plain (unencrypted) ESPHome protocol
#[derive(Clone, Default)]
pub struct PlainDecoder;

impl PlainDecoder {
  pub fn new() -> Self {
    Self
  }
}

impl Decoder for PlainDecoder {
  type Item = ProtobufMessage;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    parse_frame(src).map_err(Into::into)
  }
}

/// Encoder for plain (unencrypted) ESPHome protocol
#[derive(Clone, Default)]
pub struct PlainEncoder;

impl PlainEncoder {
  pub fn new() -> Self {
    Self
  }
}

impl Encoder<ProtobufMessage> for PlainEncoder {
  type Error = std::io::Error;

  fn encode(&mut self, item: ProtobufMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
    encode_message(item, dst);
    Ok(())
  }
}
