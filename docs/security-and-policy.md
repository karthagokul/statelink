<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security and Policy

StateLink v0.1 is designed around authenticated identities, single-writer ownership, and deny-by-default read/declare authorization.

This document describes the current reference implementation. Also read the repository-level [`../SECURITY.md`](../SECURITY.md) before deployment.

## Security model at a glance

A read is allowed only when all relevant layers permit it:

```text
producer access.read
        ∩
system policy
        ∩
authenticated consumer identity
        =
effective access
```

The system policy is authoritative. A producer cannot grant access beyond it.

## Authentication

The reference server accepts either:

### Bearer token

```http
Authorization: Bearer <token>
```

This is the normal path for native/embedded clients and the Rust client.

### Browser cookie

```text
statelink_token=<token>
```

This supports browser session integration, but production systems should normally have a real authentication/session layer rather than copying the development token file design unchanged.

The token maps to an identity:

```json
{
  "id": "camera-service",
  "roles": ["producer", "device"]
}
```

The authenticated identity is then used for:

- context ownership;
- system policy checks;
- producer exposure checks;
- discovery filtering;
- subscription authorization.

## Identity selectors

The reference implementation supports these selectors:

```text
*
id:<glob>
service:<glob>
role:<glob>
```

Examples:

```text
id:camera-service
id:camera-*
service:camera-service
role:hmi
role:cloud-*
*
```

`id:` and `service:` currently match the same authenticated identity id field. `service:` is provided as an expressive selector name for service-style identities.

The `*` selector means every authenticated identity still permitted by system policy. It does **not** mean anonymous/public access.

The selector wildcard is a simple `*` glob over identity/role strings. It is separate from MQTT topic wildcards.

## System policy

The policy file contains an array of allow rules.

Example:

```json
{
  "rules": [
    {
      "subjects": ["id:camera-*", "role:device"],
      "operations": ["declare", "read"],
      "topic_filter": "camera/#"
    },
    {
      "subjects": ["role:hmi"],
      "operations": ["read"],
      "topic_filter": "camera/#"
    }
  ]
}
```

Each rule contains:

- `subjects`: one or more identity selectors;
- `operations`: `declare` and/or `read`;
- `topic_filter`: an MQTT-style topic filter.

The policy is deny-by-default. If no rule matches, the operation is denied.

## `declare` permission

A producer needs `declare` permission for the concrete topic it wants to own.

Example rule:

```json
{
  "subjects": ["id:camera-*"],
  "operations": ["declare"],
  "topic_filter": "camera/#"
}
```

This does not automatically make the producer owner of all `camera/#` topics. It only allows the identity to attempt declarations in that namespace. Ownership is acquired per successfully declared context.

## `read` permission

Read permission controls:

- `GET`;
- `DISCOVER` visibility;
- `SUBSCRIBE` filter eligibility;
- subscription updates.

A consumer cannot broaden a subscription beyond the topic region allowed by a policy rule.

For example, if the only read rule allows:

```text
camera/+/status
```

then subscribing to:

```text
camera/CAM01/status
```

or:

```text
camera/+/status
```

can be allowed, but a broader:

```text
camera/#
```

is rejected because the requested filter is not fully covered by the policy ceiling.

## Producer exposure (`access.read`)

When declaring a context, the producer can further restrict who may read it:

```json
{
  "access": {
    "read": ["role:hmi", "role:cloud"]
  }
}
```

This is a reduction layer, not a grant layer.

If the system policy does not allow role `cloud` to read the topic, adding `role:cloud` to the producer's access list does not override the central policy.

An empty producer read list exposes the context only to its owner, subject to the system policy behavior of the implementation.

## Ownership

Ownership comes from the authenticated connection that declares the context.

Clients cannot set fields such as:

```text
owner
producer_id
writer
```

in the wire JSON to claim ownership.

Only the active owner may `SET` or `REMOVE` the context.

This prevents one authenticated client from modifying another producer's context even if both have permission to declare within the same namespace.

## Discovery confidentiality

Discovery is filtered after authorization. A caller asking for:

```text
#
```

must not learn names of contexts it cannot read.

This is important because topic names may themselves reveal sensitive device names, capabilities, tenant IDs, or deployment structure.

## Browser Origin checks

Native clients normally omit `Origin` and are accepted if authentication/policy succeeds.

Browser clients send `Origin`. The reference server compares that value against `STATELINK_ALLOWED_ORIGINS`.

Example:

```text
STATELINK_ALLOWED_ORIGINS=https://hmi.example.com,https://ops.example.com
```

Do not use Origin as identity. It only reduces cross-site browser abuse.

## TLS

The reference server currently listens as plain HTTP/WebSocket by default.

For production, terminate TLS so clients use:

```text
wss://
```

Suitable patterns include:

- TLS directly in a future server binding;
- a reverse proxy such as nginx/Envoy/Caddy;
- a service mesh;
- an authenticated ingress/gateway.

Protect authentication headers/cookies in transit.

## Credentials

The development auth file uses static tokens because it is easy to run locally.

Production options should prefer, where appropriate:

- short-lived signed access tokens;
- mTLS identity;
- a trusted local sidecar/proxy that authenticates clients;
- OS/container-bound credentials;
- an identity provider and secure browser session.

Do not ship `camera-token` or `hmi-token` as real credentials.

## Payload and queue limits

Two server controls reduce resource exhaustion risk:

```text
STATELINK_MAX_INBOUND_BYTES
STATELINK_OUTBOUND_QUEUE
```

Keep them bounded.

A client that sends an oversized message receives `PAYLOAD_TOO_LARGE`. Slow consumers can lose asynchronous updates or be disconnected rather than causing unbounded memory growth.

## Multi-tenant deployments

For multi-tenant systems, include tenant/site boundaries in both identity policy and topic namespaces.

Example:

```text
tenant/T001/camera/CAM01/status
```

Then define policy rules that prevent identities from requesting filters that cross tenant boundaries.

Do not rely only on application UI filtering.

## Recommended least-privilege pattern

For a camera deployment:

```text
camera service:
  declare camera/<its-id>/#
  read    camera/<its-id>/config/desired

HMI:
  read    camera/#

cloud exporter:
  read    selected telemetry/status namespaces
```

Narrow rules per service/device are safer than a universal `#` policy.

## Security review checklist

Before release/deployment verify:

- authentication cannot be bypassed;
- no client-supplied owner identity is trusted;
- system policy is deny-by-default;
- subscription filters cannot be broader than allowed policy;
- discovery does not leak unauthorized topic names;
- producer `access.read` cannot expand system authorization;
- browser Origins are explicit;
- TLS protects credentials;
- secrets are not committed/logged;
- message and queue limits are bounded;
- retained sensitive state is handled according to application security requirements.
