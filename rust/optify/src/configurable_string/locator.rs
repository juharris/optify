pub(crate) const TYPE_KEY: &str = "$type";
// TODO Think of a better name. Everything is "configurable".
// like PrecomputedValue? ConfigurableOption? OptifyOption?
pub(crate) const TYPE: &str = "Optify.ConfigurableString";

/// Finds pointers like JSON pointers to configurable values
// that have a `"$type"` property with a value of "Optify.ConfigurableString".
pub(crate) fn find_configurable_values(options: Option<&serde_json::Value>) -> Vec<String> {
    let mut result = Vec::new();

    if let Some(value) = options {
        find_configurable_strings_recursive(value, "", &mut result);
    }

    result
}

fn find_configurable_strings_recursive(
    value: &serde_json::Value,
    current_pointer: &str,
    result: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(obj) => {
            // Check if this object is a configurable string
            if let Some(type_value) = obj.get(TYPE_KEY) {
                if let Some(type_str) = type_value.as_str() {
                    if type_str == TYPE {
                        result.push(current_pointer.to_string());
                        return;
                    }
                }
            }

            // Recursively search object properties
            for (key, val) in obj {
                // Escape values in the key because "/" needs to be escaped.
                let key = key.replace("~", "~0").replace("/", "~1");
                let new_path = if current_pointer.is_empty() {
                    key.to_string()
                } else {
                    format!("{current_pointer}/{key}")
                };
                find_configurable_strings_recursive(val, &new_path, result);
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
            "base": "Root level configurable string",
            "arguments": {}
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(pointers, vec!["".to_string()]);
    }

    #[test]
    fn test_find_single_configurable_string() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {}
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(pointers, vec!["feature".to_string()]);
    }

    #[test]
    fn test_find_nested_configurable_string() {
        let json_value = json!({
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "base": "Deep nested",
                        "arguments": {}
                    }
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(pointers, vec!["nested/deep/value".to_string()]);
    }

    #[test]
    fn test_find_configurable_string_in_array() {
        let json_value = json!({
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "base": "Array item",
                    "arguments": {}
                }
            ]
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(pointers, vec!["array/0".to_string()]);
    }

    #[test]
    fn test_find_multiple_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {}
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "base": "Deep nested",
                        "arguments": {}
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "base": "Array item",
                    "arguments": {}
                },
                {
                    "regular": "object"
                },
                {
                    TYPE_KEY: TYPE,
                    "base": "Second array item",
                    "arguments": {}
                }
            ],
            "regular": "not configurable"
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers,
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
        assert!(pointers.is_empty());

        let pointers = find_configurable_values(Some(&json!({})));
        assert!(pointers.is_empty());

        let pointers = find_configurable_values(Some(&json!([])));
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

        let pointers = find_configurable_values(Some(&json_value));
        assert!(pointers.is_empty());
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
            pointers,
            vec!["level1/level2/level3/level4/level5".to_string()]
        );
    }

    #[test]
    fn test_complex_array_structure() {
        let json_value = json!({
            "items": [
                [
                    {
                        TYPE_KEY: TYPE,
                        "base": "Nested array item",
                        "arguments": {}
                    }
                ],
                {
                    "nested_object": {
                        TYPE_KEY: TYPE,
                        "base": "Object in array",
                        "arguments": {}
                    }
                }
            ]
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers,
            vec!["items/0/0".to_string(), "items/1/nested_object".to_string()]
        );
    }

    #[test]
    fn test_does_not_recurse_into_configurable_strings() {
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "base": "Hello {{ name }}!",
                "arguments": {
                    "nested": {
                        TYPE_KEY: TYPE, // This should not be found
                        "base": "Should not be found",
                        "arguments": {}
                    }
                }
            }
        });

        let pointers = find_configurable_values(Some(&json_value));

        // Should only find the top-level configurable string, not the nested one
        assert_eq!(pointers, vec!["feature".to_string()]);
    }

    #[test]
    fn test_with_real_config_structure() {
        // Test with the structure from the test config file
        let json_value = json!({
            "feature": {
                TYPE_KEY: TYPE,
                "base": "Hello {{ name }}! Welcome to {{ resources.app_name }}.",
                "arguments": {
                    "simple.txt": "simple.txt",
                    "template.liquid": "template.liquid"
                }
            },
            "nested": {
                "deep": {
                    "value": {
                        TYPE_KEY: TYPE,
                        "base": "Deep nested: {{ resources.template }}",
                        "arguments": {
                            "template.liquid": "template.liquid"
                        }
                    }
                }
            },
            "array": [
                {
                    TYPE_KEY: TYPE,
                    "base": "Array item: {{ index }}",
                    "arguments": {}
                }
            ],
            "regular_value": "not a configurable string"
        });

        let pointers = find_configurable_values(Some(&json_value));

        assert_eq!(
            pointers,
            vec![
                "array/0".to_string(),
                "feature".to_string(),
                "nested/deep/value".to_string()
            ]
        );
    }
}
