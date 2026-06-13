fn collect_file_values(value: &serde_json::Value, files: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, nested_value) in obj {
                if key == "file" {
                    if let Some(file) = nested_value.as_str() {
                        files.push(file.to_string());
                    }
                }
                collect_file_values(nested_value, files);
            }
        }
        serde_json::Value::Array(arr) => {
            for nested_value in arr {
                collect_file_values(nested_value, files);
            }
        }
        _ => {}
    }
}

pub(crate) fn extract_files_from_config(raw_config: &serde_json::Value) -> Vec<String> {
    let mut result = Vec::new();
    let options_obj = match raw_config.get("options") {
        Some(v) => v,
        None => return result,
    };

    collect_file_values(options_obj, &mut result);

    result
}
