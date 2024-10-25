use std::sync::atomic::AtomicI64;

unsafe impl Send for AeronPublication {}
unsafe impl Send for AeronCounter {}

impl AeronCounter {
    pub fn addr_atomic(&self) -> &AtomicI64 {
        unsafe { AtomicI64::from_ptr(self.addr()) }
    }
}