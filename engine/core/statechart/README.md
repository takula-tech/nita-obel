# Obel Statechart

[![license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel)
[![downloads](https://img.shields.io/crates/d/obel.svg)](https://crates.io/crates/obel)
[![docs.rs](https://docs.rs/obel/badge.svg)](https://docs.rs/obel/latest/obel/)
[![discord.online](https://img.shields.io/discord/1335036405788971020.svg?label=&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3jq8js8u)

You can think of statechart(SC) as kind of event-driven behavior tree (BT).
But actually SC is more powerful and has more benefits than traditional BT.

This impls of SC is ported from well-known statechart JS implementation - [XState]('https://github.com/statelyai/xstate').
For more details on statechart, please refer to the [xstate docs]('https://stately.ai/docs/xstate').

## Statechart implemented by ECS

### Statechart Component

A Statechart (SC) is an advanced form of event-driven Behavior Tree (BT) that offers enhanced capabilities and flexibility. This implementation is based on [XState](https://github.com/statelyai/xstate), a popular JavaScript statechart library. For comprehensive documentation, visit the [XState documentation](https://stately.ai/docs/xstate).

## ECS Implementation

### Components

The Statechart Component (SCCmpt) is composed of four main components:

1. **Context Component**

   - Implements a Rust enum-based JSON-like data structure
   - Provides CRUD operations for state management

2. **State Node Component**

   - Uses a cache-friendly double-linked array list structure
   - Optimized for efficient state transitions between nodes

3. **Incoming Mailbox Component**

   - Functions as a queue for received events
   - Manages event reception and processing

4. **Outgoing Mailbox Component**
   - Serves as a queue for events to be sent
   - Handles event distribution

### System Architecture

#### 1. Incoming Mailbox System

**Functionality:**

- Processes events sequentially
- Executes event handler functions
- Manages state node transitions

**Concurrency Features:**

- Multi-threaded execution utilized with all cpu cores
- Automatic components load balancing to prevent thread starvation
- Dedicated system group for sequential runs of systems

#### 2. Outgoing Mailbox System

- Derived events are stored in the outgoing mailbox component
- System collects and routes events to appropriate incoming mailboxes
- Target entities receive events via their incoming mailbox components
- Incoming mailbox system processes new events
