<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# statelink-mqtt

One-way StateLink-to-MQTT current-state export adapter.

## Flow

```text
StateLink SUBSCRIBE
      |
      v
Response::Update
      |
      v
state.value serialized as JSON
      |
      v
MQTT publish to same topic
QoS 1 + retained=true
```

The adapter does not make MQTT a replacement for StateLink ownership, security, discovery, freshness, or revision semantics.

## Required configuration

`STATELINK_TOKEN` must be provided.

Common environment variables:

```text
STATELINK_URL
STATELINK_TOKEN
STATELINK_FILTER
MQTT_HOST
MQTT_PORT
MQTT_CLIENT_ID
MQTT_USERNAME
MQTT_PASSWORD
RUST_LOG
```

## Run

```bash
STATELINK_TOKEN=cloud-export-token \
STATELINK_FILTER='camera/#' \
MQTT_HOST=127.0.0.1 \
cargo run -p statelink-mqtt
```

## Documentation

See [`../../docs/mqtt-bridge.md`](../../docs/mqtt-bridge.md) for configuration, mapping semantics, limitations, and production notes.
