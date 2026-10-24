#![deny(clippy::all)]

use napi::bindgen_prelude::{Either, FromNapiValue, JavaScriptClassExt, Reference, Unknown};
use napi::Env;

use crate::preferences::JsGetOptionsPreferences;
use crate::provider::JsOptionsProvider;
use crate::watcher::JsOptionsWatcher;

/// The provider or watcher that a scope gets options from.
/// The `Reference` keeps the JavaScript object alive without copying it.
pub enum Registry {
  Provider(Reference<JsOptionsProvider>),
  Watcher(Reference<JsOptionsWatcher>),
}

impl FromNapiValue for Registry {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    value: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    // `Reference` does not check the class of the object that it unwraps,
    // so the class must be checked first to avoid treating one class as another.
    // `instance_of` is used because the generated `ValidateNapiValue` for classes looks up the class by its Rust name
    // instead of its `js_name`, so it rejects every instance of classes with a custom `js_name`.
    let js_env = Env::from_raw(env);
    let js_value = unsafe { Unknown::from_napi_value(env, value) }?;
    if JsOptionsProvider::instance_of(&js_env, &js_value)? {
      return unsafe { Reference::from_napi_value(env, value) }.map(Self::Provider);
    }
    if JsOptionsWatcher::instance_of(&js_env, &js_value)? {
      return unsafe { Reference::from_napi_value(env, value) }.map(Self::Watcher);
    }
    Err(napi::Error::new(
      napi::Status::InvalidArg,
      "The registry must be an OptionsProvider or an OptionsWatcher.",
    ))
  }
}

/// Provides options for a fixed set of canonical features and overrides using an `OptionsProvider` or `OptionsWatcher`.
/// All of the logic is in the Rust `optify::Scope`, which keeps the features and overrides,
/// so they are not copied to JavaScript to get options.
///
/// Constraints are not stored; callers should resolve any constraint-based feature
/// filtering (such as with `getFilteredFeatures`) before creating the scope.
#[napi(js_name = "ScopedProvider")]
pub struct JsScopedProvider {
  registry: Registry,
  scope: optify::Scope,
}

#[napi]
impl JsScopedProvider {
  /// Creates a provider for canonical feature names and optional overrides.
  ///
  /// Feature names are assumed to be canonical and are not converted from aliases.
  /// Configurable values are enabled by default.
  #[napi(constructor)]
  pub fn new(
    #[napi(ts_arg_type = "OptionsProvider | OptionsWatcher")] registry: Registry,
    features: Vec<String>,
    overrides: Option<serde_json::Value>,
  ) -> Self {
    Self {
      registry,
      scope: optify::Scope::new(features, overrides),
    }
  }

  /// Restores a scope from the result of `serialize`.
  /// `overrides` is optional.
  ///
  /// Feature names are assumed to be canonical and are not converted from aliases.
  #[napi]
  pub fn deserialize(
    #[napi(ts_arg_type = "OptionsProvider | OptionsWatcher")] registry: Registry,
    value: serde_json::Value,
  ) -> napi::Result<JsScopedProvider> {
    let scope = optify::Scope::deserialize(value).map_err(napi::Error::from_reason)?;
    Ok(Self { registry, scope })
  }

  /// A copy of the default preferences for the scope.
  /// Only needed to use the cache of the provider or watcher.
  #[napi(js_name = "_defaultPreferences")]
  pub fn default_preferences(&self) -> JsGetOptionsPreferences {
    JsGetOptionsPreferences {
      inner: self.scope.preferences(None).into_owned(),
    }
  }

  /// A copy of the canonical feature names in the scope.
  /// Only needed to use the cache of the provider or watcher, which is keyed by feature names.
  #[napi(js_name = "_features")]
  pub fn features(&self) -> Vec<String> {
    self.scope.features().to_vec()
  }

  /// Gets all options using the scope's features and overrides.
  /// Given `preferences` are copied with the scope's overrides, if there are any, and are never changed.
  #[napi(js_name = "_getAllOptions")]
  pub fn get_all_options(
    &self,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<serde_json::Value> {
    let preferences = preferences.map(|p| &p.inner);
    match &self.registry {
      Registry::Provider(provider) => {
        self
          .scope
          .get_all_options(provider.inner.as_ref().unwrap(), None, preferences)
      }
      Registry::Watcher(watcher) => {
        self
          .scope
          .get_all_options(watcher.inner.as_ref().unwrap(), None, preferences)
      }
    }
    .map_err(napi::Error::from_reason)
  }

  /// Gets options for a key using the scope's features and overrides.
  /// Given `preferences` are copied with the scope's overrides, if there are any, and are never changed.
  #[napi(js_name = "_getOptions")]
  pub fn get_options(
    &self,
    key: String,
    preferences: Option<&JsGetOptionsPreferences>,
  ) -> napi::Result<serde_json::Value> {
    let preferences = preferences.map(|p| &p.inner);
    match &self.registry {
      Registry::Provider(provider) => {
        self
          .scope
          .get_options(provider.inner.as_ref().unwrap(), &key, None, preferences)
      }
      Registry::Watcher(watcher) => {
        self
          .scope
          .get_options(watcher.inner.as_ref().unwrap(), &key, None, preferences)
      }
    }
    .map_err(napi::Error::from_reason)
  }

  /// The preferences to use with the scope's features instead of the given preferences.
  /// Returns `null` when the given preferences can be used as they are, which avoids copying them.
  /// Otherwise, returns a copy with the scope's overrides.
  /// The given preferences are never changed.
  /// Only needed to use the cache of the provider or watcher.
  #[napi(js_name = "_preferences")]
  pub fn preferences(
    &self,
    preferences: &JsGetOptionsPreferences,
  ) -> Option<JsGetOptionsPreferences> {
    match self.scope.preferences(Some(&preferences.inner)) {
      std::borrow::Cow::Borrowed(_) => None,
      std::borrow::Cow::Owned(inner) => Some(JsGetOptionsPreferences { inner }),
    }
  }

  /// The provider or watcher that the options are from.
  /// Only needed to use its cache.
  #[napi(
    js_name = "_registry",
    ts_return_type = "OptionsProvider | OptionsWatcher"
  )]
  pub fn registry(
    &self,
    env: Env,
  ) -> napi::Result<Either<Reference<JsOptionsProvider>, Reference<JsOptionsWatcher>>> {
    Ok(match &self.registry {
      Registry::Provider(provider) => Either::A(provider.clone(env)?),
      Registry::Watcher(watcher) => Either::B(watcher.clone(env)?),
    })
  }

  /// Serializes the scope's features and overrides, not its registry or constraints.
  /// `overrides` is omitted when there are none.
  ///
  /// This is intended for transferring or persisting the scope.
  /// There should be no reason for application code to inspect the features directly:
  /// the system is configuration driven to avoid conditional code.
  /// Checking whether a feature is present introduces brittle conditional code because feature names can change.
  /// Use the option accessors instead.
  #[napi(ts_return_type = "{ features: Array<string>; overrides?: any }")]
  pub fn serialize(&self) -> serde_json::Value {
    self.scope.serialize()
  }
}
