// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

//! StateLink protocol wire types.
//!
//! StateLink is state-oriented: revisions identify newer complete state, not a
//! guaranteed event history. Consumers must tolerate revision gaps.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod topic;

pub const PROTOCOL_VERSION: &str = "0.1";
pub const WEBSOCKET_SUBPROTOCOL: &str = "statelink.v1";

pub type RequestId = u64;
pub type Revision = u64;
pub type SubscriptionId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccessPolicy {
    /// Subject selectors allowed to read/subscribe to this context.
    ///
    /// Supported by the reference implementation: `*`, `id:<glob>`,
    /// `service:<glob>`, and `role:<glob>`. An empty list exposes the context
    /// only to its owner, subject to system policy.
    #[serde(default)]
    pub read: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Retention {
    #[default]
    Session,
    Retained,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Freshness {
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Declare {
        id: RequestId,
        topic: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        schema: Option<String>,
        value: Value,
        #[serde(default)]
        access: AccessPolicy,
        #[serde(default)]
        retention: Retention,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ttl: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        applied_revision: Option<Revision>,
    },
    Set {
        id: RequestId,
        topic: String,
        value: Value,
        /// Optional only so an implementation can explicitly reject an attempt
        /// to mutate the immutable schema contract.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        schema: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        applied_revision: Option<Revision>,
    },
    Get {
        id: RequestId,
        topic: String,
    },
    Discover {
        id: RequestId,
        filter: String,
    },
    Subscribe {
        id: RequestId,
        filter: String,
    },
    Unsubscribe {
        id: RequestId,
        subscription: SubscriptionId,
    },
    Remove {
        id: RequestId,
        topic: String,
    },
}

impl Request {
    pub fn id(&self) -> RequestId {
        match self {
            Self::Declare { id, .. }
            | Self::Set { id, .. }
            | Self::Get { id, .. }
            | Self::Discover { id, .. }
            | Self::Subscribe { id, .. }
            | Self::Unsubscribe { id, .. }
            | Self::Remove { id, .. } => *id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextState {
    pub topic: String,
    pub revision: Revision,
    /// Unix time in milliseconds.
    pub timestamp_ms: u64,
    pub freshness: Freshness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// When present, identifies the desired-context revision that this state
    /// reports as successfully applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_revision: Option<Revision>,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredContext {
    pub topic: String,
    pub revision: Revision,
    pub freshness: Freshness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Response {
    Ok {
        id: RequestId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscription: Option<SubscriptionId>,
    },
    State {
        id: RequestId,
        #[serde(flatten)]
        state: ContextState,
    },
    Discovery {
        id: RequestId,
        contexts: Vec<DiscoveredContext>,
    },
    Update {
        subscription: SubscriptionId,
        #[serde(flatten)]
        state: ContextState,
    },
    Error {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<RequestId>,
        code: ErrorCode,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    AlreadyExists,
    NotOwner,
    InvalidTopic,
    InvalidSchema,
    SchemaImmutable,
    PayloadTooLarge,
    RateLimited,
    InternalError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_round_trip() {
        let request = Request::Declare {
            id: 1,
            topic: "system/health".into(),
            schema: Some("system-health/v1".into()),
            value: json!({"status": "healthy"}),
            access: AccessPolicy {
                read: vec!["role:hmi".into()],
            },
            retention: Retention::Session,
            ttl: Some(30),
            applied_revision: None,
        };

        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, request);
        assert_eq!(decoded.id(), 1);
    }

    #[test]
    fn error_codes_are_stable_uppercase_names() {
        let response = Response::Error {
            id: Some(9),
            code: ErrorCode::SchemaImmutable,
            message: "schema is immutable".into(),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        assert!(encoded.contains("SCHEMA_IMMUTABLE"));
    }

    #[test]
    fn applied_revision_is_optional() {
        let state = ContextState {
            topic: "camera/CAM01/config/reported".into(),
            revision: 2,
            timestamp_ms: 100,
            freshness: Freshness::Fresh,
            schema: None,
            applied_revision: Some(27),
            value: json!({"fps": 15}),
        };
        let encoded = serde_json::to_string(&state).unwrap();
        assert!(encoded.contains("\"applied_revision\":27"));
    }
}
