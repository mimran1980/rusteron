#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("../../rusteron-client/src/aeron.rs");

unsafe impl Send for AeronDriverContext {}

impl AeronDriver {
    pub fn launch_embedded(aeron_context: &AeronDriverContext) -> Arc<AtomicBool> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_copy = stop.clone();
        let stop_copy2 = stop.clone();
        let aeron_context = aeron_context.clone();
        // Register signal handler for SIGINT (Ctrl+C)
        ctrlc::set_handler(move || {
            stop_copy2.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        std::thread::spawn(move || {
            let aeron_driver = AeronDriver::new(aeron_context)?;
            aeron_driver.start(true)?;

            // Poll for work until Ctrl+C is pressed
            while !stop.load(Ordering::Acquire) {
                while aeron_driver.main_do_work()? > 0 {
                    // busy spin
                }
            }

            Ok::<_, AeronCError>(())
        });
        stop_copy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::sync::atomic::Ordering;

    use std::time::Duration;

    #[test]
    fn version_check() {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0"; // env!("CARGO_PKG_VERSION");
        assert_eq!(aeron_version, cargo_version);
    }

    #[test]
    fn send_message() -> Result<(), AeronCError> {
        let topic = "aeron:ipc";
        let stream_id = 32;

        let aeron_context = AeronDriverContext::new()?;
        aeron_context.set_dir_delete_on_shutdown(true)?;
        aeron_context.set_dir_delete_on_start(true)?;

        let stop = AeronDriver::launch_embedded(&aeron_context);

        // aeron_driver
        //     .conductor()
        //     .context()
        //     .print_configuration();
        // aeron_driver.main_do_work()?;
        println!("aeron dir: {:?}", aeron_context.get_dir());

        let dir = aeron_context.get_dir().to_string();
        let ctx = AeronContext::new()?;
        ctx.set_dir(&dir)?;

        let client = Aeron::new(ctx.clone())?;

        unsafe {
            struct Test {}
            impl AeronAvailableCounterHandler for Test {
                fn handle(
                    &mut self,
                    counters_reader: AeronCountersReader,
                    registration_id: i64,
                    counter_id: i32,
                ) -> () {
                    println!("new counter");
                }
            }

            impl AeronNewPublicationHandler for Test {
                fn handle(
                    &mut self,
                    async_: AeronAsyncAddPublication,
                    channel: &str,
                    stream_id: i32,
                    session_id: i32,
                    correlation_id: i64,
                ) -> () {
                    println!("new publication");
                }
            }
            let handler = Some(Test {});
            ctx.set_on_available_counter(handler.as_ref())?;
            ctx.set_on_new_publication(handler.as_ref())?;
        }

        client.start()?;
        println!("aeron driver started");
        assert!(client.epoch_clock() > 0);
        assert!(client.nano_clock() > 0);

        let counter_async = AeronAsyncAddCounter::new(
            client.clone(),
            2543543,
            "12312312".as_ptr(),
            "12312312".len(),
            "abcd",
            4,
        )?;

        let counter = counter_async.poll_blocking(Duration::from_secs(15))?;
        unsafe {
            *counter.addr() += 1;
        }

        let result = AeronAsyncAddPublication::new(client.clone(), topic, stream_id)?;

        let publication = result.poll_blocking(std::time::Duration::from_secs(15))?;

        let sub: AeronAsyncAddSubscription = AeronAsyncAddSubscription::new_zeroed()?;

        println!("publication channel: {:?}", publication.channel());
        println!("publication stream_id: {:?}", publication.stream_id());
        println!("publication status: {:?}", publication.channel_status());

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
