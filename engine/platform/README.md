# Obel Platform

[![license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel)
[![downloads](https://img.shields.io/crates/d/obel.svg)](https://crates.io/crates/obel)
[![docs.rs](https://docs.rs/obel/badge.svg)](https://docs.rs/obel/latest/obel/)
[![discord.online](https://img.shields.io/discord/1335036405788971020.svg?label=&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3jq8js8u)

## Table of Contents
- [Obel Platform](#obel-platform)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Installation](#installation)
    - [Using Cargo](#using-cargo)
    - [No Standard Library Support](#no-standard-library-support)
  - [Features](#features)
    - [`std` (default)](#std-default)
    - [`alloc` (default)](#alloc-default)
    - [`portable-atomic`](#portable-atomic)
    - [`critical-section`](#critical-section)
  - [Platform Support](#platform-support)
  - [Troubleshooting](#troubleshooting)
    - [Common Issues](#common-issues)

## Overview

Rust is a fantastic multi-platform language with extensive support for modern targets through its [standard library](https://doc.rust-lang.org/stable/std/).
However, some items within the standard library have alternatives that are better suited for [obel](https://crates.io/crates/obel) and game engines in general.
Additionally, to support embedded and other esoteric platforms, it's often necessary to shed reliance on `std`, making your crate [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html).

The `obel_platform` crate addresses these needs by providing alternatives and extensions to the Rust standard library. Our goal is to minimize friction when developing with and for obel across multiple platforms, from desktop to embedded systems.

## Installation

### Using Cargo

Add the dependency to your `Cargo.toml` file:

```toml
[dependencies]
obel_platform = "0.1.0"
```

Or use the cargo command:

```sh
cargo add obel_platform
```

### No Standard Library Support

For `no_std` platforms, disable default features in your `Cargo.toml`:

```toml
[dependencies]
obel_platform = { version = "0.1.0", default-features = false }
```

## Features

### `std` (default)

Enables usage of the standard library. Even with `std` enabled, this crate provides optimized alternatives to standard library components where beneficial for game engine performance. Key benefits include:

- Optimized collections for game engine use cases
- Enhanced error handling suited for game development
- Platform-specific optimizations

**Note**: This feature is incompatible with `no_std` targets.

### `alloc` (default)

Provides support for heap allocation through the [`alloc`](https://doc.rust-lang.org/stable/alloc/) crate. Features include:

- Dynamic memory allocation
- Collection types (Vec, String, etc.)
- Smart pointers (Box, Rc, Arc)

**Note**: This feature is automatically enabled with `std` and works on most `no_std` targets.

### `portable-atomic`

Implements atomic operations using [`portable-atomic`](https://docs.rs/portable-atomic/latest/portable_atomic/) as the backend. Essential for:

- Platforms lacking native atomic operations
- Consistent atomic behavior across different architectures
- Support for atomic types like `Arc`, `AtomicU8`, etc.

Enable this feature when targeting platforms with limited atomic operation support or requiring [atomic CAS](https://en.wikipedia.org/wiki/Compare-and-swap) operations.

### `critical-section`

Provides synchronization primitives using [`critical-section`](https://docs.rs/critical-section/latest/critical_section/). Useful for:

- Platforms with minimal atomic operation support
- Embedded systems requiring careful resource management
- Synchronization in interrupt-heavy environments

Often used in conjunction with the `portable-atomic` feature for comprehensive synchronization support.

## Platform Support

This crate supports a wide range of platforms:

- Desktop (Windows, macOS, Linux)
- Mobile (iOS, Android)
- Web (WebAssembly)
- Embedded systems
- Custom platforms

## Troubleshooting

### Common Issues

1. **Atomic Operations Not Available**
   - Enable the `portable-atomic` feature
   - Ensure target platform is supported

2. **Allocation Errors in No-STD**
   - Verify `alloc` feature configuration
   - Check for proper allocator setup

3. **Synchronization Problems**
   - Consider using `critical-section` feature
   - Review atomic operation requirements

For more help, visit our [Discord community](https://discord.gg/3jq8js8u) or file an issue on GitHub.