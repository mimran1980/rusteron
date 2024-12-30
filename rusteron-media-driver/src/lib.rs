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
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/aeron_custom.rs"));

unsafe impl Sync for AeronDriverContext {}
unsafe impl Send for AeronDriverContext {}

impl AeronDriver {
    pub fn launch_embedded(
        aeron_context: AeronDriverContext,
        register_sigint: bool,
    ) -> (Arc<AtomicBool>, JoinHandle<Result<(), AeronCError>>) {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_copy = stop.clone();
        // Register signal handler for SIGINT (Ctrl+C)
        if register_sigint {
            let stop_copy2 = stop.clone();
            ctrlc::set_handler(move || {
                stop_copy2.store(true, Ordering::SeqCst);
            })
            .expect("Error setting Ctrl-C handler");
        }

        let started = Arc::new(AtomicBool::new(false));
        let started2 = started.clone();

        info!("Starting media driver [dir={}]", aeron_context.get_dir());
        let handle = std::thread::spawn(move || {
            let aeron_driver = AeronDriver::new(&aeron_context)?;
            aeron_driver.start(true)?;

            info!(
                "Aeron driver started [dir={}]",
                aeron_driver.context().get_dir()
            );

            started2.store(true, Ordering::SeqCst);

            // Poll for work until Ctrl+C is pressed
            while !stop.load(Ordering::Acquire) {
                while aeron_driver.main_do_work()? > 0 {
                    // busy spin
                }
            }

            info!("stopping media driver");

            Ok::<_, AeronCError>(())
        });

        while !started.load(Ordering::SeqCst) && !handle.is_finished() {
            sleep(Duration::from_millis(100));
        }

        if handle.is_finished() {
            panic!("failed to start media driver {:?}", handle.join())
        }
        info!("started media driver");

        (stop_copy, handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::error;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

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
    fn send_message() -> Result<(), AeronCError> {
        let _ = env_logger::Builder::new()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
        let topic = AERON_IPC_STREAM;
        let stream_id = 32;

        let aeron_context = AeronDriverContext::new()?;
        aeron_context.set_dir_delete_on_shutdown(true)?;
        aeron_context.set_dir_delete_on_start(true)?;

        let (stop, _driver_handle) = AeronDriver::launch_embedded(aeron_context.clone(), false);

        // aeron_driver
        //     .conductor()
        //     .context()
        //     .print_configuration();
        // aeron_driver.main_do_work()?;
        info!("aeron dir: {:?}", aeron_context.get_dir());

        let dir = aeron_context.get_dir().to_string();
        let ctx = AeronContext::new()?;
        ctx.set_dir(&dir)?;

        let client = Aeron::new(&ctx)?;
        let mut error_count = 0;

        let error_handler = Some(Handler::leak(AeronErrorHandlerClosure::from(
            |error_code, msg| {
                error!("Aeron error {}: {}", error_code, msg);
                error_count += 1;
            },
        )));
        ctx.set_error_handler(error_handler.as_ref())?;

        struct Test {}
        impl AeronAvailableCounterCallback for Test {
            fn handle_aeron_on_available_counter(
                &mut self,
                counters_reader: AeronCountersReader,
                registration_id: i64,
                counter_id: i32,
            ) -> () {
                info!("new counter counters_reader={counters_reader:?} registration_id={registration_id} counter_id={counter_id}");
            }
        }

        impl AeronNewPublicationCallback for Test {
            fn handle_aeron_on_new_publication(
                &mut self,
                async_: AeronAsyncAddPublication,
                channel: &str,
                stream_id: i32,
                session_id: i32,
                correlation_id: i64,
            ) -> () {
                info!("on new publication {async_:?} {channel} {stream_id} {session_id} {correlation_id}")
            }
        }
        let handler = Some(Handler::leak(Test {}));
        ctx.set_on_available_counter(handler.as_ref())?;
        ctx.set_on_new_publication(handler.as_ref())?;

        client.start()?;
        info!("aeron driver started");
        assert!(Aeron::epoch_clock() > 0);
        assert!(Aeron::nano_clock() > 0);

        let counter_async =
            AeronAsyncAddCounter::new(&client, 2543543, "12312312".as_bytes(), "abcd")?;

        let counter = counter_async.poll_blocking(Duration::from_secs(15))?;
        unsafe {
            *counter.addr() += 1;
        }

        let result = AeronAsyncAddPublication::new(&client, topic, stream_id)?;

        let publication = result.poll_blocking(std::time::Duration::from_secs(15))?;

        let _sub: AeronAsyncAddSubscription = AeronAsyncAddSubscription::new_zeroed()?;

        info!("publication channel: {:?}", publication.channel());
        info!("publication stream_id: {:?}", publication.stream_id());
        info!("publication status: {:?}", publication.channel_status());

        // client.main_do_work();
        // let claim = AeronBufferClaim::default();
        // assert!(publication.try_claim(100, &claim) > 0, "publication claim is empty");

        stop.store(true, Ordering::SeqCst);

        Ok(())
    }
}

// fn cleanup_subscription(clientd: *mut ::std::os::raw::c_void) {
//     cleanup_closure::<OnAvailableImageClosure>(clientd);
// }
