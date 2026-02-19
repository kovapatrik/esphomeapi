use base64::prelude::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use noise_protocol::{patterns::noise_nn_psk0, CipherState, HandshakeState};
use noise_rust_crypto::{ChaCha20Poly1305, Sha256, X25519};
use tokio_util::codec::{Decoder, Encoder};

use super::{CodecError, EspHomeDecoder, EspHomeEncoder, HandshakeResult, ProtobufMessage};

/// Prologue for the Noise protocol handshake
const NOISE_PROLOGUE: &[u8] = b"NoiseAPIInit\x00\x00";

/// Hello frame sent to initiate the connection
const NOISE_HELLO: &[u8] = &[0x01, 0x00, 0x00];

/// Preamble byte for noise-encrypted frames
const NOISE_PREAMBLE: u8 = 0x01;

/// Header size for noise frames (preamble + 2 bytes length)
const HEADER_SIZE: usize = 3;

/// Maximum allowed frame size
const MAX_FRAME_SIZE: usize = 65535;

/// Overhead added by encryption (Poly1305 tag)
const ENCRYPTION_OVERHEAD: usize = 16;

/// Size of the message header inside encrypted payload (type + length, both u16)
const MESSAGE_HEADER_SIZE: usize = 4;

/// Parse a frame from the buffer, returning the frame data if complete
fn parse_frame(src: &mut BytesMut) -> Result<Option<BytesMut>, CodecError> {
  if src.len() < HEADER_SIZE {
    return Ok(None);
  }

  let preamble = src[0];
  if preamble != NOISE_PREAMBLE {
    return Err(CodecError::InvalidPreamble {
      expected: NOISE_PREAMBLE,
      actual: preamble,
    });
  }

  let length = u16::from_be_bytes([src[1], src[2]]) as usize;

  if length > MAX_FRAME_SIZE {
    return Err(CodecError::FrameTooLarge {
      size: length,
      max: MAX_FRAME_SIZE,
    });
  }

  if src.len() < HEADER_SIZE + length {
    src.reserve(HEADER_SIZE + length - src.len());
    return Ok(None);
  }

  // Consume header
  src.advance(HEADER_SIZE);

  // Extract frame data
  Ok(Some(src.split_to(length)))
}

/// Internal state during handshake
enum HandshakePhase {
  /// Initial state - ready to send Hello frame
  Initial,
  /// Waiting for server hello response
  AwaitingServerHello,
  /// Waiting for handshake response after sending noise message
  AwaitingHandshakeResponse,
}

/// Noise handshake handler
///
/// This type manages the Noise protocol handshake. Once the handshake completes,
/// it produces separate encoder and decoder.
pub struct NoiseHandshake {
  phase: HandshakePhase,
  initiator: HandshakeState<X25519, ChaCha20Poly1305, Sha256>,
  expected_server_name: Option<String>,
}

impl NoiseHandshake {
  /// Create a new Noise handshake handler
  ///
  /// # Arguments
  /// * `psk` - Base64-encoded pre-shared key
  /// * `expected_server_name` - Optional server name to verify
  pub fn new(psk: &str, expected_server_name: Option<String>) -> Result<Self, CodecError> {
    let psk_bytes = BASE64_STANDARD.decode(psk.as_bytes())?;

    let mut initiator = HandshakeState::new(
      noise_nn_psk0(),
      true,
      NOISE_PROLOGUE,
      None,
      None,
      None,
      None,
    );
    initiator.push_psk(&psk_bytes);

    Ok(Self {
      phase: HandshakePhase::Initial,
      initiator,
      expected_server_name,
    })
  }

  /// Process handshake step
  ///
  /// Returns:
  /// - `HandshakeResult::NeedMoreData` if more data is needed from the server
  /// - `HandshakeResult::SendFrame` if a frame needs to be sent
  /// - `HandshakeResult::Complete` when handshake is done
  pub fn process(&mut self, src: &mut BytesMut) -> Result<HandshakeResult, CodecError> {
    match self.phase {
      HandshakePhase::Initial => {
        // Transition to awaiting server hello
        self.phase = HandshakePhase::AwaitingServerHello;
        Ok(HandshakeResult::SendFrame(Bytes::from_static(NOISE_HELLO)))
      }
      HandshakePhase::AwaitingServerHello => {
        let data = match parse_frame(src)? {
          Some(data) => data,
          None => return Ok(HandshakeResult::NeedMoreData),
        };
        self.handle_server_hello(data)
      }
      HandshakePhase::AwaitingHandshakeResponse => {
        let data = match parse_frame(src)? {
          Some(data) => data,
          None => return Ok(HandshakeResult::NeedMoreData),
        };
        self.handle_handshake_response(data)
      }
    }
  }

  /// Handle the Server Hello response
  ///
  /// After validating, generates the Noise handshake frame to send.
  fn handle_server_hello(&mut self, data: BytesMut) -> Result<HandshakeResult, CodecError> {
    // Validate protocol version
    let chosen_proto = data.first().copied().unwrap_or(0);
    if chosen_proto != NOISE_PREAMBLE {
      return Err(CodecError::InvalidProtocol {
        expected: NOISE_PREAMBLE,
        actual: chosen_proto,
      });
    }

    // Check for server name (added in ESPHome 2022.2)
    if let Some(null_pos) = data.iter().skip(1).position(|&x| x == 0x00) {
      let server_name = String::from_utf8(data[1..1 + null_pos].to_vec())?;

      if let Some(expected) = &self.expected_server_name {
        if server_name != *expected {
          return Err(CodecError::ServerNameMismatch {
            expected: expected.clone(),
            actual: server_name,
          });
        }
      }
    }

    // Generate the Noise handshake message
    let handshake_payload = self
      .initiator
      .write_message_vec(&[])
      .map_err(|e| CodecError::HandshakeFailed(format!("Failed to write handshake: {:?}", e)))?;

    let payload_len = handshake_payload.len() + 1; // +1 for the 0x00 byte

    let mut frame = BytesMut::with_capacity(HEADER_SIZE + payload_len);

    // Handshake frame header
    frame.put_u8(NOISE_PREAMBLE);
    frame.put_u16(payload_len as u16);

    // Handshake payload (0x00 prefix + noise message)
    frame.put_u8(0x00);
    frame.extend_from_slice(&handshake_payload);

    // Transition to awaiting handshake response
    self.phase = HandshakePhase::AwaitingHandshakeResponse;

    Ok(HandshakeResult::SendFrame(frame.freeze()))
  }

  /// Handle the handshake response and complete the handshake
  fn handle_handshake_response(
    &mut self,
    mut data: BytesMut,
  ) -> Result<HandshakeResult, CodecError> {
    // Check for error response (error flag = 0x01)
    let first_byte = data.first().copied().unwrap_or(0);
    if first_byte == 0x01 {
      // Error response: [0x01] [error message...]
      data.advance(1);
      let error_msg = String::from_utf8_lossy(&data).to_string();
      return Err(CodecError::HandshakeFailed(error_msg));
    }

    // Success response starts with 0x00
    if first_byte != 0x00 {
      return Err(CodecError::HandshakeFailed(format!(
        "Invalid handshake response prefix: {:#04x}",
        first_byte
      )));
    }
    data.advance(1);

    // Process the Noise handshake response
    self
      .initiator
      .read_message_vec(&data)
      .map_err(|e| CodecError::HandshakeFailed(format!("Noise error: {:?}", e)))?;

    if !self.initiator.completed() {
      return Err(CodecError::HandshakeFailed(
        "Handshake did not complete as expected".to_string(),
      ));
    }

    let (encoder_cipher, decoder_cipher) = self.initiator.get_ciphers();

    Ok(HandshakeResult::Complete(
      EspHomeDecoder::Noise(NoiseDecoder::new(decoder_cipher)),
      EspHomeEncoder::Noise(NoiseEncoder::new(encoder_cipher)),
    ))
  }
}

/// Decoder for noise-encrypted messages
///
/// Created after handshake completes. Decrypts incoming messages.
pub struct NoiseDecoder {
  cipher: CipherState<ChaCha20Poly1305>,
}

impl NoiseDecoder {
  fn new(cipher: CipherState<ChaCha20Poly1305>) -> Self {
    Self { cipher }
  }
}

impl Decoder for NoiseDecoder {
  type Item = ProtobufMessage;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    let data = match parse_frame(src).map_err(std::io::Error::from)? {
      Some(data) => data,
      None => return Ok(None),
    };

    // Decrypt the frame
    let buffer = self
      .cipher
      .decrypt_vec(&data)
      .map_err(|_| CodecError::DecryptionFailed)?;

    if buffer.len() < MESSAGE_HEADER_SIZE {
      return Err(
        CodecError::MessageTooShort {
          expected: MESSAGE_HEADER_SIZE,
          actual: buffer.len(),
        }
        .into(),
      );
    }

    // Message layout:
    // - 2 bytes: message type (big-endian)
    // - 2 bytes: message length (big-endian)
    // - N bytes: message data
    let msg_type = u16::from_be_bytes([buffer[0], buffer[1]]) as u32;

    Ok(Some(ProtobufMessage {
      protobuf_type: msg_type,
      protobuf_data: buffer[MESSAGE_HEADER_SIZE..].to_vec(),
    }))
  }
}

/// Encoder for noise-encrypted messages
///
/// Created after handshake completes. Encrypts outgoing messages.
pub struct NoiseEncoder {
  cipher: CipherState<ChaCha20Poly1305>,
}

impl NoiseEncoder {
  fn new(cipher: CipherState<ChaCha20Poly1305>) -> Self {
    Self { cipher }
  }
}

impl Encoder<ProtobufMessage> for NoiseEncoder {
  type Error = std::io::Error;

  fn encode(&mut self, item: ProtobufMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
    // Build the plaintext message
    let mut plaintext = BytesMut::with_capacity(MESSAGE_HEADER_SIZE + item.protobuf_data.len());

    // Message type (big-endian u16)
    plaintext.put_u16(item.protobuf_type as u16);

    // Message length (big-endian u16)
    plaintext.put_u16(item.protobuf_data.len() as u16);

    // Message data
    plaintext.extend_from_slice(&item.protobuf_data);

    // Encrypt
    let mut ciphertext = BytesMut::zeroed(plaintext.len() + ENCRYPTION_OVERHEAD);
    self.cipher.encrypt(&plaintext, &mut ciphertext);

    // Write frame header
    dst.put_u8(NOISE_PREAMBLE);
    dst.put_u16(ciphertext.len() as u16);

    // Write encrypted data
    dst.extend_from_slice(&ciphertext);

    Ok(())
  }
}
