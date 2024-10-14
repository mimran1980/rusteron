use std::rc::Rc;
use libaeron_sys::*;
use super::AeronError;
use rusteron_common::ManagedCResource;

pub struct Subscription {
    resource: Rc<ManagedCResource<aeron_subscription_t>>,
}

impl Subscription {
    pub fn new(channel: &str, stream_id: i32) -> Result<Self, AeronError> {
        unsafe {
            let mut subscription: *mut aeron_subscription_t = std::ptr::null_mut();
            let result = aeron_async_add_subscription(
                &mut subscription,
                channel.as_ptr() as *const i8,
                stream_id,
            );

            if result < 0 {
                return Err(AeronError::from(result));
            }

            Ok(Self {
                resource: Rc::new(ManagedCResource::new(subscription, |res| unsafe {
                    aeron_subscription_close(res, None, std::ptr::null_mut())
                })),
            })
        }
    }

    pub fn poll<F>(&self, handler: F, fragment_limit: i32) -> Result<i32, AeronError>
    where
        F: Fn(&aeron_header_t, &[u8]),
    {
        let result = unsafe { aeron_subscription_poll(self.resource.as_ptr(), handler_adapter(handler), fragment_limit) };
        if result < 0 {
            Err(AeronError::from(result))
        } else {
            Ok(result)
        }
    }
}

unsafe extern "C" fn handler_adapter<F>(handler: F) -> aeron_fragment_handler_t
where
    F: Fn(&aeron_header_t, &[u8]),
{
    unsafe extern "C" fn handler_wrapper<F>(clientd: *mut std::ffi::c_void, header: *mut aeron_header_t, buffer: *const std::ffi::c_uint8, length: i32)
    where
        F: Fn(&aeron_header_t, &[u8]),
    {
        let handler = &*(clientd as *mut F);
        let slice = std::slice::from_raw_parts(buffer, length as usize);
        handler(&*header, slice);
    }

    std::mem::transmute::<_, aeron_fragment_handler_t>(handler_wrapper::<F>)
}