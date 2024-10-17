#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("../../rusteron-client/src/aeron.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn version_check() {
        let major = unsafe { crate::aeron_version_major() };
        let minor = unsafe { crate::aeron_version_minor() };
        let patch = unsafe { crate::aeron_version_patch() };

        let aeron_version = format!("{}.{}.{}", major, minor, patch);
        let cargo_version = "1.47.0"; // env!("CARGO_PKG_VERSION");
        assert_eq!(aeron_version, cargo_version);
    }

    #[test]
    fn send_message() -> Result<(), AeronCError> {
        let topic = "aeron:ipc";
        let stream_id = 32;

        let aeron_context = AeronDriverContext::new()?;
        let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
        aeron_driver.start(false)?;

        aeron_driver
            .conductor()
            .context()
            .print_configuration();
        aeron_driver.main_do_work()?;

        println!("aeron dir: {:?}", aeron_context.get_dir());

        let dir = aeron_context.get_dir().to_string();
        let ctx = AeronContext::new()?;
        ctx.set_idle_sleep_duration_ns(0)?;
        ctx.set_dir(CString::new(dir).unwrap().into_raw())?;
        let client = Aeron::new(ctx.get_inner())?;
        client.start()?;

        assert!(client.epoch_clock() > 0);
        assert!(client.nano_clock() > 0);
        let result = AeronAsyncAddPublication::new(client.clone(), topic, stream_id)?;

        let publication = result.poll_blocking(Duration::from_secs(10))?;

        let sub: AeronAsyncAddSubscription = AeronAsyncAddSubscription::new_zeroed()?;

        aeron_async_add_subscription_with_closure(
            client.get_inner(),
            CString::new(topic).unwrap().as_c_str().as_ptr(),
            stream_id,
            Box::new(move |subscription, image| {
                println!("subscription: {:?}", subscription);
                println!("image: {:?}", image);
            })
        ).unwrap();

        println!("publication channel: {:?}", publication.channel());
        println!("publication stream_id: {:?}", publication.stream_id());
        println!("publication status: {:?}", publication.channel_status());

        let claim = AeronBufferClaim::default();
        assert!(publication.try_claim(100, &claim) > 0);

        Ok(())
    }
}


// generated code


fn aeron_async_add_subscription_with_closure<F>(
    client: *mut aeron_t,
    uri: *const ::std::os::raw::c_char,
    stream_id: i32,
    on_available_image_closure: F, // Generic closure
) -> Result<(), std::os::raw::c_int>
where
    F: FnMut(AeronSubscription, AeronImage),
{
    let mut async_ptr: *mut aeron_async_add_subscription_t = std::ptr::null_mut();

    // Box the closure and turn it into a raw pointer
    let boxed_closure: *mut F = Box::into_raw(Box::new(on_available_image_closure));

    let result = unsafe {
        aeron_async_add_subscription(
            &mut async_ptr,
            client,
            uri,
            stream_id,
            Some(aeron_on_available_image_t_callback::<F>),  // Pass the callback function
            boxed_closure as *mut ::std::os::raw::c_void, // Pass the boxed closure as the clientd
            None, // on_unavailable_image_handler
            std::ptr::null_mut(), // on_unavailable_image_clientd
        )
    };

    if result == 0 {
        Ok(())
    } else {
        // If there's an error, clean up the boxed closure
        unsafe { Box::from_raw(boxed_closure); } // Clean up the box to avoid a leak
        Err(result)
    }
}
// genera


