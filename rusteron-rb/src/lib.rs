#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    use std::os::raw::c_void;

    unsafe impl Sync for AeronSpscRb {}
    unsafe impl Sync for AeronMpscRb {}

    impl AeronSpscRb {
        #[inline]
        pub fn buffer_mut(&self) -> &mut [u8] {
            debug_assert!(!self.buffer.is_null());
            unsafe { std::slice::from_raw_parts_mut(self.buffer, self.capacity) }
        }

        #[inline]
        pub fn buffer_at_mut(&self, idx: usize, len: usize) -> &mut [u8] {
            debug_assert!(idx + len < self.capacity);
            debug_assert!(!self.buffer.is_null());
            unsafe { std::slice::from_raw_parts_mut(self.buffer.add(idx), len) }
        }

        pub fn from_vec(buffer: Vec<u8>, max_msg_size: usize) -> Result<Self, AeronCError> {
            assert!(!buffer.is_empty());
            let buffer = buffer.leak();
            Self::new(
                buffer.as_mut_ptr(),
                &AeronRbDescriptor::default(),
                buffer.len(),
                max_msg_size,
            )
        }

        pub fn new_with_capacity(
            capacity: usize,
            max_msg_size: usize,
        ) -> Result<Self, AeronCError> {
            assert!(capacity.is_power_of_two());
            Self::from_vec(vec![0u8; capacity], max_msg_size)
        }

        pub fn read_msgs<T: AeronRingBufferHandlerCallback>(
            &self,
            handler: &Handler<AeronRingBufferHandlerWrapper<T>>,
            message_count_limit: usize,
        ) -> usize {
            self.read(Some(handler), message_count_limit)
        }

        pub fn controlled_read_msgs<T: AeronRingBufferControlledHandlerCallback>(
            &self,
            handler: &Handler<AeronRingBufferControlledHandlerWrapper<T>>,
            message_count_limit: usize,
        ) -> usize {
            self.controlled_read(Some(handler), message_count_limit)
        }
    }

    impl AeronMpscRb {
        #[inline]
        pub fn buffer_mut(&self) -> &mut [u8] {
            debug_assert!(!self.buffer.is_null());
            unsafe { std::slice::from_raw_parts_mut(self.buffer, self.capacity) }
        }

        #[inline]
        pub fn buffer_at_mut(&self, idx: usize, len: usize) -> &mut [u8] {
            debug_assert!(idx + len < self.capacity);
            debug_assert!(!self.buffer.is_null());
            unsafe { std::slice::from_raw_parts_mut(self.buffer.add(idx), len) }
        }

        pub fn from_vec(buffer: Vec<u8>, max_msg_size: usize) -> Result<Self, AeronCError> {
            assert!(!buffer.is_empty());
            let buffer = buffer.leak();
            Self::new(
                buffer.as_mut_ptr(),
                &AeronRbDescriptor::default(),
                buffer.len(),
                max_msg_size,
            )
        }

        pub fn new_with_capacity(
            capacity: usize,
            max_msg_size: usize,
        ) -> Result<Self, AeronCError> {
            assert!(capacity.is_power_of_two());
            Self::from_vec(vec![0u8; capacity], max_msg_size)
        }

        pub fn read_msgs<T: AeronRingBufferHandlerCallback>(
            &self,
            handler: &Handler<AeronRingBufferHandlerWrapper<T>>,
            message_count_limit: usize,
        ) -> usize {
            self.read(Some(handler), message_count_limit)
        }

        pub fn controlled_read_msgs<T: AeronRingBufferControlledHandlerCallback>(
            &self,
            handler: &Handler<AeronRingBufferControlledHandlerWrapper<T>>,
            message_count_limit: usize,
        ) -> usize {
            self.controlled_read(Some(handler), message_count_limit)
        }
    }

    pub struct AeronRingBufferHandlerWrapper<T: AeronRingBufferHandlerCallback> {
        handler: T,
    }

    impl<T: AeronRingBufferHandlerCallback> AeronRingBufferHandlerWrapper<T> {
        pub fn new(handler: T) -> Handler<Self> {
            Handler::leak(Self { handler })
        }
    }
    impl<T: AeronRingBufferHandlerCallback> AeronRbHandlerCallback
        for AeronRingBufferHandlerWrapper<T>
    {
        fn handle_aeron_rb_handler(
            &mut self,
            msg_id: i32,
            buffer: *const c_void,
            length: usize,
        ) -> () {
            let buffer = unsafe { std::slice::from_raw_parts(buffer as *const u8, length) };
            self.handler.handle_aeron_rb_handler(msg_id, buffer);
        }
    }

    pub trait AeronRingBufferHandlerCallback {
        fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) -> ();
    }

    pub struct AeronRingBufferControlledHandlerWrapper<T: AeronRingBufferControlledHandlerCallback> {
        handler: T,
    }

    impl<T: AeronRingBufferControlledHandlerCallback> AeronRingBufferControlledHandlerWrapper<T> {
        pub fn new(handler: T) -> Handler<Self> {
            Handler::leak(Self { handler })
        }
    }
    impl<T: AeronRingBufferControlledHandlerCallback> AeronRbControlledHandlerCallback
        for AeronRingBufferControlledHandlerWrapper<T>
    {
        fn handle_aeron_rb_controlled_handler(
            &mut self,
            msg_id: i32,
            buffer: *const c_void,
            length: usize,
        ) -> aeron_rb_read_action_t {
            let buffer = unsafe { std::slice::from_raw_parts(buffer as *const u8, length) };
            self.handler
                .handle_aeron_controlled_rb_handler(msg_id, buffer)
        }
    }

    pub trait AeronRingBufferControlledHandlerCallback {
        fn handle_aeron_controlled_rb_handler(
            &mut self,
            msg_type_id: i32,
            buffer: &[u8],
        ) -> aeron_rb_read_action_t;
    }

    #[test]
    pub fn spsc_normal() -> Result<(), Box<dyn error::Error>> {
        let rb = AeronSpscRb::new_with_capacity(1024 * 1024, 1024)?;

        for i in 0..100 {
            // msg_type_id must >0
            let idx = rb.try_claim(i + 1, 4);
            assert!(idx >= 0);
            let slot = rb.buffer_at_mut(idx as usize, 4);
            slot[0] = i as u8;
            rb.commit(idx)?;
        }

        struct Reader {}
        impl AeronRingBufferHandlerCallback for Reader {
            fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) -> () {
                println!("msg_type_id: {msg_type_id}, buffer: {buffer:?}");
                assert_eq!(buffer[0], (msg_type_id - 1) as u8)
            }
        }
        let handler = AeronRingBufferHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb.read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb.read(Some(&handler), 10));

        Ok(())
    }

    #[test]
    pub fn spsc_control() -> Result<(), Box<dyn error::Error>> {
        let rb = AeronSpscRb::new_with_capacity(1024 * 1024, 1024)?;

        for i in 0..100 {
            // msg_type_id must >0
            let idx = rb.try_claim(i + 1, 4);
            assert!(idx >= 0);
            let slot = rb.buffer_at_mut(idx as usize, 4);
            slot[0] = i as u8;
            rb.commit(idx)?;
        }

        struct Reader {}
        impl AeronRingBufferControlledHandlerCallback for Reader {
            fn handle_aeron_controlled_rb_handler(
                &mut self,
                msg_type_id: i32,
                buffer: &[u8],
            ) -> aeron_rb_read_action_t {
                println!("msg_type_id: {msg_type_id}, buffer: {buffer:?}");
                assert_eq!(buffer[0], (msg_type_id - 1) as u8);
                aeron_rb_read_action_stct::AERON_RB_COMMIT
            }
        }
        let handler = AeronRingBufferControlledHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb.controlled_read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb.controlled_read_msgs(&handler, 10));

        Ok(())
    }

    #[test]
    pub fn mpsc_normal() -> Result<(), Box<dyn error::Error>> {
        let rb = AeronMpscRb::new_with_capacity(1024 * 1024, 1024)?;

        for i in 0..100 {
            // msg_type_id must >0
            let idx = rb.try_claim(i + 1, 4);
            assert!(idx >= 0);
            let slot = rb.buffer_at_mut(idx as usize, 4);
            slot[0] = i as u8;
            rb.commit(idx)?;
        }

        struct Reader {}
        impl AeronRingBufferHandlerCallback for Reader {
            fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) -> () {
                println!("msg_type_id: {msg_type_id}, buffer: {buffer:?}");
                assert_eq!(buffer[0], (msg_type_id - 1) as u8)
            }
        }
        let handler = AeronRingBufferHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb.read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb.read(Some(&handler), 10));

        Ok(())
    }

    #[test]
    pub fn mpsc_control() -> Result<(), Box<dyn error::Error>> {
        let rb = AeronMpscRb::new_with_capacity(1024 * 1024, 1024)?;

        for i in 0..100 {
            // msg_type_id must >0
            let idx = rb.try_claim(i + 1, 4);
            assert!(idx >= 0);
            let slot = rb.buffer_at_mut(idx as usize, 4);
            slot[0] = i as u8;
            rb.commit(idx)?;
        }

        struct Reader {}
        impl AeronRingBufferControlledHandlerCallback for Reader {
            fn handle_aeron_controlled_rb_handler(
                &mut self,
                msg_type_id: i32,
                buffer: &[u8],
            ) -> aeron_rb_read_action_t {
                println!("msg_type_id: {msg_type_id}, buffer: {buffer:?}");
                assert_eq!(buffer[0], (msg_type_id - 1) as u8);
                aeron_rb_read_action_stct::AERON_RB_COMMIT
            }
        }
        let handler = AeronRingBufferControlledHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb.controlled_read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb.controlled_read_msgs(&handler, 10));

        Ok(())
    }
}
