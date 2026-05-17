use config::FileStoredFormat;
use std::collections::HashSet;

pub(super) fn get_supported_extensions() -> HashSet<&'static str> {
    [
        config::FileFormat::Ini.file_extensions(),
        config::FileFormat::Json.file_extensions(),
        config::FileFormat::Json5.file_extensions(),
        config::FileFormat::Ron.file_extensions(),
        config::FileFormat::Toml.file_extensions(),
        config::FileFormat::Yaml.file_extensions(),
    ]
    .iter()
    .flat_map(|exts| exts.iter().copied())
    .collect()
}
