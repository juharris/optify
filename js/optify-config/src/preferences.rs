#![deny(clippy::all)]

/// Preferences when getting options.
#[napi(js_name = "GetOptionsPreferences")]
pub struct JsGetOptionsPreferences {
  pub(crate) inner: optify::provider::GetOptionsPreferences,
}

#[napi]
impl JsGetOptionsPreferences {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: optify::provider::GetOptionsPreferences::new(),
    }
  }

  /// Deprecated: Use `areConfigurableValuesEnabled` instead.
  #[napi]
  pub fn are_configurable_strings_enabled(&self) -> bool {
    self.inner.are_configurable_values_enabled
  }

  /// Deprecated: Use `enableConfigurableValues` instead.
  #[napi]
  pub fn enable_configurable_strings(&mut self) {
    self.inner.are_configurable_values_enabled = true;
  }

  /// Deprecated: Use `disableConfigurableValues` instead.
  #[napi]
  pub fn disable_configurable_strings(&mut self) {
    self.inner.are_configurable_values_enabled = false;
  }

  /// Indicates if configurable values are enabled.
  #[napi]
  pub fn are_configurable_values_enabled(&self) -> bool {
    self.inner.are_configurable_values_enabled
  }

  /// Enables configurable values which are disabled by default.
  #[napi]
  pub fn enable_configurable_values(&mut self) {
    self.inner.are_configurable_values_enabled = true;
  }

  /// Disables configurable values which are disabled by default.
  #[napi]
  pub fn disable_configurable_values(&mut self) {
    self.inner.are_configurable_values_enabled = false;
  }

  #[napi]
  pub fn set_constraints(&mut self, constraints: Option<serde_json::Value>) {
    self.inner.set_constraints(constraints);
  }

  #[napi]
  pub fn set_constraints_json(&mut self, constraints_json: Option<String>) {
    self.inner.set_constraints_json(constraints_json.as_deref());
  }

  #[napi]
  pub fn set_overrides(&mut self, overrides: Option<serde_json::Value>) {
    self.inner.overrides = overrides;
  }

  #[napi]
  pub fn set_overrides_json(&mut self, overrides_json: Option<String>) {
    self.inner.overrides =
      overrides_json.map(|s| serde_json::from_str(&s).expect("overrides should be valid JSON"));
  }

  #[napi]
  pub fn has_overrides(&self) -> bool {
    self.inner.overrides.is_some()
  }

  #[napi]
  pub fn set_skip_feature_name_conversion(&mut self, skip_feature_name_conversion: bool) {
    self.inner.skip_feature_name_conversion = skip_feature_name_conversion;
  }
}
