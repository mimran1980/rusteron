use std::time::{Duration, Instant};

impl AeronPublication {
    pub fn new(client: AeronAsyncAddPublication) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx| unsafe {
                aeron_async_add_publication_poll(ctx, client.get_inner())
            },
            move |_ctx| {
                // TODO is there any cleanup to do
                0
            },
            false
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
}

impl AeronAsyncAddPublication {
    pub fn new(client: Aeron, uri: &str, stream_id: i32) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx| unsafe {
                aeron_async_add_publication(ctx, client.get_inner(),
                                            std::ffi::CString::new(uri).unwrap().into_raw(), stream_id)
            },
            move |_ctx| {
                // TODO is there any cleanup to do
                0
            },
            false
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }

    pub fn poll(&self) -> Option<AeronPublication> {
        if let Ok(publication) = AeronPublication::new(self.clone()) {
            Some(publication)
        } else {
            None
        }
    }

    pub fn poll_blocking(&self, timeout: Duration) -> Result<AeronPublication, AeronCError> {
        if let Some(publication) = self.poll() {
            return Ok(publication);
        }

        let time = Instant::now();
        while time.elapsed() < timeout {
            if let Some(publication) = self.poll() {
                return Ok(publication);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(AeronCError::from_code(-255))
    }
}