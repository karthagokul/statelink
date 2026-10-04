// Copyright (C) 2026 Gokul Kartha
// SPDX-License-Identifier: GPL-3.0-or-later

//! MQTT-compatible StateLink topic and filter helpers.

use thiserror::Error;

const MAX_TOPIC_BYTES: usize = 65_535;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TopicError {
    #[error("topic or filter must not be empty")]
    Empty,
    #[error("topic or filter exceeds {MAX_TOPIC_BYTES} bytes")]
    TooLong,
    #[error("topic or filter must not contain NUL")]
    ContainsNul,
    #[error("topic names must not contain MQTT wildcards")]
    WildcardInTopic,
    #[error("'+' must occupy an entire topic-filter level")]
    InvalidSingleLevelWildcard,
    #[error("'#' must occupy the final topic-filter level")]
    InvalidMultiLevelWildcard,
}

pub fn validate_topic_name(topic: &str) -> Result<(), TopicError> {
    validate_common(topic)?;
    if topic.contains('+') || topic.contains('#') {
        return Err(TopicError::WildcardInTopic);
    }
    Ok(())
}

pub fn validate_filter(filter: &str) -> Result<(), TopicError> {
    validate_common(filter)?;

    let levels: Vec<&str> = filter.split('/').collect();
    for (index, level) in levels.iter().enumerate() {
        if level.contains('+') && *level != "+" {
            return Err(TopicError::InvalidSingleLevelWildcard);
        }
        if level.contains('#') && (*level != "#" || index != levels.len() - 1) {
            return Err(TopicError::InvalidMultiLevelWildcard);
        }
    }

    Ok(())
}

pub fn matches(filter: &str, topic: &str) -> bool {
    if validate_filter(filter).is_err() || validate_topic_name(topic).is_err() {
        return false;
    }

    let filter_levels: Vec<&str> = filter.split('/').collect();
    let topic_levels: Vec<&str> = topic.split('/').collect();

    let mut fi = 0;
    let mut ti = 0;

    while fi < filter_levels.len() {
        match filter_levels[fi] {
            "#" => return true,
            "+" => {
                if ti >= topic_levels.len() {
                    return false;
                }
                fi += 1;
                ti += 1;
            }
            literal => {
                if ti >= topic_levels.len() || literal != topic_levels[ti] {
                    return false;
                }
                fi += 1;
                ti += 1;
            }
        }
    }

    ti == topic_levels.len()
}

/// Returns true when every topic matched by `requested` is also matched by `allowed`.
///
/// StateLink uses this when authorizing future subscriptions. It intentionally
/// supports only MQTT `+` and `#` wildcard semantics.
pub fn filter_covers(allowed: &str, requested: &str) -> bool {
    if validate_filter(allowed).is_err() || validate_filter(requested).is_err() {
        return false;
    }

    let a: Vec<&str> = allowed.split('/').collect();
    let r: Vec<&str> = requested.split('/').collect();

    let mut ai = 0;
    let mut ri = 0;

    loop {
        if ai == a.len() && ri == r.len() {
            return true;
        }

        if ai < a.len() && a[ai] == "#" {
            return true;
        }

        if ri == r.len() {
            return ai + 1 == a.len() && a[ai] == "#";
        }

        if ai == a.len() {
            return false;
        }

        if r[ri] == "#" {
            return false;
        }

        match a[ai] {
            "+" => {
                // A single-level wildcard can cover a requested literal or '+',
                // but never a requested multi-level wildcard (handled above).
                ai += 1;
                ri += 1;
            }
            literal => {
                // A literal cannot cover a broader requested '+'.
                if r[ri] == "+" || literal != r[ri] {
                    return false;
                }
                ai += 1;
                ri += 1;
            }
        }
    }
}

fn validate_common(value: &str) -> Result<(), TopicError> {
    if value.is_empty() {
        return Err(TopicError::Empty);
    }
    if value.len() > MAX_TOPIC_BYTES {
        return Err(TopicError::TooLong);
    }
    if value.contains('\0') {
        return Err(TopicError::ContainsNul);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mqtt_style_matching_works() {
        assert!(matches("camera/+/status", "camera/CAM01/status"));
        assert!(matches("camera/#", "camera/CAM01/status"));
        assert!(matches("#", "system/health"));
        assert!(!matches("camera/+/status", "camera/CAM01/config/desired"));
    }

    #[test]
    fn invalid_filters_are_rejected() {
        assert_eq!(
            validate_filter("camera/foo+bar/status"),
            Err(TopicError::InvalidSingleLevelWildcard)
        );
        assert_eq!(
            validate_filter("camera/#/status"),
            Err(TopicError::InvalidMultiLevelWildcard)
        );
    }

    #[test]
    fn filter_coverage_is_conservative() {
        assert!(filter_covers("camera/#", "camera/+/status"));
        assert!(filter_covers("camera/+/status", "camera/CAM01/status"));
        assert!(filter_covers("#", "system/#"));
        assert!(!filter_covers("camera/CAM01/#", "camera/+/status"));
        assert!(!filter_covers("camera/+/status", "camera/#"));
        assert!(!filter_covers("system/health", "system/+"));
    }

    #[test]
    fn topic_names_cannot_contain_wildcards() {
        assert_eq!(
            validate_topic_name("camera/+/status"),
            Err(TopicError::WildcardInTopic)
        );
    }
}
