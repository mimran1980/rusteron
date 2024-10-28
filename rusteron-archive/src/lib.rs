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
    use std::thread::sleep;
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
    pub fn test_failed_connect() {
        println!("creating archive");
        let ctx = AeronArchiveContext::new().unwrap();
        sleep(Duration::from_secs(1));
        println!("setting timeout");
        ctx.set_message_timeout_ns(1).unwrap();
        sleep(Duration::from_secs(1));
        println!("trying async connect");
        let connect = AeronArchiveAsyncConnect::new(ctx);
        assert_eq!(
            Some(AeronErrorType::NullOrNotConnected.into()),
            connect.err()
        );
    }
}
