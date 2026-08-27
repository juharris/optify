use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// The policy for the requester identifier passed via preferences.
///
/// Either `allowed` or `blocked` must be specified, not both.
///
/// - `allowed`: Only the listed requesters may use this feature.
///   An empty set means no requester is currently allowed.
/// - `blocked`: The listed requesters may not use this feature.
///   All other requesters are allowed.
///
/// See https://github.com/juharris/optify#policies for more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequesterPolicy {
    Allowed {
        allowed: HashSet<String>,
    },
    Blocked {
        blocked: HashSet<String>,
    },
}

impl RequesterPolicy {
    /// Returns `true` if the given requester is permitted by this policy.
    pub fn is_permitted(&self, value: &str) -> bool {
        match self {
            Self::Allowed { allowed } => allowed.contains(value),
            Self::Blocked { blocked } => !blocked.contains(value),
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
    /// Use `allowed` to specify an allowlist (only those requesters may use this feature).
    /// Use `blocked` to specify a denylist (all requesters except those listed may use this feature).
    /// `allowed` and `blocked` are mutually exclusive.
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
