<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Troubleshooting

This guide covers common problems when running or integrating StateLink.

## Server will not start

### Missing auth or policy file

The defaults are:

```text
config/auth.json
config/policy.json
```

The repository ships development files named:

```text
config/dev-auth.json
config/dev-policy.json
```

So for local development set them explicitly:

Linux/macOS:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

PowerShell:

```powershell
$env:STATELINK_AUTH_FILE = "config/dev-auth.json"
$env:STATELINK_POLICY_FILE = "config/dev-policy.json"
cargo run -p statelink-server
```

### Port already in use

Default bind:

```text
0.0.0.0:8080
```

Choose another port:

```bash
STATELINK_BIND=127.0.0.1:8090 cargo run -p statelink-server
```

Then point clients to:

```text
ws://127.0.0.1:8090/statelink
```

## WebSocket connection returns HTTP 400

The server requires the WebSocket subprotocol:

```text
statelink.v1
```

A generic WebSocket client that does not request this subprotocol will be rejected.

The Rust `statelink-client` configures it automatically.

## WebSocket connection returns HTTP 401

Authentication is missing or invalid.

Native client header:

```text
Authorization: Bearer <token>
```

Browser cookie option:

```text
statelink_token=<token>
```

For the repository demo:

```text
camera-token
hmi-token
```

are defined in `config/dev-auth.json`.

## Browser connection returns HTTP 403

If the browser sends an `Origin` header, that exact Origin must be included in `STATELINK_ALLOWED_ORIGINS`.

Example:

```text
STATELINK_ALLOWED_ORIGINS=http://localhost:3000,https://hmi.example.com
```

Origin matching is exact in the reference implementation.

## `FORBIDDEN`

The authenticated identity does not have effective permission for the requested operation/topic.

Check both:

1. system policy (`config/...policy.json`);
2. producer `access.read` for reads/subscriptions.

Remember that producer exposure cannot override a system-policy denial.

## `ALREADY_EXISTS`

A context with that topic already exists.

StateLink uses single-writer ownership. A second producer cannot redeclare another active context to take it over.

Choose a unique context topic or explicitly coordinate lifecycle/removal.

## `NOT_OWNER`

The current authenticated session tried to `SET` or `REMOVE` a context it does not own.

Common causes:

- a different process declared the topic;
- the producer reconnected and assumed old session ownership;
- multiple replicas are trying to publish the same single-writer context.

On reconnect, re-establish ownership according to the application's lifecycle design.

## `NOT_FOUND`

The requested context does not currently exist or is not available through the requested operation.

For consumers that need to wait for future producers, use `SUBSCRIBE` rather than repeatedly polling `GET`.

## `INVALID_TOPIC`

Check topic/filter syntax.

Rules include:

- context names cannot contain `+` or `#`;
- `+` is allowed only in filters and matches one level;
- `#` is allowed only in filters and must be the final filter level.

Examples:

```text
valid topic:   camera/CAM01/status
valid filter:  camera/+/status
valid filter:  camera/#
invalid topic: camera/+/status
invalid filter: camera/#/status
```

## `SCHEMA_IMMUTABLE`

A client attempted to change the schema identifier of an existing context.

Schema is fixed for that context lifecycle. To change it:

1. `REMOVE` the context;
2. `DECLARE` it again with the new schema identifier.

## `PAYLOAD_TOO_LARGE`

The inbound WebSocket text frame exceeded:

```text
STATELINK_MAX_INBOUND_BYTES
```

Default:

```text
1048576 bytes
```

Prefer reducing context size before increasing the server limit. StateLink is intended for current JSON state, not large binary/media payloads.

## Subscription succeeds but I receive nothing

Check these possibilities:

- no matching context has been declared yet — this is valid;
- the producer's `access.read` does not include your identity/role;
- your system policy allows the subscription filter but not the actual context read;
- the producer is not updating/declaring the topic you expect;
- the filter does not match the topic;
- you connected to a different server instance.

Use `DISCOVER` with an authorized narrow filter to inspect visible contexts.

## I see revision gaps

This is expected and valid.

StateLink synchronizes latest complete state and may coalesce/drop intermediate updates for slow consumers. A consumer must not require every revision.

If every transition must be processed, use a dedicated event/queue system alongside StateLink.

## A retained context disappeared after server restart

In the current reference implementation, retained contexts survive producer disconnect but are still held in server memory. They are not yet durable across process/container restart.

If restart durability is required, add/use a persistence layer rather than treating `retained` as disk persistence.

## Context becomes stale

A context with TTL becomes stale when it is not refreshed within its freshness lifetime.

This does not necessarily remove it. Freshness and retention are independent.

Check:

- producer update interval;
- TTL value;
- producer connectivity;
- whether the state really should use a TTL.

## Docker demo does not build

Run from the repository root:

```bash
docker compose -f demos/basic/docker-compose.yml build --no-cache
```

Then:

```bash
docker compose -f demos/basic/docker-compose.yml up --abort-on-container-exit
```

On Windows, make sure Docker Desktop is running and Linux containers are available.

## Docker test works but native Cargo fails

Compare Rust versions:

```bash
rustc --version
```

The workspace currently targets Rust 1.85+.

Also ensure required components are installed:

```bash
rustup component add rustfmt clippy
```

## Clippy fails CI

CI runs:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Warnings are errors. Fix warnings locally before pushing.

## Format check fails

Run:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

CI should verify formatting, not rewrite source.

## MQTT bridge sees no updates

Check:

- `STATELINK_TOKEN` is set;
- its identity has read permission;
- `STATELINK_FILTER` matches the contexts;
- producer `access.read` permits the bridge identity;
- MQTT broker address/port are reachable;
- the MQTT event loop has not logged a connection error.

Try a narrow StateLink filter first rather than `#`.

## Increase logging

Server:

```bash
RUST_LOG=statelink_server=debug cargo run -p statelink-server
```

MQTT bridge:

```bash
RUST_LOG=statelink_mqtt=debug cargo run -p statelink-mqtt
```

Avoid enabling verbose logs containing sensitive application data in production.

## Still stuck?

Collect:

- exact command used;
- server log around connection/request time;
- client error code/message;
- sanitized auth identity/roles (not the secret token);
- relevant policy rule;
- topic/filter involved;
- Rust/Docker versions;
- whether the issue reproduces with `make demo` or `make docker-test`.

That information usually isolates whether the problem is transport, authentication, authorization, topic matching, ownership, or build tooling.
