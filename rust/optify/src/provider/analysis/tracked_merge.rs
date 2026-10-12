use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::origin::Origin;
use super::provenance::Provenance;
use crate::json::merge::push_json_pointer_segment;

/// Remembers which origin froze each path.
pub(super) type TrackedFrozenPaths = HashMap<String, Arc<Origin>>;

/// Merges like `merge_json_with_defaults` while tracking the origin of each value.
///
/// This must stay consistent with `merge_json_with_defaults`.
/// It is separate to avoid slowing down the merge that is used to get options.
pub(super) fn merge_json_with_defaults_tracked(
    target: &mut Value,
    target_provenance: &mut Provenance,
    defaults: &Value,
    defaults_provenance: &Provenance,
    frozen_paths: &mut TrackedFrozenPaths,
) {
    if let Some(frozen_by) = frozen_paths.get("") {
        target_provenance.block(defaults_provenance, frozen_by);
        return;
    }

    let mut path = String::new();
    merge_at_path(
        target,
        target_provenance,
        defaults,
        defaults_provenance,
        frozen_paths,
        &mut path,
    );
}

fn merge_at_path(
    target: &mut Value,
    target_provenance: &mut Provenance,
    defaults: &Value,
    defaults_provenance: &Provenance,
    frozen_paths: &mut TrackedFrozenPaths,
    path: &mut String,
) {
    match (target, defaults) {
        (Value::Object(target_map), Value::Object(defaults_map)) => {
            target_provenance.append_losses(defaults_provenance);
            if defaults_map.is_empty() {
                return;
            }

            target_provenance.expand(target_map);
            let children = target_provenance
                .children
                .as_mut()
                .expect("children should be set after expanding");
            for (key, defaults_value) in defaults_map {
                let previous_path_len = path.len();
                push_json_pointer_segment(path, key);
                let defaults_child = defaults_provenance.child(key);

                if let Some(frozen_by) = frozen_paths.get(path.as_str()) {
                    // The target has an object at a frozen path because the path was frozen when the target had an object there.
                    if let Some(target_child) = children.get_mut(key) {
                        target_child.block(&defaults_child, frozen_by);
                    }
                } else if let Some(target_value) = target_map.get_mut(key) {
                    let target_child = children
                        .get_mut(key)
                        .expect("provenance should exist for each key in the target");
                    merge_at_path(
                        target_value,
                        target_child,
                        defaults_value,
                        &defaults_child,
                        frozen_paths,
                        path,
                    );
                } else {
                    // The default value is missing from the target.
                    target_map.insert(key.clone(), defaults_value.clone());
                    children.insert(key.clone(), defaults_child.into_owned());
                }

                path.truncate(previous_path_len);
            }
        }
        (Value::Object(_), _) => {
            // `defaults` is not an object, so it loses and freezes the path like in `merge_json_with_defaults`.
            target_provenance.override_with(defaults_provenance);
            frozen_paths.insert(path.clone(), defaults_provenance.origin.clone());
        }
        _ => {
            // `target` is not an object, so it wins.
            target_provenance.override_with(defaults_provenance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::merge::{merge_json_with_defaults, FrozenPaths};
    use crate::provider::analysis::OptionSource;
    use serde_json::json;

    fn origin(rank: usize) -> Arc<Origin> {
        Arc::new(Origin {
            rank,
            source: OptionSource::Feature {
                import_chain: vec![format!("feature_{rank}")],
            },
        })
    }

    /// Merges sources from highest to lowest priority with both merges and checks that the values match.
    fn merge_all(sources: &[Value]) -> Provenance {
        let mut expected = sources[0].clone();
        let mut frozen_paths = FrozenPaths::new();
        let mut value = sources[0].clone();
        let mut provenance = Provenance::new(origin(0));
        let mut tracked_frozen_paths = TrackedFrozenPaths::new();
        for (rank, source) in sources.iter().enumerate().skip(1) {
            merge_json_with_defaults(&mut expected, source, &mut frozen_paths);
            merge_json_with_defaults_tracked(
                &mut value,
                &mut provenance,
                source,
                &Provenance::new(origin(rank)),
                &mut tracked_frozen_paths,
            );
        }
        assert_eq!(value, expected, "sources: {sources:?}");
        provenance
    }

    fn ranks(origins: &[Arc<Origin>]) -> Vec<usize> {
        origins.iter().map(|origin| origin.rank).collect()
    }

    #[test]
    fn test_tracked_merge_matches_merge() {
        let cases = [
            vec![json!({"a": 1, "b": 2}), json!({"b": 3, "c": 4})],
            vec![
                json!({"l1": {"l2": {"l3": {"a": 1}}}}),
                json!({"l1": {"l2": {"l3": {"a": 57, "b": 2}, "new": "value"}}}),
            ],
            vec![json!({"key": {"nested": 1}}), json!({"key": "string"})],
            vec![
                json!({"key/with~special": {"high": true}}),
                json!({"key/with~special": false}),
                json!({"key/with~special": {"low": true}}),
            ],
            vec![json!({"arr": [1, 2, 3]}), json!({"arr": [4, 5]})],
            vec![
                json!({"k": {"a": {"b": 1}}}),
                json!({"k": {"a": null}}),
                json!({"k": {"a": {"c": 2}, "d": [1]}}),
                json!({"k": "low"}),
            ],
        ];
        for sources in cases {
            merge_all(&sources);
        }
    }

    #[test]
    fn test_tracked_merge_attributes_keys() {
        let provenance = merge_all(&[
            json!({"l1": {"a": 1, "b": 2}}),
            json!({"l1": {"b": 3, "c": 4}}),
        ]);
        let l1 = provenance.child("l1");
        assert_eq!(l1.child("a").origin.rank, 0);
        assert_eq!(l1.child("b").origin.rank, 0);
        assert_eq!(ranks(&l1.child("b").overridden), vec![1]);
        assert_eq!(l1.child("c").origin.rank, 1);
        assert!(l1.child("c").overridden.is_empty());
    }

    #[test]
    fn test_tracked_merge_frozen_path_blocks_lower_priority() {
        let provenance = merge_all(&[
            json!({"k": {"y": 1}}),
            json!({"k": 5}),
            json!({"k": {"x": 3}}),
        ]);
        let k = provenance.child("k");
        assert_eq!(k.origin.rank, 0);
        assert_eq!(ranks(&k.overridden), vec![1]);
        assert_eq!(k.blocked.len(), 1);
        assert_eq!(k.blocked[0].frozen_by.rank, 1);
        assert_eq!(k.blocked[0].source.rank, 2);
    }

    #[test]
    fn test_tracked_merge_non_object_target_wins() {
        let provenance = merge_all(&[
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [4, 5]}),
            json!({"arr": {"x": 1}}),
        ]);
        let arr = provenance.child("arr");
        assert_eq!(arr.origin.rank, 0);
        assert_eq!(ranks(&arr.overridden), vec![1, 2]);
    }
}
