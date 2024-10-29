use rusteron_media_driver::bindings::aeron_threading_mode_enum;
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

    aeron_context.set_dir("target/test")?;
    aeron_context.set_threading_mode(aeron_threading_mode_enum::AERON_THREADING_MODE_INVOKER)?;
    // aeron_context.set_shared_idle_strategy_init_args(std::ffi::CString::new("busy_spin")?.into_raw())?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(&aeron_context)?;
    aeron_driver.start(true)?;
    // Start the Aeron driver
    println!("Aeron media driver started successfully. Press Ctrl+C to stop.");

    aeron_driver.conductor().context().print_configuration();
    aeron_driver.main_do_work()?;

    println!("aeron dir: {:?}", aeron_context.get_dir());

    let dir = aeron_context.get_dir().to_string();
    std::thread::spawn(move || {
        let ctx = AeronContext::new()?;
        ctx.set_idle_sleep_duration_ns(0)?;
        ctx.set_dir(&dir)?;
        let client = Aeron::new(&ctx)?;
        client.start()?;

        assert!(Aeron::epoch_clock() > 0);
        assert!(Aeron::nano_clock() > 0);
        let result = AeronAsyncAddPublication::new(&client, "aeron:ipc", 32)?;

        loop {
            if let Some(publication) = result.poll() {
                println!("aeron publication: {:?}", publication);

                // let publication = AeronPublication{};
                println!("publication channel: {:?}", publication.channel());
                println!("publication stream_id: {:?}", publication.stream_id());
                println!("publication status: {:?}", publication.channel_status());

                let claim = AeronBufferClaim::default();
                assert!(publication.try_claim(100, &claim) > 0);

                println!("send message");
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
