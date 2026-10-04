<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Rust Examples

The executable Rust examples are intentionally kept in Cargo's conventional location:

```text
crates/statelink-client/examples/consumer.rs
crates/statelink-client/examples/producer.rs
```

This page exists so users can find and run them without knowing the crate layout first.

## Run the server

Linux/macOS:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

Windows PowerShell:

```powershell
$env:STATELINK_AUTH_FILE = "config/dev-auth.json"
$env:STATELINK_POLICY_FILE = "config/dev-policy.json"
cargo run -p statelink-server
```

## Run the consumer

Start this before the producer to demonstrate future subscriptions.

Linux/macOS:

```bash
STATELINK_URL=ws://127.0.0.1:8080/statelink \
STATELINK_TOKEN=hmi-token \
cargo run -p statelink-client --example consumer
```

Windows PowerShell:

```powershell
$env:STATELINK_URL = "ws://127.0.0.1:8080/statelink"
$env:STATELINK_TOKEN = "hmi-token"
cargo run -p statelink-client --example consumer
```

The consumer subscribes to:

```text
demo/#
```

## Run the producer

Linux/macOS:

```bash
STATELINK_URL=ws://127.0.0.1:8080/statelink \
STATELINK_TOKEN=camera-token \
cargo run -p statelink-client --example producer
```

Windows PowerShell:

```powershell
$env:STATELINK_URL = "ws://127.0.0.1:8080/statelink"
$env:STATELINK_TOKEN = "camera-token"
cargo run -p statelink-client --example producer
```

The producer declares:

```text
demo/device/status
```

with:

- schema `demo-status/v1`;
- `session` retention;
- TTL 5 seconds;
- `role:hmi` read exposure.

It then publishes a sequence of complete state replacements.

## Expected consumer output

You should see an initial subscription acknowledgment followed by `Update` responses similar to:

```text
Update { subscription: ..., state: ContextState { topic: "demo/device/status", revision: ..., ... } }
```

Exact debug formatting and revision values can change; the important behavior is that the consumer was already subscribed before the producer declared the context.

## Source links

Producer:

```text
crates/statelink-client/examples/producer.rs
```

Consumer:

```text
crates/statelink-client/examples/consumer.rs
```

For building a real application from these patterns, read [`../../docs/rust-client.md`](../../docs/rust-client.md).
