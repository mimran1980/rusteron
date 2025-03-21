/* automatically generated by rust-bindgen 0.71.1 */

pub const AERON_COMPILER_GCC: u32 = 1;
pub const AERON_COMPILER_LLVM: u32 = 1;
pub const AERON_CPU_X64: u32 = 1;
pub const AERON_CACHE_LINE_LENGTH: u32 = 64;
pub const AERON_BROADCAST_PADDING_MSG_TYPE_ID: i32 = -1;
pub const AERON_BROADCAST_SCRATCH_BUFFER_LENGTH: u32 = 4096;
pub const AERON_RB_PADDING_MSG_TYPE_ID: i32 = -1;
unsafe extern "C" {
    pub fn aeron_randomised_int32() -> i32;
}
#[repr(C, packed(4))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_broadcast_descriptor_stct {
    pub tail_intent_counter: i64,
    pub tail_counter: i64,
    pub latest_counter: i64,
    pub pad: [u8; 104usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_broadcast_descriptor_stct"]
        [::std::mem::size_of::<aeron_broadcast_descriptor_stct>() - 128usize];
    ["Alignment of aeron_broadcast_descriptor_stct"]
        [::std::mem::align_of::<aeron_broadcast_descriptor_stct>() - 4usize];
    ["Offset of field: aeron_broadcast_descriptor_stct::tail_intent_counter"]
        [::std::mem::offset_of!(aeron_broadcast_descriptor_stct, tail_intent_counter) - 0usize];
    ["Offset of field: aeron_broadcast_descriptor_stct::tail_counter"]
        [::std::mem::offset_of!(aeron_broadcast_descriptor_stct, tail_counter) - 8usize];
    ["Offset of field: aeron_broadcast_descriptor_stct::latest_counter"]
        [::std::mem::offset_of!(aeron_broadcast_descriptor_stct, latest_counter) - 16usize];
    ["Offset of field: aeron_broadcast_descriptor_stct::pad"]
        [::std::mem::offset_of!(aeron_broadcast_descriptor_stct, pad) - 24usize];
};
impl Default for aeron_broadcast_descriptor_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_broadcast_descriptor_t = aeron_broadcast_descriptor_stct;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_broadcast_record_descriptor_stct {
    pub length: i32,
    pub msg_type_id: i32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_broadcast_record_descriptor_stct"]
        [::std::mem::size_of::<aeron_broadcast_record_descriptor_stct>() - 8usize];
    ["Alignment of aeron_broadcast_record_descriptor_stct"]
        [::std::mem::align_of::<aeron_broadcast_record_descriptor_stct>() - 4usize];
    ["Offset of field: aeron_broadcast_record_descriptor_stct::length"]
        [::std::mem::offset_of!(aeron_broadcast_record_descriptor_stct, length) - 0usize];
    ["Offset of field: aeron_broadcast_record_descriptor_stct::msg_type_id"]
        [::std::mem::offset_of!(aeron_broadcast_record_descriptor_stct, msg_type_id) - 4usize];
};
pub type aeron_broadcast_record_descriptor_t = aeron_broadcast_record_descriptor_stct;
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_broadcast_receiver_stct {
    pub scratch_buffer: [u8; 4096usize],
    pub buffer: *mut u8,
    pub descriptor: *mut aeron_broadcast_descriptor_t,
    pub capacity: usize,
    pub mask: usize,
    pub record_offset: usize,
    pub cursor: i64,
    pub next_record: i64,
    pub lapped_count: ::std::os::raw::c_long,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_broadcast_receiver_stct"]
        [::std::mem::size_of::<aeron_broadcast_receiver_stct>() - 4160usize];
    ["Alignment of aeron_broadcast_receiver_stct"]
        [::std::mem::align_of::<aeron_broadcast_receiver_stct>() - 8usize];
    ["Offset of field: aeron_broadcast_receiver_stct::scratch_buffer"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, scratch_buffer) - 0usize];
    ["Offset of field: aeron_broadcast_receiver_stct::buffer"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, buffer) - 4096usize];
    ["Offset of field: aeron_broadcast_receiver_stct::descriptor"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, descriptor) - 4104usize];
    ["Offset of field: aeron_broadcast_receiver_stct::capacity"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, capacity) - 4112usize];
    ["Offset of field: aeron_broadcast_receiver_stct::mask"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, mask) - 4120usize];
    ["Offset of field: aeron_broadcast_receiver_stct::record_offset"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, record_offset) - 4128usize];
    ["Offset of field: aeron_broadcast_receiver_stct::cursor"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, cursor) - 4136usize];
    ["Offset of field: aeron_broadcast_receiver_stct::next_record"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, next_record) - 4144usize];
    ["Offset of field: aeron_broadcast_receiver_stct::lapped_count"]
        [::std::mem::offset_of!(aeron_broadcast_receiver_stct, lapped_count) - 4152usize];
};
impl Default for aeron_broadcast_receiver_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_broadcast_receiver_t = aeron_broadcast_receiver_stct;
pub type aeron_broadcast_receiver_handler_t = ::std::option::Option<
    unsafe extern "C" fn(
        type_id: i32,
        buffer: *mut u8,
        length: usize,
        clientd: *mut ::std::os::raw::c_void,
    ),
>;
unsafe extern "C" {
    pub fn aeron_broadcast_receiver_init(
        receiver: *mut aeron_broadcast_receiver_t,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_broadcast_receiver_receive(
        receiver: *mut aeron_broadcast_receiver_t,
        handler: aeron_broadcast_receiver_handler_t,
        clientd: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_broadcast_transmitter_stct {
    pub buffer: *mut u8,
    pub descriptor: *mut aeron_broadcast_descriptor_t,
    pub capacity: usize,
    pub max_message_length: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_broadcast_transmitter_stct"]
        [::std::mem::size_of::<aeron_broadcast_transmitter_stct>() - 32usize];
    ["Alignment of aeron_broadcast_transmitter_stct"]
        [::std::mem::align_of::<aeron_broadcast_transmitter_stct>() - 8usize];
    ["Offset of field: aeron_broadcast_transmitter_stct::buffer"]
        [::std::mem::offset_of!(aeron_broadcast_transmitter_stct, buffer) - 0usize];
    ["Offset of field: aeron_broadcast_transmitter_stct::descriptor"]
        [::std::mem::offset_of!(aeron_broadcast_transmitter_stct, descriptor) - 8usize];
    ["Offset of field: aeron_broadcast_transmitter_stct::capacity"]
        [::std::mem::offset_of!(aeron_broadcast_transmitter_stct, capacity) - 16usize];
    ["Offset of field: aeron_broadcast_transmitter_stct::max_message_length"]
        [::std::mem::offset_of!(aeron_broadcast_transmitter_stct, max_message_length) - 24usize];
};
impl Default for aeron_broadcast_transmitter_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_broadcast_transmitter_t = aeron_broadcast_transmitter_stct;
unsafe extern "C" {
    pub fn aeron_broadcast_transmitter_init(
        transmitter: *mut aeron_broadcast_transmitter_t,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_broadcast_transmitter_transmit(
        transmitter: *mut aeron_broadcast_transmitter_t,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> ::std::os::raw::c_int;
}
#[repr(C, packed(4))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_rb_descriptor_stct {
    pub begin_pad: [u8; 128usize],
    pub tail_position: i64,
    pub tail_pad: [u8; 120usize],
    pub head_cache_position: i64,
    pub head_cache_pad: [u8; 120usize],
    pub head_position: i64,
    pub head_pad: [u8; 120usize],
    pub correlation_counter: i64,
    pub correlation_counter_pad: [u8; 120usize],
    pub consumer_heartbeat: i64,
    pub consumer_heartbeat_pad: [u8; 120usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_rb_descriptor_stct"]
        [::std::mem::size_of::<aeron_rb_descriptor_stct>() - 768usize];
    ["Alignment of aeron_rb_descriptor_stct"]
        [::std::mem::align_of::<aeron_rb_descriptor_stct>() - 4usize];
    ["Offset of field: aeron_rb_descriptor_stct::begin_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, begin_pad) - 0usize];
    ["Offset of field: aeron_rb_descriptor_stct::tail_position"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, tail_position) - 128usize];
    ["Offset of field: aeron_rb_descriptor_stct::tail_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, tail_pad) - 136usize];
    ["Offset of field: aeron_rb_descriptor_stct::head_cache_position"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, head_cache_position) - 256usize];
    ["Offset of field: aeron_rb_descriptor_stct::head_cache_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, head_cache_pad) - 264usize];
    ["Offset of field: aeron_rb_descriptor_stct::head_position"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, head_position) - 384usize];
    ["Offset of field: aeron_rb_descriptor_stct::head_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, head_pad) - 392usize];
    ["Offset of field: aeron_rb_descriptor_stct::correlation_counter"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, correlation_counter) - 512usize];
    ["Offset of field: aeron_rb_descriptor_stct::correlation_counter_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, correlation_counter_pad) - 520usize];
    ["Offset of field: aeron_rb_descriptor_stct::consumer_heartbeat"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, consumer_heartbeat) - 640usize];
    ["Offset of field: aeron_rb_descriptor_stct::consumer_heartbeat_pad"]
        [::std::mem::offset_of!(aeron_rb_descriptor_stct, consumer_heartbeat_pad) - 648usize];
};
impl Default for aeron_rb_descriptor_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_rb_descriptor_t = aeron_rb_descriptor_stct;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_rb_record_descriptor_stct {
    pub length: i32,
    pub msg_type_id: i32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_rb_record_descriptor_stct"]
        [::std::mem::size_of::<aeron_rb_record_descriptor_stct>() - 8usize];
    ["Alignment of aeron_rb_record_descriptor_stct"]
        [::std::mem::align_of::<aeron_rb_record_descriptor_stct>() - 4usize];
    ["Offset of field: aeron_rb_record_descriptor_stct::length"]
        [::std::mem::offset_of!(aeron_rb_record_descriptor_stct, length) - 0usize];
    ["Offset of field: aeron_rb_record_descriptor_stct::msg_type_id"]
        [::std::mem::offset_of!(aeron_rb_record_descriptor_stct, msg_type_id) - 4usize];
};
pub type aeron_rb_record_descriptor_t = aeron_rb_record_descriptor_stct;
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum aeron_rb_write_result_stct {
    AERON_RB_SUCCESS = 0,
    AERON_RB_ERROR = -2,
    AERON_RB_FULL = -1,
}
pub use self::aeron_rb_write_result_stct as aeron_rb_write_result_t;
pub type aeron_rb_handler_t = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
        arg4: *mut ::std::os::raw::c_void,
    ),
>;
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum aeron_rb_read_action_stct {
    AERON_RB_ABORT = 0,
    AERON_RB_BREAK = 1,
    AERON_RB_COMMIT = 2,
    AERON_RB_CONTINUE = 3,
}
pub use self::aeron_rb_read_action_stct as aeron_rb_read_action_t;
pub type aeron_rb_controlled_handler_t = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
        arg4: *mut ::std::os::raw::c_void,
    ) -> aeron_rb_read_action_t,
>;
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_mpsc_rb_stct {
    pub buffer: *mut u8,
    pub descriptor: *mut aeron_rb_descriptor_t,
    pub capacity: usize,
    pub max_message_length: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_mpsc_rb_stct"][::std::mem::size_of::<aeron_mpsc_rb_stct>() - 32usize];
    ["Alignment of aeron_mpsc_rb_stct"][::std::mem::align_of::<aeron_mpsc_rb_stct>() - 8usize];
    ["Offset of field: aeron_mpsc_rb_stct::buffer"]
        [::std::mem::offset_of!(aeron_mpsc_rb_stct, buffer) - 0usize];
    ["Offset of field: aeron_mpsc_rb_stct::descriptor"]
        [::std::mem::offset_of!(aeron_mpsc_rb_stct, descriptor) - 8usize];
    ["Offset of field: aeron_mpsc_rb_stct::capacity"]
        [::std::mem::offset_of!(aeron_mpsc_rb_stct, capacity) - 16usize];
    ["Offset of field: aeron_mpsc_rb_stct::max_message_length"]
        [::std::mem::offset_of!(aeron_mpsc_rb_stct, max_message_length) - 24usize];
};
impl Default for aeron_mpsc_rb_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_mpsc_rb_t = aeron_mpsc_rb_stct;
unsafe extern "C" {
    pub fn aeron_mpsc_rb_init(
        ring_buffer: *mut aeron_mpsc_rb_t,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_write(
        ring_buffer: *mut aeron_mpsc_rb_t,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> aeron_rb_write_result_t;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_try_claim(
        ring_buffer: *mut aeron_mpsc_rb_t,
        msg_type_id: i32,
        length: usize,
    ) -> i32;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_commit(
        ring_buffer: *mut aeron_mpsc_rb_t,
        offset: i32,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_abort(
        ring_buffer: *mut aeron_mpsc_rb_t,
        offset: i32,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_read(
        ring_buffer: *mut aeron_mpsc_rb_t,
        handler: aeron_rb_handler_t,
        clientd: *mut ::std::os::raw::c_void,
        message_count_limit: usize,
    ) -> usize;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_controlled_read(
        ring_buffer: *mut aeron_mpsc_rb_t,
        handler: aeron_rb_controlled_handler_t,
        clientd: *mut ::std::os::raw::c_void,
        message_count_limit: usize,
    ) -> usize;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_next_correlation_id(ring_buffer: *mut aeron_mpsc_rb_t) -> i64;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_consumer_heartbeat_time(ring_buffer: *mut aeron_mpsc_rb_t, now_ms: i64);
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_consumer_heartbeat_time_value(ring_buffer: *mut aeron_mpsc_rb_t) -> i64;
}
unsafe extern "C" {
    pub fn aeron_mpsc_rb_unblock(ring_buffer: *mut aeron_mpsc_rb_t) -> bool;
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct iovec {
    pub iov_base: *mut ::std::os::raw::c_void,
    pub iov_len: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of iovec"][::std::mem::size_of::<iovec>() - 16usize];
    ["Alignment of iovec"][::std::mem::align_of::<iovec>() - 8usize];
    ["Offset of field: iovec::iov_base"][::std::mem::offset_of!(iovec, iov_base) - 0usize];
    ["Offset of field: iovec::iov_len"][::std::mem::offset_of!(iovec, iov_len) - 8usize];
};
impl Default for iovec {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct aeron_spsc_rb_stct {
    pub buffer: *mut u8,
    pub descriptor: *mut aeron_rb_descriptor_t,
    pub capacity: usize,
    pub max_message_length: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of aeron_spsc_rb_stct"][::std::mem::size_of::<aeron_spsc_rb_stct>() - 32usize];
    ["Alignment of aeron_spsc_rb_stct"][::std::mem::align_of::<aeron_spsc_rb_stct>() - 8usize];
    ["Offset of field: aeron_spsc_rb_stct::buffer"]
        [::std::mem::offset_of!(aeron_spsc_rb_stct, buffer) - 0usize];
    ["Offset of field: aeron_spsc_rb_stct::descriptor"]
        [::std::mem::offset_of!(aeron_spsc_rb_stct, descriptor) - 8usize];
    ["Offset of field: aeron_spsc_rb_stct::capacity"]
        [::std::mem::offset_of!(aeron_spsc_rb_stct, capacity) - 16usize];
    ["Offset of field: aeron_spsc_rb_stct::max_message_length"]
        [::std::mem::offset_of!(aeron_spsc_rb_stct, max_message_length) - 24usize];
};
impl Default for aeron_spsc_rb_stct {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type aeron_spsc_rb_t = aeron_spsc_rb_stct;
unsafe extern "C" {
    pub fn aeron_spsc_rb_init(
        ring_buffer: *mut aeron_spsc_rb_t,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_write(
        ring_buffer: *mut aeron_spsc_rb_t,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> aeron_rb_write_result_t;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_writev(
        ring_buffer: *mut aeron_spsc_rb_t,
        msg_type_id: i32,
        iov: *const iovec,
        iovcnt: ::std::os::raw::c_int,
    ) -> aeron_rb_write_result_t;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_try_claim(
        ring_buffer: *mut aeron_spsc_rb_t,
        msg_type_id: i32,
        length: usize,
    ) -> i32;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_commit(
        ring_buffer: *mut aeron_spsc_rb_t,
        offset: i32,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_abort(
        ring_buffer: *mut aeron_spsc_rb_t,
        offset: i32,
    ) -> ::std::os::raw::c_int;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_read(
        ring_buffer: *mut aeron_spsc_rb_t,
        handler: aeron_rb_handler_t,
        clientd: *mut ::std::os::raw::c_void,
        message_count_limit: usize,
    ) -> usize;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_controlled_read(
        ring_buffer: *mut aeron_spsc_rb_t,
        handler: aeron_rb_controlled_handler_t,
        clientd: *mut ::std::os::raw::c_void,
        message_count_limit: usize,
    ) -> usize;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_next_correlation_id(ring_buffer: *mut aeron_spsc_rb_t) -> i64;
}
unsafe extern "C" {
    pub fn aeron_spsc_rb_consumer_heartbeat_time(ring_buffer: *mut aeron_spsc_rb_t, time_ms: i64);
}
