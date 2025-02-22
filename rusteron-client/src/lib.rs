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
//! - **`backtrace`** - When enabled will log a backtrace for each AeronCError
//! - **`extra-logging`** - When enabled will log when resource is created and destroyed. useful if your seeing a segfault due to a resource being closed

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;

include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/aeron_custom.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use log::{error, info};
    use serial_test::serial;
    use std::error;
    use std::io::Write;
    use std::os::raw::c_int;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[derive(Default, Debug)]
    struct ErrorCount {
        error_count: usize,
    }

    impl AeronErrorHandlerCallback for ErrorCount {
        fn handle_aeron_error_handler(&mut self, error_code: c_int, msg: &str) {
            error!("Aeron error {}: {}", error_code, msg);
            self.error_count += 1;
        }
    }

    #[test]
    #[serial]
    fn version_check() -> Result<(), Box<dyn error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let cargo_version = "1.47.3";
        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        assert_eq!(aeron_version, cargo_version);

        let ctx = AeronContext::new()?;
        let error_count = 1;
        ctx.set_error_handler(Some(&Handler::leak(ErrorCount::default())))?;

        assert!(Aeron::epoch_clock() > 0);

        Ok(())
    }

    #[test]
    #[serial]
    pub fn simple_large_send() -> Result<(), Box<dyn error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        media_driver_ctx.set_dir_delete_on_shutdown(true)?;
        media_driver_ctx.set_dir_delete_on_start(true)?;
        media_driver_ctx.set_dir(&format!(
            "{}{}",
            media_driver_ctx.get_dir(),
            Aeron::epoch_clock()
        ))?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let error_count = 1;
        ctx.set_error_handler(Some(&Handler::leak(ErrorCount::default())))?;
        ctx.set_on_new_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;
        ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
        ctx.set_on_close_client(Some(&Handler::leak(AeronCloseClientLogger)))?;
        ctx.set_on_new_subscription(Some(&Handler::leak(AeronNewSubscriptionLogger)))?;
        ctx.set_on_unavailable_counter(Some(&Handler::leak(AeronUnavailableCounterLogger)))?;
        ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
        ctx.set_on_new_exclusive_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;

        info!("creating client [simple_large_send test]");
        let aeron = Aeron::new(&ctx)?;
        info!("starting client");

        aeron.start()?;
        info!("client started");
        let publisher = aeron.add_publication(AERON_IPC_STREAM, 123, Duration::from_secs(5))?;
        info!("created publisher");

        assert!(AeronCncMetadata::load_from_file(ctx.get_dir())?.pid > 0);
        assert!(AeronCnc::new(ctx.get_dir())?.get_to_driver_heartbeat_ms()? > 0);

        let subscription = aeron.add_subscription(
            AERON_IPC_STREAM,
            123,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
            Duration::from_secs(5),
        )?;
        info!("created subscription");

        subscription
            .poll_once(|msg, header| println!("foo"), 1024)
            .unwrap();

        // pick a large enough size to confirm fragement assembler is working
        let string_len = media_driver_ctx.ipc_mtu_length * 100;
        info!("string length: {}", string_len);

        let publisher_handler = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                let binding = "1".repeat(string_len);
                let large_msg = binding.as_bytes();
                loop {
                    if stop.load(Ordering::Acquire) || publisher.is_closed() {
                        break;
                    }
                    let result =
                        publisher.offer(large_msg, Handlers::no_reserved_value_supplier_handler());

                    if result < large_msg.len() as i64 {
                        let error = AeronCError::from_code(result as i32);
                        match error.kind() {
                            AeronErrorType::PublicationBackPressured
                            | AeronErrorType::PublicationAdminAction => {
                                // ignore
                            }
                            _ => {
                                error!(
                                    "ERROR: failed to send message {:?}",
                                    AeronCError::from_code(result as i32)
                                );
                            }
                        }
                        sleep(Duration::from_millis(500));
                    }
                }
                info!("stopping publisher thread");
            })
        };

        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let stop2 = stop.clone();

        struct FragmentHandler {
            count_copy: Arc<AtomicUsize>,
            stop2: Arc<AtomicBool>,
            string_len: usize,
        }

        impl AeronFragmentHandlerCallback for FragmentHandler {
            fn handle_aeron_fragment_handler(
                &mut self,
                buffer: &'static [u8],
                header: AeronHeader,
            ) {
                self.count_copy.fetch_add(1, Ordering::SeqCst);

                if buffer.len() != self.string_len {
                    self.stop2.store(true, Ordering::SeqCst);
                    error!(
                        "ERROR: message was {} but was expecting {} [header={:?}]",
                        buffer.len(),
                        self.string_len,
                        header
                    );
                    sleep(Duration::from_secs(1));
                }

                assert_eq!(buffer.len(), self.string_len);
                assert_eq!(buffer, "1".repeat(self.string_len).as_bytes());
            }
        }

        let (closure, _inner) = Handler::leak_with_fragment_assembler(FragmentHandler {
            count_copy,
            stop2,
            string_len,
        })?;

        // Start the timer
        let start_time = Instant::now();

        loop {
            if start_time.elapsed() > Duration::from_secs(30) {
                info!("Failed: exceeded 30-second timeout");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout exceeded",
                )));
            }
            let c = count.load(Ordering::SeqCst);
            if c > 100 {
                break;
            }
            subscription.poll(Some(&closure), 128)?;
        }

        info!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();

        let cnc = AeronCnc::new(ctx.get_dir())?;
        cnc.counters_reader().foreach_counter_once(
            |value: i64, id: i32, type_id: i32, key: &[u8], label: &str| {
                println!("counter reader id={id}, type_id={type_id}, key={key:?}, label={label}, value={value}");
            },
        );
        cnc.error_log_read_once(| observation_count: i32,
                                     first_observation_timestamp: i64,
                                     last_observation_timestamp: i64,
                                     error: &str| {
            println!("error: {error} observationCount={observation_count}, first_observation_timestamp={first_observation_timestamp}, last_observation_timestamp={last_observation_timestamp}");
        }, 0);
        cnc.loss_reporter_read_once(|    observation_count: i64,
                                    total_bytes_lost: i64,
                                    first_observation_timestamp: i64,
                                    last_observation_timestamp: i64,
                                    session_id: i32,
                                    stream_id: i32,
                                    channel: &str,
                                    source: &str,| {
            println!("loss reporter observationCount={observation_count}, totalBytesLost={total_bytes_lost}, first_observed={first_observation_timestamp}, last_observed={last_observation_timestamp}, session_id={session_id}, stream_id={stream_id}, channel={channel} source={source}");
        })?;

        Ok(())
    }

    #[test]
    #[serial]
    pub fn try_claim() -> Result<(), Box<dyn error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        media_driver_ctx.set_dir_delete_on_shutdown(true)?;
        media_driver_ctx.set_dir_delete_on_start(true)?;
        media_driver_ctx.set_dir(&format!(
            "{}{}",
            media_driver_ctx.get_dir(),
            Aeron::epoch_clock()
        ))?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        ctx.set_error_handler(Some(&Handler::leak(ErrorCount::default())))?;

        info!("creating client [try_claim test]");
        let aeron = Aeron::new(&ctx)?;
        info!("starting client");

        aeron.start()?;
        info!("client started");
        let publisher = aeron.add_publication(AERON_IPC_STREAM, 123, Duration::from_secs(5))?;
        info!("created publisher");

        let subscription = aeron.add_subscription(
            AERON_IPC_STREAM,
            123,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
            Duration::from_secs(5),
        )?;
        info!("created subscription");

        // pick a large enough size to confirm fragement assembler is working
        let string_len = 156;
        info!("string length: {}", string_len);

        let publisher_handler = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                let binding = "1".repeat(string_len);
                let msg = binding.as_bytes();
                let buffer = AeronBufferClaim::default();
                loop {
                    if stop.load(Ordering::Acquire) || publisher.is_closed() {
                        break;
                    }

                    let result = publisher.try_claim(string_len, &buffer);

                    if result < msg.len() as i64 {
                        error!(
                            "ERROR: failed to send message {:?}",
                            AeronCError::from_code(result as i32)
                        );
                    } else {
                        buffer.data().write_all(&msg).unwrap();
                        buffer.commit().unwrap();
                    }
                }
                info!("stopping publisher thread");
            })
        };

        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let stop2 = stop.clone();

        struct FragmentHandler {
            count_copy: Arc<AtomicUsize>,
            stop2: Arc<AtomicBool>,
            string_len: usize,
        }

        impl AeronFragmentHandlerCallback for FragmentHandler {
            fn handle_aeron_fragment_handler(
                &mut self,
                buffer: &'static [u8],
                header: AeronHeader,
            ) {
                self.count_copy.fetch_add(1, Ordering::SeqCst);

                if buffer.len() != self.string_len {
                    self.stop2.store(true, Ordering::SeqCst);
                    error!(
                        "ERROR: message was {} but was expecting {} [header={:?}]",
                        buffer.len(),
                        self.string_len,
                        header
                    );
                    sleep(Duration::from_secs(1));
                }

                assert_eq!(buffer.len(), self.string_len);
                assert_eq!(buffer, "1".repeat(self.string_len).as_bytes());
            }
        }

        let (closure, _inner) = Handler::leak_with_fragment_assembler(FragmentHandler {
            count_copy,
            stop2,
            string_len,
        })?;
        let start_time = Instant::now();

        loop {
            if start_time.elapsed() > Duration::from_secs(30) {
                info!("Failed: exceeded 30-second timeout");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout exceeded",
                )));
            }
            let c = count.load(Ordering::SeqCst);
            if c > 100 {
                break;
            }
            subscription.poll(Some(&closure), 128)?;
        }

        info!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[test]
    #[serial]
    pub fn counters() -> Result<(), Box<dyn error::Error>> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        media_driver_ctx.set_dir_delete_on_shutdown(true)?;
        media_driver_ctx.set_dir_delete_on_start(true)?;
        media_driver_ctx.set_dir(&format!(
            "{}{}",
            media_driver_ctx.get_dir(),
            Aeron::epoch_clock()
        ))?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        ctx.set_error_handler(Some(&Handler::leak(ErrorCount::default())))?;
        ctx.set_on_unavailable_counter(Some(&Handler::leak(AeronUnavailableCounterLogger)))?;

        struct AvailableCounterHandler {
            found_counter: bool,
        }

        impl AeronAvailableCounterCallback for AvailableCounterHandler {
            fn handle_aeron_on_available_counter(
                &mut self,
                counters_reader: AeronCountersReader,
                registration_id: i64,
                counter_id: i32,
            ) -> () {
                info!(
            "on counter key={:?}, label={:?} registration_id={registration_id}, counter_id={counter_id}, value={}, {counters_reader:?}",
            String::from_utf8(counters_reader.get_counter_key(counter_id).unwrap()),
            counters_reader.get_counter_label(counter_id, 1000),
            counters_reader.addr(counter_id)
        );

                assert_eq!(
                    counters_reader.counter_registration_id(counter_id).unwrap(),
                    registration_id
                );

                if let Ok(label) = counters_reader.get_counter_label(counter_id, 1000) {
                    if label == "label_buffer" {
                        self.found_counter = true;
                        assert_eq!(
                            &counters_reader.get_counter_key(counter_id).unwrap(),
                            "key".as_bytes()
                        );
                    }
                }
            }
        }

        let handler = &Handler::leak(AvailableCounterHandler {
            found_counter: false,
        });
        ctx.set_on_available_counter(Some(handler))?;

        info!("creating client");
        let aeron = Aeron::new(&ctx)?;
        info!("starting client");

        aeron.start()?;
        info!("client started [counters test]");

        let counter = aeron.add_counter(
            123,
            "key".as_bytes(),
            "label_buffer",
            Duration::from_secs(5),
        )?;
        let constants = counter.get_constants()?;
        let counter_id = constants.counter_id;

        let publisher_handler = {
            let stop = stop.clone();
            let counter = counter.clone();
            std::thread::spawn(move || {
                for _ in 0..150 {
                    if stop.load(Ordering::Acquire) || counter.is_closed() {
                        break;
                    }
                    counter.addr_atomic().fetch_add(1, Ordering::SeqCst);
                }
                info!("stopping publisher thread");
            })
        };

        let now = Instant::now();
        while counter.addr_atomic().load(Ordering::SeqCst) < 100
            && now.elapsed() < Duration::from_secs(10)
        {
            sleep(Duration::from_micros(10));
        }

        assert!(now.elapsed() < Duration::from_secs(10));

        info!(
            "counter is {}",
            counter.addr_atomic().load(Ordering::SeqCst)
        );

        info!("stopping client");

        #[cfg(not(target_os = "windows"))] // not sure why windows version doesn't fire event
        assert!(handler.found_counter);

        stop.store(true, Ordering::SeqCst);

        let reader = aeron.counters_reader();
        assert_eq!(reader.get_counter_label(counter_id, 256)?, "label_buffer");
        assert_eq!(reader.get_counter_key(counter_id)?, "key".as_bytes());
        let buffers = AeronCountersReaderBuffers::default();
        reader.get_buffers(&buffers)?;

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[doc = include_str!("../../README.md")]
    mod readme_tests {}
}
