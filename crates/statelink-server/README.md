<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# statelink-server

Reference HTTP/WebSocket server for StateLink v0.1.

## Responsibilities

This crate handles transport/integration concerns:

- HTTP listener;
- `/healthz` health endpoint;
- `/statelink` WebSocket endpoint;
- required `statelink.v1` subprotocol;
- bearer-token / browser-cookie authentication adapter;
- browser Origin allow-list;
- authenticated session creation;
- WebSocket JSON framing;
- bounded outbound queues;
- dispatching `statelink-core` deliveries to connected peers.

Protocol/state semantics belong in `statelink-protocol` / `statelink-core` whenever possible.

## Important files

```text
src/main.rs  server, connection loop, dispatch, maintenance
src/auth.rs  token authentication and Origin policy
```

## Run locally

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

Default endpoints:

```text
ws://127.0.0.1:8080/statelink
http://127.0.0.1:8080/healthz
```

## Documentation

- [`../../docs/server-configuration.md`](../../docs/server-configuration.md)
- [`../../docs/security-and-policy.md`](../../docs/security-and-policy.md)
- [`../../docs/architecture.md`](../../docs/architecture.md)
