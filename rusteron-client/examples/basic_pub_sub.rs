use rusteron_client::*;
use std::error;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn error::Error>> {
    let ctx = AeronContext::new()?;

    // set the directory
    // ctx.set_dir(media_driver_ctx.get_dir())?;

    let mut error_count = 1;
    let error_handler = AeronErrorHandlerClosure::from(|error_code, msg| {
        eprintln!("aeron error {}: {}", error_code, msg);
        error_count += 1;
    });
    ctx.set_error_handler(Some(&Handler::leak(error_handler)))?;
    ctx.set_on_new_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;
    ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
    ctx.set_on_close_client(Some(&Handler::leak(AeronCloseClientLogger)))?;
    ctx.set_on_new_subscription(Some(&Handler::leak(AeronNewSubscriptionLogger)))?;
    ctx.set_on_unavailable_counter(Some(&Handler::leak(AeronUnavailableCounterLogger)))?;
    ctx.set_on_available_counter(Some(&Handler::leak(AeronAvailableCounterLogger)))?;
    ctx.set_on_new_exclusive_publication(Some(&Handler::leak(AeronNewPublicationLogger)))?;

    println!("creating client");
    let aeron = Aeron::new(ctx)?;
    println!("starting client");

    aeron.start()?;
    println!("client started");
    let publisher = aeron
        .async_add_publication("aeron:ipc", 123)?
        .poll_blocking(Duration::from_secs(5))?;
    println!("created publisher");

    let subscription = aeron
        .async_add_subscription(
            "aeron:ipc",
            123,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )?
        .poll_blocking(Duration::from_secs(5))
        .unwrap();
    println!("created subscription");

    // pick a large enough size to confirm fragment assembler is working
    let string_len = 1024 * 1024;
    println!("string length: {}", string_len);

    let publisher_handler = {
        std::thread::spawn(move || loop {
            println!("sending message");
            if publisher.offer(
                "1".repeat(string_len).as_bytes(),
                Handlers::no_reserved_value_supplier_handler(),
            ) < 1
            {
                eprintln!("failed to send message");
            }
        })
    };

    let mut count = 0;
    let closure = AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
        println!(
            "received a message from aeron [position: {:?}, msg length:{}]",
            header.position(),
            msg.len()
        );
        count += 1;
        assert_eq!(msg.as_slice(), "1".repeat(string_len).as_bytes())
    });
    let closure = Handler::leak(closure);

    for _ in 0..100 {
        if count > 100 {
            break;
        }
        subscription.poll(Some(&closure), 1024)?;
    }

    println!("stopping client");

    Ok(())
}
