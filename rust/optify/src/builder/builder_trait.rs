use std::path::Path;

use crate::provider::OptionsRegistry;

/// Trait defining the core functionality for building an options provider.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public yet and mainly available to support bindings for other languages.
pub trait OptionsRegistryBuilder<T: OptionsRegistry> {
    /// Adds a directory containing feature configurations.
    fn add_directory(&mut self, directory: impl AsRef<Path>) -> Result<&Self, String>;

    /// Sets a JSON schema for validation.
    /// When provided, files will be validated against this schema during loading when `add_directory` is called.
    /// Must be called before `add_directory` to take effect.
    fn with_schema(&mut self, schema_path: impl AsRef<Path>) -> Result<&Self, String>;

    /// Builds the options provider.
    fn build(&mut self) -> Result<T, String>;
}
