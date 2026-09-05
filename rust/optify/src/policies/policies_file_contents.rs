use serde::Deserialize;

use super::requester_feature_policy::RequesterPoliciesMap;

#[derive(Deserialize)]
pub(crate) struct PoliciesFileContents {
    // Helps ignore the `$schema` property when parsing.
    #[serde(rename = "$schema")]
    #[allow(dead_code)]
    pub(crate) schema: Option<String>,
    #[serde(flatten)]
    pub(crate) requesters: RequesterPoliciesMap,
}
