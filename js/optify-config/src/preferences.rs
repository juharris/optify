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

  #[napi]
  pub fn set_constraints_json(&mut self, constraints_json: Option<String>) {
    self.inner.set_constraints_json(constraints_json.as_deref());
  }

  #[napi]
  pub fn set_overrides_json(&mut self, overrides_json: Option<String>) {
    self.inner.overrides_json = overrides_json;
  }

  #[napi]
  pub fn set_skip_feature_name_conversion(&mut self, skip_feature_name_conversion: bool) {
    self.inner.skip_feature_name_conversion = skip_feature_name_conversion;
  }
}
