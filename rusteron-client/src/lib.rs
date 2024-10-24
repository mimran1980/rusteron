#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("aeron.rs");

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::*;
    use std::{error, thread};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    fn version_check() -> Result<(), Box<dyn error::Error>> {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0"; // env!("CARGO_PKG_VERSION");
        assert_eq!(aeron_version, cargo_version);

        // don't want to run just want to enfore that it compiles
        if SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            < 1
        {
            let ctx = AeronContext::new()?;
            let mut error_count = 1;
            let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
                eprintln!("aeron error {}: {}", error_code, msg);
                error_count += 1;
            });

            ctx.set_error_handler(&Handler::new(error_handler))?;
        }

        Ok(())
    }

    fn no_handler<T>() -> Option<&'static T> {
        None
    }

    #[test]
    pub fn simple_send() -> Result<(), Box<dyn error::Error>> {
        println!("creating media driver ctx");
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) = rusteron_media_driver::AeronDriver::launch_embedded(&media_driver_ctx);

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
        ctx.set_error_handler(&Handler::new(error_handler))?;

        println!("creating client");
        let aeron = Aeron::new(ctx.clone())?;
        println!("starting client");

        aeron.start()?;
        println!("client started");
        let publisher = aeron.async_add_publication("aeron:ipc", 123)?.poll_blocking(Duration::from_secs(5)).unwrap();
        println!("created publisher");

        let subscription = aeron.async_add_subscription("aeron:ipc", 123, &Handler::<AeronAvailableImageLogger>::none(), &Handler::<AeronUnavailableImageLogger>::none())?.poll_blocking(Duration::from_secs(5)).unwrap();
        println!("created subscription");

        let publisher_handler = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                loop {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    println!("sending message");
                    if publisher.offer("123".as_bytes(), &Handler::<AeronReservedValueSupplierLogger>::none()) < 1 {
                        eprintln!("failed to send message");
                    }
                }
                println!("stopping publisher thread");
            })
        };


        let count = Arc::new(AtomicUsize::new(0usize));
        let count_copy = Arc::clone(&count);
        let closure = AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
            println!("received a message from aeron {:?}, count: {}, msg length:{}", header.position(), count_copy.fetch_add(1, Ordering::SeqCst), msg.len());
        });
        let closure = Handler::new(closure);

        for _ in 0..100 {
            let c = count.load(Ordering::SeqCst);
            println!("count {c:?}");
            if c > 100 {
                stop.store(true, Ordering::SeqCst);
                break;
            }
            subscription.poll(&closure, 1024)?;
        }

        println!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = publisher_handler.join().unwrap();
        drop(aeron);
        let _ = driver_handle.join().unwrap();
        Ok(())
    }
}
