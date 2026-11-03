use optify::Feature;

/// Creates a `Feature` named `name` with options `{"test": value}`.
#[must_use]
pub fn feature_from_json(name: &str, value: i64) -> Feature {
    serde_json::from_value(serde_json::json!({
        "metadata": {"name": name},
        "options": {"test": value},
    }))
    .unwrap()
}
