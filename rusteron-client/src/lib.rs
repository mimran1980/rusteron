#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(unused_unsafe)]
#![doc = include_str!("../README.md")]
//! # Features
//!
//! - **`static`**: When enabled, this feature statically links the Aeron C code.
//!   By default, the library uses dynamic linking to the Aeron C libraries.

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/aeron_custom.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::error;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    #[serial]
    fn version_check() -> Result<(), Box<dyn error::Error>> {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0";
        assert_eq!(aeron_version, cargo_version);

        let ctx = AeronContext::new()?;
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("ERROR: aeron error {}: {}", error_code, msg);
            error_count += 1;
        });

        ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;

        assert!(Aeron::epoch_clock() > 0);

        Ok(())
    }

    #[test]
    #[serial]
    pub fn simple_large_send() -> Result<(), Box<dyn error::Error>> {
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("ERROR: aeron error {}: {}", error_code, msg);
            error_count += 1;
        });
        ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;
        ctx.set_on_new_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;
        ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
        ctx.set_on_close_client(Some(&Handler::leak(AeronCloseClientLogger)))?;
        ctx.set_on_new_subscription(Some(&Handler::leak(AeronNewSubscriptionLogger)))?;
        ctx.set_on_unavailable_counter(Some(&Handler::leak(AeronUnavailableCounterLogger)))?;
        ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
        ctx.set_on_new_exclusive_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;

        println!("creating client");
        let aeron = Aeron::new(ctx)?;
        println!("starting client");

        aeron.start()?;
        println!("client started");
        let publisher = aeron
            .async_add_publication("aeron:ipc", 123)?
            .poll_blocking(Duration::from_secs(5))?;
        println!("created publisher");

        let subscription = aeron
            .async_add_subscription(
                "aeron:ipc",
                123,
                Handlers::no_available_image_handler(),
                Handlers::no_unavailable_image_handler(),
            )?
            .poll_blocking(Duration::from_secs(5))
            .unwrap();
        println!("created subscription");

        // pick a large enough size to confirm fragement assembler is working
        let string_len = media_driver_ctx.ipc_mtu_length * 100;
        println!("string length: {}", string_len);

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
                        eprintln!("ERROR: failed to send message");
                    } else {
                        println!("send message [result={}]", result);
                    }
                }
                println!("stopping publisher thread");
            })
        };

        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let stop2 = stop.clone();

        let closure =
            AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
                println!(
                    "received a message from aeron {:?}, count: {}, msg length:{}",
                    header.position(),
                    count_copy.fetch_add(1, Ordering::SeqCst),
                    msg.len()
                );
                if msg.len() != string_len {
                    stop2.store(true, Ordering::SeqCst);
                    eprintln!(
                        "ERROR: message was {} was expecting {} [header={:?}]",
                        msg.len(),
                        string_len,
                        header
                    );
                    sleep(Duration::from_secs(1));
                }
                assert_eq!(msg.len(), string_len);
                assert_eq!(msg.as_slice(), "1".repeat(string_len).as_bytes())
            });
        let closure = Handler::leak_with_fragment_assembler(closure)?;

        loop {
            let c = count.load(Ordering::SeqCst);
            if c > 100 {
                break;
            }
            subscription.poll(Some(&closure), 128)?;
        }

        println!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[test]
    #[serial]
    pub fn try_claim() -> Result<(), Box<dyn error::Error>> {
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("ERROR: aeron error {}: {}", error_code, msg);
            error_count += 1;
        });
        ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;

        println!("creating client");
        let aeron = Aeron::new(ctx)?;
        println!("starting client");

        aeron.start()?;
        println!("client started");
        let publisher = aeron
            .async_add_publication("aeron:ipc", 123)?
            .poll_blocking(Duration::from_secs(5))?;
        println!("created publisher");

        let subscription = aeron
            .async_add_subscription(
                "aeron:ipc",
                123,
                Handlers::no_available_image_handler(),
                Handlers::no_unavailable_image_handler(),
            )?
            .poll_blocking(Duration::from_secs(5))
            .unwrap();
        println!("created subscription");

        // pick a large enough size to confirm fragement assembler is working
        let string_len = 156;
        println!("string length: {}", string_len);

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
                        eprintln!(
                            "ERROR: failed to send message {:?}",
                            AeronCError::from_code(result as i32)
                        );
                    } else {
                        buffer.data().write_all(&msg).unwrap();
                        buffer.commit().unwrap();
                        println!("send message [result={}]", result);
                    }
                }
                println!("stopping publisher thread");
            })
        };

        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let stop2 = stop.clone();

        let closure =
            AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
                println!(
                    "received a message from aeron {:?}, count: {}, msg length:{}",
                    header.position(),
                    count_copy.fetch_add(1, Ordering::SeqCst),
                    msg.len()
                );
                if msg.len() != string_len {
                    stop2.store(true, Ordering::SeqCst);
                    eprintln!(
                        "ERROR: message was {} was expecting {} [header={:?}]",
                        msg.len(),
                        string_len,
                        header
                    );
                    sleep(Duration::from_secs(1));
                }
                assert_eq!(msg.len(), string_len);
                assert_eq!(msg.as_slice(), "1".repeat(string_len).as_bytes())
            });
        let closure = Handler::leak_with_fragment_assembler(closure)?;

        loop {
            let c = count.load(Ordering::SeqCst);
            println!("count {c:?}");
            if c > 100 {
                break;
            }
            subscription.poll(Some(&closure), 128)?;
        }

        println!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[test]
    #[serial]
    pub fn counters() -> Result<(), Box<dyn error::Error>> {
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("ERROR: aeron error {}: {}", error_code, msg);
            error_count += 1;
        });
        ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;
        ctx.set_on_unavailable_counter(Some(&Handler::leak(AeronUnavailableCounterLogger)))?;
        let mut found_counter = false;
        ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterClosure::from(
            |counters_reader: AeronCountersReader,
             registration_id: i64,
             counter_id: i32| {
                println!("on counter {:?} {counters_reader:?}, registration_id={registration_id}, counter_id={counter_id}, value={}", counters_reader.get_counter_label(counter_id, 1000), counters_reader.addr(counter_id));
                assert_eq!(counters_reader.counter_registration_id(counter_id).unwrap(), registration_id);
                if let Ok(label) = counters_reader.get_counter_label(counter_id, 1000) {
                    if label == "test_counter" {
                        found_counter = true;
                    }
                }
            }
        ))))?;

        println!("creating client");
        let aeron = Aeron::new(ctx.clone())?;
        println!("starting client");

        aeron.start()?;
        println!("client started");

        let counter = aeron
            .async_add_counter(123, "test_counter".as_bytes(), "this is a test")?
            .poll_blocking(Duration::from_secs(5))?;

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
                println!("stopping publisher thread");
            })
        };

        let now = Instant::now();
        while counter.addr_atomic().load(Ordering::SeqCst) < 100
            && now.elapsed() < Duration::from_secs(10)
        {
            sleep(Duration::from_micros(10));
        }

        assert!(now.elapsed() < Duration::from_secs(10));

        println!(
            "counter is {}",
            counter.addr_atomic().load(Ordering::SeqCst)
        );

        println!("stopping client");

        #[cfg(not(target_os = "windows"))] // not sure why windows version doesn't fire event
        assert!(found_counter);

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[doc = include_str!("../../README.md")]
    mod readme_tests {}
}
