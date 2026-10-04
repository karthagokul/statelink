<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# statelink-client

Minimal async Rust client SDK for the StateLink WebSocket binding.

## Public client flow

```rust
let mut client = statelink_client::Client::connect(url, token).await?;
client.send(&request).await?;
let response = client.next_response().await?;
```

The client automatically:

- sends bearer authentication;
- requests WebSocket subprotocol `statelink.v1`;
- serializes/deserializes StateLink JSON messages;
- replies to WebSocket ping frames with pong frames.

Application-level reconnect/subscription restoration is intentionally outside the minimal v0.1 API.

## Runnable examples

```text
examples/producer.rs
examples/consumer.rs
```

Run:

```bash
cargo run -p statelink-client --example consumer
cargo run -p statelink-client --example producer
```

The top-level discoverable guide is [`../../examples/rust/README.md`](../../examples/rust/README.md).

## Tests/build

```bash
cargo test -p statelink-client --all-targets
cargo build -p statelink-client --examples
```

## Documentation

- [`../../docs/rust-client.md`](../../docs/rust-client.md)
- [`../../docs/getting-started.md`](../../docs/getting-started.md)
