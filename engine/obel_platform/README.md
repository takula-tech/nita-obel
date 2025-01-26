# Obel Platform

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/ntakulatech/nita_obel#license)
[![Crates.io](https://img.shields.io/crates/v/obel_platform.svg)](https://crates.io/crates/obel_platform)
[![Downloads](https://img.shields.io/crates/d/obel_platform.svg)](https://crates.io/crates/obel_platform)
[![Docs](https://docs.rs/obel_platform/badge.svg)](https://docs.rs/obel_platform/latest/obel_platform/)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/obel)

## Overview

`obel_platform` is a specialized crate designed to enhance cross-platform development  
for [Obel](https://crates.io/crates/nita_obel) game engine projects. While Rust's [standard library](https://doc.rust-lang.org/stable/std/) provides excellent multi-platform support,  
this crate offers optimized alternatives specifically tailored for game development and embedded systems.

Key benefits:

- Platform-optimized alternatives to standard library components
- First-class support for [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments
- Seamless integration with Bevy ecosystem

## Installation

Add the crate to your project using cargo:

```sh
cargo add obel_platform
```

## Usage

Simply import from `obel_platform` instead of `std` for supported items. Refer to the [documentation](https://docs.rs/obel_platform/latest/obel_platform/) for available items and their benefits.

## Features

### Standard Library Support (`std`) [default]

- Enables standard library integration
- Provides optimized alternatives where beneficial
- Incompatible with `no_std` targets

### Allocation Support (`alloc`) [default]

- Enables [`alloc`](https://doc.rust-lang.org/stable/alloc/) crate functionality
- Automatically enabled with `std` feature
- Compatible with most `no_std` targets

### Portable Atomics (`portable-atomic`)

- Uses [`portable-atomic`](https://docs.rs/portable-atomic/latest/portable_atomic/) for atomic operations
- Essential for platforms with limited atomic operation support
- Provides consistent atomic behavior across platforms

### Critical Section Support (`critical-section`)

- Implements synchronization using [`critical-section`](https://docs.rs/critical-section/latest/critical_section/)
- Ideal for platforms with minimal atomic operation support
- Often used in conjunction with `portable-atomic`

## No-std Configuration

To use on generic(`no_std`) platforms, disable default features but enable `other` feature in your `Cargo.toml`:

```toml
obel_platform = { version = "x.y.z", default-features = false, features = ["generic"]  }
```
