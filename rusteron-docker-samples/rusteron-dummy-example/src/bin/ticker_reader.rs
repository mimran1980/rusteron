use log::{error, info};
use rusteron_archive::*;
use rusteron_dummy_example::{archive_connect, init_logger, TICKER_CHANNEL, TICKER_STREAM_ID};
use signal_hook::consts::*;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use websocket_lite::Result;

fn main() -> Result<()> {
    init_logger();

    let (archive, aeron) = archive_connect()?;

    let shutdown = register_exit_signals()?;

    let mut archive_log_time = Instant::now();
    let archive_log = Duration::from_secs(120);

    let mut live_log_time = Instant::now();
    let live_log = Duration::from_secs(30);

    let mut record_reader = Handler::leak(RecorderDescriptorReader::default());
    let mut replay_msg_count_handler = Handler::leak(MessageCountHandler::default());
    let mut live_msg_count_handler = Handler::leak(MessageCountHandler::default());

    let channel = TICKER_CHANNEL;
    let stream_id = TICKER_STREAM_ID;

    let mut live_subscription: Option<AeronSubscription> = None;

    while !shutdown.load(Ordering::Acquire) {
        if archive_log_time.elapsed() > archive_log {
            archive_log_time = Instant::now();
            record_reader.reset();
            match archive.list_recordings_for_uri(
                0,
                i32::MAX,
                channel,
                stream_id,
                Some(&record_reader),
            ) {
                Ok(recordings) => {
                    info!("found {recordings} recordings");

                    if let Some(record) = &record_reader.last_recording_with_stop_position {
                        let params = AeronArchiveReplayParams::new(
                            0,
                            i32::MAX,
                            record.start_position,
                            record.stop_position - record.start_position,
                            0,
                            0,
                        )?;

                        // change from ephemeral port to real port
                        let replay_channel = aeron
                            .add_subscription(
                                "aeron:udp?endpoint=localhost:0",
                                stream_id,
                                Handlers::no_available_image_handler(),
                                Handlers::no_unavailable_image_handler(),
                                Duration::from_secs(5),
                            )?
                            .try_resolve_channel_endpoint_uri()?;
                        info!("resolved replay channel: {}", replay_channel);

                        let replay_session_id = archive.start_replay(
                            record.recording_id,
                            &replay_channel,
                            stream_id,
                            &params,
                        )?;
                        let session_id = replay_session_id as i32;

                        let channel_replay = format!("{}?session-id={}", channel, session_id);
                        info!("replay subscription {}", channel_replay);
                        match aeron
                            .async_add_subscription(
                                &channel_replay,
                                stream_id,
                                Some(&Handler::leak(AeronAvailableImageLogger)),
                                Some(&Handler::leak(AeronUnavailableImageLogger)),
                            )?
                            .poll_blocking(Duration::from_secs(1))
                        {
                            Ok(subscription) => {
                                replay_msg_count_handler.reset();
                                let time = Instant::now();

                                while subscription
                                    .poll(Some(&replay_msg_count_handler), 1000)
                                    .is_ok()
                                {
                                    // prevent live sub from building up
                                    if let Some(live_subscription) = &live_subscription {
                                        let _ = live_subscription
                                            .poll(Some(&live_msg_count_handler), 1000);
                                    }
                                }

                                info!(
                                    "replay finished of last inactive recording [took={:?} {:?}]",
                                    time.elapsed(),
                                    *replay_msg_count_handler
                                );
                            }
                            Err(err) => {
                                error!("failed to subscribe to replay channel in time {err:?}");
                            }
                        }
                    }
                }
                Err(e) => {
                    // ideally should retry
                    error!("failed to read from aeron archiver {}", e);
                }
            }
        }

        if live_subscription.is_none() {
            live_subscription = aeron
                .add_subscription(
                    channel,
                    stream_id,
                    Handlers::no_available_image_handler(),
                    Handlers::no_unavailable_image_handler(),
                    Duration::from_millis(100),
                )
                .ok();
        }

        if let Some(live_subscription) = &live_subscription {
            let _ = live_subscription.poll(Some(&live_msg_count_handler), 1000);
        }

        if live_log_time.elapsed() > live_log {
            live_log_time = Instant::now();
            info!(
                "live channel sent {:?} since previous log",
                *live_msg_count_handler
            );
            live_msg_count_handler.reset();
        }
    }

    info!("shutting down");

    Ok(())
}

#[derive(Debug, Default)]
struct RecorderDescriptorReader {
    last_recording_with_stop_position: Option<AeronArchiveRecordingDescriptor>,
}

impl RecorderDescriptorReader {
    fn reset(&mut self) {
        self.last_recording_with_stop_position = None;
    }
}

impl AeronArchiveRecordingDescriptorConsumerFuncCallback for RecorderDescriptorReader {
    fn handle_aeron_archive_recording_descriptor_consumer_func(
        &mut self,
        recording_descriptor: AeronArchiveRecordingDescriptor,
    ) {
        info!("found recording {:?}", recording_descriptor);
        if recording_descriptor.stop_position > 0 {
            self.last_recording_with_stop_position = Some(recording_descriptor);
        }
    }
}

fn register_exit_signals() -> Result<Arc<AtomicBool>> {
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let signals = &[SIGINT, SIGTERM, SIGKILL, SIGQUIT];
    for &signal in signals {
        let flag_clone = Arc::clone(&shutdown_flag);
        signal_hook::flag::register(signal, flag_clone.clone())?;
    }

    Ok(shutdown_flag)
}

#[derive(Debug, Default)]
struct MessageCountHandler {
    count: usize,
    bytes: usize,
}

impl MessageCountHandler {
    fn reset(&mut self) {
        self.count = 0;
        self.bytes = 0;
    }
}

impl AeronFragmentHandlerCallback for MessageCountHandler {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], _header: AeronHeader) {
        self.count += 1;
        self.bytes += buffer.len();
    }
}
