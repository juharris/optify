use config;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

use crate::provider::{Aliases, OptionsProvider, Sources};
use crate::schema::feature::FeatureConfiguration;

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
#[derive(Clone)]
pub struct OptionsProviderBuilder {
    aliases: Aliases,
    imports: HashMap<String, Vec<String>>,
    sources: Sources,
}

impl Default for OptionsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn add_alias(aliases: &mut Aliases, alias: &String, key: &String) -> Result<(), String> {
    let uni_case_alias = unicase::UniCase::new(alias.clone());
    let res = aliases.insert(uni_case_alias, key.clone());
    if res.is_some() {
        return Err(format!(
            "The alias {:?} for key {:?} is already mapped to {:?}",
            alias,
            key,
            res.unwrap()
        ));
    }
    Ok(())
}

impl OptionsProviderBuilder {
    pub fn new() -> Self {
        OptionsProviderBuilder {
            aliases: Aliases::new(),
            imports: HashMap::new(),
            sources: Sources::new(),
        }
    }

    pub fn add_directory(&mut self, directory: &Path) -> Result<&Self, String> {
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

            // TODO Optimization: Find a more efficient way to build a more generic view of the file.
            // The `config` library is helpful because it handles many file types.
            // It would also be nice to support comments in .json files, even though it is not standard.
            // The `config` library does support .json5 which supports comments.
            let file = config::File::from(path);
            let config_for_path = match config::Config::builder().add_source(file).build() {
                Ok(conf) => conf,
                Err(e) => {
                    return Err(format!(
                        "Error reading file {:?}: {:?}",
                        path.to_string_lossy(),
                        e
                    ))
                }
            };
            let feature_config: FeatureConfiguration = match config_for_path.try_deserialize() {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!(
                        "Error deserializing feature configuration from file {:?}: {:?}",
                        path.to_string_lossy(),
                        e
                    ))
                }
            };
            let options_as_json: serde_json::Value =
                feature_config.options.try_deserialize().unwrap();
            let options_as_json_str = serde_json::to_string(&options_as_json).unwrap();
            let source = config::File::from_str(&options_as_json_str, config::FileFormat::Json);
            let canonical_feature_name = self.get_canonical_feature_name(path, directory);

            match feature_config.imports {
                None => {}
                Some(imports) => {
                    self.imports.insert(canonical_feature_name.clone(), imports);
                }
            }
            let res = self.sources.insert(canonical_feature_name.clone(), source);
            if res.is_some() {
                return Err(format!(
                    "Duplicate key found: `{}` for path `{}`",
                    canonical_feature_name,
                    path.to_string_lossy()
                ));
            }

            match add_alias(
                &mut self.aliases,
                &canonical_feature_name,
                &canonical_feature_name,
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            // Add aliases.
            if let Some(aliases) = feature_config.metadata.aliases {
                for alias in aliases {
                    match add_alias(&mut self.aliases, &alias, &canonical_feature_name) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn build(&self) -> Result<OptionsProvider, String> {
        let mut resolved_imports: HashSet<String> = HashSet::new();
        for (canonical_feature_name, imports) in &self.imports {
            if resolved_imports.insert(canonical_feature_name.clone()) {
                self.resolve_imports(canonical_feature_name, imports, &mut resolved_imports);
            }
        }

        Ok(OptionsProvider::new(&self.aliases, &self.sources))
    }

    fn get_canonical_feature_name(&self, path: &Path, directory: &Path) -> String {
        path.strip_prefix(directory)
            .unwrap()
            .with_extension("")
            .to_str()
            .unwrap()
            .to_owned()
    }

    fn resolve_imports(
        &self,
        canonical_feature_name: &str,
        imports_for_feature: &[String],
        resolved_imports: &mut HashSet<String>,
    ) -> Result<(), String> {
        // TODO Validate imports.
        // Gather errors.
        // All imports must be canonical feature names for clarity and to help navigate to the right file.
        // If any are aliases, then show a nice error message to say what to change it to.

        // Build each import so that we don't need to traverse at runtime.
        let mut config_builder = config::Config::builder();
        for import in imports_for_feature {
            // Make sure it's a canonical feature name by checking if it's in `self.sources`.
            let mut source = self.sources.get(import).unwrap().clone();
            if resolved_imports.insert(import.clone()) {
                match self.imports.get(import) {
                    None => {}
                    Some(imports_for_import) => {
                        self.resolve_imports(import, imports_for_import, resolved_imports);
                    }
                }

                // Get the source again because it may have been updated after resolving imports.
                let mut source = self.sources.get(import).unwrap().clone();
            }

            config_builder = config_builder.add_source(source);
        }

        let new_config = config_builder.build();
        match new_config {
            Ok(cfg) => {
                let key = canonical_feature_name.to_owned();
                // FIXME Convert to something that can be inserted as a source.
                let res = self.sources.insert(key, cfg);
                if res.is_some() {
                    panic!("Duplicate key found: `{}`", key);
                }
            }
            Err(e) => {
                return Err(format!(
                    "Error building configuration for feature {:?}: {:?}",
                    canonical_feature_name, e
                ))
            }
        }

        Ok(())
    }
}
