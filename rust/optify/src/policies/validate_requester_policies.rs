use crate::provider::{Aliases, Features};

use super::feature_policies::{Policies, PoliciesMap};
use super::requester_feature_policy::{RequesterFeaturePolicy, RequesterPoliciesMap};
use super::requester_policy::RequesterPolicy;

/// Validates the requester policies declared in `.optify/policies.json` files against the
/// features loaded so far.
///
/// - Every feature name referenced must be a canonical feature name.
/// - A requester policy that explicitly grants a requester access to a feature while the
///   feature's own `policies.requester` does not permit that requester (whether the feature
///   explicitly blocks the requester or only explicitly allows other requesters) is a
///   conflict and causes an error, since it is likely a mistake.
/// - A requester policy that explicitly blocks a requester from a feature while the
///   feature's own `policies.requester` explicitly allows that requester is also a conflict.
///
/// Both `.optify/policies.json` and each feature's own `policies.requester` are
/// independently configurable and are both checked at runtime (see
/// `PolicyStore::is_requester_permitted_for_feature`); this only validates that they
/// don't explicitly contradict each other.
/// Nothing is merged here.
pub(crate) fn validate_requester_policies(
    requester_policies: &RequesterPoliciesMap,
    features: &Features,
    aliases: &Aliases,
    policies: &PoliciesMap,
) -> Result<(), String> {
    for (requester, policy) in requester_policies {
        for feature_name in policy.feature_names() {
            if features.contains_key(feature_name) {
                continue;
            }
            let uni_case_feature_name = unicase::UniCase::new(feature_name.clone());
            if let Some(canonical_feature_name) = aliases.get(&uni_case_feature_name) {
                return Err(format!(
                    "Error validating policies for requester '{requester}': '{feature_name}' is an alias for canonical feature name '{canonical_feature_name}'. Policies must use canonical feature names for clarity and easier navigation."
                ));
            }
            return Err(format!(
                "Error validating policies for requester '{requester}': feature '{feature_name}' does not exist."
            ));
        }

        match policy {
            RequesterFeaturePolicy::Allow { allow } => {
                for feature_name in allow {
                    if let Some(policies) = policies.get(feature_name) {
                        if !policies.is_requester_permitted(requester) {
                            return Err(format!(
                                "Conflicting policies for requester '{requester}' and feature '{feature_name}': '.optify/policies.json' allows it, but the feature's own policies do not permit this requester."
                            ));
                        }
                    }
                }
            }
            RequesterFeaturePolicy::Block { block } => {
                for feature_name in block {
                    if let Some(Policies {
                        requester: Some(RequesterPolicy::Allow { allow }),
                    }) = policies.get(feature_name)
                    {
                        if allow.contains(requester) {
                            return Err(format!(
                                "Conflicting policies for requester '{requester}' and feature '{feature_name}': '.optify/policies.json' blocks it, but the feature's own policies explicitly allow it."
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
