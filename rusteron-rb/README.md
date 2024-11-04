# rusteron-rb

**rusteron-rb** is a core component of the **rusteron** project, providing ring buffer and broadcast functionalities to interact with the Aeron messaging system in a Rust environment. It enables Rust developers to leverage Aeron's high-performance, low-latency communication protocols.

## Overview

The **rusteron-rb** module acts as a Rust wrapper around the Aeron C ring buffer API. It offers functions for establishing connections, transmitting and receiving messages, and broadcasting data streams, allowing seamless communication between distributed applications. Since it is built on top of Aeron's C bindings, this library operates in an `unsafe` context, requiring extra care from developers to ensure correctness.

> **Note**: Since this module leverages Aeron C bindings, it is inherently unsafe and should be used with caution. Incorrect usage can lead to undefined behavior, such as segmentation faults.

## Features

- **Broadcast Receiver**: Receive messages from Aeron broadcast channels using `AeronBroadcastReceiver`.
- **Broadcast Transmitter**: Send messages to Aeron broadcast channels using `AeronBroadcastTransmitter`.
- **MPSC Ring Buffer**: Multi-producer, single-consumer ring buffer implementation using `AeronMpscRb`.
- **SPSC Ring Buffer**: Single-producer, single-consumer ring buffer implementation using `AeronSpscRb`.

# Examples

Sure thing! Here are three sections with detailed code examples: one for Single Producer, Single Consumer (SPSC) Ring Buffer, one for Multi-Producer, Single Consumer (MPSC) Ring Buffer, and one for the Broadcast Transmitter and Receiver. These examples are based on your unit tests and will help illustrate the usage for different parts of **rusteron-rb**.

### Single Producer, Single Consumer Ring Buffer

This example demonstrates how to use the `AeronSpscRb` to create a simple single producer, single consumer ring buffer.

```rust
use rusteron_rb::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let rb = AeronSpscRb::new_with_capacity(1024 * 1024, 1024)?;

    // Producer writes data to the ring buffer
    for i in 0..100 {
        let idx = rb.try_claim(i + 1, 4);
        assert!(idx >= 0);
        let slot = rb.buffer_at_mut(idx as usize, 4);
        slot[0] = i as u8;
        rb.commit(idx)?;
    }

    // Consumer reads data from the ring buffer
    struct Reader;
    impl AeronRingBufferHandlerCallback for Reader {
        fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) {
            println!("msg_type_id: {}, buffer: {:?}", msg_type_id, buffer);
            assert_eq!(buffer[0], (msg_type_id - 1) as u8);
        }
    }

    let handler = AeronRingBufferHandlerWrapper::new(Reader);
    for _ in 0..10 {
        let read = rb.read_msgs(&handler, 10);
        assert_eq!(10, read);
    }

    Ok(())
}
```

### Multi-Producer, Single Consumer Ring Buffer

The following example demonstrates how to use the `AeronMpscRb` for a multi-producer, single consumer scenario, enabling multiple producers to write to the same ring buffer while a single consumer reads from it.

```rust
use rusteron_rb::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let rb = AeronMpscRb::new_with_capacity(1024 * 1024, 1024)?;

    // Producers write data to the ring buffer
    for i in 0..100 {
        let idx = rb.try_claim(i + 1, 4);
        assert!(idx >= 0);
        let slot = rb.buffer_at_mut(idx as usize, 4);
        slot[0] = i as u8;
        rb.commit(idx)?;
    }

    // Consumer reads data from the ring buffer
    struct Reader;
    impl AeronRingBufferHandlerCallback for Reader {
        fn handle_aeron_rb_handler(&mut self, msg_type_id: i32, buffer: &[u8]) {
            println!("msg_type_id: {}, buffer: {:?}", msg_type_id, buffer);
            assert_eq!(buffer[0], (msg_type_id - 1) as u8);
        }
    }

    let handler = AeronRingBufferHandlerWrapper::new(Reader);
    for _ in 0..10 {
        let read = rb.read_msgs(&handler, 10);
        assert_eq!(10, read);
    }

    Ok(())
}
```

### Broadcast Transmitter and Receiver

This example demonstrates how to set up a broadcast transmitter and receiver. The transmitter sends messages that are then received by the receiver, illustrating a simple broadcast communication scenario.

```rust
use rusteron_rb::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up broadcast transmitter and receiver
    let mut vec = vec![0u8; 1024 * 1024 + AERON_BROADCAST_BUFFER_TRAILER_LENGTH];
    let transmitter = AeronBroadcastTransmitter::from_slice(vec.as_mut_slice(), 1024)?;
    let receiver = AeronBroadcastReceiver::from_slice(vec.as_mut_slice())?;

    // Transmit messages
    for i in 0..100 {
        let mut msg = [0u8; 4];
        msg[0] = i as u8;
        let idx = transmitter.transmit_msg(i + 1, &msg).unwrap();
        println!("sent {}", idx);
        assert!(idx >= 0);
    }

    // Receive messages
    struct Reader;
    impl AeronBroadcastReceiverHandlerCallback for Reader {
        fn handle_aeron_broadcast_receiver_handler(&mut self, msg_type_id: i32, buffer: &mut [u8]) {
            println!("msg_type_id: {}, buffer: {:?}", msg_type_id, buffer);
            assert_eq!(buffer[0], (msg_type_id - 1) as u8);
        }
    }

    let handler = AeronBroadcastReceiverHandlerWrapper::new(Reader);
    for _ in 0..100 {
        let read = receiver.receive(Some(&handler)).unwrap();
        println!("read {}", read);
        assert!(read > 0);
    }

    Ok(())
}
```

## Installation

Add the following to your `Cargo.toml` file to include **rusteron-rb**:

dynamic lib
```toml
[dependencies]
rusteron-rb = "0.1"
```

static lib
```toml
[dependencies]
rusteron-rb = { version = "0.1", features = ["static"] }
```

Ensure you have also set up the necessary Aeron C libraries required by **rusteron-rb**.

## Usage Example

Here is a basic example demonstrating the initialization and usage of `AeronBroadcastReceiver` and `AeronBroadcastTransmitter`:

```rust
use rusteron_rb::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    Ok(())
}
```

## Safety Considerations

Since **rusteron-rb** relies on Aeron C bindings, it involves `unsafe` Rust code. Users must ensure:

- Resources are properly managed.
- Proper synchronization when accessing shared data in a multi-threaded environment.

Failing to uphold these safety measures can lead to crashes or undefined behavior.

## Contributing

Contributions are welcome! Please feel free to open issues, submit pull requests, or suggest new features. We're particularly interested in:

- Feedback on API usability.
- Bug reports and feature requests.
- Documentation improvements.

If you wish to contribute, refer to our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md).

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-rb/)
- [API Reference on GitHub](https://mimran1980.github.io/rusteron/rusteron_rb)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!