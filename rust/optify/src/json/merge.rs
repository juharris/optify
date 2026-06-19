use std::collections::HashSet;

pub(crate) type FrozenPaths = HashSet<String>;

/// Merges objects recursively while remembering paths where lower-priority defaults are blocked.
/// Merges objects recursively with a target for the correct information
/// and defaults to use if the target is missing a value.
/// The `target` is the final value after many merges.
/// `defaults` has values to use if the `target` is missing a value.
/// I.e., values in `target` override values in `defaults`.
/// If a key is missing from `target`, it is copied from `defaults`.
/// If the key is present in both `target` and `defaults` and the value is an object, the objects are merged recursively.
#[inline]
pub(crate) fn merge_json_with_defaults(
    target: &mut serde_json::Value,
    defaults: &serde_json::Value,
    frozen_paths: &mut FrozenPaths,
) {
    let mut path = String::new();
    merge_json_with_defaults_at_path(target, defaults, frozen_paths, &mut path);
}

#[inline]
fn merge_json_with_defaults_at_path(
    target: &mut serde_json::Value,
    defaults: &serde_json::Value,
    frozen_paths: &mut FrozenPaths,
    path: &mut String,
) {
    if frozen_paths.contains(path.as_str()) {
        return;
    }

    match (target, defaults) {
        (serde_json::Value::Object(target_map), serde_json::Value::Object(defaults_map)) => {
            for (key, defaults_value) in defaults_map {
                let previous_path_len = path.len();
                push_json_pointer_segment(path, key);

                if !frozen_paths.contains(path.as_str()) {
                    match target_map.get_mut(key) {
                        Some(target_value) => {
                            // They both have a value for this key.
                            merge_json_with_defaults_at_path(
                                target_value,
                                defaults_value,
                                frozen_paths,
                                path,
                            );
                        }
                        None => {
                            // Default value is missing from target.
                            target_map.insert(key.clone(), defaults_value.clone());
                        }
                    }
                }

                path.truncate(previous_path_len);
            }
        }
        (serde_json::Value::Object(_), _) => {
            // `defaults` is not an object.
            // A lower-priority primitive would have stopped deeper defaults if it had been the target.
            // Remember that lower-priority objects cannot add children at this path later
            // because the default would have overwritten them if we were merge for low to high,
            // but we merge from high priority to low priority to attempt to copy less.
            frozen_paths.insert(path.clone());
        }
        _ => {
            // `target` is not an object.
            // The target already has the winning value at this path.
            // Defaults only merge through this path when both values are objects.
        }
    }
}

fn push_json_pointer_segment(path: &mut String, key: &str) {
    path.push('/');
    for character in key.chars() {
        match character {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            _ => path.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn merge_json(target: &mut serde_json::Value, defaults: &serde_json::Value) {
        let mut frozen_paths = FrozenPaths::new();
        merge_json_with_defaults(target, defaults, &mut frozen_paths);
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut target = json!({"a": 1, "b": 2});
        let defaults = json!({"b": 3, "c": 4});
        merge_json(&mut target, &defaults);
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
        merge_json(&mut target, &defaults);
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
        merge_json(&mut target, &defaults);
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
        merge_json(&mut target, &defaults);
        assert_eq!(target, json!({"key": {"nested": 1}}));
    }

    #[test]
    fn test_merge_with_defaults_using_frozen_paths_blocks_lower_priority_object_defaults() {
        let mut target = json!({
            "key/with~special": {
                "high": true
            }
        });
        let middle_priority_defaults = json!({"key/with~special": false});
        let low_priority_defaults = json!({
            "key/with~special": {
                "low": true
            }
        });
        let mut frozen_paths = FrozenPaths::new();

        merge_json_with_defaults(&mut target, &middle_priority_defaults, &mut frozen_paths);
        merge_json_with_defaults(&mut target, &low_priority_defaults, &mut frozen_paths);

        assert_eq!(target, json!({"key/with~special": {"high": true}}));
        assert!(frozen_paths.contains("/key~1with~0special"));
    }

    #[test]
    fn test_merge_with_defaults_missing_key() {
        let mut target = json!({"a": 1});
        let defaults = json!({"b": 2});
        merge_json(&mut target, &defaults);
        assert_eq!(target, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_with_defaults_arrays_not_merged() {
        let mut target = json!({"arr": [1, 2, 3]});
        let defaults = json!({"arr": [4, 5]});
        merge_json(&mut target, &defaults);
        // Target array is preserved, not replaced or merged.
        assert_eq!(target, json!({"arr": [1, 2, 3]}));
    }
}
