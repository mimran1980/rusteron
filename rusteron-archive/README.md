Below is the README with all references to Closure::from removed and replaced with direct struct instantiations. The “2. Using a Closure” section under Handlers and Callbacks is also removed as requested. All code snippets are in Markdown code fences.

# rusteron-archive

**rusteron-archive** is a module within the **rusteron** project that provides functionalities for interacting with Aeron's archive capabilities in a Rust environment. This module extends **rusteron-client** by offering features for recording streams, managing archives, and handling replay capabilities.

## Overview

The **rusteron-archive** module is intended to help Rust developers leverage Aeron's archive functionalities, including the recording and replaying of messages.

## Installation

Add **rusteron-archive** to your `Cargo.toml`:

*dynamic lib*:
```toml
[dependencies]
rusteron-archive = "0.1"

static lib
```toml
[dependencies]
rusteron-archive = { version = "0.1", features= ["static"] }
```

You must also ensure that you include Aeron C libraries required by **rusteron-archive** when using default feature. (Using static will automatically include these dependancies in binary).

## Features

- **Stream Recording**: Enables recording of Aeron streams.
- **Replay Handling**: Provides capabilities for replaying recorded messages.
- **Publication**: Send messages to various Aeron channels.
- **Subscription**: Receive messages from Aeron channels.
- **Callbacks**: Handle events such as new publications, new subscriptions, and errors.
- **Automatic Resource Management (`new` method only)**: The wrappers attempt to automatically manage resources, specifically when using the `new` method. This includes calling the appropriate `xxx_init` method during initialisation and automatically invoking `xxx_close` or `xxx_destroy` methods (if one exists) during cleanup. However, this management is partial. For other methods, such as `AeronArchive::set_aeron`, it is the developer's responsibility to ensure that the arguments remain valid and alive during their use. Proper resource management beyond initialisation requires manual handling by the user to avoid undefined behaviour or resource leaks.

## General Patterns

Much like **rusteron-client**, the **rusteron-archive** module follows several general patterns to simplify usage of Aeron functionalities in Rust:

- **Cloneable Wrappers**: All Rust wrappers in **rusteron-archive** can be cloned, and they will refer to the same underlying Aeron C instance/resource. This allows safe use of multiple references to the same object. If you need a shallow copy, use `clone_struct()`, which copies only the underlying C struct.

- **Mutable and Immutable Operations**: Modifications can be performed directly with `&self`, allowing flexibility without needing additional ownership complexities.
- **Automatic Resource Management (`new` method only)**: The wrappers attempt to automatically manage resources, clearing objects and calling the appropriate close, destroy, or remove methods when needed.
- **Manual Handler Management**: Callbacks and handlers require manual management. Handlers are passed into the C bindings using `Handlers::leak(xxx)`, and need to be explicitly released by calling `release()`. This manual process is required due to the complexity of determining when these handlers should be cleaned up once handed off to C.
  For methods where the callback is not stored and only used there and then e.g. poll, you can pass in a closure directory e.g.
```rust ,ignore
  subscription.poll_once(|msg, header| { println!("msg={:?}, header={:?}", msg, header) })
```

## Handlers and Callbacks

Handlers within **rusteron-archive** work just like those in **rusteron-client**. You can attach and manage them using two main approaches:

Defining a trait for your handler and implementing it within your own struct is the recommended, most performant approach. For instance:

The recommended approach is to define a trait for your handler and implement it within your own struct. This pattern is performant and safe as it does not require additional allocations. For instance:

```rust ,no_run
use rusteron_archive::*;

pub trait AeronErrorHandlerCallback {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &str) -> ();
}

pub struct AeronErrorHandlerLogger;

impl AeronErrorHandlerCallback for AeronErrorHandlerLogger {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &str) -> () {
        eprintln!("Error {}: {}", errcode, message);
    }
}

By passing instances of this trait to the archive context, you gain a reusable and safe way to respond to errors or other events without incurring unnecessary runtime overhead.

Wrapping Callbacks with Handler

Callbacks must be wrapped in a Handler. This ensures proper integration with the Aeron C API. Use Handlers::leak(xxx) to pass a handler into the C bindings. When your handler is no longer needed, call release() to free resources and avoid memory leaks.

### Wrapping Callbacks with Handler

Regardless of the approach, callbacks must be wrapped in a `Handler`. This ensures proper integration with the Aeron C API. Use `Handlers::leak(xxx)` to pass a handler into C bindings. When your handler is no longer needed, call `release()` to free up resources and avoid memory leaks.

### Handler Convenience Methods

If you do not need to set a particular handler, you can pass `None`. However, doing so manually can be awkward due to static type requirements. To simplify this, **rusteron-archive** (like **rusteron-client**) provides convenience methods prefixed with `Handlers::no_...`, returning `None` with the correct type signature. For example:

```rust ,ignore
use rusteron_archive::*;
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_error_handler_handler() -> Option<&'static Handler<AeronErrorHandlerLogger>> {
        None::<&Handler<AeronErrorHandlerLogger>>
    }
}
```

These methods make it easy to specify that no handler is required, keeping your code concise.

## Error Handling with Aeron C Bindings

**rusteron-archive** relies on the same Aeron C bindings as **rusteron-client**, using `i32` error codes to indicate the outcome of operations. In Rust, these are wrapped within a `Result<i32, AeronCError>` to provide clearer, more idiomatic error handling.

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

## Safety Considerations

**Resource Management**:  
1. **Lifetime of `Aeron`**: The `AeronArchive` does not take full ownership or manage the lifetime of the `Aeron` instance. Instead, it calls `AeronArchive::set_aeron`, meaning you must ensure the `Aeron` object remains valid throughout the archive's usage. Dropping or losing reference to the `Aeron` too soon can lead to segmentation faults or undefined behaviour.

2. **Unsafe Bindings**: Since **rusteron-archive** relies on Aeron C bindings, you must carefully manage resources (publishers, subscriptions, handlers, etc.) to avoid crashes or undefined behaviour. This includes ensuring you do not publish messages after closing the Aeron client or the associated archive context.

3. **Partial Automatic Resource Management**: While constructors aim to manage resources automatically, many aspects of resource lifecycles remain manual. For instance, handlers require a call to `release()` to clean up memory. Be especially cautious in multithreaded environments, ensuring synchronisation is properly handled.

Failure to follow these guidelines can lead to unstable or unpredictable results.

### Workflow Overview
1. **Initialise Contexts**: Set up archive and client contexts.
2. **Start Recording**: Begin recording a specified channel and stream.
3. **Publish Messages**: Send messages to be captured by the archive.
4. **Stop Recording**: Conclude the recording session.
5. **Locate Recording**: Identify and retrieve details about the recorded stream.
6. **Replay Setup**: Configure replay parameters and replay the recorded messages on a new stream.
7. **Subscribe and Receive**: Subscribe to the replay stream, receiving the replayed messages as they appear.

## Building This Project98

For full details on building the **rusteron** project, please refer to the [HOW_TO_BUILD.md](../HOW_TO_BUILD.md) file.

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