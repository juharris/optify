use config;
use std::path::Path;
use walkdir::WalkDir;

use crate::provider::{Aliases, OptionsProvider, Sources};
use crate::schema::feature::FeatureConfiguration;

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProviderBuilder {
    aliases: Aliases,
    sources: Sources,
}

impl Default for OptionsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsProviderBuilder {
    pub fn new() -> Self {
        OptionsProviderBuilder {
            aliases: Aliases::new(),
            sources: Sources::new(),
        }
    }

    pub fn add_directory(mut self, directory: &Path) -> Result<Self, String> {
        for entry in WalkDir::new(directory) {
            let entry = entry.unwrap();
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            // Skip .md files because they are not handled by the `config` library and we may have README.md files in the directory.
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                continue;
            }

            let key = self.get_path_key(path, directory);
            // TODO Optimization: Find a better way to build a more generic view of the file.
            // The config library is helpful because it handles many file types.
            let file = config::File::from(path);
            let config = config::Config::builder().add_source(file).build().unwrap();
            let feature_config: FeatureConfiguration = config.try_deserialize().unwrap();
            let options_as_json: serde_json::Value =
                feature_config.options.try_deserialize().unwrap();
            let options_as_json_str = serde_json::to_string(&options_as_json).unwrap();
            let source = config::File::from_str(&options_as_json_str, config::FileFormat::Json);
            let key = unicase::UniCase::new(key);
            let res = self.sources.insert(key.clone(), source);
            if res.is_some() {
                return Err(format!(
                    "Duplicate key found: `{}` for path `{}`",
                    key,
                    path.to_string_lossy()
                ));
            }

            // Add aliases.
            if let Some(aliases) = feature_config.metadata.aliases {
                for alias in aliases {
                    let alias = unicase::UniCase::new(alias);
                    let res = self.aliases.insert(alias.clone(), key.clone());
                    if res.is_some() {
                        return Err(format!(
                            "Duplicate alias found: `{}` for key `{}`",
                            alias, key
                        ));
                    }
                }
            }

            // TODO Add `key` as an alias.
        }

        Ok(self)
    }

    pub fn build(self) -> OptionsProvider {
        OptionsProvider::new(self.aliases, self.sources)
    }

    fn get_path_key(&self, path: &Path, directory: &Path) -> String {
        path.strip_prefix(directory)
            .unwrap()
            .with_extension("")
            .to_str()
            .unwrap()
            .to_owned()
    }
}
