use config;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::builder::OptionsRegistryBuilder;
use crate::provider::{Aliases, Conditions, Features, OptionsProvider, Sources};
use crate::schema::conditions::ConditionExpression;
use crate::schema::feature::FeatureConfiguration;
use crate::schema::metadata::OptionsMetadata;

type Imports = HashMap<String, Vec<String>>;

/// A builder to use in production to create an `OptionsProvider`.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public yet and mainly available to support bindings for other languages.
#[derive(Clone)]
pub struct OptionsProviderBuilder {
    aliases: Aliases,
    conditions: Conditions,
    features: Features,
    imports: Imports,
    sources: Sources,
}

impl Default for OptionsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn add_alias(
    aliases: &mut Aliases,
    alias: &String,
    canonical_feature_name: &String,
) -> Result<(), String> {
    let uni_case_alias = unicase::UniCase::new(alias.clone());
    if let Some(ref res) = aliases.insert(uni_case_alias, canonical_feature_name.clone()) {
        return Err(format!(
            "The alias '{alias}' for canonical feature name '{canonical_feature_name}' is already mapped to '{res}'."
        ));
    }
    Ok(())
}

fn get_canonical_feature_name(path: &Path, directory: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap()
        .with_extension("")
        .to_str()
        .expect("path should be valid Unicode")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(clippy::too_many_arguments)]
fn resolve_imports(
    canonical_feature_name: &str,
    imports_for_feature: &[String],
    resolved_imports: &mut HashSet<String>,
    features_in_resolution_path: &mut HashSet<String>,
    aliases: &Aliases,
    all_imports: &Imports,
    sources: &mut Sources,
    conditions: &Conditions,
) -> Result<(), String> {
    // Build full configuration for the feature so that we don't need to traverse imports for the feature when configurations are requested from the provider.
    let mut config_builder = config::Config::builder();
    for import in imports_for_feature {
        // Validate imports.
        if !features_in_resolution_path.insert(import.clone()) {
            // The import is already in the path, so there is a cycle.
            return Err(format!(
                    "Error when resolving imports for '{canonical_feature_name}': Cycle detected with import '{import}'. The features in the path (not in order): {features_in_resolution_path:?}"
                ));
        }

        if conditions.contains_key(import) {
            return Err(format!(
                "Error when resolving imports for '{canonical_feature_name}': The import '{import}' \
                 has conditions. Conditions cannot be used in imported features. This helps keep \
                 retrieving and building configuration options for a list of features fast and more \
                 predictable because imports do not need to be re-evaluated. Instead, keep each \
                 feature file as granular and self-contained as possible, then use conditions and \
                 import the required granular features in a feature file that defines a common \
                 scenario."
            ));
        }

        // Get the source so that we can build the configuration.
        // Getting the source also ensures the import is a canonical feature name.
        let mut source = match sources.get(import) {
                Some(s) => s,
                // The import is not a canonical feature name.
                None => match aliases.get(&unicase::UniCase::new(import.clone())) {
                    Some(canonical_name_for_import) => {
                        return Err(format!(
                            "Error when resolving imports for '{canonical_feature_name}': The import '{import}' is not a canonical feature name. Use '{canonical_name_for_import}' instead of '{import}' in order to keep dependencies clear and to help with navigating through files."
                        ))
                    }
                    None => {
                        return Err(format!(
                            "Error when resolving imports for '{canonical_feature_name}': The import '{import}' is not a canonical feature name and not a recognized alias. Use a canonical feature name in order to keep dependencies clear and to help with navigating through files."
                        ))
                    }
                },
            };
        if resolved_imports.insert(import.clone()) {
            if let Some(imports_for_import) = all_imports.get(import) {
                let mut _features_in_resolution_path = features_in_resolution_path.clone();
                _features_in_resolution_path.insert(import.clone());
                resolve_imports(
                    import,
                    imports_for_import,
                    resolved_imports,
                    &mut _features_in_resolution_path,
                    aliases,
                    all_imports,
                    sources,
                    conditions,
                )?
            }

            // Get the source again because it may have been updated after resolving imports.
            source = sources.get(import).unwrap();
        }

        config_builder = config_builder.add_source(source.clone());
    }

    // Include the current feature's configuration last to override any imports.
    let source = sources.get(canonical_feature_name).unwrap();
    config_builder = config_builder.add_source(source.clone());

    // Build the configuration and store it.
    match config_builder.build() {
        Ok(new_config) => {
            // Convert to something that can be inserted as a source.
            let options_as_config_value: config::Value = match new_config.try_deserialize() {
                Ok(v) => v,
                Err(e) => {
                    // Should never happen.
                    return Err(format!(
                        "Error deserializing feature configuration for '{canonical_feature_name}': {e}"
                    ));
                }
            };
            let options_as_json: serde_json::Value = options_as_config_value
                .try_deserialize()
                .expect("configuration should be deserializable to JSON");
            let options_as_json_str = serde_json::to_string(&options_as_json).unwrap();
            let source = config::File::from_str(&options_as_json_str, config::FileFormat::Json);
            sources.insert(canonical_feature_name.to_owned(), source);
        }
        Err(e) => {
            return Err(format!(
                "Error building configuration for feature {canonical_feature_name:?}: {e:?}"
            ))
        }
    }

    Ok(())
}

/// The result of loading a feature configuration file.
struct LoadingResult {
    canonical_feature_name: String,
    conditions: Option<ConditionExpression>,
    source: config::File<config::FileSourceString, config::FileFormat>,
    imports: Option<Vec<String>>,
    metadata: OptionsMetadata,
}

impl OptionsProviderBuilder {
    pub fn new() -> Self {
        OptionsProviderBuilder {
            aliases: Aliases::new(),
            conditions: Conditions::new(),
            features: Features::new(),
            imports: HashMap::new(),
            sources: Sources::new(),
        }
    }

    fn process_entry(path: &Path, directory: &Path) -> Option<Result<LoadingResult, String>> {
        if !path.is_file()
            // Skip .md files because they are not handled by the `config` library, but there may be README.md files in the directory.
            || path.extension().filter(|e| *e == "md").is_some()
            // Skip .optify folders because they mark settings such as the root folder.
            || path
                .components()
                .any(|component| component.as_os_str() == ".optify")
        {
            return None;
        }

        // TODO Optimization: Find a more efficient way to build a more generic view of the file.
        // The `config` library is helpful because it handles many file types.
        // It would also be nice to support comments in .json files, even though it is not standard.
        // The `config` library does support .json5 which supports comments.
        let file = config::File::from(path);
        let config_for_path = match config::Config::builder().add_source(file).build() {
            Ok(conf) => conf,
            Err(e) => {
                return Some(Err(format!(
                    "Error loading file '{}': {e}",
                    path.to_string_lossy(),
                )))
            }
        };

        let feature_config: FeatureConfiguration = match config_for_path.try_deserialize() {
            Ok(v) => v,
            Err(e) => {
                return Some(Err(format!(
                    "Error deserializing configuration for file '{}': {e}",
                    path.display(),
                )))
            }
        };

        let options_as_json_str = match feature_config.options {
            Some(options) => match options.try_deserialize::<serde_json::Value>() {
                Ok(options_as_json) => serde_json::to_string(&options_as_json).unwrap(),
                Err(e) => {
                    return Some(Err(format!(
                        "Error deserializing options for '{}': {e}",
                        path.display()
                    )))
                }
            },
            None => "{}".to_owned(),
        };
        let source = config::File::from_str(&options_as_json_str, config::FileFormat::Json);
        let canonical_feature_name = get_canonical_feature_name(path, directory);

        // Ensure the name is set in the metadata.
        let metadata = match feature_config.metadata {
            Some(mut metadata) => {
                metadata.name = Some(canonical_feature_name.clone());
                metadata
            }
            None => OptionsMetadata::new(None, None, Some(canonical_feature_name.clone()), None),
        };

        Some(Ok(LoadingResult {
            canonical_feature_name,
            conditions: feature_config.conditions,
            source,
            imports: feature_config.imports,
            metadata,
        }))
    }

    fn process_loading_result(
        &mut self,
        loading_result: &Result<LoadingResult, String>,
    ) -> Result<(), String> {
        let info = loading_result.as_ref()?;
        let canonical_feature_name = &info.canonical_feature_name;
        if self
            .sources
            .insert(canonical_feature_name.clone(), info.source.clone())
            .is_some()
        {
            return Err(format!(
                "Error when loading feature. The canonical feature name '{canonical_feature_name}' was already added. It may be an alias for another feature."
            ));
        }
        if let Some(conditions) = &info.conditions {
            self.conditions
                .insert(canonical_feature_name.clone(), conditions.clone());
        }
        if let Some(imports) = &info.imports {
            self.imports
                .insert(canonical_feature_name.clone(), imports.clone());
        }
        add_alias(
            &mut self.aliases,
            canonical_feature_name,
            canonical_feature_name,
        )?;
        if let Some(ref aliases) = info.metadata.aliases {
            for alias in aliases {
                add_alias(&mut self.aliases, alias, canonical_feature_name)?;
            }
        }
        self.features
            .insert(canonical_feature_name.clone(), info.metadata.clone());
        Ok(())
    }
}

impl OptionsRegistryBuilder<OptionsProvider> for OptionsProviderBuilder {
    fn add_directory(&mut self, directory: &Path) -> Result<&Self, String> {
        if !directory.is_dir() {
            return Err(format!(
                "Error adding directory: {directory:?} is not a directory"
            ));
        }

        let loading_results: Vec<Result<LoadingResult, String>> = walkdir::WalkDir::new(directory)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| {
                Self::process_entry(
                    entry
                        .unwrap_or_else(|_| {
                            panic!("Error walking directory: {}", directory.display())
                        })
                        .path(),
                    directory,
                )
            })
            .collect();
        for loading_result in loading_results {
            self.process_loading_result(&loading_result)?;
        }

        Ok(self)
    }

    fn build(&mut self) -> Result<OptionsProvider, String> {
        let mut resolved_imports: HashSet<String> = HashSet::new();
        for (canonical_feature_name, imports_for_feature) in &self.imports {
            if resolved_imports.insert(canonical_feature_name.clone()) {
                // Check for infinite loops by starting a path here.
                let mut features_in_resolution_path: HashSet<String> =
                    HashSet::from([canonical_feature_name.clone()]);
                resolve_imports(
                    canonical_feature_name,
                    imports_for_feature,
                    &mut resolved_imports,
                    &mut features_in_resolution_path,
                    &self.aliases,
                    &self.imports,
                    &mut self.sources,
                    &self.conditions,
                )?;
            }
        }

        Ok(OptionsProvider::new(
            &self.aliases,
            &self.conditions,
            &self.features,
            &self.sources,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_canonical_feature_name() {
        let directory = std::path::Path::new("wtv");
        let path = directory.join("dir1").join("dir2").join("feature_B.json");
        assert_eq!(
            "dir1/dir2/feature_B",
            get_canonical_feature_name(&path, directory)
        );
    }
}
