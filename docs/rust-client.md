<!-- Copyright (C) 2026 Gokul Kartha -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Rust Client Guide

The `statelink-client` crate is the async Rust client for the StateLink WebSocket binding.

Canonical source:

```text
crates/statelink-client/src/lib.rs
```

Canonical runnable examples:

```text
crates/statelink-client/examples/producer.rs
crates/statelink-client/examples/consumer.rs
```

## Client model

The current API intentionally stays small:

```rust
Client::connect(url, bearer_token).await
client.send(&request).await
client.next_response().await
```

`Client::connect`:

- opens the WebSocket;
- sends `Authorization: Bearer <token>`;
- requests the `statelink.v1` WebSocket subprotocol.

`send` serializes a `statelink_protocol::Request` as one WebSocket text frame.

`next_response` waits for the next protocol response, automatically answers WebSocket ping frames with pong frames, rejects binary protocol messages, and returns a typed `statelink_protocol::Response`.

## Add the workspace crates

Inside this repository, depend on:

```toml
[dependencies]
statelink-client = { path = "../statelink-client" }
statelink-protocol = { path = "../statelink-protocol" }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

For an external project, use the appropriate Git/path dependency until published crates are available.

## Producer example

A producer connects with a token whose authenticated identity has permission to declare the chosen topic.

```rust
use serde_json::json;
use statelink_client::Client;
use statelink_protocol::{AccessPolicy, Request, Retention};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "ws://127.0.0.1:8080/statelink",
        "camera-token",
    )
    .await?;

    client
        .send(&Request::Declare {
            id: 1,
            topic: "camera/CAM01/status".into(),
            schema: Some("camera-status/v1".into()),
            value: json!({
                "online": true,
                "fps": 15
            }),
            access: AccessPolicy {
                read: vec!["role:hmi".into()],
            },
            retention: Retention::Session,
            ttl: Some(30),
            applied_revision: None,
        })
        .await?;

    println!("{:?}", client.next_response().await?);

    client
        .send(&Request::Set {
            id: 2,
            topic: "camera/CAM01/status".into(),
            value: json!({
                "online": true,
                "fps": 20
            }),
            schema: None,
            applied_revision: None,
        })
        .await?;

    println!("{:?}", client.next_response().await?);
    Ok(())
}
```

Important producer rules:

- declare a context before setting it;
- do not include a sender/owner identity in payloads — ownership comes from authentication;
- treat `value` as complete current state, not a patch;
- keep schema identifiers stable for the life of the context;
- choose `session` or `retained` deliberately;
- use TTL for state whose freshness matters;
- expose only the consumer roles/identities that actually need the context.

## Consumer example

```rust
use statelink_client::Client;
use statelink_protocol::{Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "ws://127.0.0.1:8080/statelink",
        "hmi-token",
    )
    .await?;

    client
        .send(&Request::Subscribe {
            id: 1,
            filter: "camera/+/status".into(),
        })
        .await?;

    loop {
        match client.next_response().await? {
            Response::Ok {
                subscription: Some(id),
                ..
            } => {
                println!("subscription active: {id}");
            }
            Response::Update { state, .. } => {
                println!(
                    "topic={} revision={} freshness={:?} value={}",
                    state.topic,
                    state.revision,
                    state.freshness,
                    state.value
                );
            }
            Response::Error { code, message, .. } => {
                eprintln!("StateLink error {code:?}: {message}");
            }
            _ => {}
        }
    }
}
```

A subscription may be started before the producer exists.

## GET

Use `GET` when you need one point-in-time state rather than a continuing subscription.

```rust
client
    .send(&Request::Get {
        id: 10,
        topic: "camera/CAM01/status".into(),
    })
    .await?;

match client.next_response().await? {
    Response::State { state, .. } => {
        println!("{}", state.value);
    }
    other => {
        println!("unexpected response: {other:?}");
    }
}
```

## DISCOVER

Use discovery to find authorized contexts without learning unauthorized names.

```rust
client
    .send(&Request::Discover {
        id: 20,
        filter: "camera/#".into(),
    })
    .await?;
```

Handle `Response::Discovery { contexts, .. }`.

## UNSUBSCRIBE

The successful `SUBSCRIBE` response includes a server-assigned subscription id:

```rust
Response::Ok {
    subscription: Some(subscription),
    ..
}
```

Later:

```rust
client
    .send(&Request::Unsubscribe {
        id: 30,
        subscription,
    })
    .await?;
```

## Desired/reported state

A device can subscribe to a desired configuration context and publish a reported context containing `applied_revision`.

Example reported update:

```rust
client
    .send(&Request::Set {
        id: 51,
        topic: "camera/CAM01/config/reported".into(),
        value: serde_json::json!({"fps": 15}),
        schema: None,
        applied_revision: Some(27),
    })
    .await?;
```

This means the reported state corresponds to desired revision 27.

## Request IDs

The current client leaves request-id generation to the application. Use unique/increasing IDs for concurrently outstanding operations so responses are easy to correlate.

Subscription updates are identified by the server-assigned subscription id rather than request id.

## Handling slow consumers

StateLink is state-oriented. The reference server may drop an update for a slow consumer when its bounded outbound queue is full. Applications must therefore:

- treat each update as complete state;
- accept revision gaps;
- never depend on receiving every intermediate revision.

If every transition matters, use an event stream alongside StateLink.

## Reconnection

The current minimal client does not automatically reconnect or reconstruct subscriptions. Production applications should wrap it with application-level reconnect logic:

1. reconnect and re-authenticate;
2. re-create subscriptions;
3. accept the current snapshot/update stream;
4. re-declare session-scoped producer contexts when applicable.

Do not assume a previous session's ownership survives reconnect for session contexts.

## Run the included Rust examples

Server:

```bash
STATELINK_AUTH_FILE=config/dev-auth.json \
STATELINK_POLICY_FILE=config/dev-policy.json \
cargo run -p statelink-server
```

Consumer:

```bash
STATELINK_TOKEN=hmi-token \
cargo run -p statelink-client --example consumer
```

Producer:

```bash
STATELINK_TOKEN=camera-token \
cargo run -p statelink-client --example producer
```

Windows equivalents are documented in [`../examples/rust/README.md`](../examples/rust/README.md).
