use crate::AeronErrorType::Unknown;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Formatter};
use std::{any, fmt, ptr};

/// A custom struct for managing C resources with automatic cleanup.
///
/// It handles initialisation and clean-up of the resource and ensures that resources
/// are properly released when they go out of scope.
pub struct ManagedCResource<T> {
    resource: *mut T,
    cleanup: Option<Box<dyn FnMut(*mut *mut T) -> i32>>,
    cleanup_struct: bool,
    borrowed: bool,
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
    /// `cleanup_struct` where it should clean up the struct in rust
    pub fn new(
        init: impl FnOnce(*mut *mut T) -> i32,
        cleanup: impl FnMut(*mut *mut T) -> i32 + 'static,
        cleanup_struct: bool,
    ) -> Result<Self, AeronCError> {
        let mut resource: *mut T = ptr::null_mut();
        let result = init(&mut resource);
        if result < 0 || resource.is_null() {
            return Err(AeronCError::from_code(result));
        }

        let result = Self {
            resource,
            cleanup: Some(Box::new(cleanup)),
            cleanup_struct,
            borrowed: false,
        };
        println!("created c resource: {:?}", result);
        Ok(result)
    }

    pub fn new_borrowed(value: *const T) -> Self {
        Self {
            resource: value as *mut _,
            cleanup: None,
            cleanup_struct: false,
            borrowed: true,
        }
    }

    /// Gets a raw pointer to the resource.
    #[inline]
    pub fn get(&self) -> *mut T {
        self.resource
    }

    /// Closes the resource by calling the cleanup function.
    ///
    /// If cleanup fails, it returns an `AeronError`.
    pub fn close(&mut self) -> Result<(), AeronCError> {
        if let Some(mut cleanup) = self.cleanup.take() {
            if !self.resource.is_null() {
                let result = (cleanup)(&mut self.resource);
                if result < 0 {
                    return Err(AeronCError::from_code(result));
                }
                self.resource = std::ptr::null_mut();
            }
        }

        Ok(())
    }
}

impl<T> Drop for ManagedCResource<T> {
    fn drop(&mut self) {
        if !self.resource.is_null() && !self.borrowed {
            let resource = self.resource.clone();
            // Ensure the clean-up function is called when the resource is dropped.
            println!("closing c resource: {:?}", self);
            let _ = self.close(); // Ignore errors during an automatic drop to avoid panics.

            if self.cleanup_struct {
                println!("closing rust struct resource: {:?}", self);
                unsafe {
                    let _ = Box::from_raw(resource);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum AeronErrorType {
    NullOrNotConnected,
    ClientErrorDriverTimeout,
    ClientErrorClientTimeout,
    ClientErrorConductorServiceTimeout,
    ClientErrorBufferFull,
    PublicationBackPressured,
    PublicationAdminAction,
    PublicationClosed,
    PublicationMaxPositionExceeded,
    PublicationError,
    TimedOut,
    Unknown(i32),
}

impl From<AeronErrorType> for AeronCError {
    fn from(value: AeronErrorType) -> Self {
        AeronCError::from_code(value.code())
    }
}

impl AeronErrorType {
    pub fn code(&self) -> i32 {
        match self {
            AeronErrorType::NullOrNotConnected => -1,
            AeronErrorType::ClientErrorDriverTimeout => -1000,
            AeronErrorType::ClientErrorClientTimeout => -1001,
            AeronErrorType::ClientErrorConductorServiceTimeout => -1002,
            AeronErrorType::ClientErrorBufferFull => -1003,
            AeronErrorType::PublicationBackPressured => -2,
            AeronErrorType::PublicationAdminAction => -3,
            AeronErrorType::PublicationClosed => -4,
            AeronErrorType::PublicationMaxPositionExceeded => -5,
            AeronErrorType::PublicationError => -6,
            AeronErrorType::TimedOut => -234324,
            AeronErrorType::Unknown(code) => *code,
        }
    }

    pub fn from_code(code: i32) -> Self {
        match code {
            -1 => AeronErrorType::NullOrNotConnected,
            -1000 => AeronErrorType::ClientErrorDriverTimeout,
            -1001 => AeronErrorType::ClientErrorClientTimeout,
            -1002 => AeronErrorType::ClientErrorConductorServiceTimeout,
            -1003 => AeronErrorType::ClientErrorBufferFull,
            -2 => AeronErrorType::PublicationBackPressured,
            -3 => AeronErrorType::PublicationAdminAction,
            -4 => AeronErrorType::PublicationClosed,
            -5 => AeronErrorType::PublicationMaxPositionExceeded,
            -6 => AeronErrorType::PublicationError,
            -234324 => AeronErrorType::TimedOut,
            _ => Unknown(code),
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            AeronErrorType::NullOrNotConnected => "Null Value or Not Connected",
            AeronErrorType::ClientErrorDriverTimeout => "Client Error Driver Timeout",
            AeronErrorType::ClientErrorClientTimeout => "Client Error Client Timeout",
            AeronErrorType::ClientErrorConductorServiceTimeout => {
                "Client Error Conductor Service Timeout"
            }
            AeronErrorType::ClientErrorBufferFull => "Client Error Buffer Full",
            AeronErrorType::PublicationBackPressured => "Publication Back Pressured",
            AeronErrorType::PublicationAdminAction => "Publication Admin Action",
            AeronErrorType::PublicationClosed => "Publication Closed",
            AeronErrorType::PublicationMaxPositionExceeded => "Publication Max Position Exceeded",
            AeronErrorType::PublicationError => "Publication Error",
            AeronErrorType::TimedOut => "Timed Out",
            AeronErrorType::Unknown(_) => "Unknown Error",
        }
    }
}

/// Represents an Aeron-specific error with a code and an optional message.
///
/// The error code is derived from Aeron C API calls.
/// Use `get_message()` to retrieve a human-readable message, if available.
pub struct AeronCError {
    pub code: i32,
}

impl AeronCError {
    /// Creates an AeronError from the error code returned by Aeron.
    ///
    /// Error codes below zero are considered failure.
    pub fn from_code(code: i32) -> Self {
        #[cfg(debug_assertions)]
        {
            if code < 0 {
                let backtrace = Backtrace::capture();
                eprintln!(
                    "Aeron C error code: {}, kind: '{:?}' - {:#?}",
                    code,
                    AeronErrorType::from_code(code),
                    backtrace
                );
            }
        }
        AeronCError { code }
    }

    pub fn kind(&self) -> AeronErrorType {
        AeronErrorType::from_code(self.code)
    }
}

impl fmt::Display for AeronCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Aeron error {}: {:?}", self.code, self.kind())
    }
}

impl fmt::Debug for AeronCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AeronCError")
            .field("code", &self.code)
            .field("kind", &self.kind())
            .finish()
    }
}

impl std::error::Error for AeronCError {}

// fn cleanup_closure<T>(clientd: *mut ::std::os::raw::c_void) {
//     if !clientd.is_null() {
//         unsafe {
//             // Convert the raw pointer back into a Box and drop it.
//             Box::from_raw(clientd as *mut T);
//             // The Box is dropped when it goes out of scope, automatically calling the destructor (drop).
//         }
//     }
// }

/// # Handler
///
/// `Handler` is a struct that wraps a raw pointer and a drop flag.
///
/// **Important:** `Handler` does not get dropped automatically.
/// You need to call the `release` method if you want to clear the memory manually.
///
/// ## Example
///
/// ```no_compile
/// use rusteron_code_gen::Handler;
/// let handler = Handler::leak(your_value);
/// // When you are done with the handler
/// handler.release();
/// ```
pub struct Handler<T> {
    raw_ptr: *mut T,
    should_drop: bool,
}

/// Utility method for setting empty handlers
pub struct Handlers;

impl<T> Handler<T> {
    pub fn leak(handler: T) -> Self {
        let raw_ptr = Box::into_raw(Box::new(handler)) as *mut _;
        Self {
            raw_ptr,
            should_drop: true,
        }
    }

    pub fn wrap(handler: Box<&T>) -> Self {
        let raw_ptr = Box::into_raw(handler) as *mut T;
        Self {
            raw_ptr,
            should_drop: false,
        }
    }

    pub fn is_none(&self) -> bool {
        self.raw_ptr.is_null()
    }

    pub fn as_raw(&self) -> *mut std::os::raw::c_void {
        self.raw_ptr as *mut std::os::raw::c_void
    }

    pub fn release(self) {
        if self.should_drop && !self.raw_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.raw_ptr as *mut Box<T>);
            }
        }
    }
}
