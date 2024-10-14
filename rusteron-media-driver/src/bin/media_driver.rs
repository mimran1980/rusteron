use rusteron_media_driver::*;
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
    aeron_context.set_dir(x)?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
    aeron_driver.start(false)?;
    // Start the Aeron driver
    println!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    aeron_driver
        .conductor()
        .context()
        .print_configuration();
    aeron_driver.main_do_work()?;

    println!("aeron dir: {:?}", aeron_context.get_dir());

    // Poll for work until Ctrl+C is pressed
    while running.load(Ordering::Acquire) {
        aeron_driver.main_do_work()?;
    }

    println!("Received signal to stop the media driver.");
    println!("Aeron media driver stopped successfully.");
    Ok(())
}
