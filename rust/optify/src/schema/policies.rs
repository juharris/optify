use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Error returned when a requester is not permitted to use a feature.
#[derive(Clone, Debug)]
pub struct PolicyDeniedError {
    pub feature_name: String,
    pub requester: String,
}

impl PolicyDeniedError {
    pub fn new(feature_name: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            feature_name: feature_name.into(),
            requester: requester.into(),
        }
    }
}

impl fmt::Display for PolicyDeniedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Requester \"{}\" is not permitted to use feature \"{}\". The requester is denied by the feature's policies.",
            self.requester, self.feature_name
        )
    }
}

impl std::error::Error for PolicyDeniedError {}

/// The policy for the requester identifier passed via preferences.
///
/// Either `allow` or `block` must be specified, not both.
///
/// - `allow`: Only the listed requesters may use this feature.
///   An empty set means no requester is currently allowed.
/// - `block`: The listed requesters may not use this feature.
///   All other requesters are allowed.
///
/// See https://github.com/juharris/optify#policies for more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequesterPolicy {
    Allow {
        allow: HashSet<String>,
    },
    Block {
        block: HashSet<String>,
    },
}

impl RequesterPolicy {
    /// Returns `true` if the given requester is permitted by this policy.
    pub fn is_permitted(&self, value: &str) -> bool {
        match self {
            Self::Allow { allow } => allow.contains(value),
            Self::Block { block } => !block.contains(value),
        }
    }
}

/// Policies that restrict access to a feature based on values in the request's preferences.
///
/// Policies are checked for the **top-level features** in a request.
/// Unlike conditions, policies are **not** evaluated on imported features — a feature may
/// freely import another feature that has policies without those policies being enforced.
///
/// See https://github.com/juharris/optify#policies for details and a comparison with conditions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    /// Restrictions based on the requester identifier passed via preferences.
    ///
    /// Use `allow` to specify an allowlist (only those requesters may use this feature).
    /// Use `block` to specify a denylist (all requesters except those listed may use this feature).
    /// `allow` and `block` are mutually exclusive.
    pub requester: Option<RequesterPolicy>,
}

impl Policies {
    /// Returns `true` if the given requester is permitted to use the feature.
    ///
    /// If no `requester` policy is set, all requesters are permitted.
    pub fn is_requester_permitted(&self, requester: &str) -> bool {
        self.requester
            .as_ref()
            .map_or(true, |p| p.is_permitted(requester))
    }
}
