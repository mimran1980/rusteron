use libaeron_sys::*;
use super::AeronError;

pub struct Publication {
    resource: *mut aeron_publication_t,
}

impl Publication {
    pub fn new(resource: *mut aeron_publication_t) -> Self {
        Self { resource }
    }

    pub fn offer(&self, buffer: &[u8], offset: usize, length: usize) -> Result<i64, AeronError> {
        let result = unsafe { aeron_publication_offer(self.resource, buffer.as_ptr() as *const _, offset as i32, length as i32, std::ptr::null()) };
        if result < 0 {
            Err(AeronError::from(result as i32))
        } else {
            Ok(result)
        }
    }
}