<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Development Guide

This document describes how to build, test, and extend the StateLink Rust workspace.

## Toolchain

The workspace currently targets:

```text
Rust 1.85+
Edition 2021
```

Check your toolchain:

```bash
rustc --version
cargo --version
```

## Workspace crates

```text
statelink-protocol
statelink-core
statelink-client
statelink-server
statelink-mqtt
```

See [Repository layout](repository-layout.md) before adding code so protocol, core semantics, transport, and adapters remain separated.

## Build

Debug workspace build:

```bash
cargo build --workspace --all-targets
```

Release build:

```bash
cargo build --release --workspace --all-targets
```

Build only the server:

```bash
cargo build -p statelink-server
```

## Unit tests

Run all workspace tests:

```bash
cargo test --workspace --all-targets
```

Run one crate:

```bash
cargo test -p statelink-core
```

Run one named test:

```bash
cargo test -p statelink-core policy_is_deny_by_default
```

Tests should be added alongside protocol/core behavior changes. New behavior should not rely only on end-to-end manual validation.

## Formatting

Check formatting:

```bash
cargo fmt --all -- --check
```

Apply formatting:

```bash
cargo fmt --all
```

## Clippy

CI treats warnings as errors:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not suppress lints globally merely to make CI green. Prefer fixing the underlying code or adding a narrow, justified allowance when genuinely required.

## One command locally

```bash
make check
```

This runs formatting checks, Clippy, and tests.

## Docker-only validation

If Rust is not installed locally:

Linux/macOS:

```bash
make docker-test
```

Windows PowerShell:

```powershell
.\scripts\docker-test.ps1
```

The source tree is mounted into a Rust 1.85 container and the same core checks are run there.

## Run the demo

Linux/macOS:

```bash
make demo
```

Windows PowerShell:

```powershell
.\scripts\demo.ps1
```

The demo definition lives at:

```text
demos/basic/docker-compose.yml
```

## Run components manually

Server:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

Consumer:

```bash
STATELINK_TOKEN=hmi-token \
cargo run -p statelink-client --example consumer
```

Producer:

```bash
STATELINK_TOKEN=camera-token \
cargo run -p statelink-client --example producer
```

## CI

GitHub Actions runs the Rust validation pipeline on pushes and pull requests.

The expected checks are:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo build --release --workspace --all-targets
```

A change is not considered ready if it requires formatting the working tree during CI. CI should verify committed source, not mutate it.

## Adding a protocol operation or field

Use this order:

1. Update `crates/statelink-protocol` types.
2. Update/extend protocol serialization tests.
3. Implement semantics in `crates/statelink-core`.
4. Add core unit tests for success, authorization, and failure paths.
5. Adapt `statelink-server` only where transport handling changes.
6. Extend `statelink-client` if a convenience API is needed.
7. Update examples.
8. Update `spec/STATE_LINK_V0.1.md` if the change is normative.
9. Update user documentation under `docs/`.

Do not start by adding protocol semantics directly inside the WebSocket handler.

## Adding a new transport

A new transport should translate authenticated connections/messages into the existing protocol/core model rather than duplicating state semantics.

Recommended shape:

```text
new transport
    |
    v
statelink-protocol Request/Response
    |
    v
statelink-core Bus
```

This is why `statelink-core` is intentionally transport-independent.

## Adding examples

Use one of two locations:

### Rust SDK examples

Put executable Cargo examples for the Rust client in:

```text
crates/statelink-client/examples/
```

Then add a discoverable link/description under:

```text
examples/rust/README.md
```

### Wire/language examples

Put protocol-level or other-language examples under:

```text
examples/
```

Do not hide user-facing examples only inside a crate without adding them to the top-level example index.

## Adding demos

A demo is a multi-component runnable scenario. Put it under:

```text
demos/<demo-name>/
```

Include a `README.md` with:

- purpose;
- prerequisites;
- exact run command;
- services/components started;
- expected output/behavior;
- cleanup command.

## Licensing/header rule

New source/config/script/documentation files should carry the project copyright and SPDX identifier appropriate for the file syntax.

Rust example:

```rust
// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later
```

YAML/shell example:

```text
# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later
```

Markdown example:

```html
<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
```

## Design rules worth preserving

When extending StateLink, preserve these properties unless a new protocol version intentionally changes them:

- current state rather than event history;
- authenticated single writer per context;
- deny-by-default system policy;
- producer exposure may restrict but not broaden system policy;
- secure discovery;
- future subscriptions;
- per-context revisions;
- bounded resources for slow consumers;
- transport-independent core semantics;
- clients tolerate revision gaps.
