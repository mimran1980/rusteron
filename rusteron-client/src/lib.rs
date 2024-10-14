use libaeron_sys::*;

pub mod client;
// pub mod publication;
// pub mod subscription;

use std::rc::Rc;
use std::ffi::CStr;
use rusteron_common::{AeronError, ManagedCResource};

/// Manages the Aeron driver context.
///
/// This struct wraps around the C API provided by Aeron and provides safe methods
/// for configuring and managing the driver context.
pub struct AeronContext {
    resource: Rc<ManagedCResource<aeron_context_t>>,
}

impl AeronContext {
    /// Creates and initializes a new AeronContext.
    ///
    /// This wraps the `aeron_context_init()` function from the Aeron C API.
    pub fn new() -> Result<Self, AeronError> {
        let resource: ManagedCResource<aeron_context_t> = ManagedCResource::new(
            |ctx| unsafe { aeron_context_init(ctx) },  // FFI call to init
            |ctx| unsafe { aeron_context_close(ctx) }, // FFI call to cleanup
        )?;

        Ok(AeronContext { resource: Rc::new(resource) })
    }

    /// Sets the directory used by the Aeron driver.
    ///
    /// This method must be called before the driver is initialized.
    pub fn set_dir(&self, dir: &str) -> Result<(), AeronError> {
        let result = unsafe {
            aeron_context_set_dir(self.resource.get(), dir.as_ptr() as *const i8)
        };
        if result < 0 {
            return Err(AeronError::from_code(result));
        }
        Ok(())
    }

    /// Retrieves the directory used by the Aeron driver.
    pub fn get_dir(&self) -> Result<&'static str, AeronError> {
        let dir_ptr = unsafe { aeron_context_get_dir(self.resource.get()) };
        if dir_ptr.is_null() {
            return Err(AeronError::from_code(-1));  // Assume -1 indicates an error
        }

        let dir_str = unsafe { CStr::from_ptr(dir_ptr) }.to_str().map_err(|_| AeronError::from_code(-1))?;
        Ok(dir_str)
    }

    ///
    /// This wraps the `aeron_context_get_aeron_dir()` method from the C API.
    pub fn get_aeron_dir(&self) -> Result<&'static str, AeronError> {
        let dir_ptr = unsafe { aeron_context_get_dir(self.resource.get()) };
        if dir_ptr.is_null() {
            return Err(AeronError::from_code(-1));  // Assume -1 indicates an error
        }

        let dir_str = unsafe { CStr::from_ptr(dir_ptr) }.to_str().map_err(|_| AeronError::from_code(-1))?;
        Ok(dir_str)
    }

    /// Sets a callback for on_close event in the Aeron driver context.
    pub fn set_on_close(&self, callback: fn() -> ()) -> Result<(), AeronError> {
        let result = unsafe { aeron_context_set_on_close_client(self.resource.get(), callback) };
        if result < 0 {
            return Err(AeronError::from_code(result));
        }
        Ok(())
    }
}
