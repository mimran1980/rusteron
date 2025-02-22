# rusteron-client

**rusteron-client** is a core component of the **rusteron** project, providing client functionalities to interact with the Aeron messaging system in a Rust environment. It enables Rust developers to leverage Aeron's high-performance, low-latency communication protocols.

## Overview

The **rusteron-client** module acts as a Rust wrapper around the Aeron C client API. It offers functions for establishing connections, publishing messages, and subscribing to data streams, allowing seamless communication between distributed applications. Since it is built on top of Aeron's C bindings, this library operates in an `unsafe` context, requiring extra care from developers to ensure correctness.

> **Note**: Since this module leverages Aeron C bindings, it is inherently unsafe and should be used with caution. Incorrect usage can lead to undefined behaviour, such as segmentation faults.

## Features

- **Client Initialization**: Set up an Aeron client in Rust.
- **Publication**: Send messages to various Aeron channels.
- **Subscription**: Receive messages from Aeron channels.
- **Callbacks**: Handle events such as new publications, new subscriptions, and errors.
- **Automatic Resource Management (`new` method only)**: The wrappers attempt to automatically manage resources, specifically when using the `new` method. This includes calling the appropriate `xxx_init` method during initialization and automatically invoking `xxx_close` or `xxx_destroy` methods (if one exists) during cleanup. However, this management is partial. For other methods, such as `AeronArchive::set_aeron`, it is the developer's responsibility to ensure that the arguments remain valid and alive during their use. Proper resource management beyond initialization requires manual handling by the user to avoid undefined behavior or resource leaks.
- Updated methods with a single mutable out primitive to return `Result<primitive, AeronCError>`, enhancing usability and consistency by encapsulating return values and error handling.

## General Patterns

The **rusteron-client** module follows several general patterns to simplify the use of Aeron functionalities in Rust:

- **Cloneable Wrappers**: All Rust wrappers in **rusteron-client** can be cloned, and they will refer to the same underlying Aeron C instance/resource. This allows you to use multiple references to the same object safely. If you need to make a shallow copy use `clone_struct()` which copies the underlying c struct.
- **Mutable and Immutable Operations**: Modifications can be performed directly with `&self`, allowing flexibility without needing additional ownership complexities.
- **Automatic Resource Management (`new` method only)**: The wrappers attempt to automatically manage resources, clearing objects and calling the appropriate close, destroy, or remove methods when needed.
- **Manual Handler Management**: Callbacks and handlers require manual management. Handlers are passed into the C bindings using `Handlers::leak(xxx)`, and need to be explicitly released by calling `release()`. This manual process is required due to the complexity of determining when these handlers should be cleaned up once handed off to C. 
  For methods where the callback is not stored and only used there and then e.g. poll, you can pass in a closure directory e.g. 
```rust,ignore
  subscription.poll_once(|msg, header| { println!("msg={:?}, header={:?}", msg, header) })
```

## Handlers and Callbacks

Handlers in **rusteron-client** play an important role in managing events such as errors, available images, and unavailable images. There are two ways to use handlers:

### 1. Implementing a Trait

The preferred approach is to implement the appropriate trait for your handler. This approach does not require allocations and allows you to maintain a performant, safe, and reusable implementation. For example:

```rust,no_ignore
use rusteron_client::*;

pub trait AeronErrorHandlerCallback {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &str) -> ();
}

pub struct AeronErrorHandlerLogger;

impl AeronErrorHandlerCallback for AeronErrorHandlerLogger {
    fn handle_aeron_error_handler(&mut self, _errcode: ::std::os::raw::c_int, _message: &str) -> () {
        println!("{}", stringify!(handle_aeron_error_handler));
    }
}
```

In this example, the `AeronErrorHandlerCallback` trait is implemented by `AeronErrorHandlerLogger`. This trait-based approach ensures the parameters are passed directly, avoiding unnecessary allocations.

### 2. Using a Closure

Alternatively, you can use closures as handlers. However, all arguments must be copied if your planning to use them later, even ones with static lifetimes. This method is not suitable for performance-sensitive roles but is more convenient for simpler, non-critical scenarios. Example:

```rust,no_ignore
use rusteron_client::*;

pub struct AeronErrorHandlerClosure<F: FnMut(::std::os::raw::c_int, &'static str) -> ()> {
    closure: F,
}

impl<F: FnMut(::std::os::raw::c_int, &'static str) -> ()> AeronErrorHandlerCallback for AeronErrorHandlerClosure<F> {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &'static str) -> () {
        (self.closure)(errcode, message)
    }
}
```

Closures are wrapped in the `AeronErrorHandlerClosure` struct, but as noted, this involves allocations.

### Wrapping Callbacks with Handler

All callbacks need to be wrapped in a `Handler`. This helps ensure proper integration with the Aeron C API. You can use `Handlers::leak(xxx)` to pass a handler into C bindings, but remember to call `release()` when the handler is no longer needed to avoid memory leaks.

### Handler Convenience Methods

If you do not wish to set a handler or callback, you can pass `None`. Since this is a static mapping without dynamic dispatch (`dyn`), specifying the `None` type can be cumbersome. To simplify this, methods starting with `Handlers::no_xxx` are provided, allowing you to easily indicate that no handler is required without manually specifying the type. For example:

```rust,ignore
use rusteron_client::*;
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_error_handler_handler() -> Option<&'static Handler<AeronErrorHandlerLogger>> {
        None::<&Handler<AeronErrorHandlerLogger>>
    }
}
```

These methods allow for more readable and concise code when handlers are not needed.

## Error Handling with Aeron C Bindings

The Aeron C bindings use `i32` error codes to indicate the result of an operation. In the **rusteron-client**, these error codes are wrapped using `Result<i32, AeronCError>`. If the error code is negative (i.e., less than 0), it is treated as an error and represented by an `AeronCError` that contains an error type enum. The error type enum provides a detailed classification of the error.

### Error Type Enum

The `AeronErrorType` enum defines various error types that may occur:

| Error Type | Description |
|------------|-------------|
| `NullOrNotConnected` | Null value or not connected |
| `ClientErrorDriverTimeout` | Driver timeout error |
| `ClientErrorClientTimeout` | Client timeout error |
| `ClientErrorConductorServiceTimeout` | Conductor service timeout error |
| `ClientErrorBufferFull` | Buffer is full |
| `PublicationBackPressured` | Back pressure on publication |
| `PublicationAdminAction` | Admin action during publication |
| `PublicationClosed` | Publication has been closed |
| `PublicationMaxPositionExceeded` | Maximum position exceeded for publication |
| `PublicationError` | General publication error |
| `TimedOut` | Operation timed out |
| `Unknown(i32)` | Unknown error code |

These error types help provide more context on the underlying issues when working with Aeron. For example, if a publication is closed or back-pressured, these specific errors can be captured and managed accordingly.

The `AeronCError` struct encapsulates the error code and provides methods to retrieve the corresponding error type and a human-readable description. Error handling in **rusteron-client** is designed to make working with Aeron C bindings more ergonomic by providing clear error types and descriptions for easier debugging.

## Installation

Add the following to your `Cargo.toml` file to include **rusteron-client**:

dynamic lib
```toml
[dependencies]
rusteron-client = "0.1"
```

static lib
```toml
[dependencies]
rusteron-client = { version = "0.1", features= ["static"] }
```

Ensure you have also set up the necessary Aeron C libraries required by **rusteron-client**.

## Usage Example

```rust,no_ignore
use rusteron_client::*;
use rusteron_media_driver::{AeronDriverContext, AeronDriver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start embedded media driver for testing purposes
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);
    let stop3 = stop.clone();

    let ctx = AeronContext::new()?;
    ctx.set_dir(media_driver_ctx.get_dir())?;
    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;
    
    // Set up the publication
    let publisher = aeron
        .async_add_publication("aeron:ipc", 123)?
        .poll_blocking(Duration::from_secs(5))?;
    let publisher2 = publisher.clone();

    // Start publishing messages
    let message = "Hello, Aeron!".as_bytes();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Acquire) {
            if publisher.offer(message, Handlers::no_reserved_value_supplier_handler()) > 0 {
                println!("Sent message: Hello, Aeron!");
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
    
    // Set up the publication with `try_claim`
    let string_len = 156;

    std::thread::spawn(move || {
        let buffer = AeronBufferClaim::default();
        let binding = "1".repeat(string_len);
        let msg = binding.as_bytes();
        while !stop3.load(Ordering::Acquire) {
            let result = publisher2.try_claim(string_len, &buffer);

            if result < msg.len() as i64 {
                eprintln!("ERROR: failed to send message {:?}", AeronCError::from_code(result as i32));
            } else {
                buffer.data().write_all(&msg).unwrap();
                buffer.commit().unwrap();
                println!("Sent message [result={}]", result);
            }
        }
    });

    // Set up the subscription
    let subscription = aeron
        .async_add_subscription("aeron:ipc", 123,                
                                Handlers::no_available_image_handler(),
                                Handlers::no_unavailable_image_handler())?
        .poll_blocking(Duration::from_secs(5))?;

    loop {
        subscription.poll_once(|msg: &[u8], header: AeronHeader| {
          println!(
            "Received a message from Aeron [position={:?}], msg length: {}",
            header.position(),
            msg.len()
          );
        }, 128)?;
    }

    stop.store(true, Ordering::SeqCst);
    driver_handle.join().unwrap();
    Ok(())
}
```

## Safety Considerations

Since **rusteron-client** relies on Aeron C bindings, it involves `unsafe` Rust code. Users must ensure:

- Resources are properly managed (e.g., not using a publisher after the Aeron client is closed).
- Proper synchronisation when accessing shared data in a multithreaded environment.

Failing to uphold these safety measures can lead to crashes or undefined behaviour.


## Building This Project Instructions

For detailed instructions on how to build **rusteron**, please refer to the [HOW_TO_BUILD.md](../HOW_TO_BUILD.md) file.

## Benchmarks

You can view the benchmarks for this project by visiting [BENCHMARKS.md](../BENCHMARKS.md).

## Contributing

Contributions are welcome! Please feel free to open issues, submit pull requests, or suggest new features. We're particularly interested in:

- Feedback on API usability.
- Bug reports and feature requests.
- Documentation improvements.

If you wish to contribute, refer to our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md).

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-client/)
- [API Reference on github](https://mimran1980.github.io/rusteron/rusteron_client)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!

