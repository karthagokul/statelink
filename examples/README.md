<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Examples

This directory is the discoverable entry point for StateLink examples.

## Rust examples

The canonical executable Rust examples live inside the client crate because that is Cargo's standard layout:

```text
crates/statelink-client/examples/producer.rs
crates/statelink-client/examples/consumer.rs
```

See [`rust/README.md`](rust/README.md) for exact Linux/macOS and Windows PowerShell commands.

Run them directly with:

```bash
cargo run -p statelink-client --example consumer
cargo run -p statelink-client --example producer
```

## Wire-protocol examples

If you are implementing a client in C, C++, Python, JavaScript, Go, Java, C#, or another language, start with:

[`wire/README.md`](wire/README.md)

It contains copy/paste JSON examples for:

- `DECLARE`
- `SET`
- `GET`
- `DISCOVER`
- `SUBSCRIBE`
- `UPDATE`
- `UNSUBSCRIBE`
- `REMOVE`
- desired/reported state
- protocol errors

## Demos are separate

Examples show API usage. Complete multi-component demonstrations are under:

[`../demos/README.md`](../demos/README.md)

The basic Docker demo starts a server, consumer, and producer together.

## Documentation

For explanations rather than runnable snippets, use:

- [`../docs/getting-started.md`](../docs/getting-started.md)
- [`../docs/rust-client.md`](../docs/rust-client.md)
- [`../docs/wire-protocol.md`](../docs/wire-protocol.md)
- [`../docs/concepts.md`](../docs/concepts.md)
