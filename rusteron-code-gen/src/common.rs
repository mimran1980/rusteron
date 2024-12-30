use crate::AeronErrorType::Unknown;
#[cfg(debug_assertions)]
use std::backtrace::Backtrace;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
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
        log::info!("created c resource: {:?}", result);
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
    #[inline(always)]
    pub fn get(&self) -> *mut T {
        self.resource
    }

    /// Closes the resource by calling the cleanup function.
    ///
    /// If cleanup fails, it returns an `AeronError`.
    pub fn close(&mut self) -> Result<(), AeronCError> {
        if let Some(mut cleanup) = self.cleanup.take() {
            if !self.resource.is_null() {
                let result = cleanup(&mut self.resource);
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
            log::info!("closing c resource: {:?}", self);
            let _ = self.close(); // Ignore errors during an automatic drop to avoid panics.

            if self.cleanup_struct {
                log::info!("closing rust struct resource: {:?}", self);
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
#[derive(Eq, PartialEq)]
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
                log::error!(
                    "Aeron C error code: {}, kind: '{:?}' - {:#?}",
                    code,
                    AeronErrorType::from_code(code),
                    backtrace
                );

                let backtrace = format!("{:?}", backtrace);
                // Regular expression to match the function, file, and line
                let re =
                    regex::Regex::new(r#"fn: "([^"]+)", file: "([^"]+)", line: (\d+)"#).unwrap();

                // Extract and print in IntelliJ format with function
                for cap in re.captures_iter(&backtrace) {
                    let function = &cap[1];
                    let file = &cap[2];
                    let line = &cap[3];
                    log::warn!("ERROR: {file}:{line} in {function}");
                }
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

impl<T> Deref for Handler<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw_ptr as &T }
    }
}

impl<T> DerefMut for Handler<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw_ptr as &mut T }
    }
}

pub fn find_unused_udp_port(start_port: u16) -> Option<u16> {
    use std::net::UdpSocket;

    let end_port = u16::MAX;

    for port in start_port..=end_port {
        if UdpSocket::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }

    None
}

/// Represents the Aeron URI parser and handler.
pub struct ChannelUri {}

impl ChannelUri {
    pub const AERON_SCHEME: &'static str = "aeron";
    pub const SPY_QUALIFIER: &'static str = "aeron-spy";
    pub const MAX_URI_LENGTH: usize = 4095;
}

/// Common constants and utilities for Aeron context.
///
/// This module provides configuration properties, default values, and a builder for creating Aeron URIs.

pub const DRIVER_TIMEOUT_MS_DEFAULT: u64 = 10_000;
pub const AERON_DIR_PROP_NAME: &str = "aeron.dir";
pub const AERON_IPC_CHANNEL: &str = "aeron:ipc";
pub const AERON_UDP_CHANNEL: &str = "aeron:udp";
pub const SPY_PREFIX: &str = "aeron-spy:";
pub const TAG_PREFIX: &str = "tag:";

/// Enum for media types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Media {
    Ipc,
    Udp,
}

impl Media {
    pub fn as_str(&self) -> &'static str {
        match self {
            Media::Ipc => "ipc",
            Media::Udp => "udp",
        }
    }
}

/// Enum for control modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlMode {
    Manual,
    Dynamic,
}

impl ControlMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlMode::Manual => "manual",
            ControlMode::Dynamic => "dynamic",
        }
    }
}

/// Builder for constructing Aeron URIs.
#[derive(Default, Debug)]
pub struct ChannelUriBuilder {
    prefix: Option<String>,
    media: Option<Media>,
    endpoint: Option<String>,
    network_interface: Option<String>,
    control_endpoint: Option<String>,
    control_mode: Option<ControlMode>,
    tags: Option<String>,
    reliable: Option<bool>,
    ttl: Option<u8>,
    mtu: Option<u32>,
    term_length: Option<u32>,
    initial_term_id: Option<i32>,
    term_id: Option<i32>,
    term_offset: Option<u32>,
    session_id: Option<i32>,
    linger: Option<u64>,
    sparse: Option<bool>,
    additional_params: BTreeMap<String, String>,
}

impl ChannelUriBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the prefix (e.g., "aeron-spy").
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Set the media type.
    pub fn media(mut self, media: Media) -> Self {
        self.media = Some(media);
        self
    }

    /// Set the endpoint (address:port).
    pub fn endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    /// Set the network interface.
    pub fn network_interface(mut self, network_interface: &str) -> Self {
        self.network_interface = Some(network_interface.to_string());
        self
    }

    /// Set the control endpoint (address:port).
    pub fn control_endpoint(mut self, control_endpoint: &str) -> Self {
        self.control_endpoint = Some(control_endpoint.to_string());
        self
    }

    /// Set the control mode.
    pub fn control_mode(mut self, control_mode: ControlMode) -> Self {
        self.control_mode = Some(control_mode);
        self
    }

    /// Set tags for the channel.
    pub fn tags(mut self, tags: &str) -> Self {
        self.tags = Some(tags.to_string());
        self
    }

    /// Set the reliable flag.
    pub fn reliable(mut self, reliable: bool) -> Self {
        self.reliable = Some(reliable);
        self
    }

    /// Set the Time To Live (TTL).
    pub fn ttl(mut self, ttl: u8) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set the Maximum Transmission Unit (MTU).
    pub fn mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Set the term length.
    pub fn term_length(mut self, term_length: u32) -> Self {
        self.term_length = Some(term_length);
        self
    }

    /// Set the initial term ID.
    pub fn initial_term_id(mut self, initial_term_id: i32) -> Self {
        self.initial_term_id = Some(initial_term_id);
        self
    }

    /// Set the term ID.
    pub fn term_id(mut self, term_id: i32) -> Self {
        self.term_id = Some(term_id);
        self
    }

    /// Set the term offset.
    pub fn term_offset(mut self, term_offset: u32) -> Self {
        self.term_offset = Some(term_offset);
        self
    }

    /// Set the session ID.
    pub fn session_id(mut self, session_id: i32) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the linger timeout.
    pub fn linger(mut self, linger: u64) -> Self {
        self.linger = Some(linger);
        self
    }

    /// Set the sparse flag.
    pub fn sparse(mut self, sparse: bool) -> Self {
        self.sparse = Some(sparse);
        self
    }

    /// Add a custom parameter to the URI.
    pub fn add_param(mut self, key: &str, value: &str) -> Self {
        self.additional_params
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Build the Aeron URI as a string.
    pub fn build(self) -> Result<String, String> {
        let media = self
            .media
            .map(|m| m.as_str())
            .ok_or_else(|| "Media must be specified".to_string())?;
        let mut uri = String::new();

        if let Some(prefix) = self.prefix {
            uri.push_str(&format!("{}:", prefix));
        }

        uri.push_str(&format!("aeron:{}?", media));

        if let Some(endpoint) = self.endpoint {
            uri.push_str(&format!("endpoint={}|", endpoint));
        }

        if let Some(control_endpoint) = self.control_endpoint {
            uri.push_str(&format!("control-endpoint={}|", control_endpoint));
        }

        if let Some(control_mode) = self.control_mode {
            uri.push_str(&format!("control-mode={}|", control_mode.as_str()));
        }

        if let Some(tags) = self.tags {
            uri.push_str(&format!("tags={}|", tags));
        }

        if let Some(reliable) = self.reliable {
            uri.push_str(&format!("reliable={}|", reliable));
        }

        if let Some(ttl) = self.ttl {
            uri.push_str(&format!("ttl={}|", ttl));
        }

        if let Some(mtu) = self.mtu {
            uri.push_str(&format!("mtu={}|", mtu));
        }

        if let Some(term_length) = self.term_length {
            uri.push_str(&format!("term-length={}|", term_length));
        }

        if let Some(initial_term_id) = self.initial_term_id {
            uri.push_str(&format!("initial-term-id={}|", initial_term_id));
        }

        if let Some(term_id) = self.term_id {
            uri.push_str(&format!("term-id={}|", term_id));
        }

        if let Some(term_offset) = self.term_offset {
            uri.push_str(&format!("term-offset={}|", term_offset));
        }

        if let Some(session_id) = self.session_id {
            uri.push_str(&format!("session-id={}|", session_id));
        }

        if let Some(linger) = self.linger {
            uri.push_str(&format!("linger={}|", linger));
        }

        if let Some(sparse) = self.sparse {
            uri.push_str(&format!("sparse={}|", sparse));
        }

        for (key, value) in self.additional_params {
            uri.push_str(&format!("{}={}|", key, value));
        }

        uri.pop();
        Ok(uri)
    }
}
