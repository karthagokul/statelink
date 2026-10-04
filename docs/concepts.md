<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Core Concepts

StateLink synchronizes **current state** between authenticated producers and consumers. It is deliberately smaller than a message bus and does not try to preserve every intermediate transition.

## Contexts

A **context** is a named current-state object identified by a topic.

Examples:

```text
system/health
camera/CAM01/status
camera/CAM01/config/desired
camera/CAM01/config/reported
```

A context contains:

- a topic;
- a complete JSON value;
- a monotonically increasing per-context revision;
- a timestamp;
- freshness (`fresh` or `stale`);
- optional schema identifier;
- optional TTL;
- retention mode;
- producer-defined read exposure;
- optional `applied_revision`.

## Producers and ownership

Every active context has exactly one authenticated writer.

A producer obtains ownership by successfully declaring the context. The server derives ownership from the authenticated session, not from a JSON field supplied by the client.

Only the owner may update or remove the context.

This means two producers cannot concurrently write the same context. If a second producer tries to declare an already-owned topic, it receives an error rather than silently taking ownership.

## Consumers

A consumer can:

- `GET` one exact topic;
- `DISCOVER` contexts matching a filter;
- `SUBSCRIBE` to current/future contexts matching a filter;
- `UNSUBSCRIBE` from a subscription.

Consumers only see contexts they are authorized to read.

## Topics and filters

StateLink uses MQTT-compatible hierarchy and wildcard syntax.

Topic levels are separated with `/`:

```text
camera/CAM01/status
```

Filters may use:

- `+` for exactly one level;
- `#` for all remaining levels, and only as the final level.

Examples:

```text
camera/+/status
camera/#
#
```

Context names themselves cannot contain `+` or `#`.

## State, not events

A StateLink update is a complete replacement of the current value for that context.

Suppose revisions progress as:

```text
10 -> 11 -> 12 -> 13
```

A slow consumer might observe only revisions `10` and `13`. That is valid. Revision `13` is complete current state; StateLink does not require revisions `11` and `12` to be replayed.

If an application requires every event, audit history, or durable ordered delivery, it should use an event system alongside StateLink rather than forcing those semantics into StateLink.

## Revisions

Each context has its own monotonically increasing revision counter.

Revisions are **not globally ordered** across contexts. Comparing revision `20` from one topic with revision `30` from another does not imply any global ordering relationship.

Use revisions to answer questions such as:

- Is this newer than the state I currently hold?
- Has the device applied this desired configuration revision?

## Retention

Retention controls what happens when the owner disconnects.

### `session`

The context belongs to the live producer session. When the owner disconnects, the context is removed.

Typical uses:

```text
camera/CAM01/status
camera/CAM01/runtime
camera/CAM01/health
```

### `retained`

The latest context value survives producer disconnect.

Typical uses:

```text
camera/CAM01/config/desired
site/settings
```

A retained value is not necessarily fresh. Retention and freshness are separate concepts.

## TTL and freshness

If a context has a TTL, the server can mark it `stale` after the freshness lifetime expires.

For example, a health context declared with:

```json
"ttl": 30
```

is expected to be refreshed often enough that its latest value remains current. If not, consumers can still receive the state but see:

```json
"freshness": "stale"
```

A retained context may remain available while stale.

## Schemas

A context can optionally declare a schema identifier:

```json
"schema": "camera-health/v1"
```

The schema identifier is an application-level contract identifier. StateLink v0.1 does not fetch or validate a JSON Schema document automatically.

Once declared, the schema identifier is immutable for the lifetime of that context. To change it, remove the context and declare it again.

This prevents a producer from changing the meaning of an existing context without an explicit lifecycle transition.

## Read exposure and system policy

Read authorization is the intersection of two independent layers:

```text
producer access.read
        ∩
system allow-list policy
        ∩
authenticated consumer identity
        =
effective read access
```

A producer may restrict who can read its state, but it cannot grant access beyond the system policy ceiling.

For example, a producer might declare:

```json
"access": {
  "read": ["role:hmi"]
}
```

Even an HMI is allowed only if the central system policy also permits that identity/role to read the topic.

## Discovery

Discovery is security-filtered. A broad discovery request does not leak protected context names.

For example:

```json
{
  "op": "discover",
  "id": 7,
  "filter": "camera/#"
}
```

returns only contexts the authenticated caller is allowed to read.

## Future subscriptions

A subscription does not require a matching producer to exist at subscription time.

A consumer can subscribe to:

```text
camera/+/status
```

before any camera is online. When a producer later declares a matching context, the new context becomes eligible for delivery after authorization checks.

This is useful for HMIs and cloud gateways because startup order does not need to be coordinated.

## Desired and reported state

For configuration and control, prefer declarative desired/reported contexts instead of command messages.

Example:

```text
camera/CAM01/config/desired
camera/CAM01/config/reported
```

A controller owns `desired` and writes the configuration it wants:

```json
{
  "fps": 15,
  "recording": true
}
```

A device subscribes to the desired topic, applies the configuration, and publishes what is actually active under `reported`.

The reported context can include:

```json
"applied_revision": 27
```

meaning that the device successfully applied desired revision 27.

Recommended lifecycle:

- desired configuration: often `retained`;
- reported configuration: often `session`;
- device disconnect: reported state disappears or becomes unavailable while the desired state remains.

## What StateLink is not

StateLink v0.1 intentionally does not provide:

- RPC request/response methods;
- durable event queues;
- event history;
- exactly-once processing;
- global transactions;
- global revision ordering;
- transparent multi-server federation.

Keeping these boundaries is what allows the protocol and implementation to stay lightweight.
