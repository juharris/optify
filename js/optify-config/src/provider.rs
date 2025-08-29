#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry};

use crate::metadata::{to_js_options_metadata, JsOptionsMetadata};
use crate::preferences::JsGetOptionsPreferences;

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
  pub fn build(directory: String) -> napi::Result<JsOptionsProvider> {
    let path = std::path::Path::new(&directory);
    match OptionsProvider::build(path) {
      Ok(provider) => Ok(JsOptionsProvider {
        inner: Some(provider),
      }),
      Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
  }

  #[napi]
  pub fn build_from_directories(directories: Vec<String>) -> napi::Result<JsOptionsProvider> {
    match OptionsProvider::build_from_directories(&directories) {
      Ok(provider) => Ok(JsOptionsProvider {
        inner: Some(provider),
      }),
      Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
  }

  /// Returns all of the canonical feature names.
  #[napi]
  pub fn features(&self) -> Vec<String> {
    self.inner.as_ref().unwrap().get_features()
  }

  /// Returns a map of all the canonical feature names to their metadata.
  #[napi(js_name = "_featuresWithMetadata")]
  pub fn features_with_metadata(&self) -> std::collections::HashMap<String, JsOptionsMetadata> {
    self
      .inner
      .as_ref()
      .unwrap()
      .get_features_with_metadata()
      .into_iter()
      .map(|(k, v)| (k, to_js_options_metadata(v)))
      .collect()
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

  /// Map an alias or canonical feature name (perhaps derived from a file name) to a canonical feature name.
  /// Canonical feature names map to themselves.
  ///
  /// Returns the canonical feature name or `null` if the feature name is not found.
  #[napi]
  pub fn get_canonical_feature_name(&self, feature_name: String) -> Option<String> {
    self
      .inner
      .as_ref()
      .unwrap()
      .get_canonical_feature_name(&feature_name)
      .ok()
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
