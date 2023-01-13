use openssl::error::ErrorStack;
use openssl::ssl::HandshakeError;
use protobuf::ProtobufError;
use serde_json::error::Error as SerializationError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use std::net::TcpStream;

/// Consolidates possible error types that can occur in the OpenSSL lib.
#[derive(Debug)]
pub enum SslError {
    /// This variant includes everything related to the existing SSL connection.
    Generic(ErrorStack),
    /// This variant describes an error or intermediate state after a TLS handshake attempt.
    Handshake(HandshakeError<TcpStream>),
}

impl Display for SslError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            SslError::Generic(ref err) => Display::fmt(&err, f),
            SslError::Handshake(ref err) => Display::fmt(&err, f),
        }
    }
}

impl StdError for SslError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            SslError::Generic(ref e) => e.source(),
            SslError::Handshake(ref e) => e.source(),
        }
    }
}

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
    /// This variant includes any error that comes from OpenSSL.
    Ssl(SslError),
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

impl From<ErrorStack> for Error {
    fn from(err: ErrorStack) -> Error {
        Error::Ssl(SslError::Generic(err))
    }
}

impl From<HandshakeError<TcpStream>> for Error {
    fn from(err: HandshakeError<TcpStream>) -> Error {
        Error::Ssl(SslError::Handshake(err))
    }
}
