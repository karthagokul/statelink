<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Wire Protocol Guide

This guide explains how to use StateLink directly over WebSocket without the Rust client library.

For normative interoperability requirements, read [`../spec/STATE_LINK_V0.1.md`](../spec/STATE_LINK_V0.1.md).

## Connection

The reference server exposes:

```text
GET /statelink
```

as a WebSocket endpoint.

The client **must** request the WebSocket subprotocol:

```text
statelink.v1
```

Each WebSocket text frame contains exactly one JSON StateLink message. Binary frames are not part of StateLink v0.1.

## Authentication

Native clients normally authenticate with:

```http
Authorization: Bearer <token>
```

Browser sessions may use a cookie named:

```text
statelink_token
```

Browser connections that send an `Origin` header must match the server's configured Origin allow-list.

Authentication establishes the identity used for ownership and authorization. A client cannot claim a different identity in the JSON message.

## Request IDs

Every request contains an integer `id` chosen by the client:

```json
{
  "op": "get",
  "id": 42,
  "topic": "system/health"
}
```

Normal request responses include the same `id`, allowing a client to correlate responses with outstanding requests.

Asynchronous subscription updates use a `subscription` identifier rather than a request `id`.

## DECLARE

Creates a new context and makes the authenticated session its owner.

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
    "read": ["role:hmi", "role:cloud"]
  },
  "retention": "session",
  "ttl": 30
}
```

Optional fields:

- `schema`
- `access` (defaults to no additional consumer exposure)
- `retention` (defaults to `session`)
- `ttl`
- `applied_revision`

Successful response:

```json
{
  "op": "ok",
  "id": 1
}
```

The server assigns the first context revision internally.

## SET

Replaces the complete value of a context owned by the same authenticated session.

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

Do not treat `SET` as a JSON merge or patch. The `value` is the complete current application state for that context.

A producer may also send `applied_revision` for a reported-state context:

```json
{
  "op": "set",
  "id": 3,
  "topic": "camera/CAM01/config/reported",
  "value": {
    "fps": 15
  },
  "applied_revision": 27
}
```

A schema supplied on `SET` is only accepted if it does not attempt to mutate the existing immutable schema contract; clients should normally omit `schema` after declaration.

## GET

Fetches one exact context if the authenticated caller is authorized to read it.

Request:

```json
{
  "op": "get",
  "id": 10,
  "topic": "camera/CAM01/status"
}
```

Example response:

```json
{
  "op": "state",
  "id": 10,
  "topic": "camera/CAM01/status",
  "revision": 8,
  "timestamp_ms": 1791120000000,
  "freshness": "fresh",
  "schema": "camera-status/v1",
  "value": {
    "online": true,
    "fps": 20
  }
}
```

## DISCOVER

Returns metadata for readable contexts matching a topic filter.

Request:

```json
{
  "op": "discover",
  "id": 20,
  "filter": "camera/+/status"
}
```

Example response:

```json
{
  "op": "discovery",
  "id": 20,
  "contexts": [
    {
      "topic": "camera/CAM01/status",
      "revision": 8,
      "freshness": "fresh",
      "schema": "camera-status/v1"
    }
  ]
}
```

Discovery is authorization-filtered. Unauthorized topic names are not returned.

## SUBSCRIBE

Creates a subscription to current and future matching contexts.

Request:

```json
{
  "op": "subscribe",
  "id": 30,
  "filter": "camera/#"
}
```

Successful response:

```json
{
  "op": "ok",
  "id": 30,
  "subscription": 5
}
```

The `subscription` value is server-assigned and is later used in updates and `UNSUBSCRIBE`.

A subscription may be created even when no matching context currently exists.

As matching state becomes available or changes, the server sends asynchronous updates:

```json
{
  "op": "update",
  "subscription": 5,
  "topic": "camera/CAM01/status",
  "revision": 9,
  "timestamp_ms": 1791120000500,
  "freshness": "fresh",
  "schema": "camera-status/v1",
  "value": {
    "online": true,
    "fps": 25
  }
}
```

Consumers must tolerate revision gaps. Each `UPDATE` is complete current state.

## UNSUBSCRIBE

Stops a previously created subscription.

```json
{
  "op": "unsubscribe",
  "id": 31,
  "subscription": 5
}
```

Successful response:

```json
{
  "op": "ok",
  "id": 31
}
```

## REMOVE

Removes a context owned by the authenticated session.

```json
{
  "op": "remove",
  "id": 40,
  "topic": "camera/CAM01/status"
}
```

Only the owner can remove a context.

## Error response

Errors are machine-readable:

```json
{
  "op": "error",
  "id": 42,
  "code": "FORBIDDEN",
  "message": "read access is not allowed"
}
```

Possible v0.1 codes:

```text
BAD_REQUEST
UNAUTHORIZED
FORBIDDEN
NOT_FOUND
ALREADY_EXISTS
NOT_OWNER
INVALID_TOPIC
INVALID_SCHEMA
SCHEMA_IMMUTABLE
PAYLOAD_TOO_LARGE
RATE_LIMITED
INTERNAL_ERROR
```

Clients should branch on `code`, not parse the human-readable `message`.

## Topic/filter examples

| Filter | Matches | Does not match |
| --- | --- | --- |
| `camera/+/status` | `camera/CAM01/status` | `camera/CAM01/health` |
| `camera/#` | `camera/CAM01/status`, `camera/CAM01/config/reported` | `system/health` |
| `#` | every topic permitted by policy | — |

The server's authorization policy may reject a broad filter even if it is syntactically valid.

## Browser clients

For browser deployments:

- use `wss://` in production;
- authenticate using a secure session/cookie design;
- configure `STATELINK_ALLOWED_ORIGINS` explicitly;
- do not treat `Origin` as identity;
- keep system policy deny-by-default;
- avoid embedding long-lived secrets in frontend source code.

## Raw examples

See [`../examples/wire/README.md`](../examples/wire/README.md) for a compact copy/paste set of request/response examples.
