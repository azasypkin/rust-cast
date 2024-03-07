#![deny(warnings)]

mod cast;
pub mod channels;
pub mod errors;
pub mod message_manager;

use std::{borrow::Cow, sync::Arc};

use channels::{
    connection::{ConnectionChannel, ConnectionResponse},
    heartbeat::{HeartbeatChannel, HeartbeatResponse},
    media::{MediaChannel, MediaResponse},
    receiver::{ReceiverChannel, ReceiverResponse},
};
use errors::Error;
use message_manager::{CastMessage, MessageManager};

use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{aws_lc_rs::default_provider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, DigitallySignedStruct, RootCertStore,
};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

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
    message_manager: Lrc<MessageManager<TlsStream<TcpStream>>>,

    /// Channel that manages connection responses/requests.
    pub connection: ConnectionChannel<'a, TlsStream<TcpStream>>,

    /// Channel that allows connection to stay alive (via ping-pong requests/responses).
    pub heartbeat: HeartbeatChannel<'a, TlsStream<TcpStream>>,

    /// Channel that manages various media stuff.
    pub media: MediaChannel<'a, TlsStream<TcpStream>>,

    /// Channel that manages receiving platform (e.g. Chromecast).
    pub receiver: ReceiverChannel<'a, TlsStream<TcpStream>>,
}

impl<'a> CastDevice<'a> {
    /// Connects to the cast device using host name and port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rust_cast::CastDevice;
    ///
    /// # tokio_test::block_on(async {
    /// let device = CastDevice::connect("192.168.1.2", 8009).await?;
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// # });
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
    pub async fn connect<S>(host: S, port: u16) -> Result<CastDevice<'a>, Error>
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
        let connor = tokio_rustls::TlsConnector::from(Arc::new(config));

        let stream = connor
            .connect(
                ServerName::try_from(host.as_ref())?.to_owned(),
                TcpStream::connect((host.as_ref(), port)).await?,
            )
            .await?;

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
    /// # tokio_test::block_on(async {
    /// let device = CastDevice::connect_without_host_verification("192.168.1.2", 8009).await?;
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// # });
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
    pub async fn connect_without_host_verification<S>(
        host: S,
        port: u16,
    ) -> Result<CastDevice<'a>, Error>
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
        let connor = tokio_rustls::TlsConnector::from(Arc::new(config));

        let stream = connor
            .connect(
                ServerName::try_from(host.as_ref())?.to_owned(),
                TcpStream::connect((host.as_ref(), port)).await?,
            )
            .await?;
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
    /// # tokio_test::block_on(async {
    /// # let cast_device = CastDevice::connect_without_host_verification("192.168.1.2", 8009).await?;
    ///
    /// match cast_device.receive().await {
    ///     Ok(ChannelMessage::Connection(res)) => log::debug!("Connection message: {:?}", res),
    ///     Ok(ChannelMessage::Heartbeat(_)) => cast_device.heartbeat.pong().await?,
    ///     Ok(_) => {},
    ///     Err(err) => log::error!("Error occurred while receiving message {}", err)
    /// }
    /// # Ok::<(), rust_cast::errors::Error>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Usually fails if message returned by device can't be parsed.
    ///
    /// # Returned values
    ///
    /// Parsed channel message.
    pub async fn receive(&self) -> Result<ChannelMessage, Error> {
        let cast_message = self.message_manager.receive().await?;

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
    fn connect_to_device(ssl_stream: TlsStream<TcpStream>) -> Result<CastDevice<'a>, Error> {
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
    #[test]
    #[cfg(feature = "thread_safe")]
    fn test_thread_safe() {
        use crate::CastDevice;

        fn is_sync<T: Sync>() {}
        fn is_send<T: Send>() {}

        is_sync::<CastDevice>();
        is_send::<CastDevice>();
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
