<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Wire Examples

These examples show StateLink v0.1 JSON messages independently of any client library.

Connection requirements:

```text
WebSocket URL: ws://127.0.0.1:8080/statelink
Subprotocol:    statelink.v1
Authentication: Authorization: Bearer <token>
```

Each JSON object is sent as one WebSocket text frame.

## Declare a context

```json
{
  "op": "declare",
  "id": 1,
  "topic": "camera/CAM01/status",
  "schema": "camera-status/v1",
  "value": {
    "online": true,
    "fps": 15
  },
  "access": {
    "read": ["role:hmi"]
  },
  "retention": "session",
  "ttl": 30
}
```

Response:

```json
{
  "op": "ok",
  "id": 1
}
```

## Set current state

```json
{
  "op": "set",
  "id": 2,
  "topic": "camera/CAM01/status",
  "value": {
    "online": true,
    "fps": 20
  }
}
```

Response:

```json
{
  "op": "ok",
  "id": 2
}
```

## Get one context

```json
{
  "op": "get",
  "id": 3,
  "topic": "camera/CAM01/status"
}
```

Example response:

```json
{
  "op": "state",
  "id": 3,
  "topic": "camera/CAM01/status",
  "revision": 2,
  "timestamp_ms": 1791120000000,
  "freshness": "fresh",
  "schema": "camera-status/v1",
  "value": {
    "online": true,
    "fps": 20
  }
}
```

## Discover readable contexts

```json
{
  "op": "discover",
  "id": 4,
  "filter": "camera/#"
}
```

Example response:

```json
{
  "op": "discovery",
  "id": 4,
  "contexts": [
    {
      "topic": "camera/CAM01/status",
      "revision": 2,
      "freshness": "fresh",
      "schema": "camera-status/v1"
    }
  ]
}
```

## Subscribe

```json
{
  "op": "subscribe",
  "id": 5,
  "filter": "camera/+/status"
}
```

Response:

```json
{
  "op": "ok",
  "id": 5,
  "subscription": 12
}
```

Later asynchronous update:

```json
{
  "op": "update",
  "subscription": 12,
  "topic": "camera/CAM01/status",
  "revision": 3,
  "timestamp_ms": 1791120001000,
  "freshness": "fresh",
  "schema": "camera-status/v1",
  "value": {
    "online": true,
    "fps": 25
  }
}
```

## Unsubscribe

```json
{
  "op": "unsubscribe",
  "id": 6,
  "subscription": 12
}
```

Response:

```json
{
  "op": "ok",
  "id": 6
}
```

## Remove a context

```json
{
  "op": "remove",
  "id": 7,
  "topic": "camera/CAM01/status"
}
```

Only the authenticated owner can remove it.

## Desired configuration

Controller declares or updates a retained desired context:

```json
{
  "op": "declare",
  "id": 20,
  "topic": "camera/CAM01/config/desired",
  "schema": "camera-config/v1",
  "value": {
    "fps": 15,
    "recording": true
  },
  "access": {
    "read": ["id:camera-CAM01"]
  },
  "retention": "retained"
}
```

Assume the server assigns desired revision 27.

## Report applied configuration

The device can report exactly which desired revision it applied:

```json
{
  "op": "declare",
  "id": 21,
  "topic": "camera/CAM01/config/reported",
  "schema": "camera-config/v1",
  "value": {
    "fps": 15,
    "recording": true
  },
  "access": {
    "read": ["role:hmi", "role:controller"]
  },
  "retention": "session",
  "applied_revision": 27
}
```

Subsequent report:

```json
{
  "op": "set",
  "id": 22,
  "topic": "camera/CAM01/config/reported",
  "value": {
    "fps": 20,
    "recording": true
  },
  "applied_revision": 28
}
```

## Error

Example authorization failure:

```json
{
  "op": "error",
  "id": 99,
  "code": "FORBIDDEN",
  "message": "read access is not allowed"
}
```

Use the machine-readable `code`; do not parse the message text.

## Remember

- Every update is complete current state, not an event or patch.
- Revision gaps are valid.
- `+` and `#` are allowed in filters, not context names.
- Authentication determines identity and ownership.
- System policy and producer exposure both apply.
- A subscription can be created before a matching producer/context exists.

For the full explanation, see [`../../docs/wire-protocol.md`](../../docs/wire-protocol.md).
