<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# StateLink

A Lightweight Secure State-Exchange Protocol for Embedded, IoT, Native, and Web Applications.

StateLink synchronizes **current JSON state** across processes, devices, containers, and browsers. It is intentionally smaller than a general message bus: no RPC, no durable event queue, no event history, and no transparent federation in v0.1.

> **Status:** early v0.1 reference implementation. The wire format and APIs may change before the first stable release.

## Start here

If you are evaluating or using StateLink, these are the important entry points:

| What you need | Where to go |
| --- | --- |
| Run StateLink in a few minutes | [`docs/getting-started.md`](docs/getting-started.md) |
| Understand how StateLink works | [`docs/concepts.md`](docs/concepts.md) |
| Find the source code | [`docs/repository-layout.md`](docs/repository-layout.md) |
| Find runnable examples | [`examples/README.md`](examples/README.md) |
| Run complete demos | [`demos/README.md`](demos/README.md) |
| Use the Rust client | [`docs/rust-client.md`](docs/rust-client.md) |
| Implement another-language client | [`docs/wire-protocol.md`](docs/wire-protocol.md) |
| Configure the server | [`docs/server-configuration.md`](docs/server-configuration.md) |
| Configure security/policy | [`docs/security-and-policy.md`](docs/security-and-policy.md) |
| Export StateLink to MQTT | [`docs/mqtt-bridge.md`](docs/mqtt-bridge.md) |
| Build/test/contribute | [`docs/development.md`](docs/development.md) |
| Troubleshoot | [`docs/troubleshooting.md`](docs/troubleshooting.md) |
| Read the normative protocol | [`spec/STATE_LINK_V0.1.md`](spec/STATE_LINK_V0.1.md) |

Full documentation index: [`docs/README.md`](docs/README.md).

## Five-minute Docker demo

Prerequisite: Docker with Compose.

Linux/macOS:

```bash
make demo
```

Windows PowerShell:

```powershell
.\scripts\demo.ps1
```

Direct command:

```bash
docker compose -f demos/basic/docker-compose.yml up --build --abort-on-container-exit
```

The demo intentionally starts an HMI-like consumer **before** the producer. The consumer subscribes to `demo/#`; the producer later declares `demo/device/status` and publishes complete current-state updates.

Server endpoints during the demo:

```text
WebSocket:  ws://localhost:8080/statelink
Health:     http://localhost:8080/healthz
Subprotocol: statelink.v1
```

Development tokens in `config/dev-auth.json` are examples only and must not be used in production.

## Core model

A producer owns one named context and publishes its latest complete state:

```text
system/health
camera/CAM01/status
camera/CAM01/config/desired
camera/CAM01/config/reported
```

StateLink reuses MQTT topic/filter semantics (`/`, `+`, `#`) and adds:

- authenticated single-writer ownership;
- producer-declared read exposure bounded by a deny-by-default system policy;
- security-filtered discovery;
- latest-state subscriptions with per-context revision numbers;
- subscriptions that may exist before a producer/context appears;
- TTL-based freshness/staleness;
- session and retained context lifecycles;
- optional immutable schema identifiers;
- desired/reported synchronization with `applied_revision`.

The WebSocket binding uses the `statelink.v1` subprotocol.

## Repository map

```text
statelink/
├── crates/                 Rust implementation
│   ├── statelink-protocol  Wire types + topic/filter semantics
│   ├── statelink-core      State bus + policy + lifecycle
│   ├── statelink-client    Async Rust client SDK
│   ├── statelink-server    Reference WebSocket server
│   └── statelink-mqtt      StateLink -> MQTT adapter
├── docs/                   Detailed usage/development documentation
├── examples/               Discoverable example index + wire examples
├── demos/                  End-to-end runnable demos
├── config/                 Development auth/policy configuration
├── scripts/                Helper scripts
├── spec/                   Normative StateLink protocol specification
└── .github/workflows/      CI
```

Detailed explanation: [`docs/repository-layout.md`](docs/repository-layout.md).

The canonical Rust producer and consumer examples are kept in Cargo's conventional location:

```text
crates/statelink-client/examples/producer.rs
crates/statelink-client/examples/consumer.rs
```

They are indexed from [`examples/rust/README.md`](examples/rust/README.md) so they are easy to find.

## Run natively with Rust

The workspace currently targets Rust 1.85+.

Start the server:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

Start the consumer in another terminal:

```bash
STATELINK_TOKEN=hmi-token \
cargo run -p statelink-client --example consumer
```

Then start the producer:

```bash
STATELINK_TOKEN=camera-token \
cargo run -p statelink-client --example producer
```

Windows PowerShell commands are in [`examples/rust/README.md`](examples/rust/README.md).

## Minimal wire example

Declare a context:

```json
{
  "op": "declare",
  "id": 1,
  "topic": "system/health",
  "schema": "system-health/v1",
  "value": {"status": "healthy"},
  "access": {"read": ["role:hmi"]},
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

State updates contain monotonically increasing **per-context** revisions. Revision gaps are valid because StateLink guarantees latest-state convergence, not delivery of every intermediate transition.

More JSON examples: [`examples/wire/README.md`](examples/wire/README.md).

## Security model

Effective read access is the intersection of:

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

The server currently supports bearer tokens for native clients and a `statelink_token` cookie for browser sessions. Browser `Origin` values are checked against `STATELINK_ALLOWED_ORIGINS`; Origin is an additional browser control, not identity.

For production, use `wss://` and a production authentication strategy such as short-lived credentials, mTLS, or an authenticated reverse proxy rather than the static development tokens.

See [`docs/security-and-policy.md`](docs/security-and-policy.md).

## Configuration

Important environment variables:

```text
STATELINK_BIND               default: 0.0.0.0:8080
STATELINK_AUTH_FILE          default: config/auth.json
STATELINK_POLICY_FILE        default: config/policy.json
STATELINK_ALLOWED_ORIGINS    comma-separated browser origins
STATELINK_OUTBOUND_QUEUE     default: 256
STATELINK_MAX_INBOUND_BYTES  default: 1048576
RUST_LOG                     tracing filter
```

Runnable development files:

```text
config/dev-auth.json
config/dev-policy.json
```

Detailed configuration: [`docs/server-configuration.md`](docs/server-configuration.md).

## Tests and quality checks

With Rust installed:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo build --release --workspace --all-targets
```

Or:

```bash
make check
```

With Docker only:

```bash
make docker-test
```

Windows PowerShell:

```powershell
.\scripts\docker-test.ps1
```

CI runs formatting, Clippy with warnings denied, workspace tests, and release build validation.

## StateLink and MQTT

StateLink topic names map naturally to MQTT topic syntax, but a generic MQTT broker does not implement StateLink ownership, read exposure, secure discovery, freshness, revisions, or schema semantics.

The included `statelink-mqtt` adapter provides a one-way current-state export using retained MQTT publications.

Read [`docs/mqtt-bridge.md`](docs/mqtt-bridge.md).

## Specification vs implementation documentation

- [`spec/STATE_LINK_V0.1.md`](spec/STATE_LINK_V0.1.md) is the normative v0.1 interoperability specification.
- [`docs/`](docs/README.md) explains the current implementation and recommended usage.

When they disagree on protocol requirements, the normative specification takes precedence.

## License

Copyright (C) 2026 Gokul Kartha.

StateLink is licensed under the GNU General Public License, version 3 or any later version (`GPL-3.0-or-later`). Source files carry SPDX license identifiers and author copyright headers.
