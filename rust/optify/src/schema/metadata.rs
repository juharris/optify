// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsMetadata.cs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct OptionsMetadata {
    // TODO Add more props.
    pub aliases: Option<Vec<String>>,
}
