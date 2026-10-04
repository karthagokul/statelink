<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Architecture

StateLink separates protocol representation, state semantics, transport, client SDK, and external adapters so the core model can remain lightweight and transport-independent.

## Logical architecture

```text
                        +-----------------------+
                        |   Application / HMI   |
                        +-----------+-----------+
                                    |
                              statelink-client
                                    |
                              WebSocket v0.1
                                    |
+----------------+      +-----------v-----------+      +----------------+
| Device/Service |----->|   statelink-server    |<-----| Other Consumer |
+----------------+      | auth + WS adaptation  |      +----------------+
                        +-----------+-----------+
                                    |
                        Request / Response types
                                    |
                        +-----------v-----------+
                        |    statelink-core      |
                        | contexts, revisions,  |
                        | policy, subscriptions |
                        +-----------+-----------+
                                    |
                                    | authorized updates
                                    v
                        +-----------------------+
                        |   statelink-mqtt      |
                        | optional export       |
                        +-----------+-----------+
                                    |
                                    v
                              MQTT broker
```

## Layer 1: `statelink-protocol`

The protocol crate defines the vocabulary shared between implementations:

- requests;
- responses;
- state metadata;
- error codes;
- retention/freshness enums;
- topic/filter rules;
- WebSocket subprotocol constant.

It should contain no network listener and no application-specific device logic.

## Layer 2: `statelink-core`

The core crate is the state engine. It receives an authenticated `Session` plus a typed protocol request and returns:

- the direct response to that request;
- zero or more deliveries for subscribed sessions.

Its responsibilities include:

- context ownership;
- revisions;
- declare/set/get/remove semantics;
- discovery;
- subscription lifecycle;
- TTL freshness transitions;
- session/retained lifecycle;
- system policy;
- producer exposure checks.

Keeping this independent from WebSocket means another transport can use the same state semantics.

## Layer 3: `statelink-server`

The server adapts HTTP/WebSocket connections to the core.

Connection flow:

```text
HTTP upgrade request
   |
   +-- require statelink.v1 subprotocol
   +-- validate browser Origin when present
   +-- authenticate bearer token/cookie
   |
   v
create authenticated Session
   |
   v
parse JSON Request
   |
   v
Bus::handle_request(...)
   |
   +--> direct Response -> requesting socket
   |
   +--> Delivery[] -> subscribed sockets
```

Each connection has a bounded outbound queue. This is intentional: StateLink prefers bounded memory and eventual current-state convergence over preserving every intermediate update.

## Layer 4: `statelink-client`

The Rust SDK is a thin transport client rather than a second state engine.

It:

- performs authenticated WebSocket connection setup;
- requests `statelink.v1`;
- serializes `Request`;
- deserializes `Response`;
- handles WebSocket ping/pong;
- exposes protocol responses to application code.

Application-level reconnect, caching, and domain models remain outside the minimal v0.1 client.

## Layer 5: adapters

`statelink-mqtt` is an adapter around the client. It subscribes to authorized StateLink state and publishes `state.value` to MQTT retained topics.

Adapters should not become alternate sources of StateLink ownership/policy truth.

## Read path

A typical subscription path is:

```text
consumer authentication
        |
        v
SUBSCRIBE camera/#
        |
        v
system policy checks requested filter
        |
        v
subscription registered
        |
        v
producer DECLARE/SET
        |
        v
context-specific system policy + producer exposure checks
        |
        v
UPDATE delivered to authorized subscription
```

## Write path

A typical producer path is:

```text
producer authentication
        |
        v
DECLARE topic
        |
        +-- topic validation
        +-- system declare policy
        +-- check topic not already owned
        |
        v
context owner = authenticated session
        |
        v
SET topic
        |
        +-- owner check
        +-- schema immutability check
        |
        v
revision increments
        |
        v
subscriptions receive new complete state
```

## Lifecycle

### Session context

```text
DECLARE -> active -> SET... -> owner disconnect -> removed
```

### Retained context

```text
DECLARE -> active -> SET... -> owner disconnect -> retained latest value/stale
```

In the current in-memory reference server, retained data does not provide process-restart durability.

## Freshness

TTL is evaluated by server maintenance. A context can remain present while transitioning:

```text
fresh -> stale
```

Freshness and retention are independent.

## Concurrency model

The reference server wraps the core `Bus` in an async mutex and handles network sessions concurrently. State mutations therefore pass through a serialized core state engine while socket I/O remains asynchronous.

This is simple and deterministic for the reference implementation. Future high-scale implementations may shard/context-partition internally while preserving protocol semantics.

## Slow consumers

The server uses bounded per-peer channels. If a consumer cannot keep up, an asynchronous update can be dropped.

That is safe only because the protocol contract says each delivered revision is complete current state and clients tolerate gaps.

This is a core architectural property, not an accidental implementation detail.

## Recommended application architecture

Use StateLink for state that other components need to converge on:

```text
health
runtime status
current configuration
capabilities
current alarms/state summaries
current operating mode
```

Use another mechanism for data that must preserve every item:

```text
audit log
financial transaction
command journal
event history
high-volume telemetry archive
```

A real system can use both:

```text
StateLink -> current operational truth
Event/DB  -> durable history
```

## Extension rule

When adding features, ask where the rule belongs:

- wire representation -> `statelink-protocol`;
- state semantics/security policy -> `statelink-core`;
- WebSocket/HTTP mechanics -> `statelink-server`;
- Rust developer ergonomics -> `statelink-client`;
- external-system mapping -> adapter crate.

This separation is the main architectural discipline of the repository.
