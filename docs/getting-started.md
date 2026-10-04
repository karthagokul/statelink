<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Getting Started

This guide gets a complete StateLink producer/consumer flow running and then shows how to run the components separately.

## Prerequisites

Choose one of the following development paths:

- **Docker only:** Docker Engine / Docker Desktop with Compose support.
- **Native Rust:** Rust 1.85 or newer, Cargo, and a normal build toolchain.

The simplest first run is the Docker demo.

## 1. Clone the repository

```bash
git clone https://github.com/karthagokul/statelink.git
cd statelink
```

## 2. Run the end-to-end Docker demo

Linux/macOS:

```bash
make demo
```

Windows PowerShell:

```powershell
.\scripts\demo.ps1
```

Or invoke Compose directly:

```bash
docker compose -f demos/basic/docker-compose.yml up --build --abort-on-container-exit
```

The demo runs three containers:

1. `statelink` — the reference server;
2. `consumer` — an HMI-like consumer that subscribes to `demo/#` before a producer exists;
3. `producer` — a device-like producer that later declares `demo/device/status` and updates it.

The order is deliberate: it demonstrates that a subscription can exist before matching contexts are declared.

The server endpoints are:

```text
WebSocket: ws://localhost:8080/statelink
Health:    http://localhost:8080/healthz
Subprotocol: statelink.v1
```

Stop the demo with `Ctrl+C`. To remove created containers/networks explicitly:

```bash
docker compose -f demos/basic/docker-compose.yml down
```

## 3. Validate the project with Docker only

Linux/macOS:

```bash
make docker-test
```

Windows PowerShell:

```powershell
.\scripts\docker-test.ps1
```

This runs formatting checks, Clippy with warnings treated as errors, and the complete workspace test suite inside a Rust container.

## 4. Run natively with Rust

### Start the server

Linux/macOS:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
RUST_LOG=statelink_server=info \
cargo run -p statelink-server
```

Windows PowerShell:

```powershell
$env:STATELINK_AUTH_FILE = "config/dev-auth.json"
$env:STATELINK_POLICY_FILE = "config/dev-policy.json"
$env:RUST_LOG = "statelink_server=info"
cargo run -p statelink-server
```

The development configuration contains these example identities:

```text
camera-token -> id demo-producer, role producer
hmi-token    -> id demo-hmi,      role hmi
```

These are development-only credentials.

### Start the consumer

In another terminal:

Linux/macOS:

```bash
STATELINK_TOKEN=hmi-token \
cargo run -p statelink-client --example consumer
```

Windows PowerShell:

```powershell
$env:STATELINK_TOKEN = "hmi-token"
cargo run -p statelink-client --example consumer
```

The consumer subscribes to:

```text
demo/#
```

It can be started before the producer.

### Start the producer

In a third terminal:

Linux/macOS:

```bash
STATELINK_TOKEN=camera-token \
cargo run -p statelink-client --example producer
```

Windows PowerShell:

```powershell
$env:STATELINK_TOKEN = "camera-token"
cargo run -p statelink-client --example producer
```

The producer declares:

```text
demo/device/status
```

and then publishes several complete state replacements. The consumer should print `UPDATE` responses as revisions change.

## 5. Understand what just happened

The producer did not send events. It created a **context** representing current state:

```json
{
  "online": true,
  "counter": 3
}
```

Each successful update receives a newer per-context revision. A consumer only needs the latest complete state. Missing an intermediate revision is valid and does not imply that the protocol must retransmit history.

The consumer subscribed using an MQTT-style filter (`demo/#`). Authorization was evaluated using both:

- the server's system policy in `config/dev-policy.json`; and
- the producer's `access.read` declaration.

Both layers must permit the consumer.

## 6. Your first application topic

A typical device might expose:

```text
camera/CAM01/status
camera/CAM01/health
camera/CAM01/config/desired
camera/CAM01/config/reported
```

Use topics for **current state**, not commands or event streams. For command-like configuration, prefer desired/reported state. See [Core concepts](concepts.md#desired-and-reported-state).

## 7. Where to go next

- Runnable examples: [`../examples/README.md`](../examples/README.md)
- Rust SDK usage: [Rust client guide](rust-client.md)
- Non-Rust clients: [Wire protocol guide](wire-protocol.md)
- Server deployment/configuration: [Server configuration](server-configuration.md)
- Authorization: [Security and policy](security-and-policy.md)
- Common errors: [Troubleshooting](troubleshooting.md)
