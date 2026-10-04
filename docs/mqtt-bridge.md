<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# MQTT Bridge

`statelink-mqtt` is a one-way adapter that subscribes to StateLink and republishes authorized current state to MQTT retained topics.

It is useful when an existing MQTT ecosystem needs to consume StateLink-managed state without making MQTT itself responsible for StateLink ownership, authorization, freshness, or discovery semantics.

## Direction

Current implementation:

```text
StateLink -> MQTT
```

It does not currently import arbitrary MQTT messages back into StateLink.

## Mapping

For every authorized StateLink `UPDATE`:

- MQTT topic = StateLink topic;
- MQTT payload = serialized StateLink `value` only;
- QoS = at least once;
- retained flag = `true`.

Example StateLink state:

```json
{
  "topic": "camera/CAM01/status",
  "revision": 12,
  "freshness": "fresh",
  "value": {
    "online": true,
    "fps": 15
  }
}
```

becomes MQTT retained publication:

```text
Topic: camera/CAM01/status
Payload: {"online":true,"fps":15}
```

The MQTT payload does not include the StateLink revision/freshness metadata in the current data profile.

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `STATELINK_URL` | `ws://127.0.0.1:8080/statelink` | StateLink WebSocket endpoint |
| `STATELINK_TOKEN` | required | StateLink credential |
| `STATELINK_FILTER` | `#` | StateLink subscription filter |
| `MQTT_HOST` | `127.0.0.1` | MQTT broker host |
| `MQTT_PORT` | `1883` | MQTT broker port |
| `MQTT_CLIENT_ID` | `statelink-export` | MQTT client id |
| `MQTT_USERNAME` | unset | Optional MQTT username |
| `MQTT_PASSWORD` | empty when username set | Optional MQTT password |
| `RUST_LOG` | `statelink_mqtt=info` | Logging filter |

## Example

Assume:

- StateLink server is on localhost port 8080;
- MQTT broker is on localhost port 1883;
- the StateLink auth/policy configuration includes an identity allowed to read `camera/#`.

Linux/macOS:

```bash
STATELINK_URL=ws://127.0.0.1:8080/statelink \
STATELINK_TOKEN=cloud-export-token \
STATELINK_FILTER='camera/#' \
MQTT_HOST=127.0.0.1 \
MQTT_PORT=1883 \
cargo run -p statelink-mqtt
```

Windows PowerShell:

```powershell
$env:STATELINK_URL = "ws://127.0.0.1:8080/statelink"
$env:STATELINK_TOKEN = "cloud-export-token"
$env:STATELINK_FILTER = "camera/#"
$env:MQTT_HOST = "127.0.0.1"
$env:MQTT_PORT = "1883"
cargo run -p statelink-mqtt
```

## Authorization still applies

The bridge receives only StateLink contexts allowed by:

- its authenticated identity;
- the StateLink system policy;
- each producer's `access.read` exposure.

Using `STATELINK_FILTER=#` does not bypass StateLink authorization.

For least privilege, prefer a narrower filter such as:

```text
camera/+/status
```

instead of `#` whenever possible.

## Why MQTT retained publication?

StateLink is current-state oriented. MQTT retained messages are the closest native MQTT mechanism for making the latest value immediately available to a newly subscribed MQTT client.

This mapping preserves the **current value** behavior reasonably well, but not all StateLink semantics.

## Semantics MQTT does not provide automatically

A generic MQTT broker does not understand:

- StateLink single-writer context ownership;
- producer `access.read` exposure;
- StateLink system authorization;
- secure discovery;
- TTL freshness/staleness;
- StateLink revision semantics;
- immutable schema contracts;
- desired/reported `applied_revision` semantics.

Do not assume that publishing a StateLink value to MQTT transfers those controls to the broker.

## Operational notes

The adapter uses an MQTT keepalive of 30 seconds and a bounded client request capacity.

If the MQTT event loop fails, the current process logs the error; production deployments should supervise/restart the process as appropriate.

Likewise, the current adapter is a simple reference bridge rather than a fully managed high-availability gateway. Production hardening may need:

- reconnect/backoff policy;
- health/metrics endpoint;
- TLS MQTT connections;
- credential rotation;
- topic remapping/prefixing;
- metadata profile options;
- persistent buffering if the application requires it.

## Recommended architecture

Treat the bridge as an integration boundary:

```text
StateLink producers
       |
       v
StateLink server -- policy/ownership/current-state semantics
       |
       v
statelink-mqtt
       |
       v
MQTT broker -- legacy/external consumers
```

Keep authoritative StateLink state rules on the StateLink side rather than attempting to reproduce them independently in every MQTT consumer.
