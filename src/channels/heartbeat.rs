use std::borrow::Cow;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    cast::proxies,
    errors::Error,
    message_manager::{CastMessage, CastMessagePayload, MessageManager},
    Lrc,
};

const CHANNEL_NAMESPACE: &str = "urn:x-cast:com.google.cast.tp.heartbeat";

const MESSAGE_TYPE_PING: &str = "PING";
const MESSAGE_TYPE_PONG: &str = "PONG";

#[derive(Clone, Debug)]
pub enum HeartbeatResponse {
    Ping,
    Pong,
    NotImplemented(String, serde_json::Value),
}

pub struct HeartbeatChannel<'a, W>
where
    W: AsyncRead + AsyncWrite,
{
    sender: Cow<'a, str>,
    receiver: Cow<'a, str>,
    message_manager: Lrc<MessageManager<W>>,
}

impl<'a, W> HeartbeatChannel<'a, W>
where
    W: AsyncRead + AsyncWrite,
{
    pub fn new<S>(
        sender: S,
        receiver: S,
        message_manager: Lrc<MessageManager<W>>,
    ) -> HeartbeatChannel<'a, W>
    where
        S: Into<Cow<'a, str>>,
    {
        HeartbeatChannel {
            sender: sender.into(),
            receiver: receiver.into(),
            message_manager,
        }
    }

    pub async fn ping(&self) -> Result<(), Error> {
        let payload = serde_json::to_string(&proxies::heartbeat::HeartBeatRequest {
            typ: MESSAGE_TYPE_PING.to_string(),
        })?;

        self.message_manager
            .send(CastMessage {
                namespace: CHANNEL_NAMESPACE.to_string(),
                source: self.sender.to_string(),
                destination: self.receiver.to_string(),
                payload: CastMessagePayload::String(payload),
            })
            .await
    }

    pub async fn pong(&self) -> Result<(), Error> {
        let payload = serde_json::to_string(&proxies::heartbeat::HeartBeatRequest {
            typ: MESSAGE_TYPE_PONG.to_string(),
        })?;

        self.message_manager
            .send(CastMessage {
                namespace: CHANNEL_NAMESPACE.to_string(),
                source: self.sender.to_string(),
                destination: self.receiver.to_string(),
                payload: CastMessagePayload::String(payload),
            })
            .await
    }

    pub fn can_handle(&self, message: &CastMessage) -> bool {
        message.namespace == CHANNEL_NAMESPACE
    }

    pub fn parse(&self, message: &CastMessage) -> Result<HeartbeatResponse, Error> {
        let reply = match message.payload {
            CastMessagePayload::String(ref payload) => {
                serde_json::from_str::<serde_json::Value>(payload)?
            }
            _ => {
                return Err(Error::Internal(
                    "Binary payload is not supported!".to_string(),
                ))
            }
        };

        let message_type = reply
            .as_object()
            .and_then(|object| object.get("type"))
            .and_then(|property| property.as_str())
            .unwrap_or("")
            .to_string();

        let response = match message_type.as_ref() {
            MESSAGE_TYPE_PING => HeartbeatResponse::Ping,
            MESSAGE_TYPE_PONG => HeartbeatResponse::Pong,
            _ => HeartbeatResponse::NotImplemented(message_type.to_string(), reply),
        };

        Ok(response)
    }
}
