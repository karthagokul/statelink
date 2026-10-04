<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# statelink-core

Transport-independent StateLink state engine and authorization policy.

## Responsibilities

This crate owns the semantics of:

- authenticated context ownership;
- `DECLARE`, `SET`, `GET`, `DISCOVER`, `SUBSCRIBE`, `UNSUBSCRIBE`, `REMOVE`;
- per-context revisions;
- session and retained lifecycle;
- TTL freshness/staleness;
- subscription delivery generation;
- security-filtered discovery;
- deny-by-default system policy;
- producer exposure checks.

It should remain independent from Axum/WebSocket framing so future transports can reuse the same behavior.

## Important files

```text
src/bus.rs       state bus, contexts, subscriptions, lifecycle
src/policy.rs    system policy rules and subject selectors
src/identity.rs  authenticated identity model
src/lib.rs       public exports
```

## Tests

```bash
cargo test -p statelink-core
```

Core semantics should be covered here with unit tests before/alongside transport-level integration tests.

## Documentation

- [`../../docs/architecture.md`](../../docs/architecture.md)
- [`../../docs/concepts.md`](../../docs/concepts.md)
- [`../../docs/security-and-policy.md`](../../docs/security-and-policy.md)
