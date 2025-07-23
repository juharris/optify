#![deny(clippy::all)]

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::{OptionsProvider, OptionsRegistry, OptionsWatcher};
use std::sync::Arc;

use crate::metadata::{to_js_options_metadata, JsOptionsMetadata};

mod metadata;

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

  /// Returns all of the canonical feature names.
  #[napi]
  pub fn features(&self) -> Vec<String> {
    self.inner.as_ref().unwrap().get_features()
  }

  /// Returns a map of all the canonical feature names to their metadata.
  #[napi]
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

#[napi(js_name = "OptionsWatcher")]
pub struct JsOptionsWatcher {
  inner: Option<OptionsWatcher>,
}

/// Input to a watcher listener.
#[napi(js_name = "OptionsWatcherListenerEvent")]
pub struct JsOptionsWatcherListenerEvent {
  pub changed_paths: Vec<String>,
}

#[napi]
impl JsOptionsWatcher {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: None }
  }

  #[napi]
  pub fn add_listener(
    &mut self,
    listener: napi::threadsafe_function::ThreadsafeFunction<JsOptionsWatcherListenerEvent>,
  ) -> napi::Result<()> {
    let tsfn = Arc::new(listener);

    let listener_fn = Arc::new(
      move |paths: &std::collections::HashSet<std::path::PathBuf>| {
        let path_strings: Vec<String> = paths
          .iter()
          .map(|p| p.to_string_lossy().to_string())
          .collect();

        tsfn.call(
          Ok(JsOptionsWatcherListenerEvent {
            changed_paths: path_strings,
          }),
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      },
    );

    self.inner.as_mut().unwrap().add_listener(listener_fn);
    Ok(())
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

  /// Returns all of the canonical feature names.
  #[napi]
  pub fn features(&self) -> Vec<String> {
    self.inner.as_ref().unwrap().get_features()
  }

  /// Returns a map of all the canonical feature names to their metadata.
  #[napi]
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
  pub fn last_modified(&self) -> napi::Result<i64> {
    self
      .inner
      .as_ref()
      .map(|w| {
        w.last_modified()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_millis() as i64
      })
      .ok_or_else(|| napi::Error::from_reason("Watcher not built yet"))
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
