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

impl AeronCnc {
    pub fn new(aeron_dir: &str) -> Result<AeronCnc, AeronCError> {
        let c_string = std::ffi::CString::new(aeron_dir).expect("CString conversion failed");
        let resource = ManagedCResource::new(
            move |cnc| unsafe { aeron_cnc_init(cnc, c_string.as_ptr(), 0) },
            Some(Box::new(move |cnc| unsafe {
                aeron_cnc_close(*cnc);
                0
            })),
            false,
            None,
        )?;

        let result = Self {
            inner: std::rc::Rc::new(resource),
        };
        Ok(result)
    }

    #[doc = " Gets the timestamp of the last heartbeat sent to the media driver from any client.\n\n @param aeron_cnc to query\n @return last heartbeat timestamp in ms."]
    pub fn get_to_driver_heartbeat_ms(&self) -> Result<i64, AeronCError> {
        unsafe {
            let timestamp = aeron_cnc_to_driver_heartbeat(self.get_inner());
            if timestamp >= 0 {
                return Ok(timestamp);
            } else {
                return Err(AeronCError::from_code(timestamp as i32));
            }
        }
    }
}

impl AeronCncMetadata {
    pub fn load_from_file(aeron_dir: &str) -> Result<Self, AeronCError> {
        let aeron_dir = std::ffi::CString::new(aeron_dir).expect("CString::new failed");
        let mapped_file = std::rc::Rc::new(std::cell::RefCell::new(aeron_mapped_file_t {
            addr: std::ptr::null_mut(),
            length: 0,
        }));
        let mapped_file2 = std::rc::Rc::clone(&mapped_file);
        let resource = ManagedCResource::new(
            move |ctx| {
                let result = unsafe {
                    aeron_cnc_map_file_and_load_metadata(
                        aeron_dir.as_ptr(),
                        mapped_file.borrow_mut().deref_mut() as *mut aeron_mapped_file_t,
                        ctx,
                    )
                };
                if result == aeron_cnc_load_result_t::AERON_CNC_LOAD_SUCCESS {
                    1
                } else {
                    -1
                }
            },
            Some(Box::new(move |ctx| unsafe {
                aeron_unmap(mapped_file2.borrow_mut().deref_mut() as *mut aeron_mapped_file_t)
            })),
            false,
            None,
        )?;

        let result = Self {
            inner: std::rc::Rc::new(resource),
        };
        Ok(result)
    }
}

impl AeronSubscription {
    pub fn close_with_no_args(&mut self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl AeronPublication {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl AeronExclusivePublication {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl AeronCounter {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl AeronCounter {
    #[inline]
    pub fn addr_atomic(&self) -> &std::sync::atomic::AtomicI64 {
        unsafe { std::sync::atomic::AtomicI64::from_ptr(self.addr()) }
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

impl std::str::FromStr for AeronUriStringBuilder {
    type Err = AeronCError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let builder = AeronUriStringBuilder::default();
        builder.init_on_string(s)?;
        Ok(builder)
    }
}

impl AeronUriStringBuilder {
    #[inline]
    pub fn build(&self, max_str_length: usize) -> Result<String, AeronCError> {
        let mut result = String::with_capacity(max_str_length);
        self.build_into(&mut result)?;
        Ok(result)
    }

    pub fn media(&self, value: Media) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_STRING_BUILDER_MEDIA_KEY);
        self.put(key, value.as_str())?;
        Ok(self)
    }

    pub fn control_mode(&self, value: ControlMode) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_CONTROL_MODE_KEY);
        self.put(key, value.as_str())?;
        Ok(self)
    }

    pub fn prefix(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_STRING_BUILDER_PREFIX_KEY);
        self.put(key, value)?;
        Ok(self)
    }

    fn strip_null_terminator(bytes: &[u8]) -> &str {
        let len = bytes.len() - 1;
        unsafe { std::str::from_utf8_unchecked(&bytes[..len]) }
    }

    pub fn initial_term_id(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_INITIAL_TERM_ID_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn term_id(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TERM_ID_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn term_offset(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TERM_OFFSET_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn alias(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_ALIAS_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn term_length(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TERM_LENGTH_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn linger_timeout(&self, value: i64) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_LINGER_TIMEOUT_KEY);
        self.put_int64(key, value)?;
        Ok(self)
    }
    pub fn mtu_length(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_MTU_LENGTH_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn ttl(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_TTL_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn sparse_term(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_SPARSE_TERM_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn reliable(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_RELIABLE_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn eos(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_EOS_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn tether(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TETHER_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn tags(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TAGS_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn endpoint(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_ENDPOINT_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn interface(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_INTERFACE_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn control(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_UDP_CHANNEL_CONTROL_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn session_id(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_SESSION_ID_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn group(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_GROUP_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn rejoin(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_REJOIN_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn fc(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_FC_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn gtag(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_GTAG_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn cc(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_CC_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn spies_simulate_connection(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_SPIES_SIMULATE_CONNECTION_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn ats(&self, value: bool) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_ATS_KEY);
        self.put(key, if value { "true" } else { "false" })?;
        Ok(self)
    }
    pub fn socket_sndbuf(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_SOCKET_SNDBUF_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn socket_rcvbuf(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_SOCKET_RCVBUF_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn receiver_window(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_RECEIVER_WINDOW_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn media_rcv_timestamp_offset(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_MEDIA_RCV_TIMESTAMP_OFFSET_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn channel_rcv_timestamp_offset(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_CHANNEL_RCV_TIMESTAMP_OFFSET_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn channel_snd_timestamp_offset(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_CHANNEL_SND_TIMESTAMP_OFFSET_KEY);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn timestamp_offset_reserved(&self, value: &str) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_TIMESTAMP_OFFSET_RESERVED);
        self.put(key, value)?;
        Ok(self)
    }
    pub fn response_correlation_id(&self, value: i64) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_RESPONSE_CORRELATION_ID_KEY);
        self.put_int64(key, value)?;
        Ok(self)
    }
    pub fn nak_delay(&self, value: i64) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_NAK_DELAY_KEY);
        self.put_int64(key, value)?;
        Ok(self)
    }
    pub fn untethered_window_limit_timeout(&self, value: i64) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_UNTETHERED_WINDOW_LIMIT_TIMEOUT_KEY);
        self.put_int64(key, value)?;
        Ok(self)
    }
    pub fn untethered_resting_timeout(&self, value: i64) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_UNTETHERED_RESTING_TIMEOUT_KEY);
        self.put_int64(key, value)?;
        Ok(self)
    }
    pub fn max_resend(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_MAX_RESEND_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn stream_id(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_STREAM_ID_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }
    pub fn publication_window(&self, value: i32) -> Result<&Self, AeronCError> {
        let key: &str = Self::strip_null_terminator(AERON_URI_PUBLICATION_WINDOW_KEY);
        self.put_int32(key, value)?;
        Ok(self)
    }

    #[inline]
    pub fn build_into(&self, dst: &mut String) -> Result<(), AeronCError> {
        self.sprint_into(dst)?;
        Ok(())
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

    #[inline]
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

pub struct AeronErrorLogger;
impl AeronErrorHandlerCallback for AeronErrorLogger {
    fn handle_aeron_error_handler(&mut self, error_code: std::ffi::c_int, msg: &str) -> () {
        log::error!("aeron error {}: {}", error_code, msg);
    }
}
unsafe impl Send for AeronErrorLogger {}
unsafe impl Sync for AeronErrorLogger {}
