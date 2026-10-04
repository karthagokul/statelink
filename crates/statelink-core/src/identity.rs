// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub type SessionId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub id: String,
    #[serde(default)]
    pub roles: BTreeSet<String>,
}

impl Identity {
    pub fn new(id: impl Into<String>, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            id: id.into(),
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    pub identity: Identity,
}

impl Session {
    pub fn new(id: SessionId, identity: Identity) -> Self {
        Self { id, identity }
    }
}
