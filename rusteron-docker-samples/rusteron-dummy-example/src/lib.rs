pub mod model;

use crate::model::Subscribe;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use rusteron_archive::*;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};
use websocket_lite::{ClientBuilder, Message, Opcode};

pub const TICKER_CHANNEL: &str = "aeron:udp?endpoint=localhost:9123";
pub const TICKER_STREAM_ID: i32 = 10;

pub trait JsonMesssageHandler {
    fn on_msg(&mut self, msg: &str);
}

pub async fn download_ws(
    url: &str,
    subscription: Subscribe,
    mut handler: impl JsonMesssageHandler,
) -> websocket_lite::Result<()> {
    loop {
        let mut client = ClientBuilder::new(url)?.async_connect().await?;
        let request = Message::text(serde_json::to_string(&subscription)?);
        info!("{url} sending request: {:#?}", subscription);
        if let Err(e) = client.send(request).await {
            info!("error sending websocket msg: {}", e);
            continue;
        }
        while let Some(msg) = client.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    info!("Error while receiving message: {:?}", e);
                    break;
                }
            };
            match msg.opcode() {
                Opcode::Text => handler.on_msg(msg.as_text().expect("should be text message")),
                Opcode::Binary => {
                    error!("unsupported binary format");
                }
                Opcode::Close => {
                    info!("closed");
                    break;
                }
                Opcode::Ping => {}
                Opcode::Pong => {}
            }
        }
    }
}

pub fn init_logger() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init()
}

pub fn archive_connect() -> Result<(AeronArchive, Aeron), io::Error> {
    let start = Instant::now();
    let signal_consumer =
        Handler::leak(crate::AeronArchiveRecordingSignalConsumerFuncClosure::from(
            |signal: AeronArchiveRecordingSignal| {
                info!("Recording signal received: {:?}", signal);
            },
        ));

    let error_handler = Handler::leak(crate::AeronErrorHandlerClosure::from(|code, msg| {
        error!("err code: {}, msg: {}", code, msg);
    }));

    while start.elapsed() < Duration::from_secs(30) {
        match AeronContext::new() {
            Ok(aeron_context) => match Aeron::new(&aeron_context) {
                Ok(aeron) => match aeron.start() {
                    Ok(_) => {
                        info!(
                            "Successfully connected to aeron client [{}]",
                            aeron.version_full()
                        );
                        let archive_context = AeronArchiveContext::new_with_no_credentials_supplier(
                                    &aeron,
                                    &std::env::var("AERON_ARCHIVE_CONTROL_CHANNEL").expect("missing environment variable AERON_ARCHIVE_CONTROL_CHANNEL"),
                                    &std::env::var("AERON_ARCHIVE_CONTROL_RESPONSE_CHANNEL").expect("missing environment variable AERON_ARCHIVE_CONTROL_RESPONSE_CHANNEL"),
                                    &std::env::var("AERON_ARCHIVE_REPLICATION_CHANNEL").expect("missing environment variable AERON_ARCHIVE_REPLICATION_CHANNEL"),
                                ).expect("error creating archive context");

                        archive_context
                            .set_recording_signal_consumer(Some(&signal_consumer))
                            .expect("Failed to set recording signal consumer");
                        archive_context
                            .set_error_handler(Some(&error_handler))
                            .expect("unable to set error handler");
                        archive_context
                            .set_idle_strategy(Some(&Handler::leak(
                                AeronIdleStrategyFuncClosure::from(|_work_count| {}),
                            )))
                            .expect("unable to set idle strategy");
                        if let Ok(connect) = AeronArchiveAsyncConnect::new(&archive_context) {
                            match connect.poll_blocking(Duration::from_secs(10)) {
                                Ok(archive) => {
                                    let i = archive.get_archive_id();
                                    assert!(i > 0);
                                    info!("aeron archive media driver is up [connected with archive id {i}]");
                                    return Ok((archive, aeron));
                                }
                                Err(e) => {
                                    error!("Failed to poll and connect to Aeron archive: {:?}", e);
                                }
                            }
                        } else {
                            error!(
                                "Failed to create AeronArchiveAsyncConnect with the given context"
                            );
                        }
                    }
                    Err(e) => {
                        error!("error creating archive context: {:?}", e);
                        error!("aeron error: {}", aeron.errmsg());
                    }
                },
                Err(e) => {
                    error!(
                        "error creating aeron client [aeron_dir={:?}, error={:?}]",
                        aeron_context.get_dir(),
                        e
                    );

                    if let Ok(entries) = std::fs::read_dir("/dev/shm") {
                        info!("/dev/shm has {} files", entries.count());
                    } else {
                        error!("Unable to read directory /dev/shm");
                    }
                }
            },
            Err(e) => {
                error!("error creating aeron context: {:?}", e);
            }
        }
        info!("waiting for aeron to start up, retrying...");
    }

    assert!(
        start.elapsed() < Duration::from_secs(60),
        "failed to start up aeron media driver"
    );

    Err(std::io::Error::other(
        "unable to start up aeron media driver client",
    ))
}
