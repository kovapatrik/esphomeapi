mod noise;
mod plain;

use bytes::{Bytes, BytesMut};
pub use noise::{NoiseDecoder, NoiseEncoder, NoiseHandshake};
pub use plain::{PlainDecoder, PlainEncoder};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

/// Errors that can occur during codec operations
#[derive(Debug, Error)]
pub enum CodecError {
  #[error("Invalid preamble: expected {expected:#04x}, got {actual:#04x}")]
  InvalidPreamble { expected: u8, actual: u8 },

  #[error("Invalid protocol version: expected {expected:#04x}, got {actual:#04x}")]
  InvalidProtocol { expected: u8, actual: u8 },

  #[error("Frame size {size} exceeds maximum {max}")]
  FrameTooLarge { size: usize, max: usize },

  #[error("Server name mismatch: expected '{expected}', got '{actual}'")]
  ServerNameMismatch { expected: String, actual: String },

  #[error("Invalid server name encoding: {0}")]
  InvalidServerName(#[from] std::string::FromUtf8Error),

  #[error("Invalid base64 PSK: {0}")]
  InvalidPsk(#[from] base64::DecodeError),

  #[error("Handshake failed: {0}")]
  HandshakeFailed(String),

  #[error("Decryption failed")]
  DecryptionFailed,

  #[error("Message too short: expected at least {expected} bytes, got {actual}")]
  MessageTooShort { expected: usize, actual: usize },

  #[error("Varint decoding failed: value too large")]
  VarintTooLarge,

  #[error("Not ready: handshake not complete")]
  NotReady,

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
}

impl From<CodecError> for std::io::Error {
  fn from(err: CodecError) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
  }
}

/// A protobuf message with its type ID and serialized data
#[derive(Debug, Clone)]
pub struct ProtobufMessage {
  pub protobuf_type: u32,
  pub protobuf_data: Vec<u8>,
}

/// Result of processing a handshake step
pub enum HandshakeResult {
  /// Need more data from the server to continue
  NeedMoreData,
  /// Send this frame to the server, then continue handshake
  SendFrame(Bytes),
  /// Handshake completed successfully, ready to split into encoder/decoder
  Complete(EspHomeDecoder, EspHomeEncoder),
}

/// Codec for handshake phase - handles both reading and writing during setup
pub enum EspHomeHandshake {
  Noise(NoiseHandshake),
  Plain(PlainDecoder), // Plain doesn't need handshake, but we keep consistent API
}

impl EspHomeHandshake {
  /// Create a new codec for the given connection parameters
  ///
  /// # Arguments
  /// * `psk` - Optional base64-encoded pre-shared key for Noise encryption
  /// * `expected_name` - Optional server name to verify during handshake
  pub fn new(psk: Option<String>, expected_name: Option<String>) -> Result<Self, CodecError> {
    match psk {
      Some(psk) => Ok(Self::Noise(NoiseHandshake::new(&psk, expected_name)?)),
      None => Ok(Self::Plain(PlainDecoder::new())),
    }
  }

  /// Process incoming data during handshake and potentially complete it
  ///
  /// Returns:
  /// - `HandshakeResult::NeedMoreData` if more data is needed from the server
  /// - `HandshakeResult::SendFrame` if a frame needs to be sent
  /// - `HandshakeResult::Complete` when handshake is done
  pub fn process(&mut self, src: &mut BytesMut) -> Result<HandshakeResult, CodecError> {
    match self {
      Self::Noise(handshake) => handshake.process(src),
      Self::Plain(_) => {
        // Plain doesn't need handshake, immediately ready
        Ok(HandshakeResult::Complete(
          EspHomeDecoder::Plain(PlainDecoder::new()),
          EspHomeEncoder::Plain(PlainEncoder::new()),
        ))
      }
    }
  }
}

/// Decoder for receiving messages after handshake is complete
pub enum EspHomeDecoder {
  Noise(NoiseDecoder),
  Plain(PlainDecoder),
}

impl Decoder for EspHomeDecoder {
  type Item = ProtobufMessage;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    match self {
      Self::Noise(decoder) => decoder.decode(src),
      Self::Plain(decoder) => decoder.decode(src),
    }
  }
}

/// Encoder for sending messages after handshake is complete
pub enum EspHomeEncoder {
  Noise(NoiseEncoder),
  Plain(PlainEncoder),
}

impl Encoder<ProtobufMessage> for EspHomeEncoder {
  type Error = std::io::Error;

  fn encode(&mut self, item: ProtobufMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
    match self {
      Self::Noise(encoder) => encoder.encode(item.clone(), dst),
      Self::Plain(encoder) => encoder.encode(item.clone(), dst),
    }
  }
}
