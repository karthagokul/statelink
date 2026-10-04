// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use futures_util::{SinkExt, StreamExt};
use statelink_protocol::{Request, Response, WEBSOCKET_SUBPROTOCOL};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest,
        http::{
            header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL},
            HeaderValue,
        },
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid authorization header: {0}")]
    Header(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),
    #[error("connection closed")]
    Closed,
    #[error("binary websocket messages are not part of StateLink v1")]
    BinaryMessage,
}

pub struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Client {
    pub async fn connect(url: &str, bearer_token: &str) -> Result<Self, ClientError> {
        let mut request = url.into_client_request()?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {bearer_token}"))?,
        );
        request.headers_mut().insert(
            SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(WEBSOCKET_SUBPROTOCOL),
        );

        let (socket, _) = connect_async(request).await?;
        Ok(Self { socket })
    }

    pub async fn send(&mut self, request: &Request) -> Result<(), ClientError> {
        let payload = serde_json::to_string(request)?;
        self.socket.send(Message::Text(payload)).await?;
        Ok(())
    }

    pub async fn next_response(&mut self) -> Result<Response, ClientError> {
        loop {
            match self.socket.next().await {
                Some(Ok(Message::Text(text))) => {
                    return Ok(serde_json::from_str(text.as_ref())?);
                }
                Some(Ok(Message::Ping(payload))) => {
                    self.socket.send(Message::Pong(payload)).await?;
                }
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Close(_))) | None => return Err(ClientError::Closed),
                Some(Ok(Message::Binary(_))) => return Err(ClientError::BinaryMessage),
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(error.into()),
            }
        }
    }
}
