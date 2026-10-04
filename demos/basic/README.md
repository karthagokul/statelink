<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Basic Docker Demo

This demo shows the smallest complete StateLink system: one server, one consumer, and one producer.

## Scenario

```text
consumer starts
    |
    | SUBSCRIBE demo/#
    v
StateLink server
    ^
    |
producer starts later
    |
    | DECLARE demo/device/status
    | SET revision 2
    | SET revision 3
    | ...
```

The consumer deliberately starts before the producer. This verifies that StateLink subscriptions are allowed to exist before matching contexts are declared.

## Services

### `statelink`

Runs `/usr/local/bin/statelink-server` and mounts:

```text
config/dev-auth.json
config/dev-policy.json
```

The server is exposed on host port 8080.

### `consumer`

Runs the Rust `consumer` example with:

```text
STATELINK_TOKEN=hmi-token
STATELINK_URL=ws://statelink:8080/statelink
```

It subscribes to:

```text
demo/#
```

### `producer`

Runs the Rust `producer` example with:

```text
STATELINK_TOKEN=camera-token
STATELINK_URL=ws://statelink:8080/statelink
```

It declares:

```text
demo/device/status
```

and updates its counter several times.

## Run

From the repository root:

```bash
make demo
```

PowerShell:

```powershell
.\scripts\demo.ps1
```

Direct Compose command:

```bash
docker compose -f demos/basic/docker-compose.yml up --build --abort-on-container-exit
```

## Expected behavior

You should see:

1. StateLink server starts and listens on port 8080.
2. Consumer connects first and creates its subscription.
3. Producer connects later using a different identity/token.
4. Producer declares `demo/device/status`.
5. Consumer receives current-state `UPDATE` messages.
6. Producer publishes newer complete states/revisions.

The consumer does not need to be restarted when the producer appears.

## Verify health

While the server is running:

```bash
curl -i http://localhost:8080/healthz
```

Expected HTTP status:

```text
204 No Content
```

## Clean up

```bash
docker compose -f demos/basic/docker-compose.yml down
```

## Development credentials

This demo uses static credentials from `config/dev-auth.json`:

```text
camera-token
hmi-token
```

They are intentionally simple local-development values and must not be used as production credentials.

## What to read next

- [`../../docs/getting-started.md`](../../docs/getting-started.md)
- [`../../docs/concepts.md`](../../docs/concepts.md)
- [`../../examples/rust/README.md`](../../examples/rust/README.md)
- [`../../docs/security-and-policy.md`](../../docs/security-and-policy.md)
