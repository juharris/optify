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
}
