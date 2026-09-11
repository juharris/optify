use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::requester_policy::RequesterPolicy;

pub(crate) type PoliciesMap = HashMap<String, Policies>;

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
    pub requester: RequesterPolicy,
}

impl Policies {
    /// Returns `true` if the given requester is permitted to use the feature.
    pub fn is_requester_permitted(&self, requester: &str) -> bool {
        self.requester.is_permitted(requester)
    }
}
