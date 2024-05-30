use std::io::Error as IoError;

use protobuf::Error as ProtobufError;
use rustls::pki_types::InvalidDnsNameError;
use serde_json::error::Error as SerializationError;
use thiserror::Error;

/// Consolidates possible error types that can occur in the lib.
#[derive(Debug, Error)]
pub enum Error {
    /// This variant is used when error occurs in the lib logic.
    #[error("an internal error occurred, {}", _0)]
    Internal(String),
    /// This variant includes everything related to the network connection.
    #[error("{}", _0)]
    Io(IoError),
    /// This variant includes all possible errors that come from Protobuf layer.
    #[error("{}", _0)]
    Protobuf(ProtobufError),
    /// Errors with JSON (de)serialization of incoming and outgoing
    /// messages.
    #[error("{}", _0)]
    Serialization(SerializationError),
    /// Errors parsing messages (valid JSON but bad semantics)
    #[error("{}", _0)]
    Parsing(String),
    /// This variant is used to indicate invalid DNS name used to connect to Cast device.
    #[error("{}", _0)]
    Dns(InvalidDnsNameError),
    /// This variant includes any error that comes from rustls.
    #[error("{}", _0)]
    Tls(rustls::Error),
    /// Problems with given namespace
    #[error("{}", _0)]
    Namespace(String),
    /// This variant is used when login blocks take too long.
    #[error("{}", _0)]
    Timeout(String),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Error {
        Error::Protobuf(err)
    }
}

impl From<SerializationError> for Error {
    fn from(err: SerializationError) -> Error {
        Error::Serialization(err)
    }
}

impl From<rustls::Error> for Error {
    fn from(err: rustls::Error) -> Error {
        Error::Tls(err)
    }
}

impl From<InvalidDnsNameError> for Error {
    fn from(err: InvalidDnsNameError) -> Error {
        Error::Dns(err)
    }
}
