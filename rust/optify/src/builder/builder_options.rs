use serde::Deserialize;
use std::path::PathBuf;

/// How file references from configurable strings should be tracked
#[derive(Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub enum TrackReferenceMode {
    #[default]
    None,
    ConfigurableStrings,
}

/// Deserializable form of builder options from `.optify/config.json`.
/// Fields are optional so that unset values fall back to the builder-level `BuilderOptions`.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuilderOptionsConfig {
    #[serde(default)]
    pub are_configurable_strings_enabled: Option<bool>,
    #[serde(default)]
    pub schema_path: Option<PathBuf>,
    #[serde(default)]
    pub track_file_references: Option<TrackReferenceMode>,
}

impl BuilderOptionsConfig {
    /// Merges this config with a base `BuilderOptions`, using config values when set
    /// and falling back to the base for unset fields.
    pub fn merge_with(self, base: &BuilderOptions) -> BuilderOptions {
        BuilderOptions {
            are_configurable_strings_enabled: self
                .are_configurable_strings_enabled
                .unwrap_or(base.are_configurable_strings_enabled),
            schema_path: self.schema_path.or(base.schema_path.clone()),
            track_file_references: self
                .track_file_references
                .unwrap_or(base.track_file_references),
        }
    }
}

/// Options for handling files in a directory.
#[derive(Clone, Default)]
pub struct BuilderOptions {
    pub are_configurable_strings_enabled: bool,
    pub schema_path: Option<PathBuf>,
    pub track_file_references: TrackReferenceMode,
}
