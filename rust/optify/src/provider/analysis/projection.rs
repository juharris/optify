use serde_json::Value;

/// Keeps only the parts of `value` that can affect the merged value at the path with `segments`.
///
/// Siblings of the path never affect the merged value at the path,
/// so only ancestors and the subtree at the path need to be merged.
/// Objects on the path are kept even when they do not have the next segment
/// because merging an object behaves differently than merging a non-object or a missing value.
#[must_use]
pub(super) fn project(value: &Value, segments: &[String]) -> Value {
    match (segments.split_first(), value) {
        (Some((segment, rest)), Value::Object(map)) => {
            let mut projected = serde_json::Map::new();
            if let Some(child) = map.get(segment) {
                projected.insert(segment.clone(), project(child, rest));
            }
            Value::Object(projected)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::merge::{merge_json_with_defaults, FrozenPaths};
    use crate::json::parse_json_pointer::parse_json_pointer;
    use serde_json::json;

    /// Merges sources from highest to lowest priority.
    fn merge_all(sources: &[Value]) -> Value {
        let mut result = sources[0].clone();
        let mut frozen_paths = FrozenPaths::new();
        for source in &sources[1..] {
            merge_json_with_defaults(&mut result, source, &mut frozen_paths);
        }
        result
    }

    #[test]
    fn test_project() {
        let segments = parse_json_pointer("/k/x").unwrap();
        assert_eq!(
            project(&json!({"k": {"x": {"deep": 1}, "y": 2}, "z": 3}), &segments),
            json!({"k": {"x": {"deep": 1}}})
        );
        assert_eq!(
            project(&json!({"k": {"y": 2}}), &segments),
            json!({"k": {}})
        );
        assert_eq!(project(&json!({"k": 5}), &segments), json!({"k": 5}));
        assert_eq!(project(&json!({"z": 3}), &segments), json!({}));
    }

    /// Analysis relies on merging projections being the same as projecting the merged value.
    #[test]
    fn test_project_of_merge_equals_merge_of_projections() {
        let cases = [
            vec![
                json!({"l1": {"a": 1, "b": 2}}),
                json!({"l1": {"b": 3, "c": 4}}),
            ],
            vec![
                json!({"key/with~special": {"high": true}}),
                json!({"key/with~special": false}),
                json!({"key/with~special": {"low": true}}),
            ],
            vec![json!({"arr": [1, 2, 3]}), json!({"arr": [4, 5]})],
            vec![
                json!({"k": {"y": 1}}),
                json!({"k": 5}),
                json!({"k": {"x": 3}}),
            ],
            vec![
                json!({"k": {"a": {"b": 1}}}),
                json!({"k": {"a": null}}),
                json!({"k": {"a": {"c": 2}, "d": [1]}}),
                json!({"k": "low"}),
            ],
        ];
        let pointers = [
            "/l1",
            "/l1/b",
            "/l1/c",
            "/key~1with~0special/low",
            "/arr/0",
            "/k/x",
            "/k/y",
            "/k/a/b",
            "/k/a/c",
            "/k/d",
            "/missing/path",
        ];
        for sources in cases {
            let merged = merge_all(&sources);
            for pointer in pointers {
                let segments = parse_json_pointer(pointer).unwrap();
                let projected_sources: Vec<Value> = sources
                    .iter()
                    .map(|source| project(source, &segments))
                    .collect();
                assert_eq!(
                    project(&merged, &segments),
                    merge_all(&projected_sources),
                    "sources: {sources:?}, pointer: {pointer:?}"
                );
            }
        }
    }
}
