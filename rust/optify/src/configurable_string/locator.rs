// Deep recursively search for JSON Pointers to objects
// that have a `"$type"` property with a value of "Optify.ConfigurableString".
pub(crate) fn find_configurable_value_pointers(options: Option<&serde_json::Value>) -> Vec<String> {
    const TYPE: &str = "Optify.ConfigurableString";
    let mut result = Vec::new();
    result
}
