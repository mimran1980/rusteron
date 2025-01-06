/**/
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![doc = include_str!("../README.md")]
//! # Features
//!
//! - **`static`**: When enabled, this feature statically links the Aeron C code.
//!   By default, the library uses dynamic linking to the Aeron C libraries.

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;
use std::ffi::{c_char, CStr};

pub mod testing;

include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/aeron_custom.rs"));
// include!(concat!(env!("OUT_DIR"), "/rb_custom.rs"));

pub type SourceLocation = bindings::aeron_archive_source_location_t;
pub const SOURCE_LOCATION_LOCAL: aeron_archive_source_location_en =
    SourceLocation::AERON_ARCHIVE_SOURCE_LOCATION_LOCAL;
pub const SOURCE_LOCATION_REMOTE: aeron_archive_source_location_en =
    SourceLocation::AERON_ARCHIVE_SOURCE_LOCATION_REMOTE;

pub struct RecordingPos;
impl RecordingPos {
    pub fn find_counter_id_by_session(
        counter_reader: &AeronCountersReader,
        session_id: i32,
    ) -> i32 {
        unsafe {
            aeron_archive_recording_pos_find_counter_id_by_session_id(
                counter_reader.get_inner(),
                session_id,
            )
        }
    }
    pub fn find_counter_id_by_recording_id(
        counter_reader: &AeronCountersReader,
        recording_id: i64,
    ) -> i32 {
        unsafe {
            aeron_archive_recording_pos_find_counter_id_by_recording_id(
                counter_reader.get_inner(),
                recording_id,
            )
        }
    }
}

unsafe extern "C" fn default_encoded_credentials(
    _clientd: *mut std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    // Allocate a zeroed instance of `aeron_archive_encoded_credentials_t`
    let empty_credentials = Box::new(aeron_archive_encoded_credentials_t {
        data: ptr::null(),
        length: 0,
    });
    Box::into_raw(empty_credentials)
}

impl AeronArchive {
    pub fn aeron(&self) -> Aeron {
        self.get_archive_context().get_aeron()
    }
}

impl AeronArchiveContext {
    // The method below sets no credentials supplier, which is essential for the operation
    // of the Aeron Archive Context. The `set_credentials_supplier` must be set to prevent
    // segmentation faults in the C bindings.
    pub fn set_no_credentials_supplier(&self) -> Result<i32, AeronCError> {
        self.set_credentials_supplier(
            Some(default_encoded_credentials),
            None,
            None::<&Handler<AeronArchiveCredentialsFreeFuncLogger>>,
        )
    }

    /// This method creates a new `AeronArchiveContext` with a no-op credentials supplier.
    /// If you do not set a credentials supplier, it will segfault.
    /// This method ensures that a non-functional credentials supplier is set to avoid the segfault.
    pub fn new_with_no_credentials_supplier(
        aeron: &Aeron,
        request_control_channel: &str,
        response_control_channel: &str,
        recording_events_channel: &str,
    ) -> Result<AeronArchiveContext, AeronCError> {
        let context = Self::new()?;
        context.set_no_credentials_supplier()?;
        context.set_aeron(aeron)?;
        context.set_control_request_channel(request_control_channel)?;
        context.set_control_response_channel(response_control_channel)?;
        context.set_recording_events_channel(recording_events_channel)?;
        Ok(context)
    }
}

impl AeronArchive {
    pub fn poll_for_error(&self) -> Option<String> {
        let mut buffer: Vec<u8> = vec![0; 100];
        let raw_ptr: *mut c_char = buffer.as_mut_ptr() as *mut c_char;
        let len = self.poll_for_error_response(raw_ptr, 100).ok()?;
        if len >= 0 {
            unsafe {
                let result = CStr::from_ptr(raw_ptr).to_string_lossy().to_string();
                if result.is_empty() {
                    None
                } else {
                    Some(result)
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, info};

    use crate::testing::EmbeddedArchiveMediaDriverProcess;
    use serial_test::serial;
    use std::cell::Cell;
    use std::error;
    use std::error::Error;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    pub const ARCHIVE_CONTROL_REQUEST: &str = "aeron:udp?endpoint=localhost:8010";
    pub const ARCHIVE_CONTROL_RESPONSE: &str = "aeron:udp?endpoint=localhost:8011";
    pub const ARCHIVE_RECORDING_EVENTS: &str =
        "aeron:udp?control-mode=dynamic|control=localhost:8012";

    #[test]
    #[serial]
    #[ignore] // TODO need to fix test, doesn't receive any response back
    fn test_simple_replay_merge() -> Result<(), AeronCError> {
        pub const STREAM_ID: i32 = 1033;
        pub const MESSAGE_PREFIX: &str = "Message-Prefix-";
        pub const CONTROL_ENDPOINT: &str = "localhost:23265";
        pub const RECORDING_ENDPOINT: &str = "localhost:23266";
        pub const LIVE_ENDPOINT: &str = "localhost:23267";
        // pub const REPLAY_ENDPOINT: &str = "localhost:0";
        pub const REPLAY_ENDPOINT: &str = "localhost:23268";

        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        assert!(is_udp_port_available(23265));
        assert!(is_udp_port_available(23266));
        assert!(is_udp_port_available(23267));
        assert!(is_udp_port_available(23268));

        EmbeddedArchiveMediaDriverProcess::kill_all_java_processes()
            .expect("failed to kill all java processes");
        let id = Aeron::nano_clock();
        let aeron_dir = format!("target/aeron/{}/shm", id);
        let archive_dir = format!("target/aeron/{}/archive", id);

        info!("starting archive media driver");
        let media_driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
            &aeron_dir,
            &format!("{}/archive", aeron_dir),
            ARCHIVE_CONTROL_REQUEST,
            ARCHIVE_CONTROL_RESPONSE,
            ARCHIVE_RECORDING_EVENTS,
        )
        .expect("Failed to start embedded media driver");

        info!("connecting to archive");
        let (archive, aeron) = media_driver
            .archive_connect()
            .expect("Could not connect to archive client");

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        info!("connected to archive, adding publication");
        assert!(!aeron.is_closed());

        let publication = aeron.add_publication(
            &format!("aeron:udp?control={CONTROL_ENDPOINT}|control-mode=dynamic|term-length=65536|fc=tagged,g:99901/1,t:5s"),
            STREAM_ID,
            Duration::from_secs(5),
        )?;

        info!(
            "publication {} [status={:?}]",
            publication.channel(),
            publication.channel_status()
        );
        assert_eq!(1, publication.channel_status());

        let session_id = publication.session_id();
        let recording_channel = format!(
            "aeron:udp?endpoint={RECORDING_ENDPOINT}|control={CONTROL_ENDPOINT}|session-id={session_id}|gtag=99901"
        );
        info!("recording channel {}", recording_channel);
        archive.start_recording(&recording_channel, STREAM_ID, SOURCE_LOCATION_REMOTE, true)?;

        info!("waiting for publisher to be connected");
        while !publication.is_connected() {
            thread::sleep(Duration::from_millis(100));
        }
        info!("publisher to be connected");
        let counters_reader = aeron.counters_reader();
        let publisher_thread = thread::spawn(move || {
            let mut message_count = 0;
            let mut counter_id = -1;
            let session_id = publication.get_constants().unwrap().session_id;

            while running.load(Ordering::Acquire) {
                let message = format!("{}{}", MESSAGE_PREFIX, message_count);
                while publication.offer(
                    message.as_bytes(),
                    Handlers::no_reserved_value_supplier_handler(),
                ) <= 0
                {
                    thread::sleep(Duration::from_millis(10));
                }
                message_count += 1;
                if message_count % 10_000 == 0 {
                    info!(
                        "Published {} messages [position={}]",
                        message_count,
                        publication.position()
                    );
                }
                // slow down publishing so can catch up
                if message_count > 100_000 {
                    while counter_id < 0 {
                        counter_id =
                            RecordingPos::find_counter_id_by_session(&counters_reader, session_id);
                    }

                    // ensure archiver is caught up
                    while counters_reader.get_counter_value(counter_id) < publication.position() {
                        thread::sleep(Duration::from_micros(1));
                    }
                    thread::sleep(Duration::from_micros(100));
                }
            }
            info!("Publisher thread terminated");
        });

        sleep(Duration::from_secs(5));

        let replay_channel = format!("aeron:udp?session-id={session_id}");
        info!("replay channel {}", replay_channel);

        let replay_destination = format!("aeron:udp?endpoint={REPLAY_ENDPOINT}");
        info!("replay destination {}", replay_destination);

        let live_destination =
            format!("aeron:udp?endpoint={LIVE_ENDPOINT}|control={CONTROL_ENDPOINT}");
        info!("live destination {}", live_destination);

        let counters_reader = aeron.counters_reader();
        let mut counter_id = -1;

        while counter_id < 0 {
            counter_id = RecordingPos::find_counter_id_by_session(&counters_reader, session_id);
        }
        info!(
            "counter id {} {:?}",
            counter_id,
            counters_reader.get_counter_label(counter_id, 1024)
        );
        info!(
            "counter id {} position={:?}",
            counter_id,
            counters_reader.get_counter_value(counter_id)
        );

        let recording_id = Cell::new(-1);
        let start_position = Cell::new(-1);

        let list_recordings_handler = Handler::leak(
            crate::AeronArchiveRecordingDescriptorConsumerFuncClosure::from(
                |descriptor: AeronArchiveRecordingDescriptor| {
                    info!("Recording descriptor: {:?}", descriptor);
                    recording_id.set(descriptor.recording_id);
                    start_position.set(descriptor.start_position);
                    assert_eq!(descriptor.session_id, session_id);
                },
            ),
        );
        assert!(archive.list_recordings(0, 1000, Some(&list_recordings_handler))? > 0);
        assert!(recording_id.get() >= 0);

        let recording_id = recording_id.get();
        let start_position = start_position.get();

        let subscribe_channel = format!("aeron:udp?control-mode=manual|session-id={session_id}");
        info!("subscribe channel {}", subscribe_channel);
        let subscription = aeron.add_subscription(
            &subscribe_channel,
            STREAM_ID,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
            Duration::from_secs(5),
        )?;

        // subscription.add_destination(&aeron, &replay_destination, Duration::from_secs(5))?;
        // subscription.add_destination(&aeron, &live_destination, Duration::from_secs(5))?;

        let replay_merge = AeronArchiveReplayMerge::new(
            &subscription,
            &archive,
            &replay_channel,
            &replay_destination,
            &live_destination,
            recording_id,
            start_position,
            Aeron::nano_clock(),
            10_000, // merge progress timeout
        )?;

        info!(
            "ReplayMerge initialization: recordingId={}, startPosition={}, replayChannel={}, replayDestination={}, liveDestination={}",
            recording_id,
            start_position,
            replay_channel,
            replay_destination,
            live_destination
        );

        // media_driver
        //     .run_aeron_stats()
        //     .expect("Failed to run aeron stats");

        // info!("Waiting for subscription to connect...");
        // while !subscription.is_connected() {
        //     thread::sleep(Duration::from_millis(100));
        // }
        // info!("Subscription connected");

        // let (handler,closure) = Handler::leak_with_fragment_assembler(crate::AeronFragmentHandlerClosure::from(
        let handler = Handler::leak(crate::AeronFragmentHandlerClosure::from(
            |buffer: Vec<u8>, header: AeronHeader| {
                let message = String::from_utf8_lossy(buffer.as_slice());
                info!("Replayed message: {}", message);
            },
        ));

        info!("about to start_replay");
        let params = AeronArchiveReplayParams::new(
            0,
            0,
            0,
            i64::MAX,
            0,
            subscription.get_constants()?.registration_id,
        )?;
        let replay_session_id =
            archive.start_replay(recording_id, &replay_destination, STREAM_ID, &params)?;
        info!("replay session id {}", replay_session_id);
        info!("replay session id {}", replay_session_id as i32);
        // aeron.add_subscription(&format!("{replay_destination}|session-id={}", replay_session_id as i32), STREAM_ID,
        // Handlers::no_available_image_handler(),
        // Handlers::no_unavailable_image_handler(),
        // Duration::from_secs(5))?;
        assert!(replay_session_id > 0, "Replay failed to start");
        if let Some(err) = archive.poll_for_error() {
            panic!("{}", err);
        }
        if aeron.errmsg().len() > 0 && "no error" != aeron.errmsg() {
            panic!("{}", aeron.errmsg());
        }

        while !replay_merge.is_merged() {
            debug!(
                "ReplayMerge state: image={:?}, is_live_added={} is_merged={} has_failed={}",
                replay_merge.image(),
                replay_merge.is_live_added(),
                replay_merge.is_merged(),
                replay_merge.has_failed()
            );
            assert!(!replay_merge.has_failed());
            if replay_merge.poll(Some(&handler), 1000)? == 0 {
                if let Some(err) = archive.poll_for_error() {
                    panic!("{}", err);
                }
                if aeron.errmsg().len() > 0 && "no error" != aeron.errmsg() {
                    panic!("{}", aeron.errmsg());
                }
                archive.poll_for_recording_signals()?;
                thread::sleep(Duration::from_millis(100));
            } else {
                panic!("yes it worked!!!!")
            }
        }
        assert!(!replay_merge.has_failed());

        running_clone.store(false, Ordering::Release);
        publisher_thread.join().unwrap();

        Ok(())
    }

    #[test]
    fn version_check() {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);

        let cargo_version = "1.47.0";
        assert_eq!(aeron_version, cargo_version);
    }

    // #[test]
    // #[serial]
    // pub fn test_failed_connect() -> Result<(), Box<dyn error::Error>> {
    //         env_logger::Builder::new()
    //         .is_test(true)
    //         .filter_level(log::LevelFilter::Debug)
    //         .init();
    //     let ctx = AeronArchiveContext::new()?;
    //     std::env::set_var("AERON_DRIVER_TIMEOUT", "1");
    //     let connect = AeronArchiveAsyncConnect::new(&ctx);
    //     std::env::remove_var("AERON_DRIVER_TIMEOUT");
    //
    //     assert_eq!(
    //         Some(AeronErrorType::NullOrNotConnected.into()),
    //         connect.err()
    //     );
    //     Ok(())
    // }

    use std::thread;

    pub fn start_aeron_archive() -> Result<
        (
            Aeron,
            AeronArchiveContext,
            EmbeddedArchiveMediaDriverProcess,
        ),
        Box<dyn Error>,
    > {
        let id = Aeron::nano_clock();
        let aeron_dir = format!("target/aeron/{}/shm", id);
        let archive_dir = format!("target/aeron/{}/archive", id);

        let request_port = find_unused_udp_port(8000).expect("Could not find port");
        let response_port = find_unused_udp_port(request_port + 1).expect("Could not find port");
        let recording_event_port =
            find_unused_udp_port(response_port + 1).expect("Could not find port");
        let request_control_channel = &format!("aeron:udp?endpoint=localhost:{}", request_port);
        let response_control_channel = &format!("aeron:udp?endpoint=localhost:{}", response_port);
        let recording_events_channel =
            &format!("aeron:udp?endpoint=localhost:{}", recording_event_port);
        assert_ne!(request_control_channel, response_control_channel);

        let archive_media_driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
            &aeron_dir,
            &archive_dir,
            request_control_channel,
            response_control_channel,
            recording_events_channel,
        )
        .expect("Failed to start Java process");

        let aeron_context = AeronContext::new()?;
        aeron_context.set_dir(&aeron_dir)?;
        aeron_context.set_client_name("test")?;
        aeron_context.set_publication_error_frame_handler(Some(&Handler::leak(
            AeronPublicationErrorFrameHandlerLogger,
        )))?;
        let error_handler = Handler::leak(AeronErrorHandlerClosure::from(|error_code, msg| {
            panic!("error {} {}", error_code, msg)
        }));
        aeron_context.set_error_handler(Some(&error_handler))?;
        let aeron = Aeron::new(&aeron_context)?;
        aeron.start()?;

        let archive_context = AeronArchiveContext::new_with_no_credentials_supplier(
            &aeron,
            request_control_channel,
            response_control_channel,
            recording_events_channel,
        )?;
        archive_context.set_error_handler(Some(&error_handler))?;
        Ok((aeron, archive_context, archive_media_driver))
    }

    #[test]
    #[serial]
    pub fn test_aeron_archive() -> Result<(), Box<dyn error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        EmbeddedArchiveMediaDriverProcess::kill_all_java_processes()
            .expect("failed to kill all java processes");

        let (aeron, archive_context, media_driver) = start_aeron_archive()?;

        assert!(!aeron.is_closed());

        info!("connected to aeron");

        let archive_connector = AeronArchiveAsyncConnect::new(&archive_context.clone())?;
        let archive = archive_connector
            .poll_blocking(Duration::from_secs(30))
            .expect("failed to connect to aeron archive media driver");

        assert!(archive.get_archive_id() > 0);

        let channel = AERON_IPC_STREAM;
        let stream_id = 10;

        let subscription_id =
            archive.start_recording(channel, stream_id, SOURCE_LOCATION_LOCAL, true)?;

        assert!(subscription_id >= 0);
        info!("subscription id {}", subscription_id);

        let publication = aeron
            .async_add_exclusive_publication(channel, stream_id)?
            .poll_blocking(Duration::from_secs(5))?;

        for i in 0..11 {
            while publication.offer(
                "123456".as_bytes(),
                Handlers::no_reserved_value_supplier_handler(),
            ) <= 0
            {
                sleep(Duration::from_millis(50));
                archive.poll_for_recording_signals()?;
                if let Some(err) = archive.poll_for_error() {
                    panic!("{}", err);
                }
            }
            info!("sent message {i} [test_aeron_archive]");
        }

        let session_id = publication.get_constants()?.session_id;
        info!("publication session id {}", session_id);
        // since this is single threaded need to make sure it did write to archiver, usually not required in multi-proccess app
        let stop_position = publication.position();
        info!(
            "publication stop position {} [publication={:?}]",
            stop_position,
            publication.get_constants()
        );
        let counters_reader = aeron.counters_reader();
        info!("counters reader ready {:?}", counters_reader);

        let mut counter_id = -1;

        let start = Instant::now();
        while counter_id <= 0 && start.elapsed() < Duration::from_secs(5) {
            let counter_id = RecordingPos::find_counter_id_by_session(&counters_reader, session_id);
            info!("counter id {}", counter_id);
        }

        assert!(counter_id >= 0);

        info!("counter id {counter_id}, session id {session_id}");
        while counters_reader.get_counter_value(counter_id) < stop_position {
            info!(
                "current archive publication stop position {}",
                counters_reader.get_counter_value(counter_id)
            );
            sleep(Duration::from_millis(50));
        }
        info!(
            "found archive publication stop position {}",
            counters_reader.get_counter_value(counter_id)
        );

        archive.stop_recording_channel_and_stream(channel, stream_id)?;
        drop(publication);

        info!("list recordings");
        let found_recording_id = Cell::new(-1);
        let start_pos = Cell::new(-1);
        let end_pos = Cell::new(-1);
        let handler = Handler::leak(
            crate::AeronArchiveRecordingDescriptorConsumerFuncClosure::from(
                |d: AeronArchiveRecordingDescriptor| {
                    info!("found recording {:#?}", d);
                    if d.stop_position > d.start_position && d.stop_position > 0 {
                        found_recording_id.set(d.recording_id);
                        start_pos.set(d.start_position);
                        end_pos.set(d.stop_position);
                    }
                },
            ),
        );
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) && found_recording_id.get() == -1 {
            archive.list_recordings_for_uri(0, i32::MAX, channel, stream_id, Some(&handler))?;
            archive.poll_for_recording_signals()?;
            if let Some(err) = archive.poll_for_error() {
                panic!("{}", err);
            }
        }
        assert!(start.elapsed() < Duration::from_secs(5));
        info!("start replay");
        let params = AeronArchiveReplayParams::new(
            0,
            i32::MAX,
            start_pos.get(),
            end_pos.get() - start_pos.get(),
            0,
            0,
        )?;
        info!("replay params {:#?}", params);
        let replay_stream_id = 45;
        let replay_session_id =
            archive.start_replay(found_recording_id.get(), channel, replay_stream_id, &params)?;
        let session_id = replay_session_id as i32;

        info!("replay session id {}", replay_session_id);
        info!("session id {}", session_id);
        let channel_replay = format!("{}?session-id={}", channel, session_id);
        info!("archive id: {}", archive.get_archive_id());

        info!("add subscription {}", channel_replay);
        let subscription = aeron
            .async_add_subscription(
                &channel_replay,
                replay_stream_id,
                Some(&Handler::leak(AeronAvailableImageLogger)),
                Some(&Handler::leak(AeronUnavailableImageLogger)),
            )?
            .poll_blocking(Duration::from_secs(10))?;

        let count = Cell::new(0);
        let poll = Handler::leak(crate::AeronFragmentHandlerClosure::from(|msg, header| {
            assert_eq!(msg, "123456".as_bytes().to_vec());
            count.set(count.get() + 1);
        }));

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) && subscription.poll(Some(&poll), 100)? <= 0
        {
            let count = archive.poll_for_recording_signals()?;
            if let Some(err) = archive.poll_for_error() {
                panic!("{}", err);
            }
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "messages not received {count:?}"
        );
        info!("aeron {:?}", aeron);
        info!("ctx {:?}", archive_context);
        assert_eq!(11, count.get());
        Ok(())
    }
}
