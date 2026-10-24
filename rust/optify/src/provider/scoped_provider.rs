use std::borrow::Cow;

use serde::Deserialize;
use serde_json::{json, Value};

use super::{CacheOptions, GetOptionsPreferences, OptionsRegistry};

#[derive(Deserialize)]
struct SerializedScope {
    features: Vec<String>,
    #[serde(default)]
    overrides: Option<Value>,
}

/// A fixed set of canonical features and overrides which is independent of any registry.
///
/// A `ScopedProvider` pairs a scope with a borrowed registry.
/// Bindings that cannot borrow a registry can keep a scope and pass the registry when getting options.
///
/// Feature names are assumed to be canonical and are not converted from aliases.
/// Constraints are not stored; callers should resolve any constraint-based feature
/// filtering before creating the scope.
pub struct Scope {
    /// Preferences with this scope's overrides and defaults, which are built once and shared.
    default_preferences: GetOptionsPreferences,
    features: Vec<String>,
}

impl Scope {
    /// Creates a scope for canonical feature names and optional overrides.
    ///
    /// The features are taken by value so that callers which already own them, such as bindings, do not copy them.
    /// Configurable values are enabled by default.
    #[must_use]
    pub fn new(features: Vec<String>, overrides: Option<Value>) -> Self {
        Self {
            default_preferences: GetOptionsPreferences {
                are_configurable_values_enabled: true,
                skip_feature_name_conversion: true,
                overrides,
                ..GetOptionsPreferences::default()
            },
            features,
        }
    }

    /// Restores a scope from the result of `serialize`.
    ///
    /// `overrides` is optional.
    pub fn deserialize(value: Value) -> Result<Self, String> {
        let SerializedScope {
            features,
            overrides,
        } = serde_json::from_value(value).map_err(|error| error.to_string())?;
        Ok(Self::new(features, overrides))
    }

    /// The canonical feature names in this scope.
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }

    /// Gets all options for this scope's features using the registry.
    ///
    /// See `Scope::preferences` for how `preferences` are used.
    pub fn get_all_options<R: OptionsRegistry + ?Sized>(
        &self,
        registry: &R,
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Value, String> {
        registry.get_all_options(
            &self.features,
            cache_options,
            Some(&self.preferences(preferences)),
        )
    }

    /// Gets options for a key and this scope's features using the registry.
    ///
    /// See `Scope::preferences` for how `preferences` are used.
    pub fn get_options<R: OptionsRegistry + ?Sized>(
        &self,
        registry: &R,
        key: &str,
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Value, String> {
        registry.get_options_with_preferences(
            key,
            &self.features,
            cache_options,
            Some(&self.preferences(preferences)),
        )
    }

    /// Gets options for a key and this scope's features using the registry,
    /// applying this scope's overrides to the mutable preferences.
    ///
    /// This avoids copying the preferences when this scope has overrides.
    pub fn get_options_with_mut_preferences<R: OptionsRegistry + ?Sized>(
        &self,
        registry: &R,
        key: &str,
        cache_options: Option<&CacheOptions>,
        preferences: &mut GetOptionsPreferences,
    ) -> Result<Value, String> {
        self.set_overrides_on(preferences);
        registry.get_options_with_preferences(key, &self.features, cache_options, Some(preferences))
    }

    /// Gets the preferences to use with this scope's features.
    ///
    /// Without `preferences`, this scope's defaults are used.
    /// Given `preferences` are used as they are unless this scope has overrides,
    /// in which case a copy is made with this scope's overrides.
    /// Given `preferences` are never changed.
    #[must_use]
    pub fn preferences<'a>(
        &'a self,
        preferences: Option<&'a GetOptionsPreferences>,
    ) -> Cow<'a, GetOptionsPreferences> {
        match preferences {
            None => Cow::Borrowed(&self.default_preferences),
            Some(preferences) if self.default_preferences.overrides.is_none() => {
                Cow::Borrowed(preferences)
            }
            Some(preferences) => {
                Cow::Owned(preferences.with_overrides(self.default_preferences.overrides.clone()))
            }
        }
    }

    /// Serializes the scope's features and overrides, not its registry or constraints.
    ///
    /// This is intended for transferring or persisting the scope. There should be no
    /// reason for application code to inspect the features directly: the system is
    /// configuration driven to avoid conditional code. Checking whether a feature is
    /// present introduces brittle conditional code because feature names can change.
    /// Use the option accessors instead.
    /// `overrides` is omitted when there are none.
    #[must_use]
    pub fn serialize(&self) -> Value {
        let mut serialized = serde_json::Map::new();
        serialized.insert("features".to_owned(), json!(self.features));
        if let Some(overrides) = &self.default_preferences.overrides {
            serialized.insert("overrides".to_owned(), overrides.clone());
        }
        Value::Object(serialized)
    }

    /// Sets this scope's overrides on the preferences, if this scope has any overrides.
    fn set_overrides_on(&self, preferences: &mut GetOptionsPreferences) {
        if self.default_preferences.overrides.is_some() {
            preferences
                .overrides
                .clone_from(&self.default_preferences.overrides);
        }
    }
}

/// Provides options for a fixed set of canonical features and overrides using a borrowed registry.
///
/// Constraints are not stored; callers should resolve any constraint-based feature
/// filtering before creating the scope.
pub struct ScopedProvider<'a, R: OptionsRegistry + ?Sized> {
    registry: &'a R,
    scope: Scope,
}

impl<'a, R: OptionsRegistry + ?Sized> ScopedProvider<'a, R> {
    /// Creates a provider for canonical feature names and optional overrides.
    ///
    /// Feature names are assumed to be canonical and are not converted from aliases.
    #[must_use]
    pub fn new(registry: &'a R, features: &[impl AsRef<str>], overrides: Option<Value>) -> Self {
        Self {
            registry,
            scope: Scope::new(
                features
                    .iter()
                    .map(|feature| feature.as_ref().to_owned())
                    .collect(),
                overrides,
            ),
        }
    }

    /// Gets options for a key using this scope's features and overrides.
    pub fn get_options(&self, key: &str) -> Result<Value, String> {
        self.get_options_with_preferences(key, None, None)
    }

    /// Gets options for a key using this scope's features, cache options, and preferences.
    ///
    /// The preferences are copied before this scope's overrides are applied.
    pub fn get_options_with_preferences(
        &self,
        key: &str,
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Value, String> {
        self.scope
            .get_options(self.registry, key, cache_options, preferences)
    }

    /// Gets options for a key, applying this scope's overrides to the mutable preferences.
    pub fn get_options_with_mut_preferences(
        &self,
        key: &str,
        cache_options: Option<&CacheOptions>,
        preferences: &mut GetOptionsPreferences,
    ) -> Result<Value, String> {
        self.scope
            .get_options_with_mut_preferences(self.registry, key, cache_options, preferences)
    }

    /// Gets all options using this scope's features and overrides.
    pub fn get_all_options(&self) -> Result<Value, String> {
        self.scope.get_all_options(self.registry, None, None)
    }

    /// Serializes the scope's features and overrides, not its registry or constraints.
    ///
    /// See `Scope::serialize`.
    #[must_use]
    pub fn serialize(&self) -> Value {
        self.scope.serialize()
    }

    /// Restores a serialized scope using the supplied registry.
    ///
    /// Feature names are assumed to be canonical and are not converted from aliases.
    pub fn deserialize(registry: &'a R, value: Value) -> Result<Self, String> {
        Ok(Self {
            registry,
            scope: Scope::deserialize(value)?,
        })
    }
}
