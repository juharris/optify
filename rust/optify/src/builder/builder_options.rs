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

/// Options for handling files in a directory.
#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuilderOptions {
    #[serde(default)]
    pub are_configurable_strings_enabled: bool,
    #[serde(default)]
    pub schema_path: Option<PathBuf>,
    #[serde(default)]
    pub track_file_references: TrackReferenceMode,
}
