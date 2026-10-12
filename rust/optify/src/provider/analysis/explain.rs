use std::borrow::Cow;
use std::sync::Arc;

use serde_json::Value;

use super::origin::Origin;
use super::provenance::{Blocked, Provenance};
use super::replay::Replayed;
use super::{OptionAnalysis, OptionOrigin};
use crate::json::merge::push_json_pointer_segment;

/// No value is set at the path.
pub(super) struct NotFound {
    /// The deepest path on the way to the requested path when its value is not an object.
    pub non_object_path: Option<String>,
}

/// Walks the merged value and provenance along `segments` to explain the value at the path.
pub(super) fn explain(
    replayed: &Replayed,
    segments: &[String],
) -> Result<OptionAnalysis, NotFound> {
    let Some((value, provenance)) = &replayed.merged else {
        return Err(NotFound {
            non_object_path: None,
        });
    };

    let mut current_value = value;
    let mut current_provenance = Cow::Borrowed(provenance);
    let mut path = String::new();
    let mut losses = Vec::new();
    let mut blocked_on_path = Vec::new();
    for segment in segments {
        current_provenance.collect_losses(&mut losses);
        blocked_on_path.extend(
            current_provenance
                .blocked
                .iter()
                .map(|blocked| (path.clone(), blocked.clone())),
        );

        let Some(next_value) = get_child_value(current_value, segment) else {
            return explain_missing(
                replayed,
                current_value,
                &current_provenance,
                path,
                &blocked_on_path,
            );
        };

        current_provenance = match current_provenance {
            Cow::Borrowed(provenance) => provenance.child(segment),
            // The entire value came from one origin, so its descendants did too.
            Cow::Owned(provenance) => Cow::Owned(Provenance::new(provenance.origin)),
        };
        current_value = next_value;
        push_json_pointer_segment(&mut path, segment);
    }

    let mut origins = Vec::new();
    flatten_origins(&current_provenance, path, losses, &mut origins);
    Ok(OptionAnalysis {
        blocker: None,
        origins,
    })
}

fn create_option_origin(
    path: String,
    origin: &Arc<Origin>,
    mut overridden: Vec<Arc<Origin>>,
) -> OptionOrigin {
    overridden.sort_by_key(|overridden_origin| overridden_origin.rank);
    overridden.dedup_by_key(|overridden_origin| overridden_origin.rank);
    OptionOrigin {
        overridden: overridden
            .into_iter()
            .filter(|overridden_origin| overridden_origin.rank != origin.rank)
            .map(|overridden_origin| overridden_origin.source.clone())
            .collect(),
        path,
        source: origin.source.clone(),
    }
}

/// Explains why the path is not in the final options when features define a value at the path.
fn explain_missing(
    replayed: &Replayed,
    deepest_value: &Value,
    deepest_provenance: &Provenance,
    deepest_path: String,
    blocked_on_path: &[(String, Blocked)],
) -> Result<OptionAnalysis, NotFound> {
    if replayed.defining_origins.is_empty() {
        let non_object_path = (!deepest_value.is_object()).then_some(deepest_path);
        return Err(NotFound { non_object_path });
    }

    let (path, blocking_origin) = if deepest_value.is_object() {
        // An object on the path is missing the next key, so a frozen path must have blocked lower-priority values.
        blocked_on_path
            .iter()
            .filter(|(_, blocked)| {
                replayed
                    .defining_origins
                    .iter()
                    .any(|origin| Arc::ptr_eq(origin, &blocked.source))
            })
            .min_by_key(|(_, blocked)| blocked.frozen_by.rank)
            .map(|(path, blocked)| (path.clone(), blocked.frozen_by.clone()))
            .ok_or(NotFound {
                non_object_path: None,
            })?
    } else {
        (deepest_path, deepest_provenance.origin.clone())
    };

    Ok(OptionAnalysis {
        blocker: Some(create_option_origin(
            path,
            &blocking_origin,
            replayed.defining_origins.clone(),
        )),
        origins: Vec::new(),
    })
}

/// Adds one origin per highest-level sub-path with a single origin.
fn flatten_origins(
    provenance: &Provenance,
    path: String,
    mut losses: Vec<Arc<Origin>>,
    origins: &mut Vec<OptionOrigin>,
) {
    provenance.collect_losses(&mut losses);
    if let Some(origin) = single_origin(provenance) {
        origins.push(create_option_origin(path, origin, losses));
        return;
    }

    let Some(children) = &provenance.children else {
        return;
    };
    let mut keys: Vec<&String> = children.keys().collect();
    keys.sort_unstable();
    for key in keys {
        let mut child_path = path.clone();
        push_json_pointer_segment(&mut child_path, key);
        flatten_origins(&children[key], child_path, losses.clone(), origins);
    }
}

fn get_child_value<'a>(value: &'a Value, segment: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => map.get(segment),
        Value::Array(array) => segment
            .parse::<usize>()
            .ok()
            .and_then(|index| array.get(index)),
        _ => None,
    }
}

/// Returns the origin when the entire value at the node came from one origin and nothing lost at its descendants.
fn single_origin(provenance: &Provenance) -> Option<&Arc<Origin>> {
    let Some(children) = &provenance.children else {
        return Some(&provenance.origin);
    };
    if children.is_empty() {
        return Some(&provenance.origin);
    }

    let mut common_origin: Option<&Arc<Origin>> = None;
    for child in children.values() {
        if !child.overridden.is_empty() || !child.blocked.is_empty() {
            return None;
        }
        let origin = single_origin(child)?;
        match common_origin {
            None => common_origin = Some(origin),
            Some(common) if Arc::ptr_eq(common, origin) => {}
            Some(_) => return None,
        }
    }
    common_origin
}
