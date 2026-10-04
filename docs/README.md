<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# StateLink Documentation

This directory is the main documentation entry point for StateLink. The normative protocol specification remains in [`../spec/STATE_LINK_V0.1.md`](../spec/STATE_LINK_V0.1.md); the documents here explain how to build, run, integrate, configure, and operate the reference implementation.

## Start here

If you are new to StateLink, read these in order:

1. [Getting started](getting-started.md) — run the Docker demo, start the server manually, and send your first state.
2. [Core concepts](concepts.md) — contexts, topics, ownership, revisions, freshness, retention, schemas, and desired/reported state.
3. [Architecture](architecture.md) — how protocol, core, server, client, and adapters fit together.
4. [Repository layout](repository-layout.md) — where source code, examples, demos, configuration, protocol specification, and scripts live.
5. [Rust client guide](rust-client.md) — integrate StateLink into a Rust producer or consumer.
6. [Wire protocol guide](wire-protocol.md) — use StateLink directly over WebSocket from any language.

## Task-oriented guide

| I want to... | Read / use |
| --- | --- |
| Run something immediately | [Getting started](getting-started.md) |
| Understand the design | [Architecture](architecture.md) |
| Find runnable examples | [`../examples/README.md`](../examples/README.md) |
| Run an end-to-end demo | [`../demos/README.md`](../demos/README.md) |
| Understand the source tree | [Repository layout](repository-layout.md) |
| Write a Rust producer | [Rust client guide](rust-client.md#producer-example) |
| Write a Rust consumer | [Rust client guide](rust-client.md#consumer-example) |
| Use another language | [Wire protocol guide](wire-protocol.md) |
| Configure the server | [Server configuration](server-configuration.md) |
| Configure authentication and authorization | [Security and policy](security-and-policy.md) |
| Export state to MQTT | [MQTT bridge](mqtt-bridge.md) |
| Build, test, or contribute | [Development guide](development.md) |
| Diagnose an error | [Troubleshooting](troubleshooting.md) |
| Implement the protocol independently | [`../spec/STATE_LINK_V0.1.md`](../spec/STATE_LINK_V0.1.md) |

## Documentation vs. specification

The documents under `docs/` are explanatory. They describe the current Rust reference implementation and recommended usage patterns.

The file [`spec/STATE_LINK_V0.1.md`](../spec/STATE_LINK_V0.1.md) is normative for StateLink v0.1. When explanatory documentation and the protocol specification disagree, the protocol specification takes precedence.

## Current implementation status

StateLink is currently an early v0.1 reference implementation. It provides:

- a transport-independent state bus and policy engine;
- MQTT-compatible topic/filter syntax;
- authenticated single-writer context ownership;
- deny-by-default system authorization;
- producer-controlled read exposure bounded by system policy;
- security-filtered discovery;
- subscriptions that may exist before a producer appears;
- monotonically increasing per-context revisions;
- TTL-based freshness and stale state;
- session and retained context lifecycles;
- immutable optional schema identifiers;
- desired/reported synchronization through `applied_revision`;
- an authenticated WebSocket server using the `statelink.v1` subprotocol;
- an async Rust client;
- a one-way StateLink-to-MQTT retained-state bridge;
- Docker demo and Docker-based validation.

StateLink v0.1 is intentionally not an RPC system, event queue, event history, or distributed transaction protocol.
