use serde::Deserialize;

/// Options for handling files in a directory.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuilderOptions {
    #[serde(default)]
    pub are_configurable_strings_enabled: bool,
}

impl BuilderOptions {
    pub(crate) fn default() -> Self {
        BuilderOptions {
            are_configurable_strings_enabled: false,
        }
    }
}
