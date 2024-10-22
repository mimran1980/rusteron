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
    use super::*;
    use std::error;
    use std::sync::atomic::Ordering;
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
            let error_handler = Some(AeronErrorHandlerClosure::from(|error_code, msg| {
                eprintln!("aeron error {}: {}", error_code, msg);
                error_count += 1;
            }));
            ctx.set_error_handler(error_handler.as_ref())?;
        }

        Ok(())
    }

    #[test]
    pub fn simple_ping_pong() -> Result<(), Box<dyn error::Error>> {
        println!("creating media driver ctx");
        println!("creating media driver ctx");
        let media_driver_ctx = rusteron_media_driver::AeronDriverContext::new()?;
        let (stop, driver_handle) = rusteron_media_driver::AeronDriver::launch_embedded(&media_driver_ctx);

        println!("started media driver");
        sleep(Duration::from_secs(1));

        let ctx = AeronContext::new()?;
        ctx.set_dir(media_driver_ctx.get_dir())?;
        assert_eq!(media_driver_ctx.get_dir(), ctx.get_dir());
        let mut error_count = 1;
        let error_handler = Some(AeronErrorHandlerClosure::from(|error_code, msg| {
            eprintln!("aeron error {}: {}", error_code, msg);
            error_count += 1;
        }));
        ctx.set_error_handler(error_handler.as_ref())?;

        println!("creating client");
        let aeron = Aeron::new(ctx.clone())?;
        println!("starting client");

        aeron.start()?;


        println!("stopping client");

        stop.store(true, Ordering::SeqCst);

        let _ = driver_handle.join().unwrap();
        Ok(())
    }
}
