use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Maps a requester identifier to the policy that determines which canonical feature names
/// the requester is permitted to use, declared in `.optify/policies.json`.
pub(crate) type RequesterPoliciesMap = HashMap<String, RequesterFeaturePolicy>;

/// The policy for the canonical feature names that a requester is permitted to use, declared
/// per-requester in `.optify/policies.json`.
///
/// Either `allow` or `block` must be specified, not both.
///
/// - `allow`: Only the listed features may be used by the requester.
///   An empty set means no feature is currently allowed.
/// - `block`: The listed features may not be used by the requester.
///   All other features are allowed.
///
/// Feature names must be canonical feature names.
/// Aliases and non-existent feature names are not permitted and will cause the build to fail.
///
/// See https://github.com/juharris/optify#policies for more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequesterFeaturePolicy {
    Allow { allow: HashSet<String> },
    Block { block: HashSet<String> },
}

impl RequesterFeaturePolicy {
    /// Returns `true` if the given canonical feature name is permitted by this policy.
    pub fn is_permitted(&self, canonical_feature_name: &str) -> bool {
        match self {
            Self::Allow { allow } => allow.contains(canonical_feature_name),
            Self::Block { block } => !block.contains(canonical_feature_name),
        }
    }

    /// Returns an iterator over the feature names referenced by this policy.
    pub fn feature_names(&self) -> impl Iterator<Item = &String> {
        match self {
            Self::Allow { allow } => allow.iter(),
            Self::Block { block } => block.iter(),
        }
    }
}
