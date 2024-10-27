# rusteron-media-driver

**rusteron-media-driver** is a module in the **rusteron** project that provides an interface to the Aeron Media Driver in a Rust environment. This module is crucial for managing the messaging infrastructure between producers and consumers, allowing for efficient low-latency communication.

## Overview

The **rusteron-media-driver** module is designed to help Rust developers interact with the Aeron Media Driver, which is responsible for managing the communication between different Aeron clients. This module provides a wrapper around the Aeron C Media Driver API and offers both standard and embedded driver options for use in different environments.

The media driver can be used to set up the messaging directory and manage data streams. The embedded media driver is particularly useful for testing purposes or for simplifying deployment scenarios where a separate media driver process is not needed.

## Usage Note

It is recommended to run the media driver using the Aeron Java or C version for production use. This crate primarily serves as a utility to start the media driver embedded within unit or integration tests.

## Installation

To use **rusteron-media-driver**, add it to your `Cargo.toml`:

```toml
[dependencies]
rusteron-media-driver = "0.1"
```

Ensure that you have also set up the necessary Aeron C libraries required by **rusteron-media-driver**.

## Features

- **Media Driver Management**: Start, stop, and configure an Aeron Media Driver instance.
- **Embedded Media Driver**: Launch an embedded media driver directly from your code for testing purposes.

## Usage Examples

### Standard Media Driver Example

```rust ,no_run
use rusteron_media_driver::{AeronDriver, AeronDriverContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aeron_context = AeronDriverContext::new()?;
    aeron_context.set_dir("target/test")?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(aeron_context.clone())?;
    aeron_driver.start(false)?;
    println!("Aeron Media Driver started");
    
    Ok(())
}
```

### Embedded Media Driver Example

```rust ,no_run
use rusteron_media_driver::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and launch an embedded media driver
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

    // Create Aeron context and link with the embedded driver
    let ctx = AeronContext::new()?;
    ctx.set_dir(media_driver_ctx.get_dir())?;
    
    // Wait a bit for demonstration purposes
    thread::sleep(Duration::from_secs(3));

    // Stop the driver
    stop.store(true, Ordering::SeqCst);
    driver_handle.join().expect("Failed to join driver thread");
    println!("Embedded Aeron Media Driver stopped");

    Ok(())
}
```

## Safety Considerations

Since **rusteron-media-driver** relies on Aeron C bindings, it involves the use of `unsafe` Rust code. Users must ensure:

- Resources are properly managed (e.g., starting and stopping drivers in a correct order).
- Proper synchronisation when accessing shared data in a multithreaded environment.

Improper use of the media driver can lead to crashes or other undefined behaviours.

## Contributing

Contributions are more than welcome! Please see our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md) for more information on how to get involved.

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!

