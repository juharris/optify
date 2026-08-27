use serde::{Deserialize, Serialize};

/// The policy for a specific string field in the request's preferences
/// (e.g. the requester identifier).
///
/// Either `allowed` or `blocked` must be specified, not both.
///
/// - `allowed`: Only the listed values may use the feature.
///   An empty list means no value is currently allowed.
/// - `blocked`: The listed values may not use the feature.
///   All other values are allowed. Must contain at least one entry.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringPolicy {
    Allowed {
        allowed: Vec<String>,
    },
    Blocked {
        blocked: Vec<String>,
    },
}

impl StringPolicy {
    /// Returns `true` if the given value is permitted by this policy.
    pub fn is_permitted(&self, value: &str) -> bool {
        match self {
            Self::Allowed { allowed } => allowed.iter().any(|s| s == value),
            Self::Blocked { blocked } => !blocked.iter().any(|s| s == value),
        }
    }
}

/// Policies that restrict access to a feature based on values in the request's preferences.
///
/// Policies are checked for the **top-level features** in a request.
/// Unlike conditions, policies are **not** evaluated on imported features — a feature may
/// freely import another feature that has policies without triggering those policies.
///
/// See the comparison table below for the key differences between policies and conditions.
///
/// | Aspect | Conditions | Policies |
/// |---|---|---|
/// | Purpose | Silently filter out features based on request constraints | Restrict access to a feature; violations should surface as errors |
/// | When evaluated | During filtering — the feature is quietly excluded if conditions are not met | At the start of a request — a violation should be reported to the caller |
/// | Effect | Feature is silently omitted from the result | Feature is rejected; the requester is not permitted to use it |
/// | Applied to | Top-level features in the request | Top-level features in the request only |
/// | Imported features | **Not allowed**: a feature with conditions cannot be imported | **Allowed**: a feature with policies can be imported freely; policies are not inherited |
/// | Data source | `constraints` on the request | `requester` (or other fields) in `preferences` |
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    /// Restrictions based on the requester identifier passed via preferences.
    ///
    /// Use `allowed` to specify an allowlist (only those requesters may use this feature).
    /// Use `blocked` to specify a denylist (all requesters except those listed may use this feature).
    /// `allowed` and `blocked` are mutually exclusive.
    pub requester: Option<StringPolicy>,
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
