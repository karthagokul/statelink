// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{env, time::Duration};

use rumqttc::{AsyncClient, MqttOptions, QoS};
use statelink_client::Client;
use statelink_protocol::{Request, Response};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "statelink_mqtt=info".into()),
        )
        .init();

    let statelink_url = env::var("STATELINK_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:8080/statelink".into());
    let statelink_token = env::var("STATELINK_TOKEN")?;
    let statelink_filter = env::var("STATELINK_FILTER").unwrap_or_else(|_| "#".into());

    let mqtt_host = env::var("MQTT_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let mqtt_port = env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".into())
        .parse::<u16>()?;
    let mqtt_client_id =
        env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "statelink-export".into());

    let mut options = MqttOptions::new(mqtt_client_id, mqtt_host, mqtt_port);
    options.set_keep_alive(Duration::from_secs(30));
    if let Ok(username) = env::var("MQTT_USERNAME") {
        options.set_credentials(username, env::var("MQTT_PASSWORD").unwrap_or_default());
    }

    let (mqtt, mut eventloop) = AsyncClient::new(options, 128);
    tokio::spawn(async move {
        loop {
            if let Err(error) = eventloop.poll().await {
                error!(%error, "MQTT event loop stopped");
                break;
            }
        }
    });

    let mut statelink = Client::connect(&statelink_url, &statelink_token).await?;
    statelink
        .send(&Request::Subscribe {
            id: 1,
            filter: statelink_filter.clone(),
        })
        .await?;

    info!(filter = %statelink_filter, "exporting authorized StateLink state to MQTT");

    loop {
        match statelink.next_response().await? {
            Response::Ok {
                subscription: Some(id),
                ..
            } => info!(subscription = id, "StateLink subscription active"),
            Response::Update { state, .. } => {
                let payload = serde_json::to_vec(&state.value)?;
                mqtt.publish(&state.topic, QoS::AtLeastOnce, true, payload)
                    .await?;
            }
            Response::Error { code, message, .. } => {
                warn!(?code, %message, "StateLink adapter received protocol error");
            }
            _ => {}
        }
    }
}
