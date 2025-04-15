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
//! - **`backtrace`** - When enabled will log a backtrace for each AeronCError
//! - **`extra-logging`** - When enabled will log when resource is created and destroyed. useful if your seeing a segfault due to a resource being closed
//! - **`precompile`** - When enabled will use precompiled c code instead of requiring cmake and java to me installed

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!(concat!(env!("OUT_DIR"), "/rb_custom.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;
    use std::rc::Rc;

    #[test]
    pub fn spsc_normal() -> Result<(), Box<dyn error::Error>> {
        let rb = Rc::new(AeronSpscRb::new_with_capacity(1024 * 1024, 1024)?);
        let rb2 = rb.clone();

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
                assert_eq!(buffer[0], (msg_type_id - 1) as u8)
            }
        }
        let handler = AeronRingBufferHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb2.read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb2.read(Some(&handler), 10));

        Ok(())
    }

    #[test]
    pub fn spsc_control() -> Result<(), Box<dyn error::Error>> {
        let mut buff = vec![0u8; 1024 * 1024];
        let rb = AeronSpscRb::from_slice(&mut buff, 1024)?;

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
                assert_eq!(buffer[0], (msg_type_id - 1) as u8);
                aeron_rb_read_action_stct::AERON_RB_COMMIT
            }
        }
        let rb = AeronSpscRb::from_slice(&mut buff, 1024)?;
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
        let rb = Rc::new(AeronMpscRb::new_with_capacity(1024 * 1024, 1024)?);

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
        let rb = Rc::new(AeronMpscRb::new_with_capacity(1024 * 1024, 1024)?);

        for i in 0..100 {
            // msg_type_id must >0
            let idx = rb.try_claim(i + 1, 4);
            assert!(idx >= 0);
            let slot = rb.buffer_at_mut(idx as usize, 4);
            slot[0] = i as u8;
            rb.commit(idx)?;
        }

        let rb2 = rb.clone();

        struct Reader {}
        impl AeronRingBufferControlledHandlerCallback for Reader {
            fn handle_aeron_controlled_rb_handler(
                &mut self,
                msg_type_id: i32,
                buffer: &[u8],
            ) -> aeron_rb_read_action_t {
                assert_eq!(buffer[0], (msg_type_id - 1) as u8);
                aeron_rb_read_action_stct::AERON_RB_COMMIT
            }
        }
        let handler = AeronRingBufferControlledHandlerWrapper::new(Reader {});
        for i in 0..10 {
            let read = rb2.controlled_read_msgs(&handler, 10);
            assert_eq!(10, read);
        }

        assert_eq!(0, rb2.controlled_read_msgs(&handler, 10));

        Ok(())
    }

    #[test]
    pub fn broadcast() -> Result<(), Box<dyn error::Error>> {
        let mut vec = vec![0u8; 1024 * 1024 + AERON_BROADCAST_BUFFER_TRAILER_LENGTH];
        let transmitter = AeronBroadcastTransmitter::from_slice(vec.as_mut_slice(), 1024)?;
        let receiver = AeronBroadcastReceiver::from_slice(vec.as_mut_slice())?;
        let receiver2 = AeronBroadcastReceiver::from_slice(vec.as_mut_slice())?;
        let receiver3 = AeronBroadcastReceiver::from_slice(vec.as_mut_slice())?;

        for i in 0..100 {
            // msg_type_id must >0
            let mut msg = [0u8; 4];
            msg[0] = i as u8;
            let idx = transmitter.transmit_msg(i + 1, &msg).unwrap();
            assert!(idx >= 0);
        }

        struct Reader {}
        impl AeronBroadcastReceiverHandlerCallback for Reader {
            fn handle_aeron_broadcast_receiver_handler(
                &mut self,
                msg_type_id: i32,
                buffer: &mut [u8],
            ) -> () {
                assert_eq!(buffer[0], (msg_type_id - 1) as u8);
            }
        }
        let handler = Handler::leak(Reader {});
        for rec in [&receiver, &receiver2, &receiver3] {
            for i in 0..100 {
                let read = rec.receive(Some(&handler)).unwrap();
                assert!(read > 0);
            }
        }

        assert_eq!(0, receiver.receive(Some(&handler)).unwrap_or_default());

        Ok(())
    }
}
