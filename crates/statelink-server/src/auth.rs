// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{BTreeSet, HashMap, HashSet};

use axum::http::{header, HeaderMap};
use serde::Deserialize;
use statelink_core::Identity;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthFile {
    pub tokens: HashMap<String, AuthIdentity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthIdentity {
    pub id: String,
    #[serde(default)]
    pub roles: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct TokenAuthenticator {
    tokens: HashMap<String, Identity>,
}

impl TokenAuthenticator {
    pub fn from_config(config: AuthFile) -> Self {
        let tokens = config
            .tokens
            .into_iter()
            .map(|(token, identity)| {
                (
                    token,
                    Identity {
                        id: identity.id,
                        roles: identity.roles,
                    },
                )
            })
            .collect();
        Self { tokens }
    }

    pub fn authenticate(&self, headers: &HeaderMap) -> Option<Identity> {
        let token = bearer_token(headers).or_else(|| cookie_token(headers))?;
        self.tokens.get(token).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct OriginPolicy {
    allowed: HashSet<String>,
}

impl OriginPolicy {
    pub fn new(origins: impl IntoIterator<Item = String>) -> Self {
        Self {
            allowed: origins
                .into_iter()
                .filter(|origin| !origin.is_empty())
                .collect(),
        }
    }

    /// Native clients normally omit Origin and are accepted. Browser clients
    /// send Origin and must match the explicit allow-list.
    pub fn allows(&self, headers: &HeaderMap) -> bool {
        match headers
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok())
        {
            None => true,
            Some(origin) => self.allowed.contains(origin),
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ")
}

fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == "statelink_token").then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn authenticator() -> TokenAuthenticator {
        TokenAuthenticator::from_config(AuthFile {
            tokens: HashMap::from([(
                "secret".into(),
                AuthIdentity {
                    id: "camera".into(),
                    roles: BTreeSet::from(["producer".into()]),
                },
            )]),
        })
    }

    #[test]
    fn bearer_authentication_works() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer secret"),
        );
        assert_eq!(authenticator().authenticate(&headers).unwrap().id, "camera");
    }

    #[test]
    fn cookie_authentication_supports_browser_sessions() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("other=x; statelink_token=secret"),
        );
        assert_eq!(authenticator().authenticate(&headers).unwrap().id, "camera");
    }

    #[test]
    fn browser_origin_is_deny_by_default() {
        let policy = OriginPolicy::new(["https://hmi.example".into()]);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.example"),
        );
        assert!(!policy.allows(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://hmi.example"),
        );
        assert!(policy.allows(&headers));
    }

    #[test]
    fn native_client_without_origin_is_allowed() {
        let policy = OriginPolicy::new(Vec::<String>::new());
        assert!(policy.allows(&HeaderMap::new()));
    }
}
