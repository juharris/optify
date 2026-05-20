use crate::configurable_string::ConfigurableString;

pub(crate) fn extract_configurable_string_files_from_config(
    raw_config: &serde_json::Value,
    configurable_value_pointers: &[String],
) -> Vec<String> {
    let mut configurable_string_files = Vec::new();
    let options_obj = match raw_config.get("options") {
        Some(v) => v,
        None => return configurable_string_files,
    };

    for pointer in configurable_value_pointers {
        let json_pointer = format!("/{}", pointer);
        if let Some(configurable_value) = options_obj.pointer(&json_pointer) {
            if let Ok(cs) = serde_json::from_value::<ConfigurableString>(configurable_value.clone())
            {
                configurable_string_files.extend(cs.get_referenced_files());
            }
        }
    }

    configurable_string_files
}
