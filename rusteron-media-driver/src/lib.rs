#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use bindings::*;
include!(concat!(env!("OUT_DIR"), "/aeron.rs"));
include!("../../rusteron-client/src/aeron.rs");

unsafe impl Send for AeronDriverContext {}

impl AeronDriver {
    pub fn launch_embedded(aeron_context: &AeronDriverContext) -> Arc<AtomicBool> {
        let mut stop = Arc::new(AtomicBool::new(false));
        let mut stop_copy = stop.clone();
        let mut stop_copy2 = stop.clone();
        let aeron_context = aeron_context.clone();
        // Register signal handler for SIGINT (Ctrl+C)
        ctrlc::set_handler(move || {
            stop_copy2.store(true, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        std::thread::spawn(move || {
            let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
            aeron_driver.start(true)?;

            // Poll for work until Ctrl+C is pressed
            while !stop.load(Ordering::Acquire) {
                while aeron_driver.main_do_work()? > 0 {
                    // busy spin
                }
            }

            Ok::<_, AeronCError>(())
        });
        stop_copy
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread::sleep;
    use std::time::Duration;

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
        aeron_context.set_dir_delete_on_shutdown(true)?;
        aeron_context.set_dir_delete_on_start(true)?;

        let stop = AeronDriver::launch_embedded(&aeron_context);

        // aeron_driver
        //     .conductor()
        //     .context()
        //     .print_configuration();
        // aeron_driver.main_do_work()?;
        println!("aeron dir: {:?}", aeron_context.get_dir());

        let dir = aeron_context.get_dir().to_string();
        let ctx = AeronContext::new()?;
        ctx.set_dir(CString::new(dir).unwrap().into_raw())?;

        let client = Aeron::new(ctx.get_inner())?;

        unsafe {
            struct A {
                client: Aeron
            }
            impl AeronAvailableCounterHandler for A {
                fn handle(&mut self, counters_reader: AeronCountersReader, registration_id: i64, counter_id: i32) {
                    println!("Aeron available counters: {registration_id} {counter_id}");
                    // unsafe {
                    //     if counters_reader.metadata_length > 100 {
                    //         println!("aeron available counters reader {}", counters_reader.metadata_length);
                    //         println!("aeron available value reader {}", counters_reader.values_length);
                    //         return;
                    //     }
                    //     // let slice = std::slice::from_raw_parts(counters_reader.metadata, counters_reader.metadata_length as usize);
                    //     // println!("Aeron available counters: {:?}, registration_id: {registration_id}, counter_id: {counter_id}",
                    //     //          std::str::from_utf8_unchecked(slice).trim());
                    // }
                }
            }

            // // Now use the trait object
            let b= Box::new(Box::new(A { client: client.clone() }));
            println!("before into raw {:p}", std::ptr::from_ref(&*b));
            let boxed_handler = Box::into_raw(b) as *mut _;
            println!("after into raw {:p}", boxed_handler);
            println!("Setting Aeron callback...");
            // AeronAvailableCounterHandler aeron_on_available_counter_t
            ctx.set_on_available_counter(
                Some(aeron_on_available_counter_t_callback::<A>),
                boxed_handler as *mut ::std::os::raw::c_void,
            );
            // panic!("result {}", result);
        }

        client.start()?;
        println!("aeron driver started");
        assert!(client.epoch_clock() > 0);
        assert!(client.nano_clock() > 0);

        let counter_async = AeronAsyncAddCounter::new(client.clone(), 2543543, "12312312".as_ptr(), "12312312".len(),
        "abcd", 4)?;

        let counter = counter_async.poll_blocking(Duration::from_secs(15))?;
        unsafe { *counter.addr() += 1; }

        let result = AeronAsyncAddPublication::new(client.clone(), topic, stream_id)?;

        let publication = result.poll_blocking(std::time::Duration::from_secs(15))?;

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

        // client.main_do_work();
        // let claim = AeronBufferClaim::default();
        // assert!(publication.try_claim(100, &claim) > 0, "publication claim is empty");

        stop.store(true, Ordering::SeqCst);

        Ok(())
    }
}


// generated code
// pub trait TestER: FnMut(&str) {}
// pub trait AeronAsyncAddSubscriptionHandler: FnMut(AeronSubscription, AeronImage) {}
//
// fn aeron_async_add_subscription_with_closure<F>(
//     client: *mut aeron_t,
//     uri: *const ::std::os::raw::c_char,
//     stream_id: i32,
//     on_available_image_closure: F, // Generic closure
// ) -> Result<(), std::os::raw::c_int>
// where
//     F: FnMut(AeronSubscription, AeronImage),
// {
//     let mut async_ptr: *mut aeron_async_add_subscription_t = std::ptr::null_mut();
//
//     // Box the closure and turn it into a raw pointer
//     let boxed_closure: *mut F = Box::into_raw(Box::new(on_available_image_closure));
//
//     let result = unsafe {
//         aeron_async_add_subscription(
//             &mut async_ptr,
//             client,
//             uri,
//             stream_id,
//             Some(aeron_on_available_image_t_callback::<F>),  // Pass the callback function
//             boxed_closure as *mut ::std::os::raw::c_void, // Pass the boxed closure as the clientd
//             None, // on_unavailable_image_handler
//             std::ptr::null_mut(), // on_unavailable_image_clientd
//         )
//     };
//
//     if result == 0 {
//         Ok(())
//     } else {
//         // If there's an error, clean up the boxed closure
//         unsafe { Box::from_raw(boxed_closure); } // Clean up the box to avoid a leak
//         Err(result)
//     }
// }
// // genera
type OnAvailableImageClosure = Box<dyn FnMut(AeronSubscription, AeronImage)>;

unsafe extern "C" fn on_available_image_callback(
    clientd: *mut ::std::os::raw::c_void,
    subscription: *mut aeron_subscription_t,
    image: *mut aeron_image_t,
) {
    if !clientd.is_null() {
        // Convert the raw pointer back to the closure and invoke it.
        let closure: &mut OnAvailableImageClosure = &mut *(clientd as *mut OnAvailableImageClosure);
        closure(subscription.into(), image.into());
    }
}

fn aeron_async_add_subscription_with_closure(
    client: *mut aeron_t,
    uri: *const ::std::os::raw::c_char,
    stream_id: i32,
    on_available_image_closure: OnAvailableImageClosure,
) -> Result<(), std::os::raw::c_int> {
    let mut async_ptr: *mut aeron_async_add_subscription_t = std::ptr::null_mut();

    // Box the closure and turn it into a raw pointer
    let boxed_closure: *mut OnAvailableImageClosure = Box::into_raw(Box::new(on_available_image_closure));

    let result = unsafe {
        aeron_async_add_subscription(
            &mut async_ptr,
            client,
            uri,
            stream_id,
            Some(on_available_image_callback),  // Pass the callback function
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

fn cleanup_subscription(clientd: *mut ::std::os::raw::c_void) {
    cleanup_closure::<OnAvailableImageClosure>(clientd);
}