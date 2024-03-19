/// Proxy classes for the `connection` channel.
pub mod connection {
    use serde_derive::Serialize;

    #[derive(Serialize, Debug)]
    pub struct ConnectionRequest {
        #[serde(rename = "type")]
        pub typ: String,
        #[serde(rename = "userAgent")]
        pub user_agent: String,
    }
}

/// Proxy classes for the `heartbeat` channel.
pub mod heartbeat {
    use serde_derive::Serialize;

    #[derive(Serialize, Debug)]
    pub struct HeartBeatRequest {
        #[serde(rename = "type")]
        pub typ: String,
    }
}

/// Proxy classes for the `media` channel.
pub mod media {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    pub struct GetStatusRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "mediaSessionId", skip_serializing_if = "Option::is_none")]
        pub media_session_id: Option<i32>,
    }

    // Really LoadRequest
    /// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.LoadRequest
    #[derive(Serialize, Debug)]
    pub struct MediaRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "sessionId")]
        pub session_id: String,

        #[serde(rename = "type")]
        pub typ: String,

        pub media: Media,

        #[serde(rename = "currentTime")]
        pub current_time: f64,

        #[serde(rename = "customData")]
        pub custom_data: CustomData,

        pub autoplay: bool,

        #[serde(rename = "queueData", skip_serializing_if = "Option::is_none")]
        pub queue_data: Option<QueueData>,
    }

    /// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.QueueItem
    #[derive(Serialize, Debug)]
    pub struct QueueItem {
        #[serde(rename = "activeTrackIds")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub active_track_ids: Option<Vec<u16>>,

        pub autoplay: bool,

        #[serde(rename = "customData")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub custom_data: Option<CustomData>,

        #[serde(rename = "itemId")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub item_id: Option<u16>,

        pub media: Media,

        #[serde(rename = "playbackDuration")]
        pub playback_duration: Option<f64>,

        #[serde(rename = "preloadTime")]
        pub preload_time: f64,

        #[serde(rename = "startTime")]
        pub start_time: f64,
    }

    /// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.QueueLoadRequest
    #[derive(Serialize, Debug)]
    pub struct QueueLoadRequest {
        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "customData")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub custom_data: Option<CustomData>,

        pub items: Vec<QueueItem>,

        // This is from https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.QueueData
        #[serde(rename = "queueType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub queue_type: Option<String>,

        #[serde(rename = "repeatMode")]
        pub repeat_mode: String,

        #[serde(rename = "startIndex")]
        pub start_index: u16,
    }

    /// https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.QueueData
    #[derive(Serialize, Debug)]
    pub struct QueueData {
        pub items: Vec<QueueItem>,

        #[serde(rename = "queueType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub queue_type: Option<String>,

        #[serde(rename = "repeatMode")]
        pub repeat_mode: String,

        #[serde(rename = "startIndex")]
        pub start_index: u16,
    }

    #[derive(Serialize, Debug)]
    pub struct PlaybackGenericRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "mediaSessionId")]
        pub media_session_id: i32,

        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "customData")]
        pub custom_data: CustomData,
    }

    #[derive(Serialize, Debug)]
    pub struct PlaybackSeekRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "mediaSessionId")]
        pub media_session_id: i32,

        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "resumeState")]
        pub resume_state: Option<String>,

        #[serde(rename = "currentTime")]
        pub current_time: Option<f32>,

        #[serde(rename = "customData")]
        pub custom_data: CustomData,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Media {
        #[serde(rename = "contentId")]
        pub content_id: String,
        #[serde(rename = "streamType", default)]
        pub stream_type: String,
        #[serde(rename = "contentType")]
        pub content_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<Metadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub duration: Option<f32>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Metadata {
        #[serde(rename = "metadataType")]
        pub metadata_type: u32,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "seriesTitle")]
        pub series_title: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "albumName")]
        pub album_name: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub subtitle: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "albumArtist")]
        pub album_artist: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub artist: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub composer: Option<String>,

        pub images: Vec<Image>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "releaseDate")]
        pub release_date: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "originalAirDate")]
        pub original_air_date: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "creationDateTime")]
        pub creation_date_time: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub studio: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub location: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub latitude: Option<f64>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub longitude: Option<f64>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub season: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub episode: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "trackNumber")]
        pub track_number: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none", rename = "discNumber")]
        pub disc_number: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub width: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub height: Option<u32>,
    }

    impl Metadata {
        pub fn new(metadata_type: u32) -> Metadata {
            Metadata {
                metadata_type,
                title: None,
                series_title: None,
                album_name: None,
                subtitle: None,
                album_artist: None,
                artist: None,
                composer: None,
                images: Vec::new(),
                release_date: None,
                original_air_date: None,
                creation_date_time: None,
                studio: None,
                location: None,
                latitude: None,
                longitude: None,
                season: None,
                episode: None,
                track_number: None,
                disc_number: None,
                width: None,
                height: None,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Image {
        pub url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub width: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub height: Option<u32>,
    }

    #[derive(Serialize, Debug)]
    pub struct CustomData {}

    impl CustomData {
        pub fn new() -> CustomData {
            CustomData {}
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct ExtendedStatus {
        #[serde(rename = "playerState")]
        pub player_state: String,
        #[serde(rename = "mediaSessionId")]
        pub media_session_id: Option<i32>,
        pub media: Option<Media>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Status {
        #[serde(rename = "mediaSessionId")]
        pub media_session_id: i32,
        #[serde(default)]
        pub media: Option<Media>,
        #[serde(rename = "playbackRate")]
        pub playback_rate: f32,
        #[serde(rename = "playerState")]
        pub player_state: String,
        #[serde(rename = "currentItemId")]
        pub current_item_id: Option<u16>,
        #[serde(rename = "loadingItemId")]
        pub loading_item_id: Option<u16>,
        #[serde(rename = "preloadedItemId")]
        pub preloaded_item_id: Option<u16>,
        #[serde(rename = "idleReason")]
        pub idle_reason: Option<String>,
        #[serde(rename = "extendedStatus")]
        pub extended_status: Option<ExtendedStatus>,
        #[serde(rename = "currentTime")]
        pub current_time: Option<f32>,
        #[serde(rename = "supportedMediaCommands")]
        pub supported_media_commands: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct StatusReply {
        #[serde(rename = "requestId", default)]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub status: Vec<Status>,
    }

    #[derive(Deserialize, Debug)]
    pub struct LoadCancelledReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct LoadFailedReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct InvalidPlayerStateReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct InvalidRequestReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub reason: Option<String>,
    }
}

/// Proxy classes for the `receiver` channel.
pub mod receiver {
    use std::borrow::Cow;

    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    pub struct AppLaunchRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "appId")]
        pub app_id: String,
    }

    #[derive(Serialize, Debug)]
    pub struct AppStopRequest<'a> {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        #[serde(rename = "sessionId")]
        pub session_id: Cow<'a, str>,
    }

    #[derive(Serialize, Debug)]
    pub struct GetStatusRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,
    }

    #[derive(Serialize, Debug)]
    pub struct SetVolumeRequest {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub volume: Volume,
    }

    #[derive(Deserialize, Debug)]
    pub struct StatusReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub status: Status,
    }

    #[derive(Deserialize, Debug)]
    pub struct Status {
        #[serde(default)]
        pub applications: Vec<Application>,

        #[serde(rename = "isActiveInput", default)]
        pub is_active_input: bool,

        #[serde(rename = "isStandBy", default)]
        pub is_stand_by: bool,

        /// Volume parameters of the currently active cast device.
        pub volume: Volume,
    }

    #[derive(Deserialize, Debug)]
    pub struct Application {
        #[serde(rename = "appId")]
        pub app_id: String,

        #[serde(rename = "sessionId")]
        pub session_id: String,

        #[serde(rename = "transportId", default)]
        pub transport_id: String,

        #[serde(default)]
        pub namespaces: Vec<AppNamespace>,

        #[serde(rename = "displayName")]
        pub display_name: String,

        #[serde(rename = "statusText")]
        pub status_text: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct AppNamespace {
        pub name: String,
    }

    /// Structure that describes possible cast device volume options.
    #[derive(Deserialize, Serialize, Debug)]
    pub struct Volume {
        /// Volume level.
        pub level: Option<f32>,
        /// Mute/unmute state.
        pub muted: Option<bool>,
    }

    #[derive(Deserialize, Debug)]
    pub struct LaunchErrorReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub reason: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct InvalidRequestReply {
        #[serde(rename = "requestId")]
        pub request_id: u32,

        #[serde(rename = "type")]
        pub typ: String,

        pub reason: Option<String>,
    }
}
