// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use statelink_client::Client;
use statelink_protocol::{Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("STATELINK_URL").unwrap_or_else(|_| "ws://127.0.0.1:8080/statelink".into());
    let token = env::var("STATELINK_TOKEN").unwrap_or_else(|_| "hmi-token".into());

    let mut client = Client::connect(&url, &token).await?;
    client
        .send(&Request::Subscribe {
            id: 1,
            filter: "demo/#".into(),
        })
        .await?;

    let mut updates = 0usize;
    loop {
        let response = client.next_response().await?;
        println!("<= {response:?}");
        if matches!(response, Response::Update { .. }) {
            updates += 1;
            if updates >= 5 {
                break;
            }
        }
    }

    Ok(())
}
