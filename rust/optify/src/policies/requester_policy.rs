use std::collections::HashSet;

use serde::{Deserialize, Serialize};

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
    Allow { allow: HashSet<String> },
    Block { block: HashSet<String> },
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
