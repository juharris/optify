use serde::Deserialize;
use std::path::PathBuf;

/// Determines how file references should be tracked.
#[derive(Debug, Deserialize, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TrackReferenceMode {
    #[default]
    None,

    /// Only check in Configurable Strings.
    ConfigurableStrings,

    /// Check for any value of keys called "file"
    /// and perhaps more in the future.
    KeyName,
}

/// Deserializable form of builder options from `.optify/config.json`.
/// Fields are optional so that unset values resolve to their defaults.
/// These values provide defaults that can be overridden by builder-level `BuilderOptions`.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuilderOptionsConfig {
    #[serde(default)]
    #[serde(alias = "areConfigurableStringsEnabled")]
    pub are_configurable_values_enabled: Option<bool>,
    #[serde(default)]
    pub schema_path: Option<PathBuf>,
    #[serde(default)]
    pub track_file_references: Option<TrackReferenceMode>,
}

impl BuilderOptionsConfig {
    /// Provides default values for fields not explicitly set in the builder-level `overrides`.
    /// Fields in `overrides` that differ from `BuilderOptions::default()` take priority.
    pub fn merge_with(self, overrides: &BuilderOptions) -> BuilderOptions {
        let defaults = BuilderOptions::default();
        let are_configurable_values_enabled = if overrides.are_configurable_values_enabled
            != defaults.are_configurable_values_enabled
        {
            overrides.are_configurable_values_enabled
        } else {
            self.are_configurable_values_enabled
                .unwrap_or(defaults.are_configurable_values_enabled)
        };
        BuilderOptions {
            are_configurable_strings_enabled: are_configurable_values_enabled,
            are_configurable_values_enabled,
            schema_path: if overrides.schema_path != defaults.schema_path {
                overrides.schema_path.clone()
            } else {
                self.schema_path.or(defaults.schema_path)
            },
            track_file_references: if overrides.track_file_references
                != defaults.track_file_references
            {
                overrides.track_file_references
            } else {
                self.track_file_references
                    .unwrap_or(defaults.track_file_references)
            },
        }
    }
}

/// Options for handling files in a directory.
#[derive(Clone, Default)]
pub struct BuilderOptions {
    /// DEPRECATED: Use `are_configurable_values_enabled` instead to enable all configurable values.
    pub are_configurable_strings_enabled: bool,
    pub are_configurable_values_enabled: bool,
    pub schema_path: Option<PathBuf>,
    pub track_file_references: TrackReferenceMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_with_overrides_win_over_config_defaults() {
        // Builder-level overrides have non-default values, so they take priority over the config file values.
        let overrides = BuilderOptions {
            are_configurable_strings_enabled: true,
            are_configurable_values_enabled: true,
            schema_path: Some(PathBuf::from("override_schema.json")),
            track_file_references: TrackReferenceMode::ConfigurableStrings,
        };
        let config = BuilderOptionsConfig {
            are_configurable_values_enabled: Some(false),
            schema_path: Some(PathBuf::from("config_schema.json")),
            track_file_references: Some(TrackReferenceMode::None),
        };

        let merged = config.merge_with(&overrides);

        assert!(merged.are_configurable_strings_enabled);
        assert!(merged.are_configurable_values_enabled);
        assert_eq!(
            merged.schema_path,
            Some(PathBuf::from("override_schema.json"))
        );
        assert_eq!(
            merged.track_file_references,
            TrackReferenceMode::ConfigurableStrings
        );
    }

    #[test]
    fn test_merge_with_config_provides_defaults_when_overrides_are_default() {
        // Builder-level overrides are all at default, so config values provide the defaults.
        let overrides = BuilderOptions::default();
        let config = BuilderOptionsConfig {
            are_configurable_values_enabled: Some(true),
            schema_path: Some(PathBuf::from("config_schema.json")),
            track_file_references: Some(TrackReferenceMode::ConfigurableStrings),
        };

        let merged = config.merge_with(&overrides);

        assert!(merged.are_configurable_strings_enabled);
        assert!(merged.are_configurable_values_enabled);
        assert_eq!(
            merged.schema_path,
            Some(PathBuf::from("config_schema.json"))
        );
        assert_eq!(
            merged.track_file_references,
            TrackReferenceMode::ConfigurableStrings
        );
    }

    #[test]
    fn test_merge_with_partial_overrides() {
        // Only track_file_references is explicitly set in overrides.
        // Config provides the default for are_configurable_strings_enabled.
        let overrides = BuilderOptions {
            are_configurable_strings_enabled: false,
            are_configurable_values_enabled: false,
            schema_path: None,
            track_file_references: TrackReferenceMode::ConfigurableStrings,
        };
        let config = BuilderOptionsConfig {
            are_configurable_values_enabled: Some(true),
            schema_path: Some(PathBuf::from("config_schema.json")),
            track_file_references: None,
        };

        let merged = config.merge_with(&overrides);

        assert!(merged.are_configurable_strings_enabled);
        assert!(merged.are_configurable_values_enabled);
        assert_eq!(
            merged.schema_path,
            Some(PathBuf::from("config_schema.json"))
        );
        assert_eq!(
            merged.track_file_references,
            TrackReferenceMode::ConfigurableStrings
        );
    }

    #[test]
    fn test_merge_with_empty_config_uses_overrides() {
        // Config has no values set; overrides provide everything.
        let overrides = BuilderOptions {
            are_configurable_values_enabled: true,
            schema_path: Some(PathBuf::from("override_schema.json")),
            track_file_references: TrackReferenceMode::ConfigurableStrings,
            ..Default::default()
        };
        let config = BuilderOptionsConfig::default();

        let merged = config.merge_with(&overrides);

        assert!(merged.are_configurable_strings_enabled);
        assert!(merged.are_configurable_values_enabled);
        assert_eq!(
            merged.schema_path,
            Some(PathBuf::from("override_schema.json"))
        );
        assert_eq!(
            merged.track_file_references,
            TrackReferenceMode::ConfigurableStrings
        );
    }

    #[test]
    fn test_merge_with_both_defaults() {
        let overrides = BuilderOptions::default();
        let config = BuilderOptionsConfig::default();

        let merged = config.merge_with(&overrides);

        assert!(!merged.are_configurable_strings_enabled);
        assert_eq!(merged.schema_path, None);
        assert_eq!(merged.track_file_references, TrackReferenceMode::None);
    }
}
