use std::{fs::File, io::BufReader, path::Path};

pub(crate) fn read_json_from_file(
    path: impl AsRef<Path>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}
