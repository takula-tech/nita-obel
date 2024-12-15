# [![obel](assets/branding/obel_logo_light_dark_and_dimmed.svg)](https://obelengine.org)

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![Crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel)
[![Downloads](https://img.shields.io/crates/d/obel.svg)](https://crates.io/crates/obel)
[![Docs](https://docs.rs/obel/badge.svg)](https://docs.rs/obel/latest/obel/)
[![CI](https://github.com/obelengine/obel/workflows/CI/badge.svg)](https://github.com/obelengine/obel/actions)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/obel)

## What is obel?

obel is a refreshingly simple data-driven game engine built in Rust. It is free and open-source forever!

## WARNING

obel is still in the early stages of development. Important features are missing. Documentation is sparse. A new version of obel containing breaking changes to the API is released [approximately once every 3 months](https://obelengine.org/news/obel-0-6/#the-train-release-schedule). We provide [migration guides](https://obelengine.org/learn/migration-guides/), but we can't guarantee migrations will always be easy. Use only if you are willing to work in this environment.

**MSRV:** obel relies heavily on improvements in the Rust language and compiler.
As a result, the Minimum Supported Rust Version (MSRV) is generally close to "the latest stable release" of Rust.

## Design Goals

* **Capable**: Offer a complete 2D and 3D feature set
* **Simple**: Easy for newbies to pick up, but infinitely flexible for power users
* **Data Focused**: Data-oriented architecture using the Entity Component System paradigm
* **Modular**: Use only what you need. Replace what you don't like
* **Fast**: App logic should run quickly, and when possible, in parallel
* **Productive**: Changes should compile quickly ... waiting isn't fun

## About

* **[Features](https://obelengine.org):** A quick overview of obel's features.
* **[News](https://obelengine.org/news/)**: A development blog that covers our progress, plans and shiny new features.

## Docs

* **[Quick Start Guide](https://obelengine.org/learn/quick-start/introduction):** obel's official Quick Start Guide. The best place to start learning obel.
* **[obel Rust API Docs](https://docs.rs/obel):** obel's Rust API docs, which are automatically generated from the doc comments in this repo.
* **[Official Examples](https://github.com/obelengine/obel/tree/latest/examples):** obel's dedicated, runnable examples, which are great for digging into specific concepts.
* **[Community-Made Learning Resources](https://obelengine.org/assets/#learning)**: More tutorials, documentation, and examples made by the obel community.

## Community

Before contributing or participating in discussions with the community, you should familiarize yourself with our [**Code of Conduct**](./CODE_OF_CONDUCT.md).

* **[Discord](https://discord.gg/obel):** obel's official discord server.
* **[Reddit](https://reddit.com/r/obel):** obel's official subreddit.
* **[GitHub Discussions](https://github.com/obelengine/obel/discussions):** The best place for questions about obel, answered right here!
* **[obel Assets](https://obelengine.org/assets/):** A collection of awesome obel projects, tools, plugins and learning materials.

### Contributing

If you'd like to help build obel, check out the **[Contributor's Guide](https://obelengine.org/learn/contribute/introduction)**.
For simple problems, feel free to [open an issue](https://github.com/obelengine/obel/issues) or
[PR](https://github.com/obelengine/obel/pulls) and tackle it yourself!

For more complex architecture decisions and experimental mad science, please open an [RFC](https://github.com/obelengine/rfcs) (Request For Comments) so we can brainstorm together effectively!

## Getting Started

We recommend checking out the [Quick Start Guide](https://obelengine.org/learn/quick-start/introduction) for a brief introduction.

Follow the [Setup guide](https://obelengine.org/learn/quick-start/getting-started/setup) to ensure your development environment is set up correctly.
Once set up, you can quickly try out the [examples](https://github.com/obelengine/obel/tree/latest/examples) by cloning this repo and running the following commands:

```sh
# Switch to the correct version (latest release, default is main development branch)
git checkout latest
# Runs the "breakout" example
cargo run --example breakout
```

To draw a window with standard functionality enabled, use:

```rust
use obel::prelude::*;

fn main(){
  App::new()
    .add_plugins(DefaultPlugins)
    .run();
}
```

### Fast Compiles

obel can be built just fine using default configuration on stable Rust. However for really fast iterative compiles, you should enable the "fast compiles" setup by [following the instructions here](https://obelengine.org/learn/quick-start/getting-started/setup).

## [obel Cargo Features][cargo_features]

This [list][cargo_features] outlines the different cargo features supported by obel. These allow you to customize the obel feature set for your use-case.

[cargo_features]: docs/cargo_features.md

## Thanks

obel is the result of the hard work of many people. A huge thanks to all obel contributors, the many open source projects that have come before us, the [Rust gamedev ecosystem](https://arewegameyet.rs/), and the many libraries we build on.

A huge thanks to obel's [generous sponsors](https://obelengine.org). obel will always be free and open source, but it isn't free to make. Please consider [sponsoring our work](https://obelengine.org/donate/) if you like what we're building.

<!-- This next line need to stay exactly as is. It is required for BrowserStack sponsorship. -->
This project is tested with BrowserStack.

## License

obel is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/obelengine/obel/issues/2373) to include both.

Some of the engine's code carries additional copyright notices and license terms due to their external origins.
These are generally BSD-like, but exact details vary by crate:
If the README of a crate contains a 'License' header (or similar), the additional copyright notices and license terms applicable to that crate will be listed.
The above licensing requirement still applies to contributions to those crates, and sections of those crates will carry those license terms.
The [license](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields) field of each crate will also reflect this.
For example, [`obel_mikktspace`](./crates/obel_mikktspace/README.md#license-agreement) has code under the Zlib license (as well as a copyright notice when choosing the MIT license).

The [assets](assets) included in this repository (for our [examples](./examples/README.md)) typically fall under different open licenses.
These will not be included in your game (unless copied in by you), and they are not distributed in the published obel crates.
See [CREDITS.md](CREDITS.md) for the details of the licenses of those files.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.

## crate.io push token
cio8tdsT1HVJJLcwTHjNLF2uG1keAcAjaZP