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
    use std::error;
    use crate::{Aeron, AeronContext, AeronErrorHandlerClosure};

    #[test]
    fn version_check() -> Result<(), Box<dyn error::Error>> {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0"; // env!("CARGO_PKG_VERSION");
        assert_eq!(aeron_version, cargo_version);

        // don't want to run just want to enfore that it compiles
        if false {
            let ctx = AeronContext::new()?;
            let client = Aeron::new(ctx.clone())?;
            let mut error_count = 1;
            let error_handler = Some(AeronErrorHandlerClosure::from(|error_code, msg| {
                eprintln!("aeron error {}: {}", error_code, msg);
                error_count += 1;
            }));
            ctx.set_error_handler(error_handler.as_ref())?;
        }

        Ok(())
    }
}
