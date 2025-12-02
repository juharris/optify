/// Recursively merges `source` into `target`.
/// - Objects are merged recursively (keys from source override/extend target)
/// - Arrays and all other values are replaced entirely (not merged)
#[inline]
pub fn merge_json_value(target: &mut serde_json::Value, source: &serde_json::Value) {
    match (target, source) {
        (serde_json::Value::Object(target_map), serde_json::Value::Object(source_map)) => {
            for (key, source_value) in source_map {
                match target_map.get_mut(key) {
                    Some(target_value) => {
                        merge_json_value(target_value, source_value);
                    }
                    None => {
                        target_map.insert(key.clone(), source_value.clone());
                    }
                }
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

/// Similar to `merge_json_value`, but meant to be applied in the reverse order.
/// The `target` is the final value after many merges.
/// `defaults` is values to use if the `target` is missing a value.
/// I.e., values in `target` override values in `defaults`.
/// If a key is missing from `target`, it is copied from `defaults`.
/// If the key is present in both `target` and `defaults` and the value is an object, the objects are merged recursively.
#[inline]
pub fn merge_json_with_defaults(target: &mut serde_json::Value, defaults: &serde_json::Value) {
    match target {
        serde_json::Value::Object(target_map) => match defaults {
            serde_json::Value::Object(defaults_map) => {
                for (key, defaults_value) in defaults_map {
                    match target_map.get_mut(key) {
                        Some(target_value) => {
                            // They both have a value for this key.
                            merge_json_with_defaults(target_value, defaults_value);
                        }
                        None => {
                            // Default value is missing from target.
                            target_map.insert(key.clone(), defaults_value.clone());
                        }
                    }
                }
            }
            _ => {
                // Defaults is not an object, but target is an object so use the target.
            }
        },
        _ => {
            // Target is already defined so use it as no defaults are needed.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_simple_objects() {
        let mut target = json!({"a": 1, "b": 2});
        let source = json!({"b": 3, "c": 4});
        merge_json_value(&mut target, &source);
        assert_eq!(target, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_merge_nested_objects() {
        let mut target = json!({
            "level1": {
                "a": 1,
                "b": 2
            }
        });
        let source = json!({
            "level1": {
                "b": 3,
                "c": 4
            }
        });
        merge_json_value(&mut target, &source);
        assert_eq!(
            target,
            json!({
                "level1": {
                    "a": 1,
                    "b": 3,
                    "c": 4
                }
            })
        );
    }

    #[test]
    fn test_merge_arrays_replaced() {
        let mut target = json!({"arr": [1, 2, 3]});
        let source = json!({"arr": [4, 5]});
        merge_json_value(&mut target, &source);
        assert_eq!(target, json!({"arr": [4, 5]}));
    }

    #[test]
    fn test_merge_deeply_nested() {
        let mut target = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "a": 1
                    }
                }
            }
        });
        let source = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "b": 2
                    },
                    "new": "value"
                }
            }
        });
        merge_json_value(&mut target, &source);
        assert_eq!(
            target,
            json!({
                "l1": {
                    "l2": {
                        "l3": {
                            "a": 1,
                            "b": 2
                        },
                        "new": "value"
                    }
                }
            })
        );
    }

    #[test]
    fn test_merge_type_override() {
        let mut target = json!({"key": {"nested": 1}});
        let source = json!({"key": "string"});
        merge_json_value(&mut target, &source);
        assert_eq!(target, json!({"key": "string"}));
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut target = json!({"a": 1, "b": 2});
        let defaults = json!({"b": 3, "c": 4});
        merge_json_with_defaults(&mut target, &defaults);
        assert_eq!(target, json!({"a": 1, "b": 2, "c": 4}));
    }

    #[test]
    fn test_merge_with_defaults_type_override() {
        let mut target = json!({"key": {"nested": 1}});
        let defaults = json!({"key": "string"});
        merge_json_with_defaults(&mut target, &defaults);
        assert_eq!(target, json!({"key": {"nested": 1}}));
    }

    #[test]
    fn test_merge_with_defaults_missing_key() {
        let mut target = json!({"a": 1});
        let defaults = json!({"b": 2});
        merge_json_with_defaults(&mut target, &defaults);
        assert_eq!(target, json!({"a": 1, "b": 2}));
    }
}
