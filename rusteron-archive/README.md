# rusteron-archive

**rusteron-archive** is a module within the **rusteron** project that provides functionalities for interacting with Aeron's archive capabilities in a Rust environment. This module aims to extend **rusteron-client** by offering features for recording streams, managing archives, and handling replay capabilities.

## Overview

The **rusteron-archive** module is intended to help Rust developers leverage Aeron's archive functionalities, including the recording and replaying of messages. However, this module is currently in an early stage and has not been thoroughly tested.

The code in **rusteron-archive** is generated as a Rust wrapper around the Aeron C archive API, making it easier for Rust developers to work with Aeron's archiving capabilities. Since this module also uses C bindings, it involves an `unsafe` context, and extra caution is advised when using it.

## Project Status

- **Current Focus**: Our primary focus is currently on **rusteron-client**. However, developers can run a unit test in **rusteron-archive** that demonstrates recording and replaying from the archive.
- **Alpha Version**: **rusteron-archive** is in beta stage, and developers are encouraged to experiment with it, but it is not recommended for production use at this point.

## Installation

To use **rusteron-archive**, add it to your `Cargo.toml`:

dynamic lib
```toml
[dependencies]
rusteron-archive = "0.1"
```

static lib
```toml
[dependencies]
rusteron-archive = { version = "0.1", features= ["static"] }
```

Ensure that you have also set up the necessary Aeron C libraries required by **rusteron-archive**.

## Features

- **Stream Recording**: Enables recording of Aeron streams.
- **Replay Handling**: Provides capabilities for replaying recorded messages.

## Safety Considerations

**Resource Management for AeronArchive**: The current implementation has a critical limitation: the `Aeron` object must be kept alive explicitly. The `AeronArchive` does not take ownership or manage its lifetime correctly. Instead of passing the `Aeron` instance through the constructor, the `set_aeron` function is used, which can lead to potential segmentation faults if the `Aeron` instance is prematurely dropped. Extra caution is required to ensure the `Aeron` instance remains valid during the lifetime of the `AeronArchive`.
Since **rusteron-archive** relies on Aeron C bindings, it uses `unsafe` Rust code. Users must ensure that resources are managed properly to avoid crashes or undefined behaviour.

## Example Usage: Recording and Replaying a Stream with Aeron Archive

Below is an example of how to use `AeronArchive` to set up a recording, publish messages, and replay the recorded stream.

```rust ,no_run
use rusteron_archive::*;
use rusteron_archive::bindings::*;
use std::time::Duration;
use std::time::Instant;
use std::cell::Cell;
use std::thread::sleep;

let request_port = find_unused_udp_port(8000).expect("Could not find port");
let response_port = find_unused_udp_port(request_port + 1).expect("Could not find port");
let request_control_channel = &format!("aeron:udp?endpoint=localhost:{}", request_port);
let response_control_channel = &format!("aeron:udp?endpoint=localhost:{}", response_port);
let recording_events_channel = &format!("aeron:udp?endpoint=localhost:{}", response_port+1);
assert_ne!(request_control_channel, response_control_channel);

let error_handler = Handler::leak(AeronErrorHandlerClosure::from(|error_code, msg| {
panic!("error {} {}", error_code, msg)
}));

let aeron_context = AeronContext::new()?;
aeron_context.set_client_name("test")?;
aeron_context.set_publication_error_frame_handler(Some(&Handler::leak(
AeronPublicationErrorFrameHandlerLogger,
)))?;
aeron_context.set_error_handler(Some(&error_handler))?;
let aeron = Aeron::new(&aeron_context)?;
aeron.start()?;
println!("connected to aeron");

let archive_context = AeronArchiveContext::new_with_no_credentials_supplier(
    &aeron,
    request_control_channel,
    response_control_channel,
    recording_events_channel,
)?;
let found_recording_signal = Cell::new(false);
archive_context.set_recording_signal_consumer(Some(&Handler::leak(
    AeronArchiveRecordingSignalConsumerFuncClosure::from(
        |signal: AeronArchiveRecordingSignal| {
            println!("signal {:?}", signal);
            found_recording_signal.set(true);
        },
    ),
)))?;
archive_context.set_idle_strategy(Some(&Handler::leak(
    AeronIdleStrategyFuncClosure::from(|work_count| {}),
)))?;
archive_context.set_error_handler(Some(&error_handler))?;


let connect = AeronArchiveAsyncConnect::new(&archive_context.clone())?;
let archive = connect.poll_blocking(Duration::from_secs(5))?;

let channel = "aeron:ipc";
let stream_id = 10;

let subscription_id = archive.start_recording(
    channel,
    stream_id,
    aeron_archive_source_location_t::AERON_ARCHIVE_SOURCE_LOCATION_LOCAL,
    true,
)?;

println!("subscription id {}", subscription_id);

let publication = aeron
    .async_add_exclusive_publication(channel, stream_id)?
    .poll_blocking(Duration::from_secs(5))?;

let start = Instant::now();
while !found_recording_signal.get() && start.elapsed().as_secs() < 5 {
    sleep(Duration::from_millis(50));
    archive.poll_for_recording_signals()?;
    if let Some(err) = archive.poll_for_error() {
        panic!("{}", err);
    }
}
assert!(start.elapsed().as_secs() < 5);

for i in 0..11 {
    while publication.offer(
        "123456".as_bytes(),
        Handlers::no_reserved_value_supplier_handler(),
    ) <= 0
    {
        sleep(Duration::from_millis(50));
        archive.poll_for_recording_signals()?;
        if let Some(err) = archive.poll_for_error() {
            panic!("{}", err);
        }
    }
    println!("sent message");
}
archive.stop_recording_channel_and_stream(channel, stream_id)?;
drop(publication);

println!("list recordings");
let found_recording_id = Cell::new(-1);
let start_pos = Cell::new(-1);
let end_pos = Cell::new(-1);
let handler = Handler::leak(
    AeronArchiveRecordingDescriptorConsumerFuncClosure::from(
        |d: AeronArchiveRecordingDescriptor| {
            println!("found recording {:?}", d);
            found_recording_id.set(d.recording_id);
            start_pos.set(d.start_position);
            end_pos.set(d.stop_position);
        },
    ),
);
let start = Instant::now();
while start.elapsed() < Duration::from_secs(5)
    && found_recording_id.get() == -1
    && archive.list_recordings_for_uri(0, i32::MAX, channel, stream_id, Some(&handler))?
        <= 0
{
    sleep(Duration::from_millis(50));
    archive.poll_for_recording_signals()?;
    if let Some(err) = archive.poll_for_error() {
        panic!("{}", err);
    }
}
assert!(start.elapsed() < Duration::from_secs(5));
println!("start replay");
let params = AeronArchiveReplayParams::new(
    0,
    i32::MAX,
    start_pos.get(),
    end_pos.get() - start_pos.get(),
    0,
    0,
)?;
let replay_stream_id = 45;
let replay_session_id =
    archive.start_replay(found_recording_id.get(), channel, replay_stream_id, &params)?;
let session_id = replay_session_id as i32;

println!("replay session id {}", replay_session_id);
println!("session id {}", session_id);
let channel_replay = format!("{}?session-id={}", channel, session_id);
println!("archive id: {}", archive.get_archive_id());

println!("add subscription {}", channel_replay);
let subscription = aeron
    .async_add_subscription(
        &channel_replay,
        replay_stream_id,
        Some(&Handler::leak(AeronAvailableImageLogger)),
        Some(&Handler::leak(AeronUnavailableImageLogger)),
    )?
    .poll_blocking(Duration::from_secs(10))?;

let count = Cell::new(0);
let poll = Handler::leak(AeronFragmentHandlerClosure::from(|msg, header| {
    assert_eq!(msg, "123456".as_bytes().to_vec());
    count.set(count.get() + 1);
}));

let start = Instant::now();
while start.elapsed() < Duration::from_secs(5) && subscription.poll(Some(&poll), 100)? <= 0
{
    archive.poll_for_recording_signals()?;
    if let Some(err) = archive.poll_for_error() {
        panic!("{}", err);
    }
}
assert!(start.elapsed() < Duration::from_secs(5));
println!("aeron {:?}", aeron);
println!("ctx {:?}", archive_context);
assert_eq!(11, count.get());
Ok::<(), AeronCError>(())
```

### Workflow Overview
1. **Initialize Context**: Configures the archive and client contexts.
2. **Start Recording**: Begins recording a specified channel and stream.
3. **Publish Messages**: Sends messages to be recorded.
4. **Stop Recording**: Ends the recording session.
5. **Locate Recording**: Finds the recorded stream in the archive.
6. **Replay Setup**: Sets up replay parameters and initiates replay on a new stream.
7. **Subscribe and Receive**: Subscribes to the replayed messages, receiving and printing them as they arrive.

This example provides a practical usage of `AeronArchive` for recording and replaying streams.

## Building This Project

For detailed instructions on how to build **rusteron**, please refer to the [HOW_TO_BUILD.md](../HOW_TO_BUILD.md) file.

## Benchmarks

You can view the benchmarks for this project by visiting [BENCHMARKS.md](../BENCHMARKS.md).

## Contributing

Contributions are welcome! Please see our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md) for more information on how to get involved.

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-archive/)
- [API Reference on github](https://mimran1980.github.io/rusteron/rusteron_archive)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!