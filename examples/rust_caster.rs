use std::str::FromStr;

use ansi_term::Colour::{Green, Red};
use docopt::Docopt;
use log::error;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Deserialize;

use rust_cast::{
    channels::{
        heartbeat::HeartbeatResponse,
        media::{Media, StatusEntry, StreamType},
        receiver::CastDeviceApp,
    },
    CastDevice, ChannelMessage,
};

const SERVICE_TYPE: &str = "_googlecast._tcp.local.";
const DEFAULT_DESTINATION_ID: &str = "receiver-0";

const USAGE: &str = "
Usage: rust-caster [-v] [-h] [-a <address>] [-p <port>] [-i | -r <app to run> | -s <app to stop> | --stop-current | [-m <media handle> [--media-type <media type>] [--media-stream-type <stream type>] [--media-app <media app>]] | [--media-volume <level> | --media-mute| --media-unmute | --media-pause | --media-play | --media-stop | --media-seek <time>] [--media-app <media app>]]

Options:
    -a, --address <address>                 Cast device network address.
    -p, --port <port>                       Cast device network port. [default: 8009]
    -r, --run <app_to_run>                  Run the app with specified id/name.
    -s, --stop <app_to_stop>                Stops the app with specified id/name.
        --stop-current                      Stops currently active app.
    -i, --info                              Returns the info about the receiver.
    -m, --media <media_handle>              Media handle (URL for image or video, URL token for youtube video etc.) to load on the Cast connected device.
        --media-type <media_type>           Type of the media to load.
        --media-app <media_app>             Media app to use for streaming. [default: default]
        --media-stream-type <stream_type>   Media stream type to use (buffered, live or none). [default: none]
        --media-volume <level>              Media volume level.
        --media-mute                        Mute cast device.
        --media-unmute                      Unmute cast device.
        --media-pause                       Pause currently active media in the app that is passed in `--media-app`.
        --media-play                        Play currently paused media in the app that is passed in `--media-app`.
        --media-stop                        Stops currently active media in the app that is passed in `--media-app`.
        --media-seek <time>                 Sets the current position in the media stream in the app that is passed in `--media-app`.
    -v, --verbose                           Toggle verbose output.
    -h, --help                              Print this help menu.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_address: Option<String>,
    flag_port: u16,
    flag_run: Option<String>,
    flag_stop: Option<String>,
    flag_stop_current: bool,
    flag_info: Option<String>,
    flag_media: Option<String>,
    flag_media_type: Option<String>,
    flag_media_app: String,
    flag_media_stream_type: String,
    flag_media_volume: Option<f32>,
    flag_media_mute: bool,
    flag_media_unmute: bool,
    flag_media_pause: bool,
    flag_media_play: bool,
    flag_media_stop: bool,
    flag_media_seek: Option<f32>,
}

async fn print_info(device: &CastDevice<'_>) {
    let status = device.receiver.get_status().await.unwrap();

    println!(
        "\n{} {}",
        Green.paint("Number of apps run:"),
        Red.paint(status.applications.len().to_string())
    );
    for i in 0..status.applications.len() {
        println!(
            "{}{}{}{}{}{}{}{}{}",
            Green.paint("App#"),
            Green.paint(i.to_string()),
            Green.paint(": "),
            Red.paint(status.applications[i].display_name.as_str()),
            Red.paint(" ("),
            Red.paint(status.applications[i].app_id.as_str()),
            Red.paint(")"),
            Red.paint(" - "),
            Red.paint(status.applications[i].status_text.as_str())
        );
    }

    if let Some(level) = status.volume.level {
        println!(
            "{} {}",
            Green.paint("Volume level:"),
            Red.paint(level.to_string())
        );
    }

    if let Some(muted) = status.volume.muted {
        println!(
            "{} {}\n",
            Green.paint("Muted:"),
            Red.paint(muted.to_string())
        );
    }
}

async fn run_app(device: &CastDevice<'_>, app_to_run: &CastDeviceApp) {
    let app = device.receiver.launch_app(app_to_run).await.unwrap();

    println!(
        "{}{}{}{}{}{}{}",
        Green.paint("The following application has been run: "),
        Red.paint(app.display_name),
        Red.paint(" ("),
        Red.paint(app.app_id),
        Red.paint(")"),
        Red.paint(" - "),
        Red.paint(app.status_text)
    );
}

async fn stop_app(device: &CastDevice<'_>, app_to_run: &CastDeviceApp) {
    let status = device.receiver.get_status().await.unwrap();

    let app = status
        .applications
        .iter()
        .find(|app| &CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_run);

    match app {
        Some(app) => {
            device
                .receiver
                .stop_app(app.session_id.as_str())
                .await
                .unwrap();

            println!(
                "{}{}{}{}{}{}{}",
                Green.paint("The following application has been stopped: "),
                Red.paint(app.display_name.as_str()),
                Red.paint(" ("),
                Red.paint(app.app_id.as_str()),
                Red.paint(")"),
                Red.paint(" - "),
                Red.paint(app.status_text.as_str())
            );
        }
        None => {
            println!(
                "{} `{}` {}",
                Green.paint("Application"),
                Red.paint(app_to_run.to_string()),
                Green.paint("is not run!")
            );
        }
    }
}

async fn stop_current_app(device: &CastDevice<'_>) {
    let status = device.receiver.get_status().await.unwrap();
    match status.applications.first() {
        Some(app) => {
            device
                .receiver
                .stop_app(app.session_id.as_str())
                .await
                .unwrap();

            println!(
                "{}{}{}{}{}{}{}",
                Green.paint("The following application has been stopped: "),
                Red.paint(app.display_name.as_str()),
                Red.paint(" ("),
                Red.paint(app.app_id.as_str()),
                Red.paint(")"),
                Red.paint(" - "),
                Red.paint(app.status_text.as_str())
            );
        }
        None => println!("{}", Green.paint("There are no applications active!")),
    }
}

async fn play_media(
    device: &CastDevice<'_>,
    app_to_run: &CastDeviceApp,
    media: String,
    media_type: String,
    media_stream_type: StreamType,
) {
    let app = device.receiver.launch_app(app_to_run).await.unwrap();

    device
        .connection
        .connect(app.transport_id.as_str())
        .await
        .unwrap();

    let status = device
        .media
        .load(
            app.transport_id.as_str(),
            app.session_id.as_str(),
            &Media {
                content_id: media,
                content_type: media_type,
                stream_type: media_stream_type,
                duration: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    for i in 0..status.entries.len() {
        println!(
            "{}{}{}",
            Green.paint("Media#"),
            Green.paint(i.to_string()),
            Green.paint(": ")
        );
        println!(
            "{} {}",
            Green.paint("Playback rate:"),
            Red.paint(status.entries[i].playback_rate.to_string())
        );
        println!(
            "{} {}",
            Green.paint("Player state:"),
            Red.paint(status.entries[i].player_state.to_string())
        );

        if let Some(time) = status.entries[i].current_time {
            println!(
                "{} {}",
                Green.paint("Current time:"),
                Red.paint(time.to_string())
            );
        }

        if let Some(ref media) = status.entries[i].media {
            println!(
                "{} {}",
                Green.paint("Content Id:"),
                Red.paint(media.content_id.as_str())
            );
            println!(
                "{} {}",
                Green.paint("Stream type:"),
                Red.paint(media.stream_type.to_string())
            );
            println!(
                "{} {}",
                Green.paint("Content type:"),
                Red.paint(media.content_type.as_str())
            );

            if let Some(duration) = media.duration {
                println!(
                    "{} {}",
                    Green.paint("Duration:"),
                    Red.paint(duration.to_string())
                );
            }
        }
    }
}

async fn discover() -> Option<(String, u16)> {
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon.");

    let receiver = mdns
        .browse(SERVICE_TYPE)
        .expect("Failed to browse mDNS services.");

    while let Ok(event) = receiver.recv_async().await {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                let mut addresses = info
                    .get_addresses()
                    .iter()
                    .map(|address| address.to_string())
                    .collect::<Vec<_>>();
                println!(
                    "{}{}",
                    Green.paint("Resolved a new service: "),
                    Red.paint(format!(
                        "{} ({})",
                        info.get_fullname(),
                        addresses.join(", ")
                    ))
                );

                // Based on mDNS crate code we should have at least one address available.
                return Some((addresses.remove(0), info.get_port()));
            }
            other_event => {
                println!(
                    "{}{}",
                    Green.paint("Received other service event: "),
                    Red.paint(format!("{:?}", other_event))
                );
            }
        }
    }
    None
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let (address, port) = match args.flag_address {
        Some(address) => (address, args.flag_port),
        None => {
            println!("Cast Device address is not specified, trying to discover...");
            discover().await.unwrap_or_else(|| {
                println!("No Cast device discovered, please specify device address explicitly.");
                std::process::exit(1);
            })
        }
    };

    let cast_device = match CastDevice::connect_without_host_verification(address, port).await {
        Ok(cast_device) => cast_device,
        Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
    };

    cast_device
        .connection
        .connect(DEFAULT_DESTINATION_ID.to_string())
        .await
        .unwrap();
    cast_device.heartbeat.ping().await.unwrap();

    // Information about cast device.
    if args.flag_info.is_some() {
        return print_info(&cast_device).await;
    }

    // Run specific application.
    if let Some(app) = args.flag_run {
        return run_app(&cast_device, &CastDeviceApp::from_str(&app).unwrap()).await;
    }

    // Stop specific application.
    if let Some(app) = args.flag_stop {
        return stop_app(&cast_device, &CastDeviceApp::from_str(&app).unwrap()).await;
    }

    // Stop currently active application.
    if args.flag_stop_current {
        return stop_current_app(&cast_device).await;
    }

    // Adjust volume level.
    if let Some(level) = args.flag_media_volume {
        let volume = cast_device.receiver.set_volume(level).await.unwrap();
        println!(
            "{}{}",
            Green.paint("Volume level has been set to: "),
            Red.paint(volume.level.unwrap_or(level).to_string())
        );
        return;
    }

    // Mute/unmute cast device.
    if args.flag_media_mute || args.flag_media_unmute {
        let mute_or_unmute = args.flag_media_mute;
        let volume = cast_device
            .receiver
            .set_volume(mute_or_unmute)
            .await
            .unwrap();
        println!(
            "{}{}",
            Green.paint("Cast device is muted: "),
            Red.paint(volume.muted.unwrap_or(mute_or_unmute).to_string())
        );
        return;
    }

    // Manage media session playback (play, pause, stop and seek).
    if args.flag_media_pause
        || args.flag_media_play
        || args.flag_media_stop
        || args.flag_media_seek.is_some()
    {
        let app_to_manage = CastDeviceApp::from_str(args.flag_media_app.as_str()).unwrap();
        let status = cast_device.receiver.get_status().await.unwrap();

        let app = status
            .applications
            .iter()
            .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

        match app {
            Some(app) => {
                cast_device
                    .connection
                    .connect(app.transport_id.as_str())
                    .await
                    .unwrap();

                let status = cast_device
                    .media
                    .get_status(app.transport_id.as_str(), None)
                    .await
                    .unwrap();
                let status = status.entries.first().unwrap();

                let mut status_entry: Option<StatusEntry> = None;

                if args.flag_media_pause {
                    status_entry = Some(
                        cast_device
                            .media
                            .pause(app.transport_id.as_str(), status.media_session_id)
                            .await
                            .unwrap(),
                    );
                } else if args.flag_media_play {
                    status_entry = Some(
                        cast_device
                            .media
                            .play(app.transport_id.as_str(), status.media_session_id)
                            .await
                            .unwrap(),
                    );
                } else if args.flag_media_stop {
                    status_entry = Some(
                        cast_device
                            .media
                            .stop(app.transport_id.as_str(), status.media_session_id)
                            .await
                            .unwrap(),
                    );
                } else if args.flag_media_seek.is_some() {
                    status_entry = Some(
                        cast_device
                            .media
                            .seek(
                                app.transport_id.as_str(),
                                status.media_session_id,
                                Some(args.flag_media_seek.unwrap()),
                                None,
                            )
                            .await
                            .unwrap(),
                    );
                }

                if let Some(status_entry) = status_entry {
                    println!("{}", Green.paint("Media:"));
                    println!(
                        "{} {}",
                        Green.paint("Playback rate:"),
                        Red.paint(status_entry.playback_rate.to_string())
                    );
                    println!(
                        "{} {}",
                        Green.paint("Player state:"),
                        Red.paint(status_entry.player_state.to_string())
                    );

                    if let Some(time) = status_entry.current_time {
                        println!(
                            "{} {}",
                            Green.paint("Current time:"),
                            Red.paint(time.to_string())
                        );
                    }

                    if let Some(ref media) = status_entry.media {
                        println!(
                            "{} {}",
                            Green.paint("Content Id:"),
                            Red.paint(media.content_id.as_str())
                        );
                        println!(
                            "{} {}",
                            Green.paint("Stream type:"),
                            Red.paint(media.stream_type.to_string())
                        );
                        println!(
                            "{} {}",
                            Green.paint("Content type:"),
                            Red.paint(media.content_type.as_str())
                        );

                        if let Some(duration) = media.duration {
                            println!(
                                "{} {}",
                                Green.paint("Duration:"),
                                Red.paint(duration.to_string())
                            );
                        }
                    }
                }
            }
            None => {
                println!(
                    "{} `{}` {}",
                    Green.paint("Application"),
                    Red.paint(app_to_manage.to_string()),
                    Green.paint("is not run!")
                );
            }
        }
        return;
    }

    // Play media and keep connection.
    if let Some(media) = args.flag_media {
        let media_type = args.flag_media_type.unwrap_or_default();

        let media_stream_type = match args.flag_media_stream_type.as_str() {
            value @ "buffered" | value @ "live" | value @ "none" => {
                StreamType::from_str(value).unwrap()
            }
            _ => panic!("Unsupported stream type {}!", args.flag_media_stream_type),
        };

        play_media(
            &cast_device,
            &CastDeviceApp::from_str(args.flag_media_app.as_str()).unwrap(),
            media,
            media_type,
            media_stream_type,
        )
        .await;

        loop {
            match cast_device.receive().await {
                Ok(ChannelMessage::Heartbeat(response)) => {
                    println!("[Heartbeat] {:?}", response);

                    if let HeartbeatResponse::Ping = response {
                        cast_device.heartbeat.pong().await.unwrap();
                    }
                }

                Ok(ChannelMessage::Connection(response)) => println!("[Connection] {:?}", response),
                Ok(ChannelMessage::Media(response)) => println!("[Media] {:?}", response),
                Ok(ChannelMessage::Receiver(response)) => println!("[Receiver] {:?}", response),
                Ok(ChannelMessage::Raw(response)) => println!(
                    "Support for the following message type is not yet supported: {:?}",
                    response
                ),

                Err(error) => error!("Error occurred while receiving message {}", error),
            }
        }
    }
}
