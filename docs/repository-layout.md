<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Repository Layout

StateLink uses a Rust workspace. The repository is organized so that protocol definition, reusable libraries, executable services, examples, demos, and documentation have distinct locations.

## Top-level map

```text
statelink/
├── crates/                 Rust implementation
│   ├── statelink-protocol  Wire types and topic/filter logic
│   ├── statelink-core      State bus, ownership, policy, lifecycle
│   ├── statelink-client    Async Rust WebSocket client
│   ├── statelink-server    Reference WebSocket server
│   └── statelink-mqtt      StateLink -> MQTT export bridge
│
├── docs/                   Human-oriented usage and architecture docs
├── examples/               Example index and protocol examples
├── demos/                  End-to-end runnable demonstrations
├── config/                 Development auth/policy examples
├── scripts/                Helper scripts for testing and demos
├── spec/                   Normative protocol specification
├── .github/workflows/      CI
│
├── Cargo.toml              Workspace definition
├── Cargo.lock              Locked dependency graph
├── Dockerfile              Multi-stage image build
├── Makefile                Common developer commands
├── README.md               Project landing page
├── SECURITY.md             Security reporting/deployment notes
└── LICENSE                 GPL-3.0-or-later
```

## `crates/`: implementation source

### `crates/statelink-protocol`

This is the protocol data model. Start here when changing the wire format.

Important files:

```text
src/lib.rs      Request/response enums, context state, error codes
src/topic.rs    Topic and filter validation/matching
```

It defines, among other things:

- `Request` and `Response`;
- `ContextState`;
- `AccessPolicy`;
- `Retention` and `Freshness`;
- `ErrorCode`;
- protocol version constants;
- MQTT-style topic semantics.

### `crates/statelink-core`

This is the transport-independent state engine. It should not know about Axum/WebSocket framing.

Important files:

```text
src/bus.rs       Context ownership, subscriptions, revisions, lifecycle
src/policy.rs    Deny-by-default authorization policy
src/identity.rs  Authenticated identity model
src/lib.rs       Public exports
```

Put state semantics here rather than in the WebSocket server.

### `crates/statelink-client`

The async Rust client SDK.

Important files:

```text
src/lib.rs           Client::connect, send, next_response
examples/producer.rs Canonical Rust producer example
examples/consumer.rs Canonical Rust consumer example
```

The Rust examples remain inside this crate because `cargo run -p statelink-client --example ...` is the idiomatic Cargo mechanism. The top-level [`examples/`](../examples/README.md) directory exists as a discoverable index and provides wire-level examples.

### `crates/statelink-server`

The executable reference server.

Important files:

```text
src/main.rs  HTTP/WebSocket listener, connection/session handling, dispatch
src/auth.rs  Bearer/cookie authentication and browser Origin checking
```

The server adapts network connections to `statelink-core`. Protocol/state rules should generally stay in `statelink-core` rather than accumulating in this crate.

### `crates/statelink-mqtt`

A one-way export bridge from authorized StateLink current state to MQTT retained publications.

It is an adapter, not the StateLink core protocol.

## `docs/`: usage documentation

Start at [`docs/README.md`](README.md). This directory explains how to use the implementation.

The protocol specification is intentionally not buried here; it remains under `spec/` so the distinction between normative specification and explanatory documentation stays clear.

## `examples/`: discoverable examples

Start at [`../examples/README.md`](../examples/README.md).

This directory answers the question "show me how to use StateLink" without requiring someone to know Cargo's crate-level `examples/` convention.

It links to the canonical Rust examples and contains wire-protocol examples for non-Rust integrations.

## `demos/`: runnable systems

Start at [`../demos/README.md`](../demos/README.md).

A demo is different from an example:

- an **example** shows one API or usage pattern;
- a **demo** starts multiple components and demonstrates a complete flow.

The `basic` demo starts a server, consumer, and producer with Docker Compose.

## `config/`: development configuration

`config/dev-auth.json` and `config/dev-policy.json` are intentionally simple development examples.

Do not use their static demo tokens in production.

## `spec/`: normative protocol

[`../spec/STATE_LINK_V0.1.md`](../spec/STATE_LINK_V0.1.md) defines StateLink v0.1 interoperability requirements.

If you are implementing StateLink in another language or building an independent server, this is the authoritative starting point.

## `scripts/`: convenience commands

Scripts should make repeatable developer tasks easier; they should not contain protocol semantics.

Current examples include Docker-based testing and demo startup for PowerShell users.

## Where should new code go?

| Change | Location |
| --- | --- |
| New request/response field | `crates/statelink-protocol` |
| Topic/filter rule | `crates/statelink-protocol` |
| Context/revision/subscription behavior | `crates/statelink-core` |
| Authorization policy semantics | `crates/statelink-core` |
| HTTP/WebSocket transport behavior | `crates/statelink-server` |
| Rust SDK convenience API | `crates/statelink-client` |
| MQTT export behavior | `crates/statelink-mqtt` |
| Runnable API sample | `examples/` or crate `examples/` |
| Multi-component scenario | `demos/` |
| User/developer explanation | `docs/` |
| Normative interoperability rule | `spec/` |

Keeping these boundaries matters: StateLink should remain usable independently of any particular transport or application.
