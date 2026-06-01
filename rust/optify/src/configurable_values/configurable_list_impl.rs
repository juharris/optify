use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Item {
    #[serde(alias = "$order")]
    order: Option<f64>,
    #[serde(alias = "$value")]
    value: serde_json::Value,
}

/// Helps build a list by components.
/// Parsed from a `serde_json::Value`.
#[derive(Deserialize, Debug)]
pub(crate) struct ConfigurableList {
    items: HashMap<String, Item>,
}

impl ConfigurableList {
    pub fn build(&self) -> Result<Vec<serde_json::Value>, String> {
        // TODO implement
        Ok(Vec::new())
    }
}
