unsafe impl Send for AeronPublication {}
unsafe impl Send for AeronCounter {}

impl AeronCounter {
    pub fn addr_atomic(&self) -> &std::sync::atomic::AtomicI64 {
        unsafe { std::sync::atomic::AtomicI64::from_ptr(self.addr()) }
    }
}

impl AeronCountersReader {
    #[inline]
    #[doc = "Get the registration id assigned to a counter."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader representing the this pointer."]
    #[doc = " \n**param** counter_id      for which the registration id is requested."]
    #[doc = " \n**param** registration_id pointer for value to be set on success."]
    #[doc = " \n**return** -1 on failure, 0 on success."]
    pub fn get_counter_registration_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        let mut result = 0;
        self.counter_registration_id(counter_id, &mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the owner id assigned to a counter which will typically be the client id."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader representing the this pointer."]
    #[doc = " \n**param** counter_id      for which the owner id is requested."]
    #[doc = " \n**param** owner_id        pointer for value to be set on success."]
    #[doc = " \n**return** -1 on failure, 0 on success."]
    pub fn get_counter_owner_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        let mut result = 0;
        self.counter_owner_id(counter_id, &mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the reference id assigned to a counter which will typically be the registration id of an associated Image,"]
    #[doc = " Subscription, Publication, etc."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader representing the this pointer."]
    #[doc = " \n**param** counter_id      for which the reference id is requested."]
    #[doc = " \n**param** reference_id    pointer for value to be set on success."]
    #[doc = " \n**return** -1 on failure, 0 on success."]
    pub fn get_counter_reference_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        let mut result = 0;
        self.counter_reference_id(counter_id, &mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the state for a counter."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader that contains the counter"]
    #[doc = " \n**param** counter_id to find"]
    #[doc = " \n**param** state out pointer for the current state to be stored in."]
    #[doc = " \n**return** -1 on failure, 0 on success."]
    pub fn get_counter_state(&self, counter_id: i32) -> Result<i32, AeronCError> {
        let mut result = 0;
        Ok(self.counter_state(counter_id, &mut result)?)
    }
    #[inline]
    #[doc = "Get the type id for a counter."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader that contains the counter"]
    #[doc = " \n**param** counter_id to find"]
    #[doc = " \n**param** type id out pointer for the current state to be stored in."]
    #[doc = " \n**return** -1 on failure, 0 on success."]
    pub fn get_counter_type_id(&self, counter_id: i32) -> Result<i32, AeronCError> {
        let mut result = 0;
        Ok(self.counter_type_id(counter_id, &mut result)?)
    }
    #[inline]
    #[doc = "Get the label for a counter."]
    #[doc = ""]
    #[doc = " \n**param** counters_reader that contains the counter"]
    #[doc = " \n**param** counter_id to find"]
    #[doc = " \n**param** buffer to store the counter in."]
    #[doc = " \n**param** buffer_length length of the output buffer"]
    #[doc = " \n**return** -1 on failure, number of characters copied to buffer on success."]
    pub fn get_counter_label(
        &self,
        counter_id: i32,
        max_length: usize,
    ) -> Result<String, AeronCError> {
        let mut buffer = vec![0; max_length];
        assert_eq!(buffer.len(), max_length);
        self.counter_label(counter_id, buffer.as_mut_ptr(), max_length)?;
        let mut result = String::with_capacity(max_length);
        for c in buffer {
            let b = c as u8;
            if b == 0 {
                break;
            }
            result.push(b as char);
        }
        Ok(result)
    }
}

impl Aeron {
    pub fn new_blocking(
        context: AeronContext,
        timeout: std::time::Duration,
    ) -> Result<Self, AeronCError> {
        if let Ok(aeron) = Aeron::new(context.clone()) {
            return Ok(aeron);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Ok(aeron) = Aeron::new(context.clone()) {
                return Ok(aeron);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!("failed to create aeron client for {:?}", context);
        Err(AeronErrorType::TimedOut.into())
    }
}

impl AeronFragmentHandlerCallback for AeronFragmentAssembler {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], header: AeronHeader) -> () {
        unsafe {
            aeron_fragment_assembler_handler(
                self.get_inner() as *mut _,
                buffer.as_ptr(),
                buffer.len(),
                header.get_inner(),
            )
        }
    }
}

impl AeronControlledFragmentHandlerCallback for AeronControlledFragmentAssembler {
    fn handle_aeron_controlled_fragment_handler(
        &mut self,
        buffer: &[u8],
        header: AeronHeader,
    ) -> aeron_controlled_fragment_handler_action_t {
        unsafe {
            aeron_controlled_fragment_assembler_handler(
                self.get_inner() as *mut _,
                buffer.as_ptr(),
                buffer.len(),
                header.get_inner(),
            )
        }
    }
}

impl<T: AeronFragmentHandlerCallback> Handler<T> {
    pub fn leak_with_fragment_assembler(
        handler: T,
    ) -> Result<Handler<AeronFragmentAssembler>, AeronCError> {
        let handler = Handler::leak(handler);
        Ok(Handler::leak(AeronFragmentAssembler::new(Some(&handler))?))
    }
}
impl<T: AeronControlledFragmentHandlerCallback> Handler<T> {
    pub fn leak_with_controlled_fragment_assembler(
        handler: T,
    ) -> Result<Handler<AeronControlledFragmentAssembler>, AeronCError> {
        let handler = Handler::leak(handler);
        Ok(Handler::leak(AeronControlledFragmentAssembler::new(Some(
            &handler,
        ))?))
    }
}

impl AeronBufferClaim {
    pub fn data_mut(&self) -> &mut [u8] {
        // std::thread::sleep(std::time::Duration::from_secs(100));
        debug_assert!(!self.data.is_null());
        unsafe { std::slice::from_raw_parts_mut(self.data, self.length) }
    }
}
