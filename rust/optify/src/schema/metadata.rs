// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsMetadata.cs

use serde::{Deserialize, Serialize};

type Details = serde_json::Value;

/// Information about a group of options.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct OptionsMetadata {
    // TODO Add more props.
    /// Alternative names for the group of options.
    ///
    /// This is helpful for using custom short names for the group of options.
    pub aliases: Option<Vec<String>>,

    /// Other metadata that may be custom and application specific.
    ///
    /// This is a good place for documentation that should be available to the application.
    /// Comments that are not parsed are still great to have in files, when supported,
    /// but some comments are useful to have here because they can be accessed programmatically.
    pub details: Option<Details>,

    /// The name of the group of options.
    ///
    /// This may be derived from the file name including subdirectories.
    /// Should never be <tt>None</tt> or an empty string.
    /// When loading the options from a file, the name is automatically derived from the file name.
    pub name: Option<String>,

    /// The creators or maintainers of this group of options.
    ///
    /// For example, emails separated by ";".
    pub owners: Option<String>,
}

impl OptionsMetadata {
    pub fn new(
        aliases: Option<Vec<String>>,
        details: Option<Details>,
        name: Option<String>,
        owners: Option<String>,
    ) -> Self {
        Self {
            aliases,
            details,
            name,
            owners,
        }
    }
}
