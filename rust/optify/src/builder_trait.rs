use std::path::Path;

use crate::OptionsProviderTrait;

/// Trait defining the core functionality for building an options provider.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub trait OptionsProviderBuilderTrait<T: OptionsProviderTrait> {
    /// Adds a directory containing feature configurations.
    fn add_directory(&mut self, directory: &Path) -> Result<&Self, String>;

    /// Builds the options provider.
    fn build(&mut self) -> Result<T, String>;
}
