// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transport-independent StateLink state bus.

pub mod bus;
pub mod identity;
pub mod policy;

pub use bus::{Bus, BusError, Delivery, HandleResult, Limits};
pub use identity::{Identity, Session, SessionId};
pub use policy::{Operation, PolicyError, PolicyRule, RulePolicy, RulePolicyConfig, SystemPolicy};
