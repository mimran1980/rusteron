# rusteron

[![github-ci](https://github.com/mimran1980/rusteron/actions/workflows/ci.yml/badge.svg)](https://github.com/amoskvin/rusteron/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/rusteron/badge.svg)](https://docs.rs/rusteron/)

A Rust client wrapper for Aeron C bindings

## Important Notice

Please note that these crates do not provide an idiomatic Rust API for interacting with Aeron; `aeron-rs` should be used instead.

## Project Background

This project is based on a fork of [libaeron-sys](https://github.com/bspeice/libaeron-sys). It takes the C bindings and then tries to automatically generate the corresponding Rust code that wrap the underlying C bindings.

## Project Status

This is an alpha version.

## Modules

- **rusteron-code-gen**: This module is used to generate the wrappers from the bindings.
- **rusteron-client**: This is the Aeron client module.
- **rusteron-archive**: This module includes the Aeron client with archive functionalities.
- **rusteron-media-driver**: This module is for the Aeron media driver.
