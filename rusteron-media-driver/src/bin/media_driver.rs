use rusteron_media_driver::*;
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Flag to indicate when the application should stop (set on Ctrl+C)
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Register signal handler for SIGINT (Ctrl+C)
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Create Aeron context
    let aeron_context = AeronDriverContext::new()?;

    let x = std::ffi::CString::new("target/test")?.into_raw();
    aeron_context.set_dir(x);
    aeron_context.aeron_driver_context_get_dir_delete_on_start();


    // Create Aeron driver
    let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
    aeron_driver.start(false);
    // Start the Aeron driver
    println!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    unsafe {
        aeron_driver
            .conductor
            .context
            .as_ref()
            .unwrap()
            .aeron_driver_context_print_configuration();
    }
    aeron_driver.main_do_work();

    // Poll for work until Ctrl+C is pressed
    while running.load(Ordering::Acquire) {
        aeron_driver.main_do_work();
    }

    println!("Received signal to stop the media driver.");
    println!("Aeron media driver stopped successfully.");
    Ok(())
}
