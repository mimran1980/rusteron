use std::fmt::{Debug, Formatter};
use std::{any, fmt, ptr};

/// A custom struct for managing C resources with automatic cleanup.
///
/// It handles initialisation and clean-up of the resource and ensures that resources
/// are properly released when they go out of scope.
pub struct ManagedCResource<T> {
    resource: *mut T,
    cleanup: Box<dyn FnMut(*mut *mut T) -> i32>,
}

impl<T> Debug for ManagedCResource<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagedCResource")
            .field("resource", &self.resource)
            .field("type", &any::type_name::<T>())
            .finish()
    }
}

impl<T> ManagedCResource<T> {
    /// Creates a new ManagedCResource with a given initializer and cleanup function.
    ///
    /// The initializer is a closure that attempts to initialize the resource.
    /// If initialization fails, the initializer should return an error code.
    /// The cleanup function is used to release the resource when it's no longer needed.
    pub fn new(
        init: impl FnOnce(*mut *mut T) -> i32,
        cleanup: impl FnMut(*mut *mut T) -> i32 + 'static,
    ) -> Result<Self, AeronError> {
        let mut resource: *mut T = ptr::null_mut();
        let result = init(&mut resource);
        if result < 0 {
            return Err(AeronError::from_code(result));
        }

        Ok(Self {
            resource,
            cleanup: Box::new(cleanup),
        })
    }

    /// Gets a raw pointer to the resource.
    pub fn get(&self) -> *mut T {
        self.resource
    }

    /// Closes the resource by calling the cleanup function.
    ///
    /// If cleanup fails, it returns an `AeronError`.
    pub fn close(&mut self) -> Result<(), AeronError> {
        if !self.resource.is_null() {
            let result = (self.cleanup)(&mut self.resource);
            if result < 0 {
                return Err(AeronError::from_code(result));
            }
            self.resource = std::ptr::null_mut();
        }
        Ok(())
    }
}

impl<T> Drop for ManagedCResource<T> {
    fn drop(&mut self) {
        // Ensure the clean-up function is called when the resource is dropped.
        let _ = self.close(); // Ignore errors during an automatic drop to avoid panics.
    }
}

/// Represents an Aeron-specific error with a code and an optional message.
///
/// The error code is derived from Aeron C API calls.
/// Use `get_message()` to retrieve a human-readable message, if available.
#[derive(Debug)]
pub struct AeronError {
    pub code: i32,
}

impl AeronError {
    /// Creates an AeronError from the error code returned by Aeron.
    ///
    /// Error codes below zero are considered failure.
    pub fn from_code(code: i32) -> Self {
        AeronError { code }
    }

    /// Retrieves the error message corresponding to the error code.
    ///
    /// The message is fetched from the Aeron C API using `aeron_driver_last_error()`.
    /// If the conversion of the C string to UTF-8 fails, this returns `None`.
    pub fn get_message(&self) -> Option<&'static str> {
        unsafe {
            // let err_ptr = aeron_driver_last_error();
            // if !err_ptr.is_null() {
            //     // Try to convert the C string to a Rust &str, handle any UTF-8 errors gracefully
            //     match CStr::from_ptr(err_ptr).to_str() {
            //         Ok(message) => Some(message),
            //         Err(_) => None, // Return None if the conversion fails
            //     }
            // } else {
            //     None
            // }
            todo!()
        }
    }
}

impl fmt::Display for AeronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_message() {
            Some(msg) => write!(f, "Aeron error {}: {}", self.code, msg),
            None => write!(f, "Aeron error {}", self.code),
        }
    }
}

impl std::error::Error for AeronError {}
