<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Rust Workspace Crates

StateLink is split into focused crates so protocol/state semantics do not become tied to one network transport.

## Dependency direction

Conceptually:

```text
statelink-protocol
      ^
      |
statelink-core
      ^
      |
statelink-server

statelink-protocol
      ^
      |
statelink-client
      ^
      |
statelink-mqtt
```

The exact Cargo dependency graph may contain additional library dependencies, but this is the intended responsibility layering.

## `statelink-protocol`

Wire-level types and topic/filter behavior.

Use it for:

- request/response serialization;
- error codes;
- context state metadata;
- MQTT-compatible topic/filter validation and matching;
- protocol/subprotocol constants.

Do not put server transport code here.

## `statelink-core`

Transport-independent state engine.

Use it for:

- single-writer context ownership;
- revisions;
- context lifecycle;
- subscriptions/discovery;
- TTL/freshness;
- retention behavior;
- authorization/system policy;
- delivery generation.

This crate should remain reusable by future transports.

## `statelink-client`

Async Rust WebSocket SDK.

Use it when writing Rust producers or consumers.

Runnable Cargo examples are intentionally here:

```text
statelink-client/examples/producer.rs
statelink-client/examples/consumer.rs
```

For user-facing navigation, see [`../examples/README.md`](../examples/README.md).

## `statelink-server`

Reference Axum/WebSocket server.

Use it for:

- HTTP listener;
- WebSocket upgrade;
- bearer/cookie authentication adapter;
- browser Origin check;
- connection/session lifecycle;
- converting WebSocket frames to/from protocol types;
- bounded per-client outbound queue.

State rules should be implemented in `statelink-core` whenever possible rather than here.

## `statelink-mqtt`

One-way adapter that subscribes to StateLink and exports `state.value` to MQTT retained publications.

It is an integration component, not the authoritative state bus.

## More detail

Read [`../docs/repository-layout.md`](../docs/repository-layout.md) for the complete repository map and placement rules for new code.
