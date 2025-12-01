#![deny(clippy::all)]

use napi::Env;
use optify::builder::{OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::{OptionsRegistry, OptionsWatcher};
use std::sync::Arc;

use crate::convert::convert_to_js_object;
use crate::metadata::{to_js_options_metadata, JsOptionsMetadata};
use crate::preferences::JsGetOptionsPreferences;
use crate::watcher_options::JsWatcherOptions;

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
  pub fn build_with_options(
    directory: String,
    options: &JsWatcherOptions,
  ) -> napi::Result<JsOptionsWatcher> {
    let path = std::path::Path::new(&directory);
    match OptionsWatcher::build_with_options(path, options.inner.clone()) {
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

  #[napi]
  pub fn build_from_directories_with_options(
    directories: Vec<String>,
    options: &JsWatcherOptions,
  ) -> napi::Result<JsOptionsWatcher> {
    match OptionsWatcher::build_from_directories_with_options(&directories, options.inner.clone()) {
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

  #[napi]
  pub fn get_all_options(
    &self,
    env: Env,
    feature_names: Vec<String>,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<napi::JsUnknown> {
    let preferences = preferences.map(|p| &p.inner);
    match self
      .inner
      .as_ref()
      .unwrap()
      .get_all_options(&feature_names, None, preferences)
    {
      Ok(options) => Ok(convert_to_js_object(env, &options)),
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

  /// Indicates if the feature has conditions.
  #[napi]
  pub fn has_conditions(&self, canonical_feature_name: String) -> bool {
    self
      .inner
      .as_ref()
      .unwrap()
      .has_conditions(&canonical_feature_name)
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
  pub fn with_watcher_options(&mut self, options: &JsWatcherOptions) -> napi::Result<()> {
    self.inner.with_watcher_options(options.inner.clone());
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
