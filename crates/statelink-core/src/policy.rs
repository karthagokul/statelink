// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use statelink_protocol::topic;
use thiserror::Error;

use crate::identity::Identity;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Declare,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRule {
    pub subjects: Vec<String>,
    pub operations: Vec<Operation>,
    pub topic_filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulePolicyConfig {
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("invalid policy topic filter '{filter}': {message}")]
    InvalidFilter { filter: String, message: String },
    #[error("policy rule must contain at least one subject")]
    EmptySubjects,
    #[error("policy rule must contain at least one operation")]
    EmptyOperations,
    #[error("invalid subject selector '{0}'")]
    InvalidSubject(String),
}

pub trait SystemPolicy: Send + Sync {
    fn allows_topic(&self, identity: &Identity, operation: Operation, topic_name: &str) -> bool;
    fn allows_filter(&self, identity: &Identity, operation: Operation, filter: &str) -> bool;
}

/// A deliberately small, deny-by-default allow-list policy.
///
/// Producer exposure rules can only reduce access further; they cannot grant
/// access that this system policy does not already allow.
#[derive(Debug, Clone)]
pub struct RulePolicy {
    rules: Vec<PolicyRule>,
}

impl RulePolicy {
    pub fn from_config(config: RulePolicyConfig) -> Result<Self, PolicyError> {
        for rule in &config.rules {
            if rule.subjects.is_empty() {
                return Err(PolicyError::EmptySubjects);
            }
            if rule.operations.is_empty() {
                return Err(PolicyError::EmptyOperations);
            }
            topic::validate_filter(&rule.topic_filter).map_err(|error| {
                PolicyError::InvalidFilter {
                    filter: rule.topic_filter.clone(),
                    message: error.to_string(),
                }
            })?;
            for subject in &rule.subjects {
                validate_selector(subject)?;
            }
        }
        Ok(Self {
            rules: config.rules,
        })
    }
}

impl SystemPolicy for RulePolicy {
    fn allows_topic(&self, identity: &Identity, operation: Operation, topic_name: &str) -> bool {
        self.rules.iter().any(|rule| {
            rule.operations.contains(&operation)
                && rule
                    .subjects
                    .iter()
                    .any(|selector| selector_matches(selector, identity))
                && topic::matches(&rule.topic_filter, topic_name)
        })
    }

    fn allows_filter(&self, identity: &Identity, operation: Operation, filter: &str) -> bool {
        self.rules.iter().any(|rule| {
            rule.operations.contains(&operation)
                && rule
                    .subjects
                    .iter()
                    .any(|selector| selector_matches(selector, identity))
                && topic::filter_covers(&rule.topic_filter, filter)
        })
    }
}

pub fn validate_selector(selector: &str) -> Result<(), PolicyError> {
    if selector == "*" {
        return Ok(());
    }

    let Some((kind, pattern)) = selector.split_once(':') else {
        return Err(PolicyError::InvalidSubject(selector.to_owned()));
    };

    if !matches!(kind, "id" | "service" | "role") || pattern.is_empty() {
        return Err(PolicyError::InvalidSubject(selector.to_owned()));
    }

    Ok(())
}

pub fn selector_matches(selector: &str, identity: &Identity) -> bool {
    if selector == "*" {
        return true;
    }

    let Some((kind, pattern)) = selector.split_once(':') else {
        return false;
    };

    match kind {
        "id" | "service" => glob_match(pattern, &identity.id),
        "role" => identity.roles.iter().any(|role| glob_match(pattern, role)),
        _ => false,
    }
}

/// Small wildcard matcher for identity selectors. `*` matches any sequence.
fn glob_match(pattern: &str, value: &str) -> bool {
    let p = pattern.as_bytes();
    let v = value.as_bytes();

    let mut pi = 0usize;
    let mut vi = 0usize;
    let mut star = None;
    let mut checkpoint = 0usize;

    while vi < v.len() {
        if pi < p.len() && p[pi] == v[vi] {
            pi += 1;
            vi += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            checkpoint = vi;
        } else if let Some(star_index) = star {
            pi = star_index + 1;
            checkpoint += 1;
            vi = checkpoint;
        } else {
            return false;
        }
    }

    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }

    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity::new("camera-service", ["producer", "device"])
    }

    #[test]
    fn selectors_match_ids_and_roles() {
        let identity = identity();
        assert!(selector_matches("id:camera-*", &identity));
        assert!(selector_matches("service:camera-service", &identity));
        assert!(selector_matches("role:prod*", &identity));
        assert!(!selector_matches("role:hmi", &identity));
    }

    #[test]
    fn policy_is_deny_by_default() {
        let policy = RulePolicy::from_config(RulePolicyConfig::default()).unwrap();
        assert!(!policy.allows_topic(&identity(), Operation::Read, "camera/CAM01/status"));
    }

    #[test]
    fn allowed_filter_must_cover_requested_subscription() {
        let policy = RulePolicy::from_config(RulePolicyConfig {
            rules: vec![PolicyRule {
                subjects: vec!["role:device".into()],
                operations: vec![Operation::Read],
                topic_filter: "camera/+/status".into(),
            }],
        })
        .unwrap();

        assert!(policy.allows_filter(&identity(), Operation::Read, "camera/CAM01/status"));
        assert!(policy.allows_filter(&identity(), Operation::Read, "camera/+/status"));
        assert!(!policy.allows_filter(&identity(), Operation::Read, "camera/#"));
    }
}
