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
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/aeron_custom.rs"));
include!(concat!(env!("OUT_DIR"), "/rb_custom.rs"));

unsafe extern "C" fn default_encoded_credentials(
    _clientd: *mut std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    // Allocate a zeroed instance of `aeron_archive_encoded_credentials_t`
    let empty_credentials = Box::new(aeron_archive_encoded_credentials_t {
        data: ptr::null(),
        length: 0,
    });
    // Convert it into a raw pointer to avoid double-free on drop
    Box::into_raw(empty_credentials)
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
    pub fn new_with_no_credentials_supplier() -> Result<AeronArchiveContext, AeronCError> {
        let context = Self::new()?;
        context.set_no_credentials_supplier()?;
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

    use serial_test::serial;
    use std::cell::Cell;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use std::{error, fs, io};

    #[test]
    fn version_check() {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0";
        assert_eq!(aeron_version, cargo_version);
    }

    #[test]
    #[serial]
    pub fn test_failed_connect() -> Result<(), Box<dyn error::Error>> {
        let ctx = AeronArchiveContext::new()?;
        std::env::set_var("AERON_DRIVER_TIMEOUT", "1");
        let connect = AeronArchiveAsyncConnect::new(&ctx);
        std::env::remove_var("AERON_DRIVER_TIMEOUT");

        assert_eq!(
            Some(AeronErrorType::NullOrNotConnected.into()),
            connect.err()
        );
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_aeron_archive() -> Result<(), Box<dyn error::Error>> {
        let id = Aeron::nano_clock();
        let aeron_dir = format!("target/aeron/{}/shm", id);
        let archive_dir = format!("target/aeron/{}/archive", id);

        let control_channel = "aeron:udp?endpoint=localhost:8010";

        let archive_media_driver =
            EmbeddedArchiveMediaDriverProcess::start(&aeron_dir, &archive_dir, control_channel)
                .expect("Failed to start Java process");

        let archive_context = AeronArchiveContext::new_with_no_credentials_supplier()?;
        let found_recording_signal = Cell::new(false);
        archive_context.set_recording_signal_consumer(Some(&Handler::leak(
            crate::AeronArchiveRecordingSignalConsumerFuncClosure::from(
                |signal: AeronArchiveRecordingSignal| {
                    println!("signal {:?}", signal);
                    found_recording_signal.set(true);
                },
            ),
        )))?;
        archive_context.set_owns_aeron_client(true)?;
        archive_context.set_idle_strategy(Some(&Handler::leak(
            AeronIdleStrategyFuncClosure::from(|work_count| {}),
        )))?;
        archive_context.set_control_request_channel(control_channel)?;
        let error_handler = Handler::leak(AeronErrorHandlerClosure::from(|error_code, msg| {
            panic!("error {} {}", error_code, msg)
        }));
        archive_context.set_error_handler(Some(&error_handler))?;

        let aeron_context = AeronContext::new()?;
        aeron_context.set_dir(&aeron_dir)?;
        aeron_context.set_client_name("test")?;
        aeron_context.set_publication_error_frame_handler(Some(&Handler::leak(
            AeronPublicationErrorFrameHandlerLogger,
        )))?;
        aeron_context.set_error_handler(Some(&error_handler))?;
        let aeron = Aeron::new(&aeron_context)?;
        aeron.start()?;
        println!("connected to aeron");

        archive_context.set_aeron(&aeron)?;
        let connect = AeronArchiveAsyncConnect::new(&archive_context.clone())?;
        let archive = connect.poll_blocking(Duration::from_secs(5))?;

        assert!(archive.get_archive_id() > 0);

        let channel = "aeron:ipc";
        // let channel = "aeron:udp?endpoint=localhost:40123";
        let stream_id = 10;

        let recording_id = archive.start_recording(
            channel,
            stream_id,
            aeron_archive_source_location_t::AERON_ARCHIVE_SOURCE_LOCATION_LOCAL,
            true,
        )?;
        println!("recording id {}", recording_id);

        let publication = aeron
            .async_add_exclusive_publication(channel, stream_id)?
            .poll_blocking(Duration::from_secs(5))?;

        let start = Instant::now();
        while !found_recording_signal.get() && start.elapsed().as_secs() < 5 {
            sleep(Duration::from_millis(100));
            archive.poll_for_recording_signals()?;
            if let Some(err) = archive.poll_for_error() {
                panic!("{}", err);
            }
        }
        assert!(start.elapsed().as_secs() < 5);

        for i in 0..10 {
            while !publication.is_connected()
                || publication.offer(
                    "123456".as_bytes(),
                    Handlers::no_reserved_value_supplier_handler(),
                ) <= 0
            {
                sleep(Duration::from_millis(100));
                archive.poll_for_recording_signals()?;
                if let Some(err) = archive.poll_for_error() {
                    panic!("{}", err);
                }
            }
        }
        archive.stop_recording_channel_and_stream(channel, stream_id)?;
        sleep(Duration::from_millis(100));
        drop(publication);

        println!("list recordings");
        let found_recording_id = Cell::new(0);
        let handler = Handler::leak(
            crate::AeronArchiveRecordingDescriptorConsumerFuncClosure::from(
                |d: AeronArchiveRecordingDescriptor| {
                    println!("found recording {:?}", d);
                    found_recording_id.set(d.recording_id);
                },
            ),
        );
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5)
            && found_recording_id.get() == 0
            && archive.list_recordings_for_uri(0, i32::MAX, channel, stream_id, Some(&handler))?
                <= 0
        {
            sleep(Duration::from_millis(100));
            archive.poll_for_recording_signals()?;
            if let Some(err) = archive.poll_for_error() {
                panic!("{}", err);
            }
        }
        assert!(start.elapsed() < Duration::from_secs(5));
        println!("start replay");
        let params = AeronArchiveReplayParams::new(0, i32::MAX, 0, i64::MAX, 0, 0)?;
        // assert_eq!(recording_id, found_recording_id.get());
        let replay_session_id =
            archive.start_replay(found_recording_id.get(), channel, stream_id, &params)?;

        let channel_replay = format!("{}?session-id={}", channel, replay_session_id);

        archive.poll_for_recording_signals()?;

        // println!("add subscription {}", channel_replay);
        // let subscription = aeron.async_add_subscription(&channel_replay, stream_id, Handlers::no_available_image_handler(), Handlers::no_unavailable_image_handler())?
        //     .poll_blocking(Duration::from_secs(10))?;
        //
        // let poll = Handler::leak(crate::AeronFragmentHandlerClosure::from(|msg, header| {
        //         panic!("{:?}", msg);
        // }));
        //
        // println!("poll");
        // let start =  Instant::now();
        // while start.elapsed() < Duration::from_secs(5)
        //     && subscription.poll(Some(&poll), 100 )? <= 0 {
        //     sleep(Duration::from_millis(1000));
        //     archive.poll_for_recording_signals()?;
        //     if let Some(err) = archive.poll_for_error() {
        //         panic!("{}", err);
        //     }
        // }
        // assert!(start.elapsed() < Duration::from_secs(5));
        // println!("Replay session id: {}", replay_session_id);
        println!("aeron {:?}", aeron);
        println!("ctx {:?}", archive_context);
        Ok(())
    }

    struct EmbeddedArchiveMediaDriverProcess {
        child: Child,
        pub aeron_dir: String,
        pub archive_dir: String,
    }

    impl EmbeddedArchiveMediaDriverProcess {
        fn start(aeron_dir: &str, archive_dir: &str, control_channel: &str) -> io::Result<Self> {
            Self::clean_directory(aeron_dir)?;
            Self::clean_directory(archive_dir)?;

            // Ensure directories are recreated
            fs::create_dir_all(aeron_dir)?;
            fs::create_dir_all(archive_dir)?;

            let binding = fs::read_dir(format!(
                "{}/aeron/aeron-all/build/libs",
                env!("CARGO_MANIFEST_DIR")
            ))?
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap())
            .filter(|f| {
                f.file_name()
                    .to_string_lossy()
                    .to_string()
                    .ends_with(".jar")
            })
            .next()
            .unwrap()
            .path();
            let jar_path = binding.to_str().unwrap();

            assert!(fs::exists(jar_path).unwrap_or_default());
            println!(
                "starting archive media driver [jar={}, shm={}, archive={}]",
                jar_path, aeron_dir, archive_dir
            );

            // Start the Java process with dynamic arguments
            let child = Command::new("java")
                .args([
                    "-cp",
                    jar_path,
                    &format!("-Daeron.dir={}", aeron_dir),
                    &format!("-Daeron.archive.dir={}", archive_dir),
                    "-Daeron.spies.simulate.connection=true",
                    "-Daeron.event.log=all", // this will only work if agent is built
                    "-Daeron.event.archive.log=all",
                    // "-Daeron.term.buffer.sparse.file=false",
                    // "-Daeron.pre.touch.mapped.memory=true",
                    // "-Daeron.threading.mode=DEDICATED",
                    // "-Daeron.sender.idle.strategy=noop",
                    // "-Daeron.receiver.idle.strategy=noop",
                    // "-Daeron.conductor.idle.strategy=spin",
                    "-Dagrona.disable.bounds.checks=true",
                    &format!("-Daeron.archive.control.channel={}", control_channel),
                    "-Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0",
                    "-Daeron.archive.control.response.channel=aeron:udp?endpoint=localhost:0",
                    "io.aeron.archive.ArchivingMediaDriver",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;

            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(30) && fs::read_dir(aeron_dir)?.count() < 2
            {
                sleep(Duration::from_millis(100));
            }
            sleep(Duration::from_millis(100));

            println!(
                "started archive media driver [{:?}",
                fs::read_dir(aeron_dir)?.collect::<Vec<_>>()
            );

            Ok(EmbeddedArchiveMediaDriverProcess {
                child,
                aeron_dir: aeron_dir.to_string(),
                archive_dir: archive_dir.to_string(),
            })
        }

        fn clean_directory(dir: &str) -> io::Result<()> {
            println!("cleaning directory {}", dir);
            let path = Path::new(dir);
            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            Ok(())
        }
    }

    // Use the Drop trait to ensure process cleanup and directory removal after test completion
    impl Drop for EmbeddedArchiveMediaDriverProcess {
        fn drop(&mut self) {
            // Attempt to kill the Java process if it’s still running
            if let Err(e) = self.child.kill() {
                eprintln!("Failed to kill Java process: {}", e);
            }

            // Clean up directories after the process has terminated
            if let Err(e) = Self::clean_directory(&self.aeron_dir) {
                eprintln!("Failed to clean up Aeron directory: {}", e);
            }
            if let Err(e) = Self::clean_directory(&self.archive_dir) {
                eprintln!("Failed to clean up Archive directory: {}", e);
            }
        }
    }
}
