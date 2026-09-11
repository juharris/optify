use crate::provider::{Aliases, Features};

use super::feature_policies::PoliciesMap;
use super::requester_feature_policy::RequesterPoliciesMap;

/// Validates the requester policies declared in `.optify/policies.json` files against the
/// features loaded so far.
///
/// - Every feature name referenced must be a canonical feature name.
/// - When either `.optify/policies.json` explicitly names a feature or a feature's own
///   `policies.requester` explicitly names a requester, the two policies must agree for
///   that requester/feature pair.
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
        // Make sure feature names in `requester_policies` are canonical and exist.
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

        for (feature_name, policies) in policies {
            let feature_policy = &policies.requester;
            if (policy.mentions_feature(feature_name)
                || feature_policy.mentions_requester(requester))
                && policy.is_permitted(feature_name) != feature_policy.is_permitted(requester)
            {
                return Err(format!(
                    "Conflicting policies for requester '{requester}' and feature '{feature_name}': '.optify/policies.json' and the feature's own policies disagree."
                ));
            }
        }
    }

    Ok(())
}
