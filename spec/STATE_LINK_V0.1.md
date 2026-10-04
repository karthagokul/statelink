# StateLink Protocol v0.1

Copyright (C) 2026 Gokul Kartha  
SPDX-License-Identifier: GPL-3.0-or-later

This document defines the initial interoperable StateLink profile implemented by this repository. The keywords **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are used normatively.

## 1. Scope

StateLink synchronizes **current state**. It is not an event log, RPC protocol, queue, or distributed transaction system.

A consumer MUST tolerate revision gaps. A gap means an intermediate state was not observed; it does not require retransmission.

## 2. Topics

Context names and filters use MQTT-compatible hierarchy and wildcard semantics:

- `/` separates levels;
- `+` matches one level;
- `#` matches all remaining levels and MUST be the final filter level.

Context names MUST NOT contain `+` or `#`.

## 3. Context ownership

Every active context has exactly one authenticated writer.

Ownership MUST be derived from the authenticated connection. A sender-supplied JSON field MUST NOT be accepted as proof of identity.

Only the active owner may `SET` or `REMOVE` its context.

## 4. Context metadata

A context may have:

- `schema`: optional contract identifier;
- `retention`: `session` or `retained`;
- `ttl`: optional freshness lifetime in seconds;
- `access.read`: producer-declared consumer selectors;
- `applied_revision`: optional desired-state revision acknowledged by reported state.

A declared schema is immutable for the lifetime of the context. Changing schema requires `REMOVE` followed by a new `DECLARE`.

## 5. Revisions

The bus assigns a monotonically increasing `revision` per context. Revisions are not globally ordered.

Implementations MAY coalesce pending updates for slow consumers. Consumers MUST interpret each delivered state as a complete replacement for the previously observed state of that context.

## 6. Freshness and retention

Retention and freshness are independent.

- `session`: the context is live only while its owner session exists;
- `retained`: the latest value survives producer disconnect;
- `fresh`: the value is within its freshness lifetime;
- `stale`: the value remains available but is no longer current.

A retained context SHOULD become stale when its active producer disconnects.

## 7. Security

Effective read access is the intersection of:

1. producer exposure policy;
2. system policy;
3. authenticated consumer identity.

System policy is authoritative. Producer policy MUST NOT expand access beyond the system-policy ceiling.

Discovery MUST be security filtered. A consumer MUST NOT receive protected context names merely because it requests a broad filter.

`*` in the reference exposure syntax means all authenticated identities that are still permitted by system policy. It does not mean unauthenticated public access.

## 8. Operations

StateLink v0.1 defines:

- `DECLARE`
- `SET`
- `GET`
- `DISCOVER`
- `SUBSCRIBE`
- `UNSUBSCRIBE`
- `REMOVE`

Responses are `OK`, `STATE`, `DISCOVERY`, `UPDATE`, or `ERROR`.

A valid `SUBSCRIBE` MUST be accepted even when no matching context currently exists. Matching contexts declared later become eligible for delivery after normal authorization checks.

## 9. WebSocket binding

The WebSocket binding uses subprotocol:

```text
statelink.v1
```

Each WebSocket text frame contains exactly one JSON StateLink message. Binary frames are not part of v1.

Example request:

```json
{"op":"get","id":42,"topic":"system/health"}
```

Example state:

```json
{
  "op":"state",
  "id":42,
  "topic":"system/health",
  "revision":18,
  "timestamp_ms":1791061200000,
  "freshness":"fresh",
  "value":{"status":"healthy"}
}
```

Browser deployments SHOULD use `wss://`, authenticated sessions, explicit Origin allow-lists, and per-context authorization. Origin MUST NOT be treated as client identity.

## 10. Desired and reported state

Declarative control should use separate topics such as:

```text
camera/CAM01/config/desired
camera/CAM01/config/reported
```

The desired context is typically retained. The reported context is typically session-scoped.

A reported context MAY set `applied_revision` to the exact desired revision it successfully applied.

## 11. MQTT interoperability

StateLink topic names and filters map directly to MQTT topic syntax.

The **MQTT Data Profile** exports the StateLink `value` as the MQTT payload and SHOULD use MQTT retained publication for current-state behavior.

A generic MQTT broker does not understand StateLink ownership, producer exposure policy, freshness, or secure discovery. A StateLink-aware gateway or broker extension is required for those semantics.

## 12. Error codes

The reference error vocabulary is:

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

Clients SHOULD depend on machine-readable error codes rather than parsing the human-readable `message` field.
