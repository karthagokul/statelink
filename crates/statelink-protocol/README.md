<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# statelink-protocol

Wire-level StateLink v0.1 types and topic/filter semantics.

## Responsibilities

This crate owns:

- `Request` and `Response` JSON models;
- `ContextState` and discovery metadata;
- error-code vocabulary;
- retention/freshness enums;
- producer `AccessPolicy` representation;
- protocol version/subprotocol constants;
- MQTT-compatible topic/filter validation and matching.

It intentionally does **not** own WebSocket server logic, authentication adapters, or context lifecycle/state-bus implementation.

## Important files

```text
src/lib.rs    protocol request/response/state types
src/topic.rs  topic/filter validation and matching
```

## WebSocket subprotocol

```text
statelink.v1
```

## Tests

```bash
cargo test -p statelink-protocol
```

## Documentation

- [`../../docs/wire-protocol.md`](../../docs/wire-protocol.md)
- [`../../docs/concepts.md`](../../docs/concepts.md)
- [`../../spec/STATE_LINK_V0.1.md`](../../spec/STATE_LINK_V0.1.md)
