use config;
use std::path::Path;
use walkdir::WalkDir;

use crate::provider::{OptionsProvider, SourceValue};
use crate::schema::feature::FeatureConfiguration;

pub struct OptionsProviderBuilder {
    sources: std::collections::HashMap<String, SourceValue>,
}

impl OptionsProviderBuilder {
    pub fn new() -> Self {
        OptionsProviderBuilder {
            sources: std::collections::HashMap::new(),
        }
    }

    pub fn add_directory(mut self, directory: &Path) -> Result<Self, String> {
        for entry in WalkDir::new(directory) {
            let entry = entry.unwrap();
            // TODO Skip README.md files.
            if entry.path().is_file() {
                let path = entry.path();
                let file = config::File::from(path);
                // TODO Find a better way to build a more generic view of the file.
                // The config library is helpful because it handles many file types.
                let key = self.get_path_key(path, directory);
                print!(
                    "Adding source: {} with key {}\n",
                    path.to_string_lossy(),
                    key
                );
                let config = config::Config::builder().add_source(file).build().unwrap();
                println!("config: {:?}", config);
                let c: FeatureConfiguration = config.try_deserialize().unwrap();
                println!("FeatureConfiguration: {:?}", c);
                let options = c.options;
                println!("options: {:?}", options);
                // let source_as_json = serde_json::to_string(options.into_table().unwrap()).unwrap();
                // let source = config::File::from_str(&options.to_string(), config::FileFormat::Ron);
                let options_as_json = options.to_string();
                println!("options string: {:?}", options_as_json);
                let source = config::File::from_str(&options_as_json, config::FileFormat::Json);
                let res = self.sources.insert(key.clone(), source);
                if res.is_some() {
                    return Err(format!(
                        "Duplicate key found: `{}` for path `{}`",
                        key,
                        path.to_string_lossy()
                    ));
                }
                println!("");
            }
        }

        // TODO Add aliases.

        return Ok(self);
    }

    pub fn build(self) -> OptionsProvider {
        OptionsProvider::new(self.sources)
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
