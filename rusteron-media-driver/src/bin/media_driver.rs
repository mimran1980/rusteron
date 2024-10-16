use std::ffi::CString;
use rusteron_media_driver::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rusteron_media_driver::bindings::{aeron_async_add_publication, aeron_context, aeron_threading_mode_enum};
use rusteron_media_driver::bindings::aeron_threading_mode_enum::AERON_THREADING_MODE_SHARED_NETWORK;

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
    aeron_context.set_threading_mode(aeron_threading_mode_enum::AERON_THREADING_MODE_INVOKER)?;
    // aeron_context.set_shared_idle_strategy_init_args(std::ffi::CString::new("busy_spin")?.into_raw())?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(aeron_context.get_inner())?;
    aeron_driver.start(true)?;
    // Start the Aeron driver
    println!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    aeron_driver
        .conductor()
        .context()
        .print_configuration();
    aeron_driver.main_do_work()?;

    println!("aeron dir: {:?}", aeron_context.get_dir());

    let dir = aeron_context.get_dir().to_string();
    std::thread::spawn(move || {
        let ctx = AeronContext::new()?;
        ctx.set_idle_sleep_duration_ns(0)?;
        ctx.set_dir(CString::new(dir).unwrap().into_raw())?;
        let client = Aeron::new(ctx.get_inner())?;
        client.start()?;

        assert!(client.epoch_clock() > 0);
        assert!(client.nano_clock() > 0);
        let result = AeronAsyncAddPublication::new(client.clone(), "aeron:ipc", 32)?;

        loop {
            if let Some(publication) = result.poll() {
                println!("aeron publication: {:?}", publication);

                // let publication = AeronPublication{};
                println!("publication channel: {:?}", publication.channel());
                println!("publication stream_id: {:?}", publication.stream_id());
                println!("publication status: {:?}", publication.channel_status());

                let claim = AeronBufferClaim::new().unwrap();
                assert!(publication.try_claim(100, &claim) > 0);

                break;
            }
            println!("waiting for publication to get set up");
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Ok::<(), AeronCError>(())
    });

    // Poll for work until Ctrl+C is pressed
    while running.load(Ordering::Acquire) {
        aeron_driver.main_do_work()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }


    println!("Received signal to stop the media driver.");
    println!("Aeron media driver stopped successfully.");
    Ok(())
}
