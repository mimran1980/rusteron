#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("../../rusteron-client/src/aeron.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

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
        let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
        aeron_driver.start(false)?;

        aeron_driver
            .conductor()
            .context()
            .print_configuration();
        aeron_driver.main_do_work()?;

        println!("aeron dir: {:?}", aeron_context.get_dir());

        let dir = aeron_context.get_dir().to_string();
        let ctx = AeronContext::new()?;
        ctx.set_idle_sleep_duration_ns(0)?;
        ctx.set_dir(CString::new(dir).unwrap().into_raw())?;
        let client = Aeron::new(ctx.get_inner())?;
        client.start()?;

        assert!(client.epoch_clock() > 0);
        assert!(client.nano_clock() > 0);
        let result = AeronAsyncAddPublication::new(client.clone(), topic, stream_id)?;

        let publication = result.poll_blocking(Duration::from_secs(10))?;

        println!("publication channel: {:?}", publication.channel());
        println!("publication stream_id: {:?}", publication.stream_id());
        println!("publication status: {:?}", publication.channel_status());

        let claim = AeronBufferClaim::default();
        assert!(publication.try_claim(100, &claim) > 0);

        Ok(())
    }
}
