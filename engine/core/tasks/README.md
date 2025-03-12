# Obel Tasks

[![license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel)
[![downloads](https://img.shields.io/crates/d/obel.svg)](https://crates.io/crates/obel)
[![docs.rs](https://docs.rs/obel/badge.svg)](https://docs.rs/obel/latest/obel/)
[![discord.online](https://img.shields.io/discord/1335036405788971020.svg?label=&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3jq8js8u)


# A Lightweight Task Executor

A refreshingly simple task executor designed specifically for Obel. :)

This library offers a minimalistic threadpool with few dependencies. Its primary purpose is to support scoped fork-join operations, where tasks are spawned from a single thread, and the same thread waits for their completion.  

It is tailored for [`obel`][obel] as a lighter alternative to [`rayon`][rayon] for this specific use case. Additionally, it provides utilities for generating tasks from data slices. Built with game development in mind, this library does not enforce task fairness or execution order.

It is powered by [`async-executor`][async-executor], a lightweight executor that allows users to manage their own threads. `async-executor` is built on `async-task`, a core component of `async-std`.

---

## Usage

Obel provides three distinct thread pools to optimize task execution in multi-threaded environments. (The same API is used in single-threaded environments, such as Wasm targets, where execution is limited to a single thread.) The choice of pool depends on the latency requirements of the tasks:

- **CPU-intensive tasks**: These are tasks that typically run continuously until completion. Obel offers two pools for such tasks:
  - [`ComputeTaskPool`]: For tasks that need to complete before the next frame.
  - [`AsyncComputeTaskPool`]: For tasks that do not need to finish before rendering the next frame.

- **IO-intensive tasks**: These are tasks that spend little time in an active state (e.g., waiting for data from disk). The [`IoTaskPool`] is designed for such tasks, which are expected to complete quickly and often signal other systems when data is ready (e.g., via channels).

---

## `no_std` Support

To enable `no_std` support in this crate, you must disable default features and enable the `edge_executor` and `critical-section` features.

---
[obel]: https://github.com/obel 
[rayon]: https://github.com/rayon-rs/rayon  
[async-executor]: https://github.com/stjepang/async-executor  
