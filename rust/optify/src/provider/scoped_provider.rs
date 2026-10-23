use serde::Deserialize;
use serde_json::{json, Value};

use super::{GetOptionsPreferences, OptionsRegistry};

#[derive(Deserialize)]
struct Scope {
    features: Vec<String>,
    overrides: Option<Value>,
}

/// Provides options for a fixed set of features and overrides using a borrowed registry.
///
/// Constraints are not stored; callers should resolve any constraint-based feature
/// filtering before creating the scope.
pub struct ScopedProvider<'a, R: OptionsRegistry + ?Sized> {
    registry: &'a R,
    scope: Scope,
}

impl<'a, R: OptionsRegistry + ?Sized> ScopedProvider<'a, R> {
    #[must_use]
    pub fn new(registry: &'a R, features: &[impl AsRef<str>], overrides: Option<Value>) -> Self {
        Self {
            registry,
            scope: Scope {
                features: features
                    .iter()
                    .map(|feature| feature.as_ref().to_owned())
                    .collect(),
                overrides,
            },
        }
    }

    /// Gets options for a key using this scope's features and overrides.
    pub fn get_options(&self, key: &str) -> Result<Value, String> {
        self.registry.get_options_with_preferences(
            key,
            &self.scope.features,
            None,
            Some(&self.preferences()),
        )
    }

    /// Gets all options using this scope's features and overrides.
    pub fn get_all_options(&self) -> Result<Value, String> {
        self.registry
            .get_all_options(&self.scope.features, None, Some(&self.preferences()))
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
        json!({
            "features": self.scope.features,
            "overrides": self.scope.overrides,
        })
    }

    /// Restores a serialized scope using the supplied registry.
    ///
    /// Feature names are resolved by the registry when options are requested.
    pub fn deserialize(registry: &'a R, value: Value) -> Result<Self, String> {
        let scope = serde_json::from_value(value).map_err(|error| error.to_string())?;
        Ok(Self { registry, scope })
    }

    fn preferences(&self) -> GetOptionsPreferences {
        GetOptionsPreferences {
            overrides: self.scope.overrides.clone(),
            ..GetOptionsPreferences::default()
        }
    }
}
