#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(unused_unsafe)]

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("aeron.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
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
            eprintln!("aeron error {}: {}", error_code, msg);
            error_count += 1;
        });

        ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;

        Ok(())
    }

    #[test]
    pub fn simple_send() -> Result<(), Box<dyn error::Error>> {
        println!("creating media driver ctx");
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(&media_driver_ctx, false);

        println!("started media driver");
        sleep(Duration::from_secs(1));

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("aeron error {}: {}", error_code, msg);
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
                loop {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    println!("sending message");
                    if publisher.offer(
                        "1".repeat(string_len).as_bytes(),
                        Handlers::no_reserved_value_supplier_handler(),
                    ) < 1
                    {
                        eprintln!("failed to send message");
                    }
                }
                println!("stopping publisher thread");
            })
        };

        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let closure =
            AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
                println!(
                    "received a message from aeron {:?}, count: {}, msg length:{}",
                    header.position(),
                    count_copy.fetch_add(1, Ordering::SeqCst),
                    msg.len()
                );
                assert_eq!(msg.as_slice(), "1".repeat(string_len).as_bytes())
            });
        let closure = Handler::leak(closure);

        for _ in 0..100 {
            let c = count.load(Ordering::SeqCst);
            println!("count {c:?}");
            if c > 100 {
                stop.store(true, Ordering::SeqCst);
                break;
            }
            subscription.poll(Some(&closure), 1024)?;
        }

        println!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }

    #[test]
    pub fn counters() -> Result<(), Box<dyn error::Error>> {
        println!("creating media driver ctx");
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) =
            rusteron_media_driver::AeronDriver::launch_embedded(&media_driver_ctx, false);

        println!("started media driver");
        sleep(Duration::from_secs(1));

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("aeron error {}: {}", error_code, msg);
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
                assert_eq!(counters_reader.get_counter_registration_id(counter_id).unwrap(), registration_id);
                if let Ok(label) = counters_reader.get_counter_label(counter_id, 1000) {
                    if label == "test" {
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
            .async_add_counter(123, "test".as_bytes(), "this is a test")?
            .poll_blocking(Duration::from_secs(5))?;

        let publisher_handler = {
            let stop = stop.clone();
            let counter = counter.clone();
            std::thread::spawn(move || {
                loop {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    counter.addr_atomic().fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_micros(1));
                }
                println!("stopping publisher thread");
            })
        };

        while counter.addr_atomic().load(Ordering::SeqCst) < 100 {
            sleep(Duration::from_micros(10));
        }

        println!(
            "counter is {}",
            counter.addr_atomic().load(Ordering::SeqCst)
        );

        println!("stopping client");

        assert!(found_counter);

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        let _ = driver_handle.join().unwrap();
        Ok(())
    }
}
