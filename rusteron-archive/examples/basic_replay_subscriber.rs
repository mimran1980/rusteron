use rusteron_archive::testing::EmbeddedArchiveMediaDriverProcess;
use rusteron_archive::*;
use std::error::Error;
use std::time::Duration;

// Replay and live destination channels
const REPLAY_DESTINATION: &str = "aeron:udp?endpoint=localhost:40124";
const LIVE_DESTINATION: &str = "aeron:udp?endpoint=localhost:40125";
const CONTROL_CHANNEL: &str = "aeron:udp?control=localhost:40123"; // Control channel for multi-destination
const STREAM_ID: i32 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Aeron Replay Merge Example...");

    // Step 1: Set up Aeron Archive and Multi-Destination Subscription
    let (mut subscription, archive) = setup_subscription_and_archive()?;

    // Step 2: Add replay and live destinations to the subscription
    add_destinations_to_subscription(
        &archive,
        &mut subscription,
        REPLAY_DESTINATION,
        LIVE_DESTINATION,
    )?;

    // Step 3: Start a recording and send some test data
    let recording_id = start_recording_and_send_test_data(&archive)?;

    // Step 4: Start the replay merge
    let replay_merge = AeronArchiveReplayMerge::new(
        &subscription,
        &archive,
        REPLAY_DESTINATION,
        REPLAY_DESTINATION,
        LIVE_DESTINATION,
        recording_id,
        0, // Start position
        epoch_clock(),
        5000, // Merge progress timeout in ms
    )?;

    println!("Replay merge started.");

    // Step 5: Process messages
    process_messages(&replay_merge, &subscription)?;

    println!("Replay merge completed successfully.");
    Ok(())
}

// Helper function to set up Aeron Archive and Subscription
fn setup_subscription_and_archive() -> Result<(AeronSubscription, AeronArchive), Box<dyn Error>> {
    // Step 1: Setup Aeron directory and Archive context
    let aeron_dir = format!("target/aeron/{}/shm", Aeron::nano_clock());
    let request_port = find_unused_udp_port(8000).expect("Could not find a udp port");
    let response_port = find_unused_udp_port(request_port + 1).expect("Could not find a udp port");
    let request_channel = format!("aeron:udp?endpoint=localhost:{}", request_port);
    let response_channel = format!("aeron:udp?endpoint=localhost:{}", response_port);

    println!(
        "Setting up archive with request: {} and response: {}",
        request_channel, response_channel
    );

    // Step 2: Start embedded Aeron Archive Media Driver
    let _embedded_driver = EmbeddedArchiveMediaDriverProcess::build_and_start(
        &aeron_dir,
        &format!("{}/archive", aeron_dir),
        &request_channel,
        &response_channel,
    )
    .expect("Failed to start embedded media driver");

    // Step 3: Create Aeron context
    let mut aeron_context = AeronContext::new()?;
    aeron_context.set_dir(&aeron_dir)?;

    // Step 4: Connect to Aeron
    let aeron = Aeron::new(&aeron_context)?;
    aeron.start()?;

    // Step 5: Configure Aeron Archive context
    let mut archive_context = AeronArchiveContext::new_with_no_credentials_supplier(
        &aeron,
        &request_channel,
        &response_channel,
    )?;

    // Step 6: Connect to Aeron Archive
    let connect = AeronArchiveAsyncConnect::new(&archive_context.clone())?;
    let archive = connect.poll_blocking(Duration::from_secs(5))?;

    // Step 7: Create multi-destination subscription
    let available_handler = Handler::leak(AeronAvailableImageLogger);
    let unavailable_handler = Handler::leak(AeronUnavailableImageLogger);
    let subscription = aeron.add_subscription(
        CONTROL_CHANNEL,
        STREAM_ID,
        Some(&available_handler),
        Some(&unavailable_handler),
        Duration::from_secs(5),
    )?;

    if !subscription.is_connected() {
        return Err("Subscription not connected".into());
    }

    Ok((subscription, archive))
}

// Helper function to add destinations to the subscription
fn add_destinations_to_subscription(
    archive: &AeronArchive,
    subscription: &mut AeronSubscription,
    replay_destination: &str,
    live_destination: &str,
) -> Result<(), Box<dyn Error>> {
    subscription.add_destination(&archive.aeron(), replay_destination, Duration::from_secs(5))?;
    println!("Replay destination added: {}", replay_destination);

    subscription.add_destination(&archive.aeron(), live_destination, Duration::from_secs(5))?;
    println!("Live destination added: {}", live_destination);

    Ok(())
}

// Helper function to start recording and send test data
fn start_recording_and_send_test_data(archive: &AeronArchive) -> Result<i64, Box<dyn Error>> {
    let channel = "aeron:udp?endpoint=localhost:40123";
    let stream_id = STREAM_ID;

    // Start recording
    let recording_id = archive.start_recording(
        channel,
        stream_id,
        SourceLocation::AERON_ARCHIVE_SOURCE_LOCATION_LOCAL,
        true,
    )?;
    println!("Recording started with ID: {}", recording_id);

    // Send test data
    let publication =
        archive
            .aeron()
            .add_publication(channel, stream_id, Duration::from_secs(5))?;
    for i in 0..10 {
        let string = format!("Message {}", i);
        let message = string.as_bytes();
        while publication.offer(message, Handlers::no_reserved_value_supplier_handler()) <= 0 {
            std::thread::sleep(Duration::from_millis(10));
        }
        println!("Sent: Message {}", i);
    }

    Ok(recording_id)
}

// Helper function to process messages from the subscription
fn process_messages(
    replay_merge: &AeronArchiveReplayMerge,
    subscription: &AeronSubscription,
) -> Result<(), Box<dyn Error>> {
    let mut running = true;

    let handler = Handler::leak(AeronFragmentHandlerClosure::from(
        |data: Vec<u8>, _header| {
            println!(
                "Received message: {}",
                std::str::from_utf8(data.as_slice()).unwrap()
            );
        },
    ));
    while running && !replay_merge.is_merged() {
        replay_merge.do_work(&mut 0)?;

        subscription.poll(Some(&handler), 10)?;

        if replay_merge.has_failed() {
            eprintln!("Replay merge failed!");
            running = false;
        }
    }

    if !running {
        Err("Replay merge terminated unexpectedly.".into())
    } else {
        Ok(())
    }
}

// Helper function to get the current epoch time in milliseconds
fn epoch_clock() -> i64 {
    Aeron::nano_clock()
}
