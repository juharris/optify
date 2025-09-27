use crate::provider::constraints::Constraints;

#[derive(Hash, PartialEq, Eq)]
pub struct GetOptionsPreferences {
    /// Allows resolving configurable strings.
    /// Defaults to false: no configurable strings will be resolved.
    /// Configurable strings must have been enabled when the options were built to have them resolved at runtime.
    pub are_configurable_strings_enabled: bool,
    pub constraints: Option<Constraints>,
    /// Overrides to apply after the built configuration.
    /// A string is used because it makes it easier to pass to the `config` library, but this may change in the future.
    /// It also makes it simpler and maybe faster to get from other programming languages.
    pub overrides_json: Option<String>,
    /// Determines if the feature names should be converted to canonical feature names.
    /// Defaults to false: given features names will be converted to canonical feature names before looking for features or options.
    pub skip_feature_name_conversion: bool,
}

impl Clone for GetOptionsPreferences {
    fn clone(&self) -> Self {
        Self {
            are_configurable_strings_enabled: self.are_configurable_strings_enabled,
            constraints: self.constraints.clone(),
            overrides_json: self.overrides_json.clone(),
            skip_feature_name_conversion: self.skip_feature_name_conversion,
        }
    }
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
            overrides_json: None,
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
