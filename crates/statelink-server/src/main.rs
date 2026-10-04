// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

mod auth;

use std::{
    collections::HashMap,
    env, fs,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use auth::{AuthFile, OriginPolicy, TokenAuthenticator};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use statelink_core::{Bus, Delivery, RulePolicy, RulePolicyConfig, Session, SessionId};
use statelink_protocol::{ErrorCode, Request, Response as ProtocolResponse, WEBSOCKET_SUBPROTOCOL};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    bus: Arc<Mutex<Bus>>,
    peers: Arc<RwLock<HashMap<SessionId, mpsc::Sender<Message>>>>,
    auth: Arc<TokenAuthenticator>,
    origins: Arc<OriginPolicy>,
    next_session: Arc<AtomicU64>,
    outbound_queue: usize,
    max_inbound_bytes: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "statelink_server=info".into()),
        )
        .init();

    let bind = env::var("STATELINK_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let auth_path = env::var("STATELINK_AUTH_FILE").unwrap_or_else(|_| "config/auth.json".into());
    let policy_path =
        env::var("STATELINK_POLICY_FILE").unwrap_or_else(|_| "config/policy.json".into());
    let allowed_origins = env::var("STATELINK_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let outbound_queue = env_usize("STATELINK_OUTBOUND_QUEUE", 256)?;
    let max_inbound_bytes = env_usize("STATELINK_MAX_INBOUND_BYTES", 1_048_576)?;

    let auth_config: AuthFile = serde_json::from_str(&fs::read_to_string(&auth_path)?)?;
    let policy_config: RulePolicyConfig = serde_json::from_str(&fs::read_to_string(&policy_path)?)?;
    let policy = Arc::new(RulePolicy::from_config(policy_config)?);

    let state = AppState {
        bus: Arc::new(Mutex::new(Bus::new(policy))),
        peers: Arc::new(RwLock::new(HashMap::new())),
        auth: Arc::new(TokenAuthenticator::from_config(auth_config)),
        origins: Arc::new(OriginPolicy::new(allowed_origins)),
        next_session: Arc::new(AtomicU64::new(1)),
        outbound_queue,
        max_inbound_bytes,
    };

    tokio::spawn(maintenance_loop(state.clone()));

    let app = Router::new()
        .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/statelink", get(websocket_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    info!(%bind, "StateLink server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Response {
    if !requested_statelink_subprotocol(&headers) {
        return (
            StatusCode::BAD_REQUEST,
            format!("WebSocket subprotocol '{WEBSOCKET_SUBPROTOCOL}' is required"),
        )
            .into_response();
    }

    if !state.origins.allows(&headers) {
        return (StatusCode::FORBIDDEN, "browser Origin is not allowed").into_response();
    }

    let Some(identity) = state.auth.authenticate(&headers) else {
        return (StatusCode::UNAUTHORIZED, "authentication required").into_response();
    };

    ws.protocols([WEBSOCKET_SUBPROTOCOL])
        .on_upgrade(move |socket| handle_socket(state, socket, identity))
        .into_response()
}

async fn handle_socket(state: AppState, socket: WebSocket, identity: statelink_core::Identity) {
    let session_id = state.next_session.fetch_add(1, Ordering::Relaxed);
    let session = Session::new(session_id, identity.clone());
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<Message>(state.outbound_queue);
    state
        .peers
        .write()
        .await
        .insert(session_id, outbound_tx.clone());

    info!(session_id, identity = %identity.id, "StateLink client connected");

    let (mut socket_tx, mut socket_rx) = socket.split();

    loop {
        tokio::select! {
            outbound = outbound_rx.recv() => {
                let Some(message) = outbound else { break; };
                if socket_tx.send(message).await.is_err() {
                    break;
                }
            }
            incoming = socket_rx.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if text.len() > state.max_inbound_bytes {
                            if !queue_protocol_response(
                                &outbound_tx,
                                ProtocolResponse::Error {
                                    id: None,
                                    code: ErrorCode::PayloadTooLarge,
                                    message: "WebSocket message exceeds configured limit".into(),
                                },
                            ) {
                                break;
                            }
                            continue;
                        }

                        let request = match serde_json::from_str::<Request>(&text) {
                            Ok(request) => request,
                            Err(error) => {
                                if !queue_protocol_response(
                                    &outbound_tx,
                                    ProtocolResponse::Error {
                                        id: None,
                                        code: ErrorCode::BadRequest,
                                        message: format!("invalid request: {error}"),
                                    },
                                ) {
                                    break;
                                }
                                continue;
                            }
                        };

                        let result = {
                            let mut bus = state.bus.lock().await;
                            bus.handle_request(&session, request, now_ms())
                        };

                        if !queue_protocol_response(&outbound_tx, result.response) {
                            warn!(session_id, "closing slow client: response queue is full");
                            break;
                        }
                        dispatch_deliveries(&state, result.deliveries).await;
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if outbound_tx.try_send(Message::Pong(payload)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Binary(_))) => {
                        if !queue_protocol_response(
                            &outbound_tx,
                            ProtocolResponse::Error {
                                id: None,
                                code: ErrorCode::BadRequest,
                                message: "binary frames are not part of StateLink v1".into(),
                            },
                        ) {
                            break;
                        }
                    }
                    Some(Err(error)) => {
                        warn!(session_id, %error, "websocket receive error");
                        break;
                    }
                }
            }
        }
    }

    state.peers.write().await.remove(&session_id);
    let deliveries = {
        let mut bus = state.bus.lock().await;
        bus.disconnect_session(session_id, now_ms())
    };
    dispatch_deliveries(&state, deliveries).await;
    info!(session_id, identity = %identity.id, "StateLink client disconnected");
}

async fn maintenance_loop(state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
    loop {
        interval.tick().await;
        let deliveries = {
            let mut bus = state.bus.lock().await;
            bus.maintenance(now_ms())
        };
        dispatch_deliveries(&state, deliveries).await;
    }
}

async fn dispatch_deliveries(state: &AppState, deliveries: Vec<Delivery>) {
    if deliveries.is_empty() {
        return;
    }

    let peers = state.peers.read().await;
    for delivery in deliveries {
        let Some(sender) = peers.get(&delivery.session_id) else {
            continue;
        };
        if !queue_protocol_response(sender, delivery.response) {
            // StateLink is state-oriented. Dropping an UPDATE for a slow
            // consumer is acceptable; a later revision can converge it again.
            warn!(
                session_id = delivery.session_id,
                "dropping StateLink update for slow consumer"
            );
        }
    }
}

fn queue_protocol_response(sender: &mpsc::Sender<Message>, response: ProtocolResponse) -> bool {
    let payload = match serde_json::to_string(&response) {
        Ok(payload) => payload,
        Err(error) => {
            warn!(%error, "failed to serialize StateLink response");
            return false;
        }
    };
    sender.try_send(Message::Text(payload)).is_ok()
}

fn requested_statelink_subprotocol(headers: &HeaderMap) -> bool {
    headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .any(|protocol| protocol == WEBSOCKET_SUBPROTOCOL)
        })
        .unwrap_or(false)
}

fn env_usize(name: &str, default: usize) -> Result<usize, Box<dyn std::error::Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn websocket_subprotocol_is_required() {
        let mut headers = HeaderMap::new();
        assert!(!requested_statelink_subprotocol(&headers));
        headers.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("other, statelink.v1"),
        );
        assert!(requested_statelink_subprotocol(&headers));
    }
}
