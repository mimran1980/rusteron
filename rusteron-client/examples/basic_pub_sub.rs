use log::error;
use rusteron_client::*;
use std::cell::Cell;
use std::error;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn error::Error>> {
    let ctx = AeronContext::new()?;

    // set the directory
    // ctx.set_dir(media_driver_ctx.get_dir())?;

    println!("creating client");
    let aeron = Aeron::new(&ctx)?;
    println!("starting client");

    aeron.start()?;
    println!("client started");
    let publisher = aeron
        .async_add_publication(AERON_IPC_STREAM, 123)?
        .poll_blocking(Duration::from_secs(5))?;
    println!("created publisher");

    let subscription = aeron
        .async_add_subscription(
            AERON_IPC_STREAM,
            123,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )?
        .poll_blocking(Duration::from_secs(5))?;
    println!("created subscription");

    // pick a large enough size to confirm fragment assembler is working
    let large_string_len = 1024 * 1024;
    println!("string length: {}", large_string_len);

    let _publisher_handler = {
        std::thread::spawn(move || loop {
            if publisher.offer(
                "1".repeat(large_string_len).as_bytes(),
                Handlers::no_reserved_value_supplier_handler(),
            ) < 1
            {
                error!("failed to send message");
            }
        })
    };

    let count = Cell::new(0usize);
    let closure = AeronFragmentHandlerClosure::from(|msg: Vec<u8>, header: AeronHeader| {
        println!(
            "received a message from aeron [position: {:?}, msg length:{}]",
            header.position(),
            msg.len()
        );
        count.set(count.get() + 1);
        assert_eq!(msg.as_slice(), "1".repeat(large_string_len).as_bytes())
    });
    // if you don't need fragmentation support use Handler::leak instead
    let (closure, _inner) = Handler::leak_with_fragment_assembler(closure)?;

    loop {
        if count.get() > 100 {
            break;
        }
        subscription.poll(Some(&closure), 1024)?;
    }

    println!("stopping client");

    Ok(())
}
