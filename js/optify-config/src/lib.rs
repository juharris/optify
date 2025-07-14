#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry, OptionsWatcher};

#[macro_use]
extern crate napi_derive;

/// Preferences when getting options.
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

  /// All of the canonical feature names.
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

// It sucks to duplicate the code for the watcher, but using a macro didn't work, possibly because of the napi macro.

#[napi(js_name = "OptionsWatcher")]
pub struct JsOptionsWatcher {
  inner: Option<OptionsWatcher>,
}

#[napi]
impl JsOptionsWatcher {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: None }
  }

  #[napi]
  pub fn build(directory: String) -> napi::Result<JsOptionsWatcher> {
    let path = std::path::Path::new(&directory);
    match OptionsWatcher::build(path) {
      Ok(provider) => Ok(JsOptionsWatcher {
        inner: Some(provider),
      }),
      Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
  }

  #[napi]
  pub fn build_from_directories(directories: Vec<String>) -> napi::Result<JsOptionsWatcher> {
    match OptionsWatcher::build_from_directories(&directories) {
      Ok(provider) => Ok(JsOptionsWatcher {
        inner: Some(provider),
      }),
      Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
  }

  /// All of the canonical feature names.
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

  /// Returns the time when the provider was finished building.
  #[napi]
  pub fn last_modified(&self) -> Option<f64> {
    self.inner.as_ref().and_then(|w| {
      w.last_modified()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as f64)
    })
  }
}

#[napi(js_name = "OptionsWatcherBuilder")]
pub struct JsOptionsWatcherBuilder {
  inner: OptionsWatcherBuilder,
}

#[napi]
impl JsOptionsWatcherBuilder {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: OptionsWatcherBuilder::new(),
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
  pub fn build(&mut self) -> napi::Result<JsOptionsWatcher> {
    let watcher = self
      .inner
      .build()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(JsOptionsWatcher {
      inner: Some(watcher),
    })
  }
}
