use std::collections::HashSet;
use std::path::Path;

use crate::provider::{Aliases, Features};

use super::feature_policies::{Policies, PoliciesMap};
use super::policies_file_contents::PoliciesFileContents;
use super::policy_denied_error::PolicyDeniedError;
use super::requester_feature_policy::{RequesterFeaturePolicy, RequesterPoliciesMap};
use super::requester_policy::RequesterPolicy;
use super::validate_requester_policies::validate_requester_policies;

/// A policy store holding feature policies and requester policies.
/// Responsible for all policy-related logic in the provider and builder.
#[derive(Clone, Debug, Default)]
pub(crate) struct PolicyStore {
    policies: PoliciesMap,
    requester_policies: RequesterPoliciesMap,
}

impl PolicyStore {
    #[allow(dead_code)]
    pub(crate) fn new(policies: PoliciesMap, requester_policies: RequesterPoliciesMap) -> Self {
        Self {
            policies,
            requester_policies,
        }
    }

    /// Checks policies for a list of features for the given requester.
    pub(crate) fn check_policies(
        &self,
        requester: &str,
        canonical_feature_names: &[impl AsRef<str>],
    ) -> Result<(), String> {
        let requester_policy = self.requester_policies.get(requester);
        for feature_name in canonical_feature_names {
            let canonical = feature_name.as_ref();
            if let Some(requester_policy) = requester_policy {
                if !requester_policy.is_permitted(canonical) {
                    return Err(PolicyDeniedError::new(canonical, requester).to_string());
                }
            }
            if let Some(policies) = self.policies.get(canonical) {
                if !policies.is_requester_permitted(requester) {
                    return Err(PolicyDeniedError::new(canonical, requester).to_string());
                }
            }
        }
        Ok(())
    }

    /// Returns the effective policies for a feature by combining the feature's own policies
    /// with the requester policies from `.optify/policies.json`.
    ///
    /// The result uses the same shape as the feature's own policies:
    /// - `allow`: the feature has its own `allow` list, so only those requesters may use it,
    ///   minus any requester that `.optify/policies.json` denies for this feature.
    /// - `block`: every requester except those listed may use the feature. The list is the feature's
    ///   own `block` list plus every requester whose `.optify/policies.json` policy denies the feature.
    ///   This includes requesters with an `allow` list that does not mention the feature.
    ///
    /// Returns `None` if nothing restricts the feature, meaning every requester is permitted.
    /// Conflicts between the two sources are already rejected when loading,
    /// so this doesn't check for them.
    ///
    /// `canonical_feature_name` should be a known canonical feature name.
    pub(crate) fn get_effective_policies(&self, canonical_feature_name: &str) -> Option<Policies> {
        let denied_by_requester_policies: HashSet<String> = self
            .requester_policies
            .iter()
            .filter(|(_, policy)| !policy.is_permitted(canonical_feature_name))
            .map(|(requester, _)| requester.clone())
            .collect();

        let requester = match self
            .policies
            .get(canonical_feature_name)
            .map(|policies| &policies.requester)
        {
            Some(RequesterPolicy::Allow { allow }) => RequesterPolicy::Allow {
                allow: allow
                    .difference(&denied_by_requester_policies)
                    .cloned()
                    .collect(),
            },
            Some(RequesterPolicy::Block { block }) => RequesterPolicy::Block {
                block: block
                    .union(&denied_by_requester_policies)
                    .cloned()
                    .collect(),
            },
            None if denied_by_requester_policies.is_empty() => return None,
            None => RequesterPolicy::Block {
                block: denied_by_requester_policies,
            },
        };
        Some(Policies { requester })
    }

    pub(crate) fn insert_policy(&mut self, canonical_feature_name: String, policies: Policies) {
        self.policies.insert(canonical_feature_name, policies);
    }

    pub(crate) fn insert_requester_policy(
        &mut self,
        requester: String,
        policy: RequesterFeaturePolicy,
    ) -> Option<RequesterFeaturePolicy> {
        self.requester_policies.insert(requester, policy)
    }

    /// Loads policies from the given file and inserts them into the store.
    pub(crate) fn load_policies_from_file(
        &mut self,
        directory: &Path,
        policies_path: &Path,
        config_path: &Path,
    ) -> Result<(), String> {
        let resolved_policies_path = directory.join(policies_path);
        if !resolved_policies_path.is_file() {
            return Err(format!(
                "Error loading policies: '{}' declared via 'policiesPath' in {} is not a file.",
                resolved_policies_path.display(),
                config_path.display()
            ));
        }
        let file = config::File::from(resolved_policies_path.as_path());
        let policies_config = config::Config::builder()
            .add_source(file)
            .build()
            .map_err(|e| {
                format!(
                    "Error loading policies from {}: {e}",
                    resolved_policies_path.display()
                )
            })?;
        let policies_file: PoliciesFileContents =
            policies_config.try_deserialize().map_err(|e| {
                format!(
                    "Error deserializing policies from {}: {e}",
                    resolved_policies_path.display()
                )
            })?;
        for (requester, policy) in policies_file.requesters {
            if self
                .insert_requester_policy(requester.clone(), policy)
                .is_some()
            {
                return Err(format!(
                    "Error loading policies from {}: policies for requester '{requester}' were already defined.",
                    resolved_policies_path.display()
                ));
            }
        }
        Ok(())
    }

    /// Checks whether the requester is permitted for the given feature.
    ///
    /// Both `requester_policies` and feature `policies` must permit the requester.
    ///
    /// Returns `Ok(true)` if permitted, no policy is set, or no requester is given.
    /// Returns `Ok(false)` if denied and `raise_if_policy_denied` is false.
    /// Returns `Err(message)` if denied and `raise_if_policy_denied` is true.
    pub(crate) fn is_feature_permitted_for_requester(
        &self,
        canonical_feature_name: &str,
        requester: Option<&str>,
        raise_if_policy_denied: bool,
    ) -> Result<bool, String> {
        if let Some(requester) = requester {
            let permitted =
                self.is_requester_permitted_for_feature(canonical_feature_name, requester);
            if !permitted {
                if raise_if_policy_denied {
                    return Err(
                        PolicyDeniedError::new(canonical_feature_name, requester).to_string()
                    );
                }
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Checks whether the requester is permitted for the given feature.
    /// Both `requester_policies` and feature `policies` must permit the requester.
    pub(crate) fn is_requester_permitted_for_feature(
        &self,
        canonical_feature_name: &str,
        requester: &str,
    ) -> bool {
        if let Some(requester_policy) = self.requester_policies.get(requester) {
            if !requester_policy.is_permitted(canonical_feature_name) {
                return false;
            }
        }
        if let Some(policies) = self.policies.get(canonical_feature_name) {
            if !policies.is_requester_permitted(requester) {
                return false;
            }
        }
        true
    }

    /// Validates the requester policies against features and aliases loaded so far.
    pub(crate) fn validate_requester_policies(
        &self,
        features: &Features,
        aliases: &Aliases,
    ) -> Result<(), String> {
        validate_requester_policies(&self.requester_policies, features, aliases, &self.policies)
    }
}
