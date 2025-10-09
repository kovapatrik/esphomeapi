mod noise;
mod plain;

use std::sync::{Arc, RwLock};

use bytes::{Bytes, BytesMut};
pub use noise::Noise;
pub use plain::Plain;
use tokio::sync::{mpsc, oneshot};
use tokio_util::codec::{Decoder, Encoder};

use crate::{Connection, Result as EspResult};

pub type Callback =
  Box<dyn Fn(Arc<RwLock<Connection>>, ProtobufMessage) -> EspResult<()> + Send + Sync + 'static>;

#[derive(Debug, Clone)]
// A common struct that holds shared fields between messages
pub struct ProtobufMessage {
  pub protobuf_type: u32,
  pub protobuf_data: Vec<u8>,
}

pub enum EspHomeMessageType {
  Response {
    protobuf_message: ProtobufMessage,
  },
  Request {
    protobuf_message: ProtobufMessage,
  },
  RequestWithAwait {
    protobuf_message: ProtobufMessage,
    response_protobuf_type: u32,
    tx: oneshot::Sender<ProtobufMessage>,
  },
  RequestWithAwaitMultipleUntil {
    protobuf_message: ProtobufMessage,
    response_protobuf_types: Vec<u32>,
    until_protobuf_type: u32,
    tx: mpsc::Sender<ProtobufMessage>,
  },
}

impl std::fmt::Debug for EspHomeMessageType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EspHomeMessageType::Response { protobuf_message } => f
        .debug_struct("Response")
        .field("protobuf_type", &protobuf_message.protobuf_type)
        .field("protobuf_data", &protobuf_message.protobuf_data)
        .finish(),
      EspHomeMessageType::Request { protobuf_message } => f
        .debug_struct("Request")
        .field("protobuf_type", &protobuf_message.protobuf_type)
        .field("protobuf_data", &protobuf_message.protobuf_data)
        .finish(),
      EspHomeMessageType::RequestWithAwait {
        protobuf_message, ..
      } => f
        .debug_struct("RequestWithAwait")
        .field("protobuf_type", &protobuf_message.protobuf_type)
        .field("protobuf_data", &protobuf_message.protobuf_data)
        .finish(),
      EspHomeMessageType::RequestWithAwaitMultipleUntil {
        protobuf_message, ..
      } => f
        .debug_struct("RequestWithAwaitMultipleUntil")
        .field("protobuf_type", &protobuf_message.protobuf_type)
        .field("protobuf_data", &protobuf_message.protobuf_data)
        .finish(),
    }
  }
}

#[derive(Debug)]
/// Define the core message that will be used in the system
pub struct EspHomeMessage {
  pub message_type: EspHomeMessageType,
}

impl EspHomeMessage {
  // Helper method to access the embedded ProtobufMessage from a Message
  pub fn get_protobuf_message(&self) -> &ProtobufMessage {
    match &self.message_type {
      EspHomeMessageType::Response { protobuf_message }
      | EspHomeMessageType::Request { protobuf_message }
      | EspHomeMessageType::RequestWithAwait {
        protobuf_message, ..
      }
      | EspHomeMessageType::RequestWithAwaitMultipleUntil {
        protobuf_message, ..
      } => protobuf_message,
    }
  }

  // Constructors for each message type
  pub fn new_response(protobuf_type: u32, protobuf_data: Vec<u8>) -> Self {
    EspHomeMessage {
      message_type: EspHomeMessageType::Response {
        protobuf_message: ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
      },
    }
  }

  pub fn new_request(protobuf_type: u32, protobuf_data: Vec<u8>) -> Self {
    EspHomeMessage {
      message_type: EspHomeMessageType::Request {
        protobuf_message: ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
      },
    }
  }

  pub fn new_request_with_await(
    protobuf_type: u32,
    protobuf_data: Vec<u8>,
    response_protobuf_type: u32,
    tx: oneshot::Sender<ProtobufMessage>,
  ) -> Self {
    EspHomeMessage {
      message_type: EspHomeMessageType::RequestWithAwait {
        protobuf_message: ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
        response_protobuf_type,
        tx,
      },
    }
  }

  pub fn new_request_with_await_multiple_until(
    protobuf_type: u32,
    protobuf_data: Vec<u8>,
    response_protobuf_types: Vec<u32>,
    until_protobuf_type: u32,
    tx: mpsc::Sender<ProtobufMessage>,
  ) -> Self {
    EspHomeMessage {
      message_type: EspHomeMessageType::RequestWithAwaitMultipleUntil {
        protobuf_message: ProtobufMessage {
          protobuf_type,
          protobuf_data,
        },
        response_protobuf_types,
        until_protobuf_type,
        tx,
      },
    }
  }
}

pub trait FrameCodec:
  Encoder<EspHomeMessage, Error = std::io::Error>
  + Decoder<Item = EspHomeMessage, Error = std::io::Error>
{
  fn parse_frame(
    &self,
    src: &mut bytes::BytesMut,
    // TODO: refactor tuple return
  ) -> Result<Option<(BytesMut, usize)>, std::io::Error>;
  fn get_handshake_frame(&mut self) -> Option<Bytes>;
  fn close(&mut self);
}

#[derive(Clone)]
pub enum EspHomeCodec {
  Noise(Arc<RwLock<Noise>>),
  Plain(Arc<RwLock<Plain>>),
}

impl FrameCodec for EspHomeCodec {
  fn parse_frame(
    &self,
    src: &mut bytes::BytesMut,
  ) -> Result<Option<(BytesMut, usize)>, std::io::Error> {
    match self {
      EspHomeCodec::Noise(codec) => codec.read().unwrap().parse_frame(src),
      EspHomeCodec::Plain(codec) => codec.read().unwrap().parse_frame(src),
    }
  }

  fn get_handshake_frame(&mut self) -> Option<Bytes> {
    match self {
      EspHomeCodec::Noise(codec) => codec.write().unwrap().get_handshake_frame(),
      EspHomeCodec::Plain(codec) => codec.write().unwrap().get_handshake_frame(),
    }
  }

  fn close(&mut self) {
    match self {
      EspHomeCodec::Noise(codec) => codec.write().unwrap().close(),
      EspHomeCodec::Plain(codec) => codec.write().unwrap().close(),
    }
  }
}

impl Encoder<EspHomeMessage> for EspHomeCodec {
  type Error = std::io::Error;

  fn encode(&mut self, item: EspHomeMessage, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    match self {
      EspHomeCodec::Noise(codec) => codec.write().unwrap().encode(item, dst),
      EspHomeCodec::Plain(codec) => codec.write().unwrap().encode(item, dst),
    }
  }
}

impl Decoder for EspHomeCodec {
  type Item = EspHomeMessage;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    match self {
      EspHomeCodec::Noise(codec) => codec.write().unwrap().decode(src),
      EspHomeCodec::Plain(codec) => codec.write().unwrap().decode(src),
    }
  }
}
