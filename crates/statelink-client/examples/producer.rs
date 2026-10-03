// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{env, time::Duration};

use serde_json::json;
use statelink_client::Client;
use statelink_protocol::{AccessPolicy, Request, Retention};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("STATELINK_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:8080/statelink".into());
    let token = env::var("STATELINK_TOKEN").unwrap_or_else(|_| "camera-token".into());

    let mut client = Client::connect(&url, &token).await?;
    client
        .send(&Request::Declare {
            id: 1,
            topic: "demo/device/status".into(),
            schema: Some("demo-status/v1".into()),
            value: json!({"online": true, "counter": 0}),
            access: AccessPolicy {
                read: vec!["role:hmi".into()],
            },
            retention: Retention::Session,
            ttl: Some(5),
            applied_revision: None,
        })
        .await?;
    println!("DECLARE => {:?}", client.next_response().await?);

    for counter in 1..=5 {
        sleep(Duration::from_secs(1)).await;
        client
            .send(&Request::Set {
                id: 1 + counter,
                topic: "demo/device/status".into(),
                value: json!({"online": true, "counter": counter}),
                schema: None,
                applied_revision: None,
            })
            .await?;
        println!("SET {counter} => {:?}", client.next_response().await?);
    }

    Ok(())
}
