// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsMetadata.cs

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct OptionsMetadata {
    // TODO Add more props.
    /// Alternative names for the group of options.
    /// This is helpful for using custom short names for the group of options.
    pub aliases: Option<Vec<String>>,
    /// The creators or maintainers of this group of options.
    /// For example, emails separated by ";".
    pub owners: Option<String>,
}
