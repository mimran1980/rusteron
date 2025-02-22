# Rusteron

[![Crates.io](https://img.shields.io/crates/v/rusteron-archive)](https://crates.io/crates/rusteron-archive)
[![github-ci](https://github.com/mimran1980/rusteron/actions/workflows/ci.yml/badge.svg)](https://github.com/amoskvin/rusteron/actions/workflows/ci.yml)
[![docsrs](https://docs.rs/rusteron-client/badge.svg)](https://docs.rs/rusteron-client/)
[![github docs](https://custom-icon-badges.demolab.com/badge/githubdocs-blue.svg?logo=log&logoSource=feather)](https://mimran1980.github.io/rusteron)

**rusteron** is a Rust client wrapper for the [Aeron](https://github.com/real-logic/aeron) C API, designed to help Rust developers interact with Aeron, a high-performance messaging system. This library is built as an extension to the Aeron C bindings, making it easier to leverage Aeron's low-latency communication capabilities in Rust applications.

> **Note**: For an idiomatic Rust API for Aeron, consider using [`aeron-rs`](https://github.com/UnitedTraders/aeron-rs)

## Project Background

This project is based on a fork of [`libaeron-sys`](https://github.com/bspeice/libaeron-sys). Since it uses C bindings, **rusteron** inherently operates in an `unsafe` context. This means it requires extra caution when using, as incorrect handling can lead to undefined behaviour such as segmentation faults. For example, using a publisher after the `Aeron` client has closed will result in a segmentation fault.

The purpose of **rusteron** is to take these C bindings and generate a Rust wrapper around them to facilitate more ergonomic and performant use in Rust-based applications. Although this is not yet fully idiomatic, it provides a working bridge to the Aeron C API.

The modules in **rusteron** aim to replicate and extend the capabilities available in the Aeron ecosystem, while simplifying their use for developers accustomed to Rust.

## Project Status

**rusteron** is currently in release candiante phase it is being planned to be used in a production system, meaning:

- It is still under active development.
- APIs may be subject to breaking changes.

Community feedback and contributions are welcome to improve its functionality and usability.

## Modules Overview

The library is divided into several modules, each focusing on specific parts of Aeron's functionality:

- **[rusteron-code-gen](https://github.com/mimran1980/rusteron/tree/main/rusteron-code-gen)**: This module is
  responsible for generating the Rust wrapper from the raw C bindings. It helps maintain a clean and repeatable way to
  bridge between the two languages.

- **[rusteron-client](https://github.com/mimran1980/rusteron/tree/main/rusteron-client)**: Provides core client
  functionalities for interacting with the Aeron protocol, such as establishing connections, subscribing, and
  publishing. It uses the Aeron C bindings from aeron-client module.

- **[rusteron-archive](https://github.com/mimran1980/rusteron/tree/main/rusteron-archive)**: Extends the Aeron client to
  include archiving features, such as recording streams and handling replay capabilities. It uses the Aeron C bindings
  from aeron-archive module.
  Note rusteron-archive also includes rusteron-client code so no need to have them both as dependancies. Also make sure
  the media driver matches lib version.

- **[rusteron-media-driver](https://github.com/mimran1980/rusteron/tree/main/rusteron-media-driver)**: Implements the
  Aeron Media Driver, a core component for managing messaging between producers and consumers. It uses the Aeron C
  bindings from aeron-driver module.

- **[rusteron-rb](https://github.com/mimran1980/rusteron/tree/main/rusteron-rb)**: Provides ring buffer and broadcast functionalities via aeron c bindings, allowing efficient, low-latency message passing between different threads or processes. This module implements Single Producer, Single Consumer (SPSC) ring buffers, Multi-Producer, Single Consumer (MPSC) ring buffers, and broadcast channels.

- **[rusteron-docker-samples](https://github.com/mimran1980/rusteron/tree/main/rusteron-docker-samples)**: A collection of simple examples demonstrating how Docker configurations might look for Aeron-based applications. The examples include Dockerfiles for Aeron Media Driver and dummy components like ticker writer and ticker reader, showcasing shared resource management and environment configurations. These are not production-ready configurations but can serve as inspiration for setting up your own Docker-based workflows. For details, see the module's [README](https://github.com/mimran1980/rusteron/tree/main/rusteron-docker-samples/README.md).

## Installation

Add the following line to your `Cargo.toml` to include the specific **rusteron** module you need in your project. Depending on your use case, you can choose from the following modules:

- **rusteron-client**: For core Aeron client functionalities.
- **rusteron-archive**: For Aeron client with archive capabilities.
- **rusteron-media-driver**: For the Aeron media driver.
- **rusteron-rb**: Ring buffer (single and multi producer) and broadcast transmitter/receiver

For detailed instructions on how to build **rusteron-archive**, please refer to the [HOW_TO_BUILD.md](./HOW_TO_BUILD.md) file.

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

Replace `rusteron-client` with `rusteron-archive`/`rusteron-media-driver`/`rusteron-rb` as per your requirement.

## Usage Example

### Aeron Client Example

Below is a step-by-step example of creating and using an Aeron client.

```rust,no_ignore
use rusteron::client::{Aeron, AeronContext};
use rusteron_media_driver::AeronDriverContext;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start embedded media driver to testing
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

    let ctx = AeronContext::new()?;
    ctx.set_dir(media_driver_ctx.get_dir())?;
    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    let subscription = aeron
        .async_add_subscription("aeron:ipc", 123,                
                                Handlers::no_available_image_handler(),
                                Handlers::no_unavailable_image_handler())?
        .poll_blocking(Duration::from_secs(5))?;

    let publisher = aeron
        .async_add_publication("aeron:ipc", 123)?
        .poll_blocking(Duration::from_secs(5))?;

    let message = "Hello, Aeron!".as_bytes();
    // if <0 its an error
    let result = publisher.offer(message, Handlers::no_reserved_value_supplier_handler());

        struct FragmentHandler;

        impl AeronFragmentHandlerCallback for FragmentHandler {
            fn handle_aeron_fragment_handler(
                &mut self,
                msg: &[u8],
                header: AeronHeader,
            ) {
              println!(
                  "received a message from aeron {:?}, msg length:{}",
                  header.position(),
                  msg.len()
              );
            }
        }
    let (closure, _inner) = Handler::leak_with_fragment_assembler(FragmentHandler)?;

    let mut count = 0;
    while count < 10000 {
        subscription.poll(Some(&closure), 128)?;
      count += 1;
    }
    Ok(())
}
```

## Benchmarks

You can view the benchmarks for this project by visiting [BENCHMARKS.md](./BENCHMARKS.md).

## Contributing

Contributions are more than welcome! Please feel free to open issues, submit pull requests, or suggest new features. We are particularly looking for:

- Feedback on the API design.
- Bug reports and feature requests.
- Documentation improvements.

If you're interested in helping, check out our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md).

## License

This project is licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), at your option.

## Acknowledgments

Special thanks to the authors of the original [`libaeron-sys`](https://github.com/bspeice/libaeron-sys) for their foundational work, and to the Aeron community for supporting open development in high-performance messaging.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-client/)
- [API Reference on github](https://mimran1980.github.io/rusteron)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

---

Feel free to reach out with any questions or suggestions via GitHub Issues!

