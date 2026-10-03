// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use statelink_protocol::{
    topic, AccessPolicy, ContextState, DiscoveredContext, ErrorCode, Freshness, Request, Response,
    Retention, Revision, SubscriptionId,
};
use thiserror::Error;

use crate::{
    identity::{Identity, Session, SessionId},
    policy::{selector_matches, validate_selector, Operation, SystemPolicy},
};

#[derive(Debug, Clone)]
pub struct Limits {
    pub max_context_bytes: usize,
    pub max_contexts: usize,
    pub max_subscriptions_per_session: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_context_bytes: 1_048_576,
            max_contexts: 10_000,
            max_subscriptions_per_session: 1_024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Delivery {
    pub session_id: SessionId,
    pub response: Response,
}

#[derive(Debug, Clone)]
pub struct HandleResult {
    pub response: Response,
    pub deliveries: Vec<Delivery>,
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct BusError {
    pub code: ErrorCode,
    pub message: String,
}

impl BusError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct ContextRecord {
    topic: String,
    owner_id: String,
    active_session: Option<SessionId>,
    schema: Option<String>,
    value: Value,
    access: AccessPolicy,
    retention: Retention,
    ttl: Option<u64>,
    revision: Revision,
    updated_ms: u64,
    applied_revision: Option<Revision>,
    force_stale: bool,
    stale_notified: bool,
}

#[derive(Debug, Clone)]
struct Subscription {
    id: SubscriptionId,
    session_id: SessionId,
    identity: Identity,
    filter: String,
}

pub struct Bus {
    policy: Arc<dyn SystemPolicy>,
    limits: Limits,
    contexts: HashMap<String, ContextRecord>,
    subscriptions: HashMap<SubscriptionId, Subscription>,
    next_subscription_id: SubscriptionId,
}

impl Bus {
    pub fn new(policy: Arc<dyn SystemPolicy>) -> Self {
        Self::with_limits(policy, Limits::default())
    }

    pub fn with_limits(policy: Arc<dyn SystemPolicy>, limits: Limits) -> Self {
        Self {
            policy,
            limits,
            contexts: HashMap::new(),
            subscriptions: HashMap::new(),
            next_subscription_id: 1,
        }
    }

    pub fn handle_request(
        &mut self,
        session: &Session,
        request: Request,
        now_ms: u64,
    ) -> HandleResult {
        let request_id = request.id();

        let result = match request {
            Request::Declare {
                topic,
                schema,
                value,
                access,
                retention,
                ttl,
                applied_revision,
                ..
            } => self
                .declare(
                    session,
                    topic,
                    schema,
                    value,
                    access,
                    retention,
                    ttl,
                    applied_revision,
                    now_ms,
                )
                .map(|deliveries| (Response::Ok {
                    id: request_id,
                    subscription: None,
                }, deliveries)),
            Request::Set {
                topic,
                value,
                schema,
                applied_revision,
                ..
            } => self
                .set(
                    session,
                    &topic,
                    value,
                    schema,
                    applied_revision,
                    now_ms,
                )
                .map(|deliveries| (Response::Ok {
                    id: request_id,
                    subscription: None,
                }, deliveries)),
            Request::Get { topic, .. } => self.get(session, &topic, now_ms).map(|state| {
                (
                    Response::State {
                        id: request_id,
                        state,
                    },
                    Vec::new(),
                )
            }),
            Request::Discover { filter, .. } => self.discover(session, &filter, now_ms).map(|contexts| {
                (
                    Response::Discovery {
                        id: request_id,
                        contexts,
                    },
                    Vec::new(),
                )
            }),
            Request::Subscribe { filter, .. } => self.subscribe(session, filter, now_ms).map(
                |(subscription, deliveries)| {
                    (
                        Response::Ok {
                            id: request_id,
                            subscription: Some(subscription),
                        },
                        deliveries,
                    )
                },
            ),
            Request::Unsubscribe { subscription, .. } => self
                .unsubscribe(session, subscription)
                .map(|()| (Response::Ok {
                    id: request_id,
                    subscription: None,
                }, Vec::new())),
            Request::Remove { topic, .. } => self.remove(session, &topic).map(|()| {
                (
                    Response::Ok {
                        id: request_id,
                        subscription: None,
                    },
                    Vec::new(),
                )
            }),
        };

        match result {
            Ok((response, deliveries)) => HandleResult {
                response,
                deliveries,
            },
            Err(error) => HandleResult {
                response: Response::Error {
                    id: Some(request_id),
                    code: error.code,
                    message: error.message,
                },
                deliveries: Vec::new(),
            },
        }
    }

    pub fn disconnect_session(&mut self, session_id: SessionId, now_ms: u64) -> Vec<Delivery> {
        self.subscriptions
            .retain(|_, subscription| subscription.session_id != session_id);

        let affected: Vec<String> = self
            .contexts
            .iter()
            .filter_map(|(topic, record)| {
                (record.active_session == Some(session_id)).then(|| topic.clone())
            })
            .collect();

        let mut stale_topics = Vec::new();
        for topic_name in affected {
            let retention = self.contexts[&topic_name].retention;
            match retention {
                Retention::Session => {
                    self.contexts.remove(&topic_name);
                }
                Retention::Retained => {
                    if let Some(record) = self.contexts.get_mut(&topic_name) {
                        record.active_session = None;
                        record.force_stale = true;
                        if !record.stale_notified {
                            record.stale_notified = true;
                            stale_topics.push(topic_name);
                        }
                    }
                }
            }
        }

        stale_topics
            .into_iter()
            .flat_map(|topic_name| self.deliveries_for_topic(&topic_name, now_ms))
            .collect()
    }

    /// Emits a single stale transition when a context TTL expires.
    pub fn maintenance(&mut self, now_ms: u64) -> Vec<Delivery> {
        let mut became_stale = Vec::new();

        for (topic_name, record) in &mut self.contexts {
            if freshness(record, now_ms) == Freshness::Stale && !record.stale_notified {
                record.stale_notified = true;
                became_stale.push(topic_name.clone());
            }
        }

        became_stale
            .into_iter()
            .flat_map(|topic_name| self.deliveries_for_topic(&topic_name, now_ms))
            .collect()
    }

    fn declare(
        &mut self,
        session: &Session,
        topic_name: String,
        schema: Option<String>,
        value: Value,
        access: AccessPolicy,
        retention: Retention,
        ttl: Option<u64>,
        applied_revision: Option<Revision>,
        now_ms: u64,
    ) -> Result<Vec<Delivery>, BusError> {
        validate_topic(&topic_name)?;
        validate_schema(schema.as_deref())?;
        validate_access(&access)?;
        validate_ttl(ttl)?;
        self.validate_value_size(&value)?;

        if !self
            .policy
            .allows_topic(&session.identity, Operation::Declare, &topic_name)
        {
            return Err(BusError::new(
                ErrorCode::Forbidden,
                "system policy does not permit ownership of this topic",
            ));
        }

        if let Some(existing) = self.contexts.get_mut(&topic_name) {
            if existing.retention == Retention::Retained
                && existing.active_session.is_none()
                && existing.owner_id == session.identity.id
            {
                if existing.schema != schema {
                    return Err(BusError::new(
                        ErrorCode::SchemaImmutable,
                        "remove the retained context before changing its schema",
                    ));
                }
                if existing.retention != retention {
                    return Err(BusError::new(
                        ErrorCode::BadRequest,
                        "retention mode is immutable while a context exists",
                    ));
                }

                existing.revision = next_revision(existing.revision)?;
                existing.active_session = Some(session.id);
                existing.value = value;
                existing.access = access;
                existing.ttl = ttl;
                existing.updated_ms = now_ms;
                existing.applied_revision = applied_revision;
                existing.force_stale = false;
                existing.stale_notified = false;

                return Ok(self.deliveries_for_topic(&topic_name, now_ms));
            }

            return Err(BusError::new(
                ErrorCode::AlreadyExists,
                "context already has an active owner",
            ));
        }

        if self.contexts.len() >= self.limits.max_contexts {
            return Err(BusError::new(
                ErrorCode::RateLimited,
                "maximum number of contexts reached",
            ));
        }

        self.contexts.insert(
            topic_name.clone(),
            ContextRecord {
                topic: topic_name.clone(),
                owner_id: session.identity.id.clone(),
                active_session: Some(session.id),
                schema,
                value,
                access,
                retention,
                ttl,
                revision: 1,
                updated_ms: now_ms,
                applied_revision,
                force_stale: false,
                stale_notified: false,
            },
        );

        Ok(self.deliveries_for_topic(&topic_name, now_ms))
    }

    fn set(
        &mut self,
        session: &Session,
        topic_name: &str,
        value: Value,
        schema: Option<String>,
        applied_revision: Option<Revision>,
        now_ms: u64,
    ) -> Result<Vec<Delivery>, BusError> {
        validate_topic(topic_name)?;
        validate_schema(schema.as_deref())?;
        self.validate_value_size(&value)?;

        let record = self
            .contexts
            .get_mut(topic_name)
            .ok_or_else(|| BusError::new(ErrorCode::NotFound, "context does not exist"))?;

        if record.owner_id != session.identity.id || record.active_session != Some(session.id) {
            return Err(BusError::new(
                ErrorCode::NotOwner,
                "only the active context owner may update it",
            ));
        }

        if let Some(requested_schema) = schema {
            if record.schema.as_deref() != Some(requested_schema.as_str()) {
                return Err(BusError::new(
                    ErrorCode::SchemaImmutable,
                    "schema cannot be changed by SET; remove and redeclare the context",
                ));
            }
        }

        record.revision = next_revision(record.revision)?;
        record.value = value;
        record.updated_ms = now_ms;
        record.applied_revision = applied_revision;
        record.force_stale = false;
        record.stale_notified = false;

        Ok(self.deliveries_for_topic(topic_name, now_ms))
    }

    fn get(
        &self,
        session: &Session,
        topic_name: &str,
        now_ms: u64,
    ) -> Result<ContextState, BusError> {
        validate_topic(topic_name)?;
        let record = self
            .contexts
            .get(topic_name)
            .ok_or_else(|| BusError::new(ErrorCode::NotFound, "context does not exist"))?;

        if !self.can_read(&session.identity, record) {
            return Err(BusError::new(
                ErrorCode::Forbidden,
                "access to this context is not permitted",
            ));
        }

        Ok(state_from_record(record, now_ms))
    }

    fn discover(
        &self,
        session: &Session,
        filter: &str,
        now_ms: u64,
    ) -> Result<Vec<DiscoveredContext>, BusError> {
        validate_filter(filter)?;

        if !self
            .policy
            .allows_filter(&session.identity, Operation::Read, filter)
        {
            return Err(BusError::new(
                ErrorCode::Forbidden,
                "system policy does not permit discovery of this namespace",
            ));
        }

        let mut contexts: Vec<_> = self
            .contexts
            .values()
            .filter(|record| topic::matches(filter, &record.topic))
            .filter(|record| self.can_read(&session.identity, record))
            .map(|record| DiscoveredContext {
                topic: record.topic.clone(),
                revision: record.revision,
                freshness: freshness(record, now_ms),
                schema: record.schema.clone(),
            })
            .collect();
        contexts.sort_by(|left, right| left.topic.cmp(&right.topic));
        Ok(contexts)
    }

    fn subscribe(
        &mut self,
        session: &Session,
        filter: String,
        now_ms: u64,
    ) -> Result<(SubscriptionId, Vec<Delivery>), BusError> {
        validate_filter(&filter)?;

        if !self
            .policy
            .allows_filter(&session.identity, Operation::Read, &filter)
        {
            return Err(BusError::new(
                ErrorCode::Forbidden,
                "system policy does not permit subscription to this namespace",
            ));
        }

        let count = self
            .subscriptions
            .values()
            .filter(|subscription| subscription.session_id == session.id)
            .count();
        if count >= self.limits.max_subscriptions_per_session {
            return Err(BusError::new(
                ErrorCode::RateLimited,
                "maximum subscriptions for this session reached",
            ));
        }

        let subscription_id = self.next_subscription_id;
        self.next_subscription_id = self
            .next_subscription_id
            .checked_add(1)
            .ok_or_else(|| BusError::new(ErrorCode::InternalError, "subscription id exhausted"))?;

        self.subscriptions.insert(
            subscription_id,
            Subscription {
                id: subscription_id,
                session_id: session.id,
                identity: session.identity.clone(),
                filter: filter.clone(),
            },
        );

        // A subscription is valid even when no context currently exists. Any
        // existing authorized states are delivered immediately as UPDATEs.
        let deliveries = self
            .contexts
            .values()
            .filter(|record| topic::matches(&filter, &record.topic))
            .filter(|record| self.can_read(&session.identity, record))
            .map(|record| Delivery {
                session_id: session.id,
                response: Response::Update {
                    subscription: subscription_id,
                    state: state_from_record(record, now_ms),
                },
            })
            .collect();

        Ok((subscription_id, deliveries))
    }

    fn unsubscribe(
        &mut self,
        session: &Session,
        subscription_id: SubscriptionId,
    ) -> Result<(), BusError> {
        let subscription = self
            .subscriptions
            .get(&subscription_id)
            .ok_or_else(|| BusError::new(ErrorCode::NotFound, "subscription does not exist"))?;

        if subscription.session_id != session.id {
            return Err(BusError::new(
                ErrorCode::Forbidden,
                "subscription belongs to another session",
            ));
        }

        self.subscriptions.remove(&subscription_id);
        Ok(())
    }

    fn remove(&mut self, session: &Session, topic_name: &str) -> Result<(), BusError> {
        validate_topic(topic_name)?;
        let record = self
            .contexts
            .get(topic_name)
            .ok_or_else(|| BusError::new(ErrorCode::NotFound, "context does not exist"))?;

        if record.owner_id != session.identity.id || record.active_session != Some(session.id) {
            return Err(BusError::new(
                ErrorCode::NotOwner,
                "only the active context owner may remove it",
            ));
        }

        self.contexts.remove(topic_name);
        Ok(())
    }

    fn deliveries_for_topic(&self, topic_name: &str, now_ms: u64) -> Vec<Delivery> {
        let Some(record) = self.contexts.get(topic_name) else {
            return Vec::new();
        };
        let state = state_from_record(record, now_ms);

        self.subscriptions
            .values()
            .filter(|subscription| topic::matches(&subscription.filter, topic_name))
            .filter(|subscription| self.can_read(&subscription.identity, record))
            .map(|subscription| Delivery {
                session_id: subscription.session_id,
                response: Response::Update {
                    subscription: subscription.id,
                    state: state.clone(),
                },
            })
            .collect()
    }

    fn can_read(&self, identity: &Identity, record: &ContextRecord) -> bool {
        if !self
            .policy
            .allows_topic(identity, Operation::Read, &record.topic)
        {
            return false;
        }

        if identity.id == record.owner_id {
            return true;
        }

        record
            .access
            .read
            .iter()
            .any(|selector| selector_matches(selector, identity))
    }

    fn validate_value_size(&self, value: &Value) -> Result<(), BusError> {
        let length = serde_json::to_vec(value)
            .map_err(|_| BusError::new(ErrorCode::BadRequest, "invalid JSON context value"))?
            .len();
        if length > self.limits.max_context_bytes {
            return Err(BusError::new(
                ErrorCode::PayloadTooLarge,
                format!(
                    "context payload is {length} bytes; maximum is {}",
                    self.limits.max_context_bytes
                ),
            ));
        }
        Ok(())
    }
}

fn freshness(record: &ContextRecord, now_ms: u64) -> Freshness {
    if record.force_stale {
        return Freshness::Stale;
    }

    match record.ttl {
        Some(ttl_seconds)
            if now_ms.saturating_sub(record.updated_ms) >= ttl_seconds.saturating_mul(1_000) =>
        {
            Freshness::Stale
        }
        _ => Freshness::Fresh,
    }
}

fn state_from_record(record: &ContextRecord, now_ms: u64) -> ContextState {
    ContextState {
        topic: record.topic.clone(),
        revision: record.revision,
        timestamp_ms: record.updated_ms,
        freshness: freshness(record, now_ms),
        schema: record.schema.clone(),
        applied_revision: record.applied_revision,
        value: record.value.clone(),
    }
}

fn next_revision(revision: Revision) -> Result<Revision, BusError> {
    revision
        .checked_add(1)
        .ok_or_else(|| BusError::new(ErrorCode::InternalError, "context revision exhausted"))
}

fn validate_topic(topic_name: &str) -> Result<(), BusError> {
    topic::validate_topic_name(topic_name).map_err(|error| {
        BusError::new(
            ErrorCode::InvalidTopic,
            format!("invalid topic '{topic_name}': {error}"),
        )
    })
}

fn validate_filter(filter: &str) -> Result<(), BusError> {
    topic::validate_filter(filter).map_err(|error| {
        BusError::new(
            ErrorCode::InvalidTopic,
            format!("invalid topic filter '{filter}': {error}"),
        )
    })
}

fn validate_schema(schema: Option<&str>) -> Result<(), BusError> {
    if let Some(schema) = schema {
        if schema.is_empty() || schema.len() > 256 {
            return Err(BusError::new(
                ErrorCode::InvalidSchema,
                "schema identifier must contain 1..=256 bytes",
            ));
        }
    }
    Ok(())
}

fn validate_access(access: &AccessPolicy) -> Result<(), BusError> {
    for selector in &access.read {
        validate_selector(selector)
            .map_err(|error| BusError::new(ErrorCode::BadRequest, error.to_string()))?;
    }
    Ok(())
}

fn validate_ttl(ttl: Option<u64>) -> Result<(), BusError> {
    if ttl == Some(0) {
        return Err(BusError::new(
            ErrorCode::BadRequest,
            "ttl must be greater than zero when specified",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use statelink_protocol::{AccessPolicy, ErrorCode, Request, Response, Retention};

    use super::*;
    use crate::policy::{PolicyRule, RulePolicy, RulePolicyConfig};

    fn policy() -> Arc<RulePolicy> {
        Arc::new(
            RulePolicy::from_config(RulePolicyConfig {
                rules: vec![
                    PolicyRule {
                        subjects: vec!["id:camera-service".into()],
                        operations: vec![Operation::Declare, Operation::Read],
                        topic_filter: "camera/#".into(),
                    },
                    PolicyRule {
                        subjects: vec!["id:config-service".into()],
                        operations: vec![Operation::Declare, Operation::Read],
                        topic_filter: "camera/#".into(),
                    },
                    PolicyRule {
                        subjects: vec!["role:hmi".into(), "role:observer".into()],
                        operations: vec![Operation::Read],
                        topic_filter: "camera/#".into(),
                    },
                ],
            })
            .unwrap(),
        )
    }

    fn camera() -> Session {
        Session::new(1, Identity::new("camera-service", ["device"]))
    }

    fn hmi() -> Session {
        Session::new(2, Identity::new("hmi-1", ["hmi"]))
    }

    fn observer() -> Session {
        Session::new(3, Identity::new("observer-1", ["observer"]))
    }

    fn declare_status(bus: &mut Bus, retention: Retention) -> HandleResult {
        bus.handle_request(
            &camera(),
            Request::Declare {
                id: 1,
                topic: "camera/CAM01/status".into(),
                schema: Some("camera-status/v1".into()),
                value: json!({"online": true}),
                access: AccessPolicy {
                    read: vec!["role:hmi".into()],
                },
                retention,
                ttl: Some(30),
                applied_revision: None,
            },
            1_000,
        )
    }

    #[test]
    fn producer_and_system_policy_are_both_enforced() {
        let mut bus = Bus::new(policy());
        declare_status(&mut bus, Retention::Session);

        let allowed = bus.handle_request(
            &hmi(),
            Request::Get {
                id: 2,
                topic: "camera/CAM01/status".into(),
            },
            2_000,
        );
        assert!(matches!(allowed.response, Response::State { .. }));

        let blocked = bus.handle_request(
            &observer(),
            Request::Get {
                id: 3,
                topic: "camera/CAM01/status".into(),
            },
            2_000,
        );
        assert!(matches!(
            blocked.response,
            Response::Error {
                code: ErrorCode::Forbidden,
                ..
            }
        ));
    }

    #[test]
    fn schema_is_immutable() {
        let mut bus = Bus::new(policy());
        declare_status(&mut bus, Retention::Session);

        let result = bus.handle_request(
            &camera(),
            Request::Set {
                id: 2,
                topic: "camera/CAM01/status".into(),
                value: json!({"online": false}),
                schema: Some("camera-status/v2".into()),
                applied_revision: None,
            },
            2_000,
        );

        assert!(matches!(
            result.response,
            Response::Error {
                code: ErrorCode::SchemaImmutable,
                ..
            }
        ));
    }

    #[test]
    fn subscription_can_exist_before_context() {
        let mut bus = Bus::new(policy());
        let subscription = bus.handle_request(
            &hmi(),
            Request::Subscribe {
                id: 10,
                filter: "camera/+/status".into(),
            },
            1_000,
        );
        assert!(subscription.deliveries.is_empty());
        assert!(matches!(
            subscription.response,
            Response::Ok {
                subscription: Some(_),
                ..
            }
        ));

        let declared = declare_status(&mut bus, Retention::Session);
        assert_eq!(declared.deliveries.len(), 1);
        assert_eq!(declared.deliveries[0].session_id, hmi().id);
        assert!(matches!(declared.deliveries[0].response, Response::Update { .. }));
    }

    #[test]
    fn ttl_transitions_to_stale_once() {
        let mut bus = Bus::new(policy());
        declare_status(&mut bus, Retention::Session);
        bus.handle_request(
            &hmi(),
            Request::Subscribe {
                id: 2,
                filter: "camera/#".into(),
            },
            1_000,
        );

        assert!(bus.maintenance(30_999).is_empty());
        let stale = bus.maintenance(31_000);
        assert_eq!(stale.len(), 1);
        assert!(matches!(
            &stale[0].response,
            Response::Update { state, .. } if state.freshness == Freshness::Stale
        ));
        assert!(bus.maintenance(40_000).is_empty());
    }

    #[test]
    fn session_context_is_removed_when_owner_disconnects() {
        let mut bus = Bus::new(policy());
        declare_status(&mut bus, Retention::Session);
        bus.disconnect_session(camera().id, 2_000);

        let result = bus.handle_request(
            &hmi(),
            Request::Get {
                id: 2,
                topic: "camera/CAM01/status".into(),
            },
            2_000,
        );
        assert!(matches!(
            result.response,
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
    }

    #[test]
    fn retained_context_becomes_stale_and_can_be_reclaimed() {
        let mut bus = Bus::new(policy());
        declare_status(&mut bus, Retention::Retained);
        bus.disconnect_session(camera().id, 2_000);

        let stale = bus.handle_request(
            &hmi(),
            Request::Get {
                id: 2,
                topic: "camera/CAM01/status".into(),
            },
            2_000,
        );
        assert!(matches!(
            stale.response,
            Response::State { state, .. } if state.freshness == Freshness::Stale
        ));

        let new_camera_session = Session::new(99, camera().identity);
        let reclaimed = bus.handle_request(
            &new_camera_session,
            Request::Declare {
                id: 3,
                topic: "camera/CAM01/status".into(),
                schema: Some("camera-status/v1".into()),
                value: json!({"online": true}),
                access: AccessPolicy {
                    read: vec!["role:hmi".into()],
                },
                retention: Retention::Retained,
                ttl: Some(30),
                applied_revision: None,
            },
            3_000,
        );
        assert!(matches!(reclaimed.response, Response::Ok { .. }));
    }

    #[test]
    fn applied_revision_is_preserved_in_reported_state() {
        let mut bus = Bus::new(policy());
        let config = Session::new(4, Identity::new("config-service", ["config"]));
        let declared = bus.handle_request(
            &config,
            Request::Declare {
                id: 1,
                topic: "camera/CAM01/config/reported".into(),
                schema: None,
                value: json!({"fps": 15}),
                access: AccessPolicy {
                    read: vec!["role:hmi".into()],
                },
                retention: Retention::Session,
                ttl: None,
                applied_revision: Some(27),
            },
            1_000,
        );
        assert!(matches!(declared.response, Response::Ok { .. }));

        let state = bus.handle_request(
            &hmi(),
            Request::Get {
                id: 2,
                topic: "camera/CAM01/config/reported".into(),
            },
            1_000,
        );
        assert!(matches!(
            state.response,
            Response::State { state, .. } if state.applied_revision == Some(27)
        ));
    }
}
