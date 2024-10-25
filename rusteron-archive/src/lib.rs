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

    #[test]
    fn version_check() {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0";
        assert_eq!(aeron_version, cargo_version);
    }
}
