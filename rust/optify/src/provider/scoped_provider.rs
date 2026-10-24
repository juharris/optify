use serde::Deserialize;
use serde_json::{json, Value};

use super::{CacheOptions, GetOptionsPreferences, OptionsRegistry};

#[derive(Deserialize)]
struct SerializedScope {
    features: Vec<String>,
    #[serde(default)]
    overrides: Option<Value>,
}

/// Provides options for a fixed set of canonical features and overrides using a borrowed registry.
///
/// Constraints are not stored; callers should resolve any constraint-based feature
/// filtering before creating the scope.
pub struct ScopedProvider<'a, R: OptionsRegistry + ?Sized> {
    registry: &'a R,
    features: Vec<String>,
    default_preferences: GetOptionsPreferences,
}

impl<'a, R: OptionsRegistry + ?Sized> ScopedProvider<'a, R> {
    /// Creates a provider for canonical feature names and optional overrides.
    ///
    /// Feature names are assumed to be canonical and are not converted from aliases.
    #[must_use]
    pub fn new(registry: &'a R, features: &[impl AsRef<str>], overrides: Option<Value>) -> Self {
        Self {
            registry,
            features: features
                .iter()
                .map(|feature| feature.as_ref().to_owned())
                .collect(),
            default_preferences: Self::default_preferences(overrides),
        }
    }

    /// Gets options for a key using this scope's features and overrides.
    pub fn get_options(&self, key: &str) -> Result<Value, String> {
        self.registry.get_options_with_preferences(
            key,
            &self.features,
            None,
            Some(&self.default_preferences),
        )
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
        match preferences {
            Some(preferences) if self.default_preferences.overrides.is_none() => {
                self.registry.get_options_with_preferences(
                    key,
                    &self.features,
                    cache_options,
                    Some(preferences),
                )
            }
            Some(preferences) => {
                let preferences =
                    preferences.with_overrides(self.default_preferences.overrides.clone());
                self.registry.get_options_with_preferences(
                    key,
                    &self.features,
                    cache_options,
                    Some(&preferences),
                )
            }
            None => self.registry.get_options_with_preferences(
                key,
                &self.features,
                cache_options,
                Some(&self.default_preferences),
            ),
        }
    }

    /// Gets options for a key, applying this scope's overrides to the mutable preferences.
    pub fn get_options_with_mut_preferences(
        &self,
        key: &str,
        cache_options: Option<&CacheOptions>,
        preferences: &mut GetOptionsPreferences,
    ) -> Result<Value, String> {
        self.set_scope_overrides(preferences);
        self.registry.get_options_with_preferences(
            key,
            &self.features,
            cache_options,
            Some(preferences),
        )
    }

    /// Gets all options using this scope's features and overrides.
    pub fn get_all_options(&self) -> Result<Value, String> {
        self.registry
            .get_all_options(&self.features, None, Some(&self.default_preferences))
    }

    /// Serializes the scope's features and overrides, not its registry or constraints.
    ///
    /// This is intended for transferring or persisting the scope. There should be no
    /// reason for application code to inspect the features directly: the system is
    /// configuration driven to avoid conditional code. Checking whether a feature is
    /// present introduces brittle conditional code because feature names can change.
    /// Use the option accessors instead.
    #[must_use]
    pub fn serialize(&self) -> Value {
        let mut serialized = serde_json::Map::new();
        serialized.insert("features".to_owned(), json!(self.features));
        if let Some(overrides) = &self.default_preferences.overrides {
            serialized.insert("overrides".to_owned(), overrides.clone());
        }
        Value::Object(serialized)
    }

    /// Restores a serialized scope using the supplied registry.
    ///
    /// Feature names are assumed to be canonical and are not converted from aliases.
    pub fn deserialize(registry: &'a R, value: Value) -> Result<Self, String> {
        let SerializedScope {
            features,
            overrides,
        } = serde_json::from_value(value).map_err(|error| error.to_string())?;
        Ok(Self {
            registry,
            features,
            default_preferences: Self::default_preferences(overrides),
        })
    }

    fn default_preferences(overrides: Option<Value>) -> GetOptionsPreferences {
        GetOptionsPreferences {
            are_configurable_values_enabled: true,
            skip_feature_name_conversion: true,
            overrides,
            ..GetOptionsPreferences::default()
        }
    }

    fn set_scope_overrides(&self, preferences: &mut GetOptionsPreferences) {
        if self.default_preferences.overrides.is_some() {
            preferences
                .overrides
                .clone_from(&self.default_preferences.overrides);
        }
    }
}
