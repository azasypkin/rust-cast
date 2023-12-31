use protobuf::Error as ProtobufError;
use rustls::pki_types::InvalidDnsNameError;
use serde_json::error::Error as SerializationError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result},
    io::Error as IoError,
};

/// Consolidates possible error types that can occur in the lib.
#[derive(Debug)]
pub enum Error {
    /// This variant is used when error occurs in the lib logic.
    Internal(String),
    /// This variant includes everything related to the network connection.
    Io(IoError),
    /// This variant includes all possible errors that come from Protobuf layer.
    Protobuf(ProtobufError),
    /// This variant includes everything related to (de)serialization of incoming and outgoing
    /// messages.
    Serialization(SerializationError),
    Dns(InvalidDnsNameError),
    /// This variant includes any error that comes from rustls.
    Ssl(rustls::Error),
    /// Problems with given namespace
    Namespace(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Error::Internal(ref message) => f.write_str(message),
            Error::Io(ref err) => Display::fmt(&err, f),
            Error::Protobuf(ref err) => Display::fmt(&err, f),
            Error::Serialization(ref err) => Display::fmt(&err, f),
            Error::Ssl(ref err) => Display::fmt(&err, f),
            Error::Dns(ref err) => Display::fmt(&err, f),
            Error::Namespace(ref err) => Display::fmt(&err, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Protobuf(ref err) => Some(err),
            Error::Ssl(ref err) => Some(err),
            Error::Dns(ref err) => Some(err),
            Error::Serialization(ref err) => Some(err),
            Error::Internal(_) => None,
            Error::Namespace(_) => None,
        }
    }
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
        Error::Ssl(err)
    }
}

impl From<InvalidDnsNameError> for Error {
    fn from(err: InvalidDnsNameError) -> Error {
        Error::Dns(err)
    }
}
