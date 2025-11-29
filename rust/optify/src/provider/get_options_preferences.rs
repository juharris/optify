use crate::provider::{constraints::Constraints, SourceValue};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GetOptionsPreferences {
    /// Allows resolving configurable strings.
    /// Defaults to false: no configurable strings will be resolved.
    /// Configurable strings must have been enabled when the options were built to have them resolved at runtime.
    pub are_configurable_strings_enabled: bool,
    pub constraints: Option<Constraints>,
    /// Overrides to apply after the built configuration.
    pub overrides: Option<SourceValue>,
    /// Determines if the feature names should be converted to canonical feature names.
    /// Defaults to false: given features names will be converted to canonical feature names before looking for features or options.
    pub skip_feature_name_conversion: bool,
}

impl Default for GetOptionsPreferences {
    fn default() -> Self {
        Self::new()
    }
}

impl GetOptionsPreferences {
    pub fn new() -> Self {
        Self {
            are_configurable_strings_enabled: false,
            constraints: None,
            overrides: None,
            skip_feature_name_conversion: false,
        }
    }

    pub fn set_constraints(&mut self, constraints: Option<serde_json::Value>) {
        self.constraints = constraints.map(|c| Constraints { constraints: c });
    }

    pub fn set_constraints_json(&mut self, constraints: Option<&str>) {
        self.constraints = constraints.map(|c| Constraints {
            constraints: serde_json::from_str(c).expect("constraints should be valid JSON"),
        });
    }
}
