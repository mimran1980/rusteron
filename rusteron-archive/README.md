# rusteron-archive

**rusteron-archive** is a module within the **rusteron** project that provides functionalities for interacting with Aeron's archive capabilities in a Rust environment. This module aims to extend **rusteron-client** by offering features for recording streams, managing archives, and handling replay capabilities.

## Overview

The **rusteron-archive** module is intended to help Rust developers leverage Aeron's archive functionalities, including the recording and replaying of messages. However, this module is currently in an early stage and has not been thoroughly tested.

The code in **rusteron-archive** is generated as a Rust wrapper around the Aeron C archive API, making it easier for Rust developers to work with Aeron's archiving capabilities. Since this module also uses C bindings, it involves an `unsafe` context, and extra caution is advised when using it.

## Project Status

- **Current Focus**: Our primary focus is currently on **rusteron-client**. As such, **rusteron-archive** has not undergone extensive testing.
- **Alpha Version**: **rusteron-archive** is in an alpha stage, and developers are encouraged to experiment with it, but it is not recommended for production use at this point.

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

Since **rusteron-archive** relies on Aeron C bindings, it uses `unsafe` Rust code. Users must ensure that resources are managed properly to avoid crashes or undefined behaviour.


## Building This Project Instructions

For detailed instructions on how to build **rusteron**, please refer to the [HOW_TO_BUILD.md](../HOW_TO_BUILD.md) file.

## Contributing

Contributions are welcome! Please see our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md) for more information on how to get involved.

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-archive/)
- [API Reference on github](https://mimran1980.github.io/rusteron/rusteron_archive)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!

