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
    // Enable ignoring the "$type" key since it's at the same level keyed items.
    #[serde(rename = "$type")]
    _type: String,

    // Flatten to "hoist" the items to the same level, allowing for arbitrary keys.
    #[serde(flatten)]
    items: serde_json::Map<String, serde_json::Value>,
}

impl ConfigurableList {
    pub fn build(self) -> Result<Vec<serde_json::Value>, String> {
        // TODO Investigate memory optimizations.
        let mut items: Vec<Item> = self
            .items
            .into_values()
            .map(|value| {
                serde_json::from_value::<Option<Item>>(value)
                    .map_err(|e| format!("Failed to deserialize ConfigurableList item: {e}"))
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect();

        items.sort_by(|a, b| a.order.unwrap_or(0.0).total_cmp(&b.order.unwrap_or(0.0)));

        Ok(items.into_iter().map(|item| item.value).collect())
    }
}
