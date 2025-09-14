const TYPE_KEY: &str = "$type";
const TYPE: &str = "Optify.ConfigurableString";

// Deep recursively search for JSON Pointers to objects
// that have a `"$type"` property with a value of "Optify.ConfigurableString".
pub(crate) fn find_configurable_value_pointers(options: Option<&serde_json::Value>) -> Vec<String> {
    let mut result = Vec::new();

    if let Some(value) = options {
        find_configurable_strings_recursive(value, "", &mut result);
    }

    result
}

fn find_configurable_strings_recursive(
    value: &serde_json::Value,
    current_path: &str,
    result: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(obj) => {
            // Check if this object is a configurable string
            if let Some(type_value) = obj.get(TYPE_KEY) {
                if let Some(type_str) = type_value.as_str() {
                    if type_str == TYPE {
                        result.push(current_path.to_string());
                        return; // Don't recurse into configurable string objects
                    }
                }
            }

            // Recursively search object properties
            for (key, val) in obj {
                let new_path = if current_path.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", current_path, key)
                };
                find_configurable_strings_recursive(val, &new_path, result);
            }
        }
        serde_json::Value::Array(arr) => {
            // Recursively search array elements
            for (index, val) in arr.iter().enumerate() {
                let new_path = format!("{}/{}", current_path, index);
                find_configurable_strings_recursive(val, &new_path, result);
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

    #[test]
    fn test_configurable_string_at_root() {
        let json_value = json!({
            TYPE_KEY: TYPE,
            "root": "Root level configurable string",
            "components": {}
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(pointers, vec!["".to_string()]);
    }

    #[test]
    fn test_find_single_configurable_string() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "root": "Hello {{ name }}!",
                "components": {}
            }
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(pointers, vec!["/feature".to_string()]);
    }

    #[test]
    fn test_find_nested_configurable_string() {
        let json_value = json!({
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "root": "Deep nested",
                        "components": {}
                    }
                }
            }
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(pointers, vec!["/nested/deep/value".to_string()]);
    }

    #[test]
    fn test_find_configurable_string_in_array() {
        let json_value = json!({
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "root": "Array item",
                    "components": {}
                }
            ]
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(pointers, vec!["/array/0".to_string()]);
    }

    #[test]
    fn test_find_multiple_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "root": "Hello {{ name }}!",
                "components": {}
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "root": "Deep nested",
                        "components": {}
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "root": "Array item",
                    "components": {}
                },
                {
                    "regular": "object"
                },
                {
                    TYPE_KEY: TYPE,
                    "root": "Second array item",
                    "components": {}
                }
            ],
            "regular": "not configurable"
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(
            pointers,
            vec![
                "/array/0".to_string(),
                "/array/2".to_string(),
                "/feature".to_string(),
                "/nested/deep/value".to_string()
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let pointers = find_configurable_value_pointers(None);
        assert!(pointers.is_empty());

        let pointers = find_configurable_value_pointers(Some(&json!({})));
        assert!(pointers.is_empty());

        let pointers = find_configurable_value_pointers(Some(&json!([])));
        assert!(pointers.is_empty());
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

        let pointers = find_configurable_value_pointers(Some(&json_value));
        assert!(pointers.is_empty());
    }

    #[test]
    fn test_wrong_type_value() {
        let json_value = json!({
            "feature": {
                "$type": "SomeOtherType",
                "root": "Hello {{ name }}!",
                "components": {}
            },
            "another": {
                "$type": 42, // Not a string
                "root": "Hello",
                "components": {}
            }
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));
        assert!(pointers.is_empty());
    }

    #[test]
    fn test_deeply_nested_structure() {
        let json_value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                TYPE_KEY: TYPE,
                                "root": "Very deep",
                                "components": {}
                            }
                        }
                    }
                }
            }
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(
            pointers,
            vec!["/level1/level2/level3/level4/level5".to_string()]
        );
    }

    #[test]
    fn test_complex_array_structure() {
        let json_value = json!({
            "items": [
                [
                    {
                        TYPE_KEY: TYPE,
                        "root": "Nested array item",
                        "components": {}
                    }
                ],
                {
                    "nested_object": {
                        TYPE_KEY: TYPE,
                        "root": "Object in array",
                        "components": {}
                    }
                }
            ]
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(
            pointers,
            vec![
                "/items/0/0".to_string(),
                "/items/1/nested_object".to_string()
            ]
        );
    }

    #[test]
    fn test_does_not_recurse_into_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "root": "Hello {{ name }}!",
                "components": {
                    "nested": {
                        TYPE_KEY: TYPE, // This should not be found
                        "root": "Should not be found",
                        "components": {}
                    }
                }
            }
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        // Should only find the top-level configurable string, not the nested one
        assert_eq!(pointers, vec!["/feature".to_string()]);
    }

    #[test]
    fn test_with_real_config_structure() {
        // Test with the structure from the test config file
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "root": "Hello {{ name }}! Welcome to {{ resources.app_name }}.",
                "components": {
                    "simple.txt": "simple.txt",
                    "template.liquid": "template.liquid"
                }
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "root": "Deep nested: {{ resources.template }}",
                        "components": {
                            "template.liquid": "template.liquid"
                        }
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "root": "Array item: {{ index }}",
                    "components": {}
                }
            ],
            "regular_value": "not a configurable string"
        });

        let pointers = find_configurable_value_pointers(Some(&json_value));

        assert_eq!(
            pointers,
            vec![
                "/array/0".to_string(),
                "/feature".to_string(),
                "/nested/deep/value".to_string()
            ]
        );
    }
}
