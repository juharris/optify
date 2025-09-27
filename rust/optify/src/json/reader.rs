use std::{fs::File, io::BufReader, path::Path};

use serde::de::DeserializeOwned;

pub(crate) fn read_json_from_file(
    path: impl AsRef<Path>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    read_json_from_file_as(path)
}

pub(crate) fn read_json_from_file_as<T>(
    path: impl AsRef<Path>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}
