use std::os::raw::c_void;

unsafe impl Sync for AeronSpscRb {}
unsafe impl Sync for AeronMpscRb {}
unsafe impl Send for AeronSpscRb {}
unsafe impl Send for AeronMpscRb {}
unsafe impl Send for AeronBroadcastTransmitter {}
unsafe impl Send for AeronBroadcastReceiver {}

pub const AERON_BROADCAST_BUFFER_TRAILER_LENGTH: usize = size_of::<aeron_broadcast_descriptor_t>();

macro_rules! impl_buffer_methods {
    ($t:ty) => {
        impl<'a> $t {
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
        }
    };
}

impl_buffer_methods!(AeronBroadcastTransmitter<'a>);
impl_buffer_methods!(AeronBroadcastReceiver<'a>);
impl_buffer_methods!(AeronSpscRb<'a>);
impl_buffer_methods!(AeronMpscRb<'a>);

macro_rules! impl_from_vec_and_new_with_capacity {
    ($t:ident, $slot:ident, $descriptor:ty) => {
        pub struct $slot<'a> {
            pub idx: i32,
            pub length: usize,
            commited: bool,
            rb: &'a $t,
        }

        impl<'a> $slot<'a> {
            pub fn commit(mut self) -> Result<i32, AeronCError> {
                self.commited = true;
                self.rb.commit(self.idx)
            }
            pub fn abort(mut self) -> Result<i32, AeronCError> {
                self.commited = true;
                self.rb.abort(self.idx)
            }
            pub fn buffer_mut(&self) -> &mut [u8] {
                self.rb.buffer_at_mut(self.idx as usize, self.length)
            }
        }

        impl<'a> std::ops::Deref for $slot<'a> {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
                self.buffer_mut()
            }
        }

        impl<'a> std::ops::DerefMut for $slot<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.buffer_mut()
            }
        }

        impl<'a> Drop for $slot<'a> {
            fn drop(&mut self) {
                if !self.commited {
                    let _ = self.rb.commit(self.idx);
                }
            }
        }

        impl $t {
            pub fn try_claim_slice<'a>(
                &'a self,
                msg_type_id: i32,
                length: usize,
            ) -> Result<$slot<'a>, AeronCError> {
                let idx = self.try_claim(msg_type_id, length);
                if idx <= 0 {
                    Err(AeronCError::from_code(idx))
                } else {
                    Ok($slot {
                        idx,
                        length,
                        commited: false,
                        rb: self,
                    })
                }
            }

            pub fn from_slice(buffer: &mut [u8], max_msg_size: usize) -> Result<Self, AeronCError> {
                assert!(!buffer.is_empty());
                assert!(buffer.len().is_power_of_two());
                Self::new(
                    buffer.as_mut_ptr(),
                    &<$descriptor>::default(),
                    buffer.len(),
                    max_msg_size,
                )
            }

            pub fn new_with_capacity(
                capacity: usize,
                max_msg_size: usize,
            ) -> Result<Self, AeronCError> {
                assert!(capacity.is_power_of_two());
                Self::from_slice(vec![0u8; capacity].leak(), max_msg_size)
            }
        }
    };
}

impl_from_vec_and_new_with_capacity!(AeronSpscRb, AeronSpscRbSlot, AeronRbDescriptor);
impl_from_vec_and_new_with_capacity!(AeronMpscRb, AeronMpscRbSlot, AeronRbDescriptor);

impl<'a> AeronBroadcastTransmitter<'a> {
    pub fn from_slice(buffer: &mut [u8], max_msg_size: usize, descriptor: &'a AeronBroadcastDescriptor) -> Result<Self, AeronCError> {
        assert!(!buffer.is_empty());
        assert!((buffer.len() - AERON_BROADCAST_BUFFER_TRAILER_LENGTH).is_power_of_two());

        let ptr = buffer.as_mut_ptr();
        let broadcast = Self::new(
            ptr,
            descriptor,
            buffer.len(),
            max_msg_size,
        )?;
        broadcast.init(ptr as *mut _, buffer.len())?;
        Ok(broadcast)
    }

    pub fn transmit_msg(&self, msg_type_id: i32, msg: &[u8]) -> Result<i32, AeronCError> {
        debug_assert!(msg.len() > 0);
        debug_assert!(msg_type_id > 0);
        self.transmit(msg_type_id, msg.as_ptr() as *const _, msg.len())
    }
}

impl<'a> AeronBroadcastReceiver<'a> {
    pub fn from_slice(buffer: &mut [u8]) -> Result<Self, AeronCError> {
        assert!(!buffer.is_empty());
        let capacity = buffer.len();
        assert!((capacity - AERON_BROADCAST_BUFFER_TRAILER_LENGTH).is_power_of_two());

        let ptr = buffer.as_mut_ptr();
        let broadcast = Self::new(
            [0u8; 4096],
            ptr,
            &AeronBroadcastDescriptor::default(),
            capacity,
            capacity - 1,
            0,
            0,
            0,
            0,
        )?;
        broadcast.init(ptr as *mut _, capacity)?;
        Ok(broadcast)
    }
}

impl<'a> AeronSpscRb<'a> {
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

impl<'a> AeronMpscRb<'a> {
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
    pub handler: T,
}

impl<T: AeronRingBufferHandlerCallback> std::ops::Deref for AeronRingBufferHandlerWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}

impl<T: AeronRingBufferHandlerCallback> std::ops::DerefMut for AeronRingBufferHandlerWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handler
    }
}

impl<T: AeronRingBufferHandlerCallback> AeronRingBufferHandlerWrapper<T> {
    pub fn new(handler: T) -> Handler<Self> {
        Handler::leak(Self { handler })
    }
}
impl<T: AeronRingBufferHandlerCallback> AeronRbHandlerCallback
    for AeronRingBufferHandlerWrapper<T>
{
    fn handle_aeron_rb_handler(&mut self, msg_id: i32, buffer: *const c_void, length: usize) -> () {
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
