/// Merges objects recursively with a target for the right information and defaults to use if the target is missing a value.
/// The `target` is the final value after many merges.
/// `defaults` has values to use if the `target` is missing a value.
/// I.e., values in `target` override values in `defaults`.
/// If a key is missing from `target`, it is copied from `defaults`.
/// If the key is present in both `target` and `defaults` and the value is an object, the objects are merged recursively.
#[inline]
pub fn merge_json_with_defaults(target: &mut serde_json::Value, defaults: &serde_json::Value) {
    match (target, defaults) {
        (serde_json::Value::Object(target_map), serde_json::Value::Object(defaults_map)) => {
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
            // Either:
            // defaults is not an object, but target is already defined so use the target.
            // target is not an object, so use it as is.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_with_defaults() {
        let mut target = json!({"a": 1, "b": 2});
        let defaults = json!({"b": 3, "c": 4});
        merge_json_with_defaults(&mut target, &defaults);
        assert_eq!(target, json!({"a": 1, "b": 2, "c": 4}));
    }

    #[test]
    fn test_merge_with_defaults_nested() {
        let mut target = json!({
            "level1": {
                "a": 1,
                "b": 2
            }
        });
        let defaults = json!({
            "level1": {
                "b": 3,
                "c": 4
            }
        });
        merge_json_with_defaults(&mut target, &defaults);
        assert_eq!(
            target,
            json!({
                "level1": {
                    "a": 1,
                    "b": 2,
                    "c": 4
                }
            })
        );
    }

    #[test]
    fn test_merge_with_defaults_deeply_nested() {
        let mut target = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "a": 1
                    }
                }
            }
        });
        let defaults = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "a": 57,
                        "b": 2
                    },
                    "new": "value"
                }
            }
        });
        merge_json_with_defaults(&mut target, &defaults);
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

    #[test]
    fn test_merge_with_defaults_arrays_not_merged() {
        let mut target = json!({"arr": [1, 2, 3]});
        let defaults = json!({"arr": [4, 5]});
        merge_json_with_defaults(&mut target, &defaults);
        // Target array is preserved, not replaced or merged.
        assert_eq!(target, json!({"arr": [1, 2, 3]}));
    }
}
