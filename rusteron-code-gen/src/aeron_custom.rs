// code here is included in all modules and extends generated classes

pub const AERON_IPC_STREAM: &'static str = "aeron:ipc";

unsafe impl Send for AeronCountersReader {}
unsafe impl Send for AeronSubscription {}
unsafe impl Sync for AeronSubscription {}
unsafe impl Send for AeronPublication {}
unsafe impl Sync for AeronPublication {}
unsafe impl Send for AeronExclusivePublication {}
unsafe impl Sync for AeronExclusivePublication {}
unsafe impl Send for AeronCounter {}
unsafe impl Sync for AeronCounter {}

impl AeronSubscription {
    /// Retrieves the channel URI for this subscription with any wildcard ports filled in.
    ///
    /// If the channel is not UDP or does not have a wildcard port (0), then it will return the original URI.
    ///
    /// # Errors
    /// Returns an `Error` if resolving the channel endpoint fails.
    ///
    /// # Returns
    /// A `Result` containing the resolved URI as a `String` on success, or an `Error` on failure.
    pub fn try_resolve_channel_endpoint_uri(&self) -> Result<String, AeronCError> {
        const BUFFER_CAPACITY: usize = 1024;
        let mut uri_buffer = vec![0u8; BUFFER_CAPACITY];
        let uri_ptr = uri_buffer.as_mut_ptr() as *mut std::os::raw::c_char;
        let bytes_written = self.try_resolve_channel_endpoint_port(uri_ptr, BUFFER_CAPACITY)?;
        let resolved_uri =
            String::from_utf8_lossy(&uri_buffer[..bytes_written as usize]).to_string();
        Ok(resolved_uri)
    }
}

impl AeronCounter {
    pub fn addr_atomic(&self) -> &std::sync::atomic::AtomicI64 {
        unsafe { std::sync::atomic::AtomicI64::from_ptr(self.addr()) }
    }

    pub fn get_constants(&self) -> Result<AeronCounterConstants, AeronCError> {
        let constants = AeronCounterConstants::default();
        self.constants(&constants)?;
        Ok(constants)
    }
}

impl AeronSubscription {
    pub fn async_add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
    ) -> Result<AeronAsyncDestination, AeronCError> {
        AeronAsyncDestination::aeron_subscription_async_add_destination(client, self, destination)
    }

    pub fn get_constants(&self) -> Result<AeronSubscriptionConstants, AeronCError> {
        let constants = AeronSubscriptionConstants::default();
        self.constants(&constants)?;
        Ok(constants)
    }

    pub fn add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
        timeout: std::time::Duration,
    ) -> Result<(), AeronCError> {
        let result = self.async_add_destination(client, destination)?;
        if result
            .aeron_subscription_async_destination_poll()
            .unwrap_or_default()
            > 0
        {
            return Ok(());
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if result
                .aeron_subscription_async_destination_poll()
                .unwrap_or_default()
                > 0
            {
                return Ok(());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {} {:?}", destination, self);
        Err(AeronErrorType::TimedOut.into())
    }
}

impl AeronExclusivePublication {
    pub fn async_add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
    ) -> Result<AeronAsyncDestination, AeronCError> {
        AeronAsyncDestination::aeron_exclusive_publication_async_add_destination(
            client,
            self,
            destination,
        )
    }

    pub fn get_constants(&self) -> Result<AeronPublicationConstants, AeronCError> {
        let constants = AeronPublicationConstants::default();
        self.constants(&constants)?;
        Ok(constants)
    }

    pub fn add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
        timeout: std::time::Duration,
    ) -> Result<(), AeronCError> {
        let result = self.async_add_destination(client, destination)?;
        if result
            .aeron_subscription_async_destination_poll()
            .unwrap_or_default()
            > 0
        {
            return Ok(());
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if result
                .aeron_subscription_async_destination_poll()
                .unwrap_or_default()
                > 0
            {
                return Ok(());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {} {:?}", destination, self);
        Err(AeronErrorType::TimedOut.into())
    }
}

impl AeronPublication {
    pub fn async_add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
    ) -> Result<AeronAsyncDestination, AeronCError> {
        AeronAsyncDestination::aeron_publication_async_add_destination(client, self, destination)
    }

    pub fn get_constants(&self) -> Result<AeronPublicationConstants, AeronCError> {
        let constants = AeronPublicationConstants::default();
        self.constants(&constants)?;
        Ok(constants)
    }

    pub fn add_destination(
        &mut self,
        client: &Aeron,
        destination: &str,
        timeout: std::time::Duration,
    ) -> Result<(), AeronCError> {
        let result = self.async_add_destination(client, destination)?;
        if result
            .aeron_subscription_async_destination_poll()
            .unwrap_or_default()
            > 0
        {
            return Ok(());
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if result
                .aeron_subscription_async_destination_poll()
                .unwrap_or_default()
                > 0
            {
                return Ok(());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {} {:?}", destination, self);
        Err(AeronErrorType::TimedOut.into())
    }
}

impl AeronCountersReader {
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
        let mut result = String::with_capacity(max_length);
        self.get_counter_label_into(counter_id, &mut result)?;
        Ok(result)
    }

    #[inline]
    #[doc = "Get the label for a counter."]
    pub fn get_counter_label_into(
        &self,
        counter_id: i32,
        dst: &mut String,
    ) -> Result<(), AeronCError> {
        unsafe {
            let capacity = dst.capacity();
            let vec = dst.as_mut_vec();
            vec.set_len(capacity);
            self.counter_label(counter_id, vec.as_mut_ptr() as *mut _, capacity)?;
            let mut len = 0;
            loop {
                if len == capacity {
                    break;
                }
                let val = vec[len];
                if val == 0 {
                    break;
                }
                len += 1;
            }
            vec.set_len(len);
        }
        Ok(())
    }

    #[inline]
    #[doc = "Get the key for a counter."]
    pub fn get_counter_key(&self, counter_id: i32) -> Result<Vec<u8>, AeronCError> {
        let mut dst = Vec::new();
        self.get_counter_key_into(counter_id, &mut dst)?;
        Ok(dst)
    }

    #[inline]
    #[doc = "Get the key for a counter."]
    pub fn get_counter_key_into(
        &self,
        counter_id: i32,
        dst: &mut Vec<u8>,
    ) -> Result<(), AeronCError> {
        let mut key_ptr: *mut u8 = std::ptr::null_mut();
        unsafe {
            let result = bindings::aeron_counters_reader_metadata_key(
                self.get_inner(),
                counter_id,
                &mut key_ptr,
            );
            if result < 0 || key_ptr.is_null() {
                return Err(AeronCError::from_code(result));
            }

            loop {
                let val = *key_ptr.add(dst.len());
                if val == 0 {
                    break;
                } else {
                    dst.push(val);
                }
            }
            Ok(())
        }
    }

    pub fn get_counter_value(&self, counter_id: i32) -> i64 {
        unsafe { *self.addr(counter_id) }
    }
}

impl Aeron {
    pub fn new_blocking(
        context: &AeronContext,
        timeout: std::time::Duration,
    ) -> Result<Self, AeronCError> {
        if let Ok(aeron) = Aeron::new(&context) {
            return Ok(aeron);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Ok(aeron) = Aeron::new(&context) {
                return Ok(aeron);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed to create aeron client for {:?}", context);
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
    ) -> Result<(Handler<AeronFragmentAssembler>, Handler<T>), AeronCError> {
        let handler = Handler::leak(handler);
        Ok((
            Handler::leak(AeronFragmentAssembler::new(Some(&handler))?),
            handler,
        ))
    }
}
impl<T: AeronControlledFragmentHandlerCallback> Handler<T> {
    pub fn leak_with_controlled_fragment_assembler(
        handler: T,
    ) -> Result<(Handler<AeronControlledFragmentAssembler>, Handler<T>), AeronCError> {
        let handler = Handler::leak(handler);
        Ok((
            Handler::leak(AeronControlledFragmentAssembler::new(Some(&handler))?),
            handler,
        ))
    }
}

impl AeronBufferClaim {
    #[inline]
    pub fn data_mut(&self) -> &mut [u8] {
        debug_assert!(!self.data.is_null());
        unsafe { std::slice::from_raw_parts_mut(self.data, self.length) }
    }

    #[inline]
    pub fn frame_header_mut(&self) -> &mut aeron_header_values_frame_t {
        unsafe { &mut *self.frame_header.cast::<aeron_header_values_frame_t>() }
    }
}
