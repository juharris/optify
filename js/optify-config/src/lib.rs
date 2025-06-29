#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry};

#[macro_use]
extern crate napi_derive;

#[napi(js_name = "GetOptionsPreferences")]
pub struct JsGetOptionsPreferences {
  inner: optify::provider::GetOptionsPreferences,
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
  pub fn set_overrides_json(&mut self, overrides_json: Option<String>) {
    self.inner.set_overrides_json(overrides_json);
  }

  #[napi]
  pub fn set_skip_feature_name_conversion(&mut self, skip_feature_name_conversion: bool) {
    self
      .inner
      .set_skip_feature_name_conversion(skip_feature_name_conversion);
  }
}

#[napi(js_name = "OptionsProvider")]
pub struct JsOptionsProvider {
  inner: Option<OptionsProvider>,
}

#[napi]
impl JsOptionsProvider {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: None }
  }

  #[napi]
  pub fn features(&self) -> Vec<String> {
    self.inner.as_ref().unwrap().get_features()
  }

  #[napi]
  pub fn get_all_options_json(
    &self,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<String> {
    let preferences = preferences.map(|p| &p.inner);
    match self
      .inner
      .as_ref()
      .unwrap()
      .get_all_options(&feature_names, None, preferences)
    {
      Ok(json) => Ok(json.to_string()),
      Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
  }

  #[napi]
  pub fn get_options_json(
    &self,
    key: String,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<String> {
    let preferences = preferences.map(|p| &p.inner);
    self
      .inner
      .as_ref()
      .unwrap()
      .get_options_with_preferences(&key, &feature_names, None, preferences)
      .map(|json| json.to_string())
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

#[napi(js_name = "OptionsProviderBuilder")]
pub struct JsOptionsProviderBuilder {
  inner: OptionsProviderBuilder,
}

#[napi]
impl JsOptionsProviderBuilder {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: OptionsProviderBuilder::new(),
    }
  }

  #[napi]
  pub fn add_directory(&mut self, directory: String) -> napi::Result<()> {
    let path = std::path::Path::new(&directory);
    self
      .inner
      .add_directory(path)
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub fn build(&mut self) -> napi::Result<JsOptionsProvider> {
    let provider = self
      .inner
      .build()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(JsOptionsProvider {
      inner: Some(provider),
    })
  }
}
