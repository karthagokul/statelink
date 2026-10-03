# StateLink

A Lightweight Secure State-Exchange Protocol for Embedded, IoT, Native, and Web Applications.

StateLink is a state-oriented protocol for sharing current JSON context across processes, devices, containers, and browsers. It deliberately stays smaller than a general message bus: no RPC, no queues, no event history, and no transparent federation in v1.

> Status: early v0.1 reference implementation. The wire format and APIs may change before the first stable release.

## Core model

A producer owns one named context and publishes its latest state:

```text
system/health
camera/CAM01/status
camera/CAM01/config/desired
camera/CAM01/config/reported
```

StateLink reuses MQTT topic and wildcard semantics (`/`, `+`, `#`) and adds:

- authenticated single-writer ownership;
- producer-declared read exposure bounded by a deny-by-default system policy;
- security-filtered discovery;
- latest-state subscriptions with revision numbers;
- TTL-based freshness/staleness;
- optional immutable schema identifiers;
- desired/reported state synchronization with `applied_revision`;
- retained and session context lifecycles;
- subscriptions that may exist before a producer declares a matching context.

The WebSocket binding uses the `statelink.v1` subprotocol.

## Workspace

```text
crates/statelink-protocol   wire types, topic validation, MQTT-style filters
crates/statelink-core       transport-independent state bus and policy engine
crates/statelink-client     async Rust WebSocket client
crates/statelink-server     authenticated WebSocket reference server
config/                     development authentication and policy examples
```

## Run the Docker demo

The included demo intentionally starts the HMI consumer before the producer to demonstrate future subscriptions.

```bash
docker compose up --build --abort-on-container-exit
```

The demo server listens on `ws://localhost:8080/statelink` and exposes a health endpoint at `http://localhost:8080/healthz`.

The credentials under `config/dev-auth.json` are development-only examples. Do not deploy those tokens.

## Run tests

With Rust installed:

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

With Docker only:

```bash
make docker-test
```

On Windows PowerShell:

```powershell
./scripts/docker-test.ps1
```

## Minimal wire example

Declare a context:

```json
{
  "op": "declare",
  "id": 1,
  "topic": "system/health",
  "schema": "system-health/v1",
  "value": { "status": "healthy" },
  "access": { "read": ["role:hmi"] },
  "retention": "session",
  "ttl": 30
}
```

Subscribe before or after a producer exists:

```json
{
  "op": "subscribe",
  "id": 2,
  "filter": "system/#"
}
```

A state update includes a monotonically increasing per-context revision. Revision gaps are valid because StateLink guarantees latest-state convergence, not delivery of every intermediate transition.

## Security model

StateLink uses two independent authorization layers:

```text
producer exposure policy
          ∩
system allow-list policy
          ∩
authenticated consumer identity
          =
effective access
```

The reference server is deny-by-default. A producer cannot expose a topic outside the system policy ceiling.

The server currently supports bearer tokens for native clients and a `statelink_token` cookie for browser sessions. Browser `Origin` values are checked against `STATELINK_ALLOWED_ORIGINS`; Origin is treated only as an additional browser security control, never as identity.

For production deployments, terminate TLS so clients use `wss://`. Prefer short-lived tokens, mTLS, or an authenticated reverse proxy rather than static development tokens.

## Server configuration

Environment variables:

```text
STATELINK_BIND               default: 0.0.0.0:8080
STATELINK_AUTH_FILE          default: config/auth.json
STATELINK_POLICY_FILE        default: config/policy.json
STATELINK_ALLOWED_ORIGINS    comma-separated browser origins
STATELINK_OUTBOUND_QUEUE     default: 256
STATELINK_MAX_INBOUND_BYTES  default: 1048576
RUST_LOG                     tracing filter
```

See `config/dev-auth.json` and `config/dev-policy.json` for runnable development examples.

## License

Copyright (C) 2026 Gokul Kartha.

StateLink is licensed under the GNU General Public License, version 3 or any later version (`GPL-3.0-or-later`). Source files carry SPDX license identifiers and author copyright headers.
