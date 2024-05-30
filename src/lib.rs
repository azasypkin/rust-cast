#![deny(warnings)]

use std::{borrow::Cow, net::TcpStream, sync::Arc};

use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{aws_lc_rs::default_provider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, ClientConnection, DigitallySignedStruct, RootCertStore, StreamOwned,
};

use channels::{
    connection::{ConnectionChannel, ConnectionResponse},
    heartbeat::{HeartbeatChannel, HeartbeatResponse},
    media::{MediaChannel, MediaResponse},
    receiver::{ReceiverChannel, ReceiverResponse},
};
use errors::Error;
use message_manager::{CastMessage, MessageManager};

#[cfg(not(feature = "cast"))]
mod cast;
#[cfg(feature = "cast")]
pub mod cast;
pub mod channels;
pub mod errors;
pub mod message_manager;
mod utils;

const DEFAULT_SENDER_ID: &str = "sender-0";
const DEFAULT_RECEIVER_ID: &str = "receiver-0";

#[cfg(feature = "thread_safe")]
type Lrc<T> = std::sync::Arc<T>;
#[cfg(not(feature = "thread_safe"))]
type Lrc<T> = std::rc::Rc<T>;

/// Supported channel message types.
#[derive(Clone, Debug)]
pub enum ChannelMessage {
    /// Message to be processed by `ConnectionChannel`.
    Connection(ConnectionResponse),
    /// Message to be processed by `HeartbeatChannel`.
    Heartbeat(HeartbeatResponse),
    /// Message to be processed by `MediaChannel`.
    Media(MediaResponse),
    /// Message to be processed by `ReceiverChannel`.
    Receiver(ReceiverResponse),
    /// Raw message is returned when built-in channels can't process it (e.g. because of unknown
    /// `namespace`).
    Raw(CastMessage),
}

/// Structure that manages connection to a cast device.
pub struct CastDevice<'a> {
    message_manager: Lrc<MessageManager<StreamOwned<ClientConnection, TcpStream>>>,

    /// Channel that manages connection responses/requests.
    pub connection: ConnectionChannel<'a, StreamOwned<ClientConnection, TcpStream>>,

    /// Channel that allows connection to stay alive (via ping-pong requests/responses).
    pub heartbeat: HeartbeatChannel<'a, StreamOwned<ClientConnection, TcpStream>>,

    /// Channel that manages various media stuff.
    pub media: MediaChannel<'a, StreamOwned<ClientConnection, TcpStream>>,

    /// Channel that manages receiving platform (e.g. Chromecast).
    pub receiver: ReceiverChannel<'a, StreamOwned<ClientConnection, TcpStream>>,
}

impl<'a> CastDevice<'a> {
    /// Connects to the cast device using host name and port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_cast::CastDevice;
    ///
    /// let device = CastDevice::connect("192.168.1.2", 8009)?;
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// ```
    ///
    /// # Arguments
    ///
    /// * `host` - Cast device host name.
    /// * `port` - Cast device port number.
    ///
    /// # Errors
    ///
    /// This method may fail if connection to Cast device can't be established for some reason
    /// (e.g. wrong host name or port).
    ///
    /// # Return value
    ///
    /// Instance of `CastDevice` that allows you to manage connection.
    pub fn connect<S>(host: S, port: u16) -> Result<CastDevice<'a>, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let host = host.into();
        log::debug!("Establishing connection with cast device at {host}:{port}…");

        let mut root_store = RootCertStore::empty();
        let (valid, invalid) = root_store.add_parsable_certificates(
            rustls_native_certs::load_native_certs().expect("Could not load platform certs."),
        );
        if invalid > 0 {
            log::warn!(
                "Failed to parse {invalid} out of {} root certificates.",
                valid + invalid
            );
        } else {
            log::debug!("Successfully parsed {valid} root certificates.");
        }

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let conn = ClientConnection::new(
            config.into(),
            ServerName::try_from(host.as_ref())?.to_owned(),
        )?;
        let stream = StreamOwned::new(conn, TcpStream::connect((host.as_ref(), port))?);

        log::debug!("Connection with {host}:{port} successfully established.");

        CastDevice::connect_to_device(stream)
    }

    /// Connects to the cast device using host name and port _without_ host verification. Use on
    /// your own risk!
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_cast::CastDevice;
    ///
    /// let device = CastDevice::connect_without_host_verification("192.168.1.2", 8009)?;
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// ```
    ///
    /// # Arguments
    ///
    /// * `host` - Cast device host name.
    /// * `port` - Cast device port number.
    ///
    /// # Errors
    ///
    /// This method may fail if connection to Cast device can't be established for some reason
    /// (e.g. wrong host name or port).
    ///
    /// # Return value
    ///
    /// Instance of `CastDevice` that allows you to manage connection.
    pub fn connect_without_host_verification<S>(host: S, port: u16) -> Result<CastDevice<'a>, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let host = host.into();

        log::debug!("Establishing non-verified connection with cast device at {host}:{port}…");

        let mut config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
            .with_no_client_auth();
        config.key_log = Arc::new(rustls::KeyLogFile::new());
        let stream = StreamOwned::new(
            ClientConnection::new(
                Arc::new(config),
                ServerName::try_from(host.as_ref())?.to_owned(),
            )?,
            TcpStream::connect((host.as_ref(), port))?,
        );

        log::debug!("Connection with {host}:{port} successfully established.");

        CastDevice::connect_to_device(stream)
    }

    /// Waits for any message returned by cast device (e.g. Chromecast) and returns its parsed
    /// version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_cast::ChannelMessage;
    ///
    /// # use rust_cast::CastDevice;
    /// # let cast_device = CastDevice::connect_without_host_verification("192.168.1.2", 8009)?;
    ///
    /// match cast_device.receive() {
    ///     Ok(ChannelMessage::Connection(res)) => log::debug!("Connection message: {:?}", res),
    ///     Ok(ChannelMessage::Heartbeat(_)) => cast_device.heartbeat.pong()?,
    ///     Ok(_) => {},
    ///     Err(err) => log::error!("Error occurred while receiving message {}", err)
    /// }
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Usually fails if message returned by device can't be parsed.
    ///
    /// # Returned values
    ///
    /// Parsed channel message.
    pub fn receive(&self) -> Result<ChannelMessage, Error> {
        let cast_message = self.message_manager.receive()?;

        if self.connection.can_handle(&cast_message) {
            return Ok(ChannelMessage::Connection(
                self.connection.parse(&cast_message)?,
            ));
        }

        if self.heartbeat.can_handle(&cast_message) {
            return Ok(ChannelMessage::Heartbeat(
                self.heartbeat.parse(&cast_message)?,
            ));
        }

        if self.media.can_handle(&cast_message) {
            return Ok(ChannelMessage::Media(self.media.parse(&cast_message)?));
        }

        if self.receiver.can_handle(&cast_message) {
            return Ok(ChannelMessage::Receiver(
                self.receiver.parse(&cast_message)?,
            ));
        }

        Ok(ChannelMessage::Raw(cast_message))
    }

    /// Connects to the cast device using provided ssl stream.
    ///
    /// # Arguments
    ///
    /// * `ssl_stream` - SSL Stream for the TCP connection established with the device.
    ///
    /// # Return value
    ///
    /// Instance of `CastDevice` that allows you to manage connection.
    fn connect_to_device(
        ssl_stream: StreamOwned<ClientConnection, TcpStream>,
    ) -> Result<CastDevice<'a>, Error> {
        let message_manager_rc = Lrc::new(MessageManager::new(ssl_stream));

        let heartbeat = HeartbeatChannel::new(
            DEFAULT_SENDER_ID,
            DEFAULT_RECEIVER_ID,
            Lrc::clone(&message_manager_rc),
        );
        let connection = ConnectionChannel::new(DEFAULT_SENDER_ID, Lrc::clone(&message_manager_rc));
        let receiver = ReceiverChannel::new(
            DEFAULT_SENDER_ID,
            DEFAULT_RECEIVER_ID,
            Lrc::clone(&message_manager_rc),
        );
        let media = MediaChannel::new(DEFAULT_SENDER_ID, Lrc::clone(&message_manager_rc));

        Ok(CastDevice {
            message_manager: message_manager_rc,
            heartbeat,
            connection,
            receiver,
            media,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use byteorder::{BigEndian, WriteBytesExt};
    use log::warn;
    use protobuf::Message;
    use std::{
        fmt::Display,
        io::{Read, Write},
        sync::{Arc, RwLock},
    };

    use crate::{cast::cast_channel, utils::read_u32_from_buffer};

    #[test]
    #[cfg(feature = "thread_safe")]
    fn test_thread_safe() {
        use crate::CastDevice;

        fn is_sync<T: Sync>() {}
        fn is_send<T: Send>() {}

        is_sync::<CastDevice>();
        is_send::<CastDevice>();
    }

    /// Represents the reader half of a split mock TCP stream for testing purposes.
    #[derive(Debug)]
    pub struct ReaderHalf(MockTcpStream);

    impl Read for ReaderHalf {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.0.inner_read(buf)
        }
    }

    /// Represents the writer half of a split mock TCP stream for testing purposes.
    #[derive(Debug)]
    pub struct WriterHalf<'a>(&'a MockTcpStream);

    impl<'a> Write for WriterHalf<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.inner_write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.inner_flush()
        }
    }

    /// A mock implementation of a TCP stream for testing purposes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_cast::channels::media::MediaChannel;
    /// use rust_cast::message_manager::MessageManager;
    /// use rust_cast::Lrc;
    ///
    /// let stream = MockTcpStream::new();
    /// let message_manager = Lrc::new(MessageManager::new(stream));
    /// let channel = MediaChannel::new(
    ///     "sender-0",
    ///     message_manager
    /// );
    /// ```
    #[derive(Debug, Default, Clone)]
    pub struct MockTcpStream {
        /// Inner stream of the TCP stream which allows cloning and referencing the same stream source.
        inner: Arc<RwLock<InnerStream>>,
    }

    impl MockTcpStream {
        /// Creates a new empty `MockTcpStream` instance.
        pub fn new() -> Self {
            MockTcpStream {
                inner: Arc::new(RwLock::new(InnerStream::default())),
            }
        }

        /// Add a response message to be returned by read operations on the stream.
        pub fn add_message<M: protobuf::Message>(&mut self, message: M) {
            let message = message.write_to_bytes().unwrap();
            let mut mutex = self.inner.write().unwrap();
            mutex.response_messages.push(message);
        }

        /// Returns the received message at the given index if present, else [None].
        pub fn received_message(&self, index: usize) -> Option<TcpMessage> {
            self.inner
                .read()
                .expect("expected to acquire read lock")
                .received_messages
                .get(index)
                .cloned()
        }

        fn inner_read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.write().unwrap().read(buf)
        }

        fn inner_write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write().unwrap().write(buf)
        }

        fn inner_flush(&self) -> std::io::Result<()> {
            self.inner.write().unwrap().flush()
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner_read(buf)
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner_write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner_flush()
        }
    }

    /// Represents a TCP message containing a received payload from the sender.
    #[derive(Debug, Clone)]
    pub struct TcpMessage {
        /// The known length of the message.
        pub message_length: u32,
        /// The payload of the message.
        pub payload: Vec<u8>,
    }

    impl TcpMessage {
        /// Parses and returns the CastMessage contained in the payload.
        pub fn message(&self) -> cast_channel::CastMessage {
            <cast_channel::CastMessage as Message>::parse_from_bytes(self.payload.as_slice())
                .unwrap()
        }
    }

    impl Display for TcpMessage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", String::from_utf8_lossy(self.payload.as_slice()))
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum CursorLocation {
        Length,
        Payload,
    }

    #[derive(Debug, Clone)]
    struct ReadCursor {
        pub location: CursorLocation,
        pub index: usize,
    }

    impl ReadCursor {
        pub fn next(&self) -> Self {
            match self.location {
                CursorLocation::Length => Self {
                    location: CursorLocation::Payload,
                    index: self.index,
                },
                CursorLocation::Payload => Self {
                    location: CursorLocation::Length,
                    index: self.index + 1,
                },
            }
        }
    }

    impl Default for ReadCursor {
        fn default() -> Self {
            Self {
                location: CursorLocation::Length,
                index: 0,
            }
        }
    }

    /// Inner representation of a stream used by `MockTcpStream` for testing purposes.
    #[derive(Debug, Default)]
    struct InnerStream {
        /// The current position of the read cursor.
        cursor: ReadCursor,
        /// Buffer containing the messages which should be returned by the read operation.
        response_messages: Vec<Vec<u8>>,
        /// Buffer for storing the payload of the current message being written.
        payload_buffer: Option<TcpMessage>,
        /// Vector containing the received messages from the sender.
        received_messages: Vec<TcpMessage>,
    }

    impl Read for InnerStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if let Some(message) = self.response_messages.get(self.cursor.index) {
                let result: std::io::Result<usize> = match &self.cursor.location {
                    CursorLocation::Length => {
                        let mut len = Vec::<u8>::new();
                        len.write_u32::<BigEndian>(message.len() as u32).unwrap();
                        buf[..4].copy_from_slice(len.as_slice());
                        Ok(4)
                    }
                    CursorLocation::Payload => {
                        let len = message.len();
                        buf[..len].copy_from_slice(message.as_slice());
                        Ok(len)
                    }
                };

                self.cursor = self.cursor.next();
                result
            } else {
                warn!("No more messages to read");
                Ok(0)
            }
        }
    }

    impl Write for InnerStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(mut payload_buffer) = self.payload_buffer.take() {
                payload_buffer.payload = buf.to_vec();
                self.received_messages.push(payload_buffer);
            } else {
                let length = read_u32_from_buffer(buf).unwrap();
                self.payload_buffer = Some(TcpMessage {
                    message_length: length,
                    payload: vec![],
                });
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            // flush is never called when sending messages
            // so we don't execute any logic here
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct NoCertificateVerification;
impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
