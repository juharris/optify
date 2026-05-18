#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry};

use crate::builder_options::JsBuilderOptions;
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
  pub fn build(
    directory: String,
    options: Option<&JsBuilderOptions>,
  ) -> napi::Result<JsOptionsProvider> {
    let path = std::path::Path::new(&directory);
    match match options {
      Some(opts) => OptionsProvider::build_with_options(path, opts.inner.clone()),
      None => OptionsProvider::build(path),
    } {
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

  /// Gets all options for the specified feature names.
  /// Should return a JavaScript object.
  ///
  /// There normally isn't much of a performance difference between using
  /// `JSON.parse(get_all_options_json(...))` and `get_all_options(...)`.
  /// Large JSON objects with over 50 keys may be slightly slower with `get_all_options(...)`.
  #[napi(js_name = "_getAllOptions")]
  pub fn get_all_options(
    &self,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<serde_json::Value> {
    let preferences = preferences.map(|p| &p.inner);
    self
      .inner
      .as_ref()
      .unwrap()
      .get_all_options(&feature_names, None, preferences)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Gets all options for the specified feature names.
  /// Returns a string which can be parsed as JSON to get the options.
  ///
  /// There normally isn't much of a performance difference between using
  /// `JSON.parse(get_all_options_json(...))` and `get_all_options(...)`.
  /// Large JSON objects with over 50 keys may be slightly slower with `get_all_options(...)`.
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

  /// Gets canonical feature names that reference a relative file path.
  ///
  /// Returns `null` when file reference tracking is disabled.
  #[napi]
  pub fn get_features_referencing_file(&self, relative_path: String) -> Option<Vec<String>> {
    self
      .inner
      .as_ref()
      .unwrap()
      .get_features_referencing_file(&relative_path)
  }

  #[napi(js_name = "_getOptions")]
  pub fn get_options(
    &self,
    key: String,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<serde_json::Value> {
    let preferences = preferences.map(|p| &p.inner);
    self
      .inner
      .as_ref()
      .unwrap()
      .get_options_with_preferences(&key, &feature_names, None, preferences)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
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

  /// Indicates if the feature has conditions.
  #[napi]
  pub fn has_conditions(&self, canonical_feature_name: String) -> bool {
    self
      .inner
      .as_ref()
      .unwrap()
      .has_conditions(&canonical_feature_name)
  }

  /// Filters feature names based on constraints and preferences.
  /// Returns canonical feature names that pass the filters.
  #[napi(js_name = "getFilteredFeatures")]
  pub fn get_filtered_features(
    &self,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<Vec<String>> {
    let preferences = preferences.map(|p| &p.inner);
    self
      .inner
      .as_ref()
      .unwrap()
      .get_filtered_feature_names(&feature_names, preferences)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Filters feature names based on constraints and preferences.
  /// Returns an array matching the input order where each element is the canonical name if the feature was kept, or null if it was filtered out.
  #[napi(js_name = "mapFeatureNames")]
  pub fn map_feature_names(
    &self,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<Vec<Option<String>>> {
    let preferences = preferences.map(|p| &p.inner);
    self
      .inner
      .as_ref()
      .unwrap()
      .map_feature_names(&feature_names, preferences)
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
