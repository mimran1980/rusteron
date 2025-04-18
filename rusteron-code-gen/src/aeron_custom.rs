// code here is included in all modules and extends generated classes
pub const AERON_IPC_STREAM: &'static str = "aeron:ipc";

unsafe impl<'a> Send for AeronCountersReader<'a> {}
unsafe impl<'a> Send for AeronSubscription<'a> {}
unsafe impl<'a> Sync for AeronSubscription<'a> {}
unsafe impl<'a> Send for AeronPublication<'a> {}
unsafe impl<'a> Sync for AeronPublication<'a> {}
unsafe impl<'a> Send for AeronExclusivePublication<'a> {}
unsafe impl<'a> Sync for AeronExclusivePublication<'a> {}
unsafe impl<'a> Send for AeronCounter<'a> {}
unsafe impl<'a> Sync for AeronCounter<'a> {}

impl<'a> AeronCnc<'a> {
    pub fn new(aeron_dir: &str) -> Result<AeronCnc<'a>, AeronCError> {
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
            inner: resource,
            _marker: Default::default()
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

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum AeronSystemCounterType {
    /// Running total of bytes sent for data over UDP, excluding IP headers.
    BytesSent = 0,
    /// Running total of bytes received for data over UDP, excluding IP headers.
    BytesReceived = 1,
    /// Failed offers to the receiver proxy suggesting back-pressure.
    ReceiverProxyFails = 2,
    /// Failed offers to the sender proxy suggesting back-pressure.
    SenderProxyFails = 3,
    /// Failed offers to the driver conductor proxy suggesting back-pressure.
    ConductorProxyFails = 4,
    /// Count of NAKs sent back to senders requesting re-transmits.
    NakMessagesSent = 5,
    /// Count of NAKs received from receivers requesting re-transmits.
    NakMessagesReceived = 6,
    /// Count of status messages sent back to senders for flow control.
    StatusMessagesSent = 7,
    /// Count of status messages received from receivers for flow control.
    StatusMessagesReceived = 8,
    /// Count of heartbeat data frames sent to indicate liveness in the absence of data to send.
    HeartbeatsSent = 9,
    /// Count of heartbeat data frames received to indicate liveness in the absence of data to send.
    HeartbeatsReceived = 10,
    /// Count of data packets re-transmitted as a result of NAKs.
    RetransmitsSent = 11,
    /// Count of packets received which under-run the current flow control window for images.
    FlowControlUnderRuns = 12,
    /// Count of packets received which over-run the current flow control window for images.
    FlowControlOverRuns = 13,
    /// Count of invalid packets received.
    InvalidPackets = 14,
    /// Count of errors observed by the driver and an indication to read the distinct error log.
    Errors = 15,
    /// Count of socket send operations which resulted in less than the packet length being sent.
    ShortSends = 16,
    /// Count of attempts to free log buffers no longer required by the driver that are still held by clients.
    FreeFails = 17,
    /// Count of the times a sender has entered the state of being back-pressured when it could have sent faster.
    SenderFlowControlLimits = 18,
    /// Count of the times a publication has been unblocked after a client failed to complete an offer within a timeout.
    UnblockedPublications = 19,
    /// Count of the times a command has been unblocked after a client failed to complete an offer within a timeout.
    UnblockedCommands = 20,
    /// Count of the times the channel endpoint detected a possible TTL asymmetry between its config and a new connection.
    PossibleTtlAsymmetry = 21,
    /// Current status of the ControllableIdleStrategy if configured.
    ControllableIdleStrategy = 22,
    /// Count of the times a loss gap has been filled when NAKs have been disabled.
    LossGapFills = 23,
    /// Count of the Aeron clients that have timed out without a graceful close.
    ClientTimeouts = 24,
    /// Count of the times a connection endpoint has been re-resolved resulting in a change.
    ResolutionChanges = 25,
    /// The maximum time spent by the conductor between work cycles.
    ConductorMaxCycleTime = 26,
    /// Count of the number of times the cycle time threshold has been exceeded by the conductor in its work cycle.
    ConductorCycleTimeThresholdExceeded = 27,
    /// The maximum time spent by the sender between work cycles.
    SenderMaxCycleTime = 28,
    /// Count of the number of times the cycle time threshold has been exceeded by the sender in its work cycle.
    SenderCycleTimeThresholdExceeded = 29,
    /// The maximum time spent by the receiver between work cycles.
    ReceiverMaxCycleTime = 30,
    /// Count of the number of times the cycle time threshold has been exceeded by the receiver in its work cycle.
    ReceiverCycleTimeThresholdExceeded = 31,
    /// The maximum time spent by the NameResolver in one of its operations.
    NameResolverMaxTime = 32,
    /// Count of the number of times the time threshold has been exceeded by the NameResolver.
    NameResolverTimeThresholdExceeded = 33,
    /// The version of the media driver.
    AeronVersion = 34,
    /// The total number of bytes currently mapped in log buffers, the CnC file, and the loss report.
    BytesCurrentlyMapped = 35,
    /// A minimum bound on the number of bytes re-transmitted as a result of NAKs.\n///\n/// MDC retransmits are only counted once; therefore, this is a minimum bound rather than the actual number\n/// of retransmitted bytes. Note that retransmitted bytes are not included in the `BytesSent` counter value.
    RetransmittedBytes = 36,
    /// A count of the number of times that the retransmit pool has been overflowed.
    RetransmitOverflow = 37,
    /// A count of the number of error frames received by this driver.
    ErrorFramesReceived = 38,
    /// A count of the number of error frames sent by this driver.
    ErrorFramesSent = 39,
    DummyLast = 40,
}

impl std::convert::TryFrom<i32> for AeronSystemCounterType {
    type Error = AeronCError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(AeronCError::from_code(value));
        }
        match value as u32 {
            0 => Ok(AeronSystemCounterType::BytesSent),
            1 => Ok(AeronSystemCounterType::BytesReceived),
            2 => Ok(AeronSystemCounterType::ReceiverProxyFails),
            3 => Ok(AeronSystemCounterType::SenderProxyFails),
            4 => Ok(AeronSystemCounterType::ConductorProxyFails),
            5 => Ok(AeronSystemCounterType::NakMessagesSent),
            6 => Ok(AeronSystemCounterType::NakMessagesReceived),
            7 => Ok(AeronSystemCounterType::StatusMessagesSent),
            8 => Ok(AeronSystemCounterType::StatusMessagesReceived),
            9 => Ok(AeronSystemCounterType::HeartbeatsSent),
            10 => Ok(AeronSystemCounterType::HeartbeatsReceived),
            11 => Ok(AeronSystemCounterType::RetransmitsSent),
            12 => Ok(AeronSystemCounterType::FlowControlUnderRuns),
            13 => Ok(AeronSystemCounterType::FlowControlOverRuns),
            14 => Ok(AeronSystemCounterType::InvalidPackets),
            15 => Ok(AeronSystemCounterType::Errors),
            16 => Ok(AeronSystemCounterType::ShortSends),
            17 => Ok(AeronSystemCounterType::FreeFails),
            18 => Ok(AeronSystemCounterType::SenderFlowControlLimits),
            19 => Ok(AeronSystemCounterType::UnblockedPublications),
            20 => Ok(AeronSystemCounterType::UnblockedCommands),
            21 => Ok(AeronSystemCounterType::PossibleTtlAsymmetry),
            22 => Ok(AeronSystemCounterType::ControllableIdleStrategy),
            23 => Ok(AeronSystemCounterType::LossGapFills),
            24 => Ok(AeronSystemCounterType::ClientTimeouts),
            25 => Ok(AeronSystemCounterType::ResolutionChanges),
            26 => Ok(AeronSystemCounterType::ConductorMaxCycleTime),
            27 => Ok(AeronSystemCounterType::ConductorCycleTimeThresholdExceeded),
            28 => Ok(AeronSystemCounterType::SenderMaxCycleTime),
            29 => Ok(AeronSystemCounterType::SenderCycleTimeThresholdExceeded),
            30 => Ok(AeronSystemCounterType::ReceiverMaxCycleTime),
            31 => Ok(AeronSystemCounterType::ReceiverCycleTimeThresholdExceeded),
            32 => Ok(AeronSystemCounterType::NameResolverMaxTime),
            33 => Ok(AeronSystemCounterType::NameResolverTimeThresholdExceeded),
            34 => Ok(AeronSystemCounterType::AeronVersion),
            35 => Ok(AeronSystemCounterType::BytesCurrentlyMapped),
            36 => Ok(AeronSystemCounterType::RetransmittedBytes),
            37 => Ok(AeronSystemCounterType::RetransmitOverflow),
            38 => Ok(AeronSystemCounterType::ErrorFramesReceived),
            39 => Ok(AeronSystemCounterType::ErrorFramesSent),
            40 => Ok(AeronSystemCounterType::DummyLast),
            _  => Err(AeronCError::from_code(-1)),
        }
    }
}

impl<'a> AeronCncMetadata<'a> {
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
            inner: resource,
            _marker: Default::default()
        };
        Ok(result)
    }
}

impl<'a> AeronSubscription<'a> {
    pub fn close_with_no_args(&mut self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl<'a> AeronPublication<'a> {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }

    /// sometimes when you first connect, is_connected = true, but you get backpressure as position is 0
    /// this will check if both publication is connected and position > 0
    pub fn is_ready(&self) -> bool {
        self.is_connected() && self.position_limit() != 0
    }
}

impl<'a> AeronExclusivePublication<'a> {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }

    /// sometimes when you first connect, is_connected = true, but you get backpressure as position is 0
    /// this will check if both publication is connected and position > 0
    pub fn is_ready(&self) -> bool {
        self.is_connected() && self.position_limit() != 0
    }
}

impl<'a> AeronCounter<'a> {
    pub fn close_with_no_args(&self) -> Result<(), AeronCError> {
        self.close(Handlers::no_notification_handler())?;
        Ok(())
    }
}

impl<'a> AeronCounter<'a> {
    #[inline]
    pub fn addr_atomic(&self) -> &std::sync::atomic::AtomicI64 {
        unsafe { std::sync::atomic::AtomicI64::from_ptr(self.addr()) }
    }
}

impl<'a> AeronSubscription<'a> {
    pub fn async_add_destination(
        &mut self,
        client: &'a Aeron,
        destination: &str,
    ) -> Result<AeronAsyncDestination, AeronCError> {
        AeronAsyncDestination::aeron_subscription_async_add_destination(client, self, destination)
    }

    pub fn add_destination(
        &mut self,
        client: &'a Aeron,
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

impl<'a> AeronExclusivePublication<'a> {
    pub fn async_add_destination(
        &mut self,
        client: &'a Aeron,
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
        client: &'a Aeron,
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

impl<'a> AeronPublication<'a> {
    pub fn async_add_destination(
        &mut self,
        client: &'a Aeron,
        destination: &str,
    ) -> Result<AeronAsyncDestination, AeronCError> {
        AeronAsyncDestination::aeron_publication_async_add_destination(client, self, destination)
    }

    pub fn add_destination(
        &mut self,
        client: &'a Aeron,
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

impl<'a> std::str::FromStr for AeronUriStringBuilder<'a> {
    type Err = AeronCError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let builder = AeronUriStringBuilder::default();
        builder.init_on_string(s)?;
        Ok(builder)
    }
}

impl<'a> AeronUriStringBuilder<'a> {
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

impl<'a> AeronCountersReader<'a> {
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

impl<'a> Aeron<'a> {
    pub fn new_blocking(
        context: &'a AeronContext,
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

impl<'a> AeronFragmentHandlerCallback for AeronFragmentAssembler<'a> {
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

impl<'a> AeronControlledFragmentHandlerCallback for AeronControlledFragmentAssembler<'a> {
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

impl<'a, T: AeronFragmentHandlerCallback> Handler<T> {
    pub fn leak_with_fragment_assembler(
        handler: T,
    ) -> Result<(Handler<AeronFragmentAssembler<'a>>, Handler<T>), AeronCError> {
        let handler = Handler::leak(handler);
        Ok((
            Handler::leak(AeronFragmentAssembler::new(Some(&handler))?),
            handler,
        ))
    }
}
impl<'a, T: AeronControlledFragmentHandlerCallback> Handler<T> {
    pub fn leak_with_controlled_fragment_assembler(
        handler: T,
    ) -> Result<(Handler<AeronControlledFragmentAssembler<'a>>, Handler<T>), AeronCError> {
        let handler = Handler::leak(handler);
        Ok((
            Handler::leak(AeronControlledFragmentAssembler::new(Some(&handler))?),
            handler,
        ))
    }
}

impl<'a> AeronBufferClaim<'a> {
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
