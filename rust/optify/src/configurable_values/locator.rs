use crate::json::escape_json_pointer;

pub(crate) const TYPE_KEY: &str = "$type";
pub(crate) const STRING_TYPE: &str = "Optify.ConfigurableString";
pub(crate) const LIST_TYPE: &str = "Optify.ConfigurableList";

pub(crate) struct ConfigurableValuePointers {
    pub configurable_string_pointers: Vec<String>,
    pub configurable_list_pointers: Vec<String>,
}

/// Finds pointers like JSON pointers to configurable values
/// that have a `"$type"` property with a supported value.
pub(crate) fn find_configurable_values(
    options: Option<&serde_json::Value>,
) -> ConfigurableValuePointers {
    let mut result = ConfigurableValuePointers {
        configurable_string_pointers: Vec::new(),
        configurable_list_pointers: Vec::new(),
    };

    if let Some(value) = options {
        find_configurable_values_recursive(value, "", &mut result);
    }

    result
}

fn find_configurable_values_recursive(
    value: &serde_json::Value,
    current_pointer: &str,
    result: &mut ConfigurableValuePointers,
) {
    match value {
        serde_json::Value::Object(obj) => {
            // Check if this object is configurable.
            if let Some(type_value) = obj.get(TYPE_KEY) {
                match type_value.as_str() {
                    Some(STRING_TYPE) => {
                        result
                            .configurable_string_pointers
                            .push(current_pointer.to_string());
                        // Do not recurse because configurable strings cannot contain nested configurable values.
                        return;
                    }
                    Some(LIST_TYPE) => {
                        result
                            .configurable_list_pointers
                            .push(current_pointer.to_string());
                        // Continue recursing because configurable lists can contain nested configurable values such as strings.
                    }
                    _ => {}
                }
            }

            // Recursively search object properties
            for (key, val) in obj {
                escape_json_pointer!(key);
                let new_path = if current_pointer.is_empty() {
                    key.to_string()
                } else {
                    format!("{current_pointer}/{key}")
                };
                find_configurable_values_recursive(val, &new_path, result);
            }
        }
        serde_json::Value::Array(arr) => {
            // Recursively search array elements
            for (index, val) in arr.iter().enumerate() {
                let new_path = if current_pointer.is_empty() {
                    format!("{index}")
                } else {
                    format!("{current_pointer}/{index}")
                };
                find_configurable_values_recursive(val, &new_path, result);
            }
        }
        _ => {
            // For primitive values (string, number, boolean, null), do nothing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // TODO Add tests for finding configurable lists.

    #[test]
    fn test_configurable_string_at_root() {
        let json_value = json!({
            TYPE_KEY: STRING_TYPE,
            "base": "Root level configurable string",
            "arguments": {}
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(pointers.configurable_string_pointers, vec!["".to_string()]);
    }

    #[test]
    fn test_find_single_configurable_string() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: STRING_TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {}
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["feature".to_string()]
        );
    }

    #[test]
    fn test_find_nested_configurable_string() {
        let json_value = json!({
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: STRING_TYPE,
                        "base": "Deep nested",
                        "arguments": {}
                    }
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["nested/deep/value".to_string()]
        );
    }

    #[test]
    fn test_find_configurable_string_in_array() {
        let json_value = json!({
            "array": [
                {
                    TYPE_KEY: STRING_TYPE,
                    "base": "Array item",
                    "arguments": {}
                }
            ]
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["array/0".to_string()]
        );
    }

    #[test]
    fn test_find_multiple_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: STRING_TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {}
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: STRING_TYPE,
                        "base": "Deep nested",
                        "arguments": {}
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: STRING_TYPE,
                    "base": "Array item",
                    "arguments": {}
                },
                {
                    "regular": "object"
                },
                {
                    TYPE_KEY: STRING_TYPE,
                    "base": "Second array item",
                    "arguments": {}
                }
            ],
            "regular": "not configurable"
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec![
                "array/0".to_string(),
                "array/2".to_string(),
                "feature".to_string(),
                "nested/deep/value".to_string()
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let pointers = find_configurable_values(None);
        assert!(pointers.configurable_list_pointers.is_empty());
        assert!(pointers.configurable_string_pointers.is_empty());

        let pointers = find_configurable_values(Some(&json!({})));
        assert!(pointers.configurable_list_pointers.is_empty());
        assert!(pointers.configurable_string_pointers.is_empty());

        let pointers = find_configurable_values(Some(&json!([])));
        assert!(pointers.configurable_list_pointers.is_empty());
        assert!(pointers.configurable_string_pointers.is_empty());
    }

    #[test]
    fn test_no_configurable_strings() {
        let json_value = json!({
            "regular": "value",
            "number": 42,
            "boolean": true,
            "null_value": null,
            "array": [1, 2, 3],
            "object": {
                "nested": "value",
                "more_nested": {
                    "deep": "value"
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));
        assert!(pointers.configurable_list_pointers.is_empty());
        assert!(pointers.configurable_string_pointers.is_empty());
    }

    #[test]
    fn test_wrong_type_value() {
        let json_value = json!({
            "feature": {
                "$type": "SomeOtherType",
                "base": "Hello {{ name }}!",
                "arguments": {}
            },
            "another": {
                "$type": 42, // Not a string
                "base": "Hello",
                "arguments": {}
            }
        });

        let pointers = find_configurable_values(Some(&json_value));
        assert!(pointers.configurable_list_pointers.is_empty());
        assert!(pointers.configurable_string_pointers.is_empty());
    }

    #[test]
    fn test_deeply_nested_structure() {
        let json_value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                TYPE_KEY: STRING_TYPE,
                                "base": "Very deep",
                                "arguments": {}
                            }
                        }
                    }
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["level1/level2/level3/level4/level5".to_string()]
        );
    }

    #[test]
    fn test_complex_array_structure() {
        let json_value = json!({
            "items": [
                [
                    {
                        TYPE_KEY: STRING_TYPE,
                        "base": "Nested array item",
                        "arguments": {}
                    }
                ],
                {
                    "nested_object": {
                        TYPE_KEY: STRING_TYPE,
                        "base": "Object in array",
                        "arguments": {}
                    }
                }
            ]
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["items/0/0".to_string(), "items/1/nested_object".to_string()]
        );
    }

    #[test]
    fn test_does_not_recurse_into_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: STRING_TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {
                    "nested": {
                        TYPE_KEY: STRING_TYPE, // This should not be found
                        "base": "Should not be found",
                        "arguments": {}
                    }
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        // Should only find the top-level configurable string, not the nested one
        assert_eq!(
            pointers.configurable_string_pointers,
            vec!["feature".to_string()]
        );
    }

    #[test]
    fn test_with_real_config_structure() {
        // Test with the structure from the test config file
        let json_value = json!({
            "feature": {
                TYPE_KEY: STRING_TYPE,
                "base": "Hello {{ name }}! Welcome to {{ resources.app_name }}.",
                "arguments": {
                    "simple.txt": "simple.txt",
                    "template.liquid": "template.liquid"
                }
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: STRING_TYPE,
                        "base": "Deep nested: {{ resources.template }}",
                        "arguments": {
                            "template.liquid": "template.liquid"
                        }
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: STRING_TYPE,
                    "base": "Array item: {{ index }}",
                    "arguments": {}
                }
            ],
            "regular_value": "not a configurable string"
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers.configurable_string_pointers,
            vec![
                "array/0".to_string(),
                "feature".to_string(),
                "nested/deep/value".to_string()
            ]
        );
    }
}
