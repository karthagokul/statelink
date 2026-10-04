<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Server Configuration

The reference server is the `statelink-server` binary in `crates/statelink-server`.

## Endpoints

The server exposes two HTTP routes:

```text
GET /healthz    health check; returns HTTP 204 when the process is serving
GET /statelink  WebSocket upgrade endpoint
```

The WebSocket client must request subprotocol:

```text
statelink.v1
```

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `STATELINK_BIND` | `0.0.0.0:8080` | TCP bind address |
| `STATELINK_AUTH_FILE` | `config/auth.json` | JSON token-to-identity map |
| `STATELINK_POLICY_FILE` | `config/policy.json` | JSON system allow-list policy |
| `STATELINK_ALLOWED_ORIGINS` | empty | Comma-separated browser Origin allow-list |
| `STATELINK_OUTBOUND_QUEUE` | `256` | Bounded outbound queue per connected client |
| `STATELINK_MAX_INBOUND_BYTES` | `1048576` | Maximum inbound WebSocket text message size |
| `RUST_LOG` | `statelink_server=info` | Tracing/log filter |

## Development startup

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

## Authentication file

Example:

```json
{
  "tokens": {
    "camera-token": {
      "id": "demo-producer",
      "roles": ["producer"]
    },
    "hmi-token": {
      "id": "demo-hmi",
      "roles": ["hmi"]
    }
  }
}
```

The JSON key is the bearer/cookie token. The value defines the authenticated identity and zero or more roles.

This file is loaded when the server starts. The current reference implementation does not dynamically reload it.

The repository's `config/dev-auth.json` is for local development only. Static demo tokens must not be copied into a real deployment.

## Policy file

Example:

```json
{
  "rules": [
    {
      "subjects": ["id:demo-producer"],
      "operations": ["declare", "read"],
      "topic_filter": "demo/#"
    },
    {
      "subjects": ["role:hmi"],
      "operations": ["read"],
      "topic_filter": "demo/#"
    }
  ]
}
```

Supported operations in the current system policy are:

```text
declare
read
```

The policy is deny-by-default. No matching rule means the action is denied.

See [Security and policy](security-and-policy.md) for selector semantics and the interaction with producer `access.read`.

## Browser Origin policy

Native clients usually do not send an `Origin` header and are not blocked by the Origin allow-list.

Browsers do send `Origin`. If the request contains an Origin, it must match one of the values in `STATELINK_ALLOWED_ORIGINS`.

Example:

```bash
STATELINK_ALLOWED_ORIGINS=https://hmi.example.com,https://admin.example.com
```

Origin is only an additional browser security control. It is not identity and does not replace authentication or topic authorization.

## Message-size limit

The reference server rejects oversized inbound WebSocket text messages with:

```text
PAYLOAD_TOO_LARGE
```

Default:

```text
1 MiB (1048576 bytes)
```

Choose a lower limit for tightly constrained devices and a larger one only when your state model genuinely needs it.

StateLink contexts should remain reasonably small; the protocol is intended for current application/device state rather than large media objects.

## Outbound queue

Each connection has a bounded outbound queue. Default:

```text
256 messages
```

This protects the server from unbounded memory growth when a consumer is slow.

Because StateLink is state-oriented, asynchronous `UPDATE` messages may be dropped for a slow consumer. The consumer must accept revision gaps and converge on a later complete state.

Direct protocol responses are also queued. A persistently slow or blocked client may be disconnected.

## Maintenance loop

The reference server runs periodic maintenance approximately every 500 ms. Maintenance drives lifecycle behavior such as freshness/staleness changes and related deliveries.

Applications should not depend on TTL changes occurring at the exact millisecond of expiry; TTL represents freshness semantics, not a real-time scheduling guarantee.

## Logging

The server uses Rust `tracing`.

Examples:

```bash
RUST_LOG=statelink_server=debug cargo run -p statelink-server
```

or:

```bash
RUST_LOG=info,statelink_server=trace cargo run -p statelink-server
```

Avoid logging bearer tokens or sensitive state values in production integrations.

## Docker

Build the repository image:

```bash
docker build -t statelink:dev .
```

The image contains:

```text
/usr/local/bin/statelink-server
/usr/local/bin/statelink-mqtt
/usr/local/bin/statelink-producer
/usr/local/bin/statelink-consumer
```

The default entrypoint is `statelink-server`.

The end-to-end Compose configuration is under:

```text
demos/basic/docker-compose.yml
```

Run it with:

```bash
make demo
```

## Production deployment checklist

Before using the reference server outside local development:

1. Put it behind TLS and expose `wss://`, not plain `ws://` across untrusted networks.
2. Replace demo tokens with an appropriate authentication strategy.
3. Use short-lived credentials, mTLS, or an authenticated reverse proxy where practical.
4. Keep system policy deny-by-default and narrowly scoped.
5. Configure explicit browser Origins if browsers connect directly.
6. Protect auth and policy files with filesystem permissions.
7. Choose inbound-size and queue limits appropriate for the target device/server.
8. Run the process with minimal OS/container privileges.
9. Monitor health, logs, reconnect rates, policy failures, and slow-consumer warnings.
10. Treat retained state as application data that may need durability beyond the in-memory reference implementation if long-term persistence is required.

## Current persistence limitation

The current reference server is primarily an in-memory implementation. `retained` means the value survives producer disconnect inside the running server process; it should not be interpreted as guaranteed durable storage across process/container restart unless an external persistence layer is added.

Design applications accordingly when durable reboot persistence is required.
