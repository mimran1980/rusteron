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
    use log::{error, info};

    use crate::testing::EmbeddedArchiveMediaDriverProcess;
    use serial_test::serial;
    use std::cell::Cell;
    use std::error;
    use std::error::Error;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    pub const ARCHIVE_CONTROL_REQUEST: &str = "aeron:udp?endpoint=localhost:8010";
    pub const ARCHIVE_CONTROL_RESPONSE: &str = "aeron:udp?endpoint=localhost:8011";
    pub const ARCHIVE_RECORDING_EVENTS: &str = "aeron:udp?endpoint=localhost:8012";

    pub const EXAMPLE_LIVE_CHANNEL: &str = "aeron:udp?endpoint=localhost:8020";
    pub const EPHEMERAL_CHANNEL: &str = "aeron:udp?endpoint=localhost:0";

    #[test]
    #[serial]
    fn test_simple_replay_merge() -> Result<(), AeronCError> {
        env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .init();
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
        // important that you have aeron and archive, else you get segfault if you try to use archive if aeron is close
        let (archive, aeron) = media_driver
            .archive_connect()
            .expect("Could not connect to archive client");

        info!("connected to archive");
        assert!(!aeron.is_closed());

        let live_uri = EXAMPLE_LIVE_CHANNEL;
        let replay_uri = EPHEMERAL_CHANNEL;
        let stream_id = 1001;

        while archive
            .start_recording(live_uri, stream_id, SOURCE_LOCATION_LOCAL, true)
            .unwrap_or(-1)
            < 0
        {
            error!("failed to start recording");
            if let Some(err) = archive.poll_for_error() {
                panic!("{}", err);
            }
            sleep(Duration::from_millis(100));
        }
        info!("asked archiver to record {}:{}", live_uri, stream_id);

        // Setup publisher
        let publication = aeron
            .add_publication(&live_uri, stream_id, Duration::from_secs(5))
            .expect("Failed to create publication");
        while !publication.is_connected() {
            thread::sleep(Duration::from_millis(100));
        }

        // Spawn a thread to simulate the publisher
        let publisher_thread = thread::spawn(move || {
            let mut i = 0;
            loop {
                let message = format!("price update: {}", i);
                while publication.offer(
                    message.as_bytes(),
                    Handlers::no_reserved_value_supplier_handler(),
                ) <= 0
                {
                    thread::sleep(Duration::from_micros(10));
                }
                i += 1;
                thread::sleep(Duration::from_millis(100));
                info!("offer price update: {}", i);
            }
        });

        // TODO add reply-merge subscription here
        info!("publisher thread started");

        let recording_id = Cell::new(-1);
        let list_recording_handler = Handler::leak(
            crate::AeronArchiveRecordingDescriptorConsumerFuncClosure::from(
                |d: AeronArchiveRecordingDescriptor| {
                    info!("descriptor {:?}", d);
                    recording_id.set(d.recording_id);
                },
            ),
        );
        while recording_id.get() < 0 {
            info!(
                "list_recordings_for_uri {:?}",
                archive.list_recordings_for_uri(
                    0,
                    1,
                    live_uri,
                    stream_id,
                    Some(&list_recording_handler)
                )
            );
            if let Some(e) = archive.poll_for_error() {
                panic!("{}", e);
            }
        }

        let replay_subscription = aeron.add_subscription(
            replay_uri,
            stream_id,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
            Duration::from_secs(5),
        )?;

        // change from empheme port to real port
        let replay_uri = replay_subscription.try_resolve_channel_endpoint_uri()?;
        info!("replay uri: {}", replay_uri);

        let replay_session_id = archive.start_replay(
            recording_id.get(),
            &replay_uri,
            stream_id,
            &AeronArchiveReplayParams::new(0, i32::MAX, 0, i64::MAX, 0, 0)?,
        )?;
        let session_id = replay_session_id as i32;

        info!("replay session id {}", replay_session_id);
        info!("session id {}", session_id);
        let channel_replay = format!("{}?session-id={}", replay_uri, session_id);
        info!("archive id: {}", archive.get_archive_id());
        info!("channel replay: {}", channel_replay);
        let subscription = aeron
            .async_add_subscription(
                &channel_replay,
                stream_id,
                Some(&Handler::leak(AeronAvailableImageLogger)),
                Some(&Handler::leak(AeronUnavailableImageLogger)),
            )?
            .poll_blocking(Duration::from_secs(10))?;

        let replay_thread = thread::spawn(move || {
            let handler = Handler::leak(crate::AeronFragmentHandlerClosure::from(
                |buffer: Vec<u8>, header: AeronHeader| {
                    let message = String::from_utf8_lossy(buffer.as_slice());
                    info!("Replayed message: {}", message);
                },
            ));
            // Simulate replaying last 24 hours of data
            for _ in 0..1000 {
                while subscription
                    .poll(Some(&handler), 10)
                    .expect("Failed to poll fragments")
                    <= 0
                {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            let handler = Handler::leak(crate::AeronFragmentHandlerClosure::from(
                |buffer: Vec<u8>, header: AeronHeader| {
                    let message = String::from_utf8_lossy(buffer.as_slice());
                    info!("Live message: {}", message);
                },
            ));

            // Merge into the live stream
            for _ in 0..100 {
                while subscription
                    .poll(Some(&handler), 10)
                    .expect("Failed to poll fragments")
                    <= 0
                {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });

        publisher_thread.join().expect("Publisher thread failed");
        replay_thread.join().expect("Replay thread failed");

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
    //         .filter_level(log::LevelFilter::Info)
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

    #[test]
    fn test_replay_merge() -> Result<(), AeronCError> {
        env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .init();
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

        let _embedded_driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
            &aeron_dir,
            &format!("{}/archive", aeron_dir),
            &request_control_channel,
            &response_control_channel,
            &recording_events_channel,
        )
        .expect("Failed to start embedded media driver");

        // important that you have aeron and archive, else you get segfault if you try to use archive if aeron is close
        let (archive, aeron) = _embedded_driver
            .archive_connect()
            .expect("Could not connect to archive client");

        assert!(!aeron.is_closed());

        let archive_uri = ChannelUriBuilder::new()
            .media("udp")
            .control_endpoint("localhost:40123")
            .control_mode("manual")
            .endpoint("localhost:40234")
            .add_param("reliable", "true")
            .build()
            .expect("Failed to build URI");
        let live_uri = "aeron:udp?endpoint=localhost:40124";
        let replay_uri = "aeron:udp?endpoint=localhost:40125";
        let stream_id = 1001;

        archive.start_recording(&archive_uri, stream_id, SOURCE_LOCATION_LOCAL, true)?;
        info!("asked archiver to record {}:{}", archive_uri, stream_id);

        // Setup publisher
        let publication = aeron
            .add_publication(&archive_uri, stream_id, Duration::from_secs(5))
            .expect("Failed to create publication");

        // Spawn a thread to simulate the publisher
        let publisher_thread = thread::spawn(move || {
            while !publication.is_connected() {
                thread::sleep(Duration::from_millis(100));
            }
            let mut i = 0;
            loop {
                let message = format!("price update: {}", i);
                while publication.offer(
                    message.as_bytes(),
                    Handlers::no_reserved_value_supplier_handler(),
                ) <= 0
                {
                    thread::sleep(Duration::from_micros(10));
                }
                i += 1;
                thread::sleep(Duration::from_millis(10));
                // info!("offer price update: {}", i);
            }
        });

        // Setup replay subscription
        let mut subscription = aeron
            .async_add_subscription(
                &archive_uri,
                stream_id,
                Handlers::no_available_image_handler(),
                Handlers::no_unavailable_image_handler(),
            )
            .expect("Failed to create subscription")
            .poll_blocking(Duration::from_secs(5))
            .expect("Subscription not available");

        // Add replay and live destinations to the subscription
        subscription
            .add_destination(&aeron, replay_uri, Duration::from_secs(5))
            .expect("Failed to add replay destination");
        subscription
            .add_destination(&aeron, live_uri, Duration::from_secs(5))
            .expect("Failed to add live destination");

        let replay_thread = thread::spawn(move || {
            let handler = Handler::leak(crate::AeronFragmentHandlerClosure::from(
                |buffer: Vec<u8>, header: AeronHeader| {
                    let message = String::from_utf8_lossy(buffer.as_slice());
                    info!("Replayed message: {}", message);
                },
            ));
            // Simulate replaying last 24 hours of data
            for _ in 0..100 {
                while subscription
                    .poll(Some(&handler), 10)
                    .expect("Failed to poll fragments")
                    <= 0
                {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            let handler = Handler::leak(crate::AeronFragmentHandlerClosure::from(
                |buffer: Vec<u8>, header: AeronHeader| {
                    let message = String::from_utf8_lossy(buffer.as_slice());
                    info!("Live message: {}", message);
                },
            ));

            // Merge into the live stream
            for _ in 0..100 {
                while subscription
                    .poll(Some(&handler), 10)
                    .expect("Failed to poll fragments")
                    <= 0
                {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });

        publisher_thread.join().expect("Publisher thread failed");
        replay_thread.join().expect("Replay thread failed");

        Ok(())
    }

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
        env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .init();
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
            info!("sent message {i}");
        }

        // since this is single threaded need to make sure it did write to archiver, usually not required in multi-proccess app
        let stop_position = publication.position();
        info!(
            "publication stop position {} [publication={:?}]",
            stop_position,
            publication.get_constants()
        );
        let counters_reader = aeron.counters_reader();
        let session_id = publication.get_constants()?.session_id;
        let counter_id = RecordingPos::find_counter_id_by_session(&counters_reader, session_id);

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
