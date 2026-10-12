use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::origin::Origin;

/// A lower-priority value that was skipped because a higher-priority origin froze its path.
#[derive(Clone, Debug)]
pub(super) struct Blocked {
    /// The origin that set a non-object value at the frozen path.
    pub frozen_by: Arc<Origin>,
    /// The origin whose value was skipped.
    pub source: Arc<Origin>,
}

/// Tracks where the value at a node in a JSON value came from.
#[derive(Clone, Debug)]
pub(super) struct Provenance {
    /// Lower-priority values at this node that were skipped because the path was frozen.
    pub blocked: Vec<Blocked>,
    /// The provenance for each key when the value is an object whose keys may have different origins.
    /// `None` when the entire value came from `origin`.
    pub children: Option<HashMap<String, Provenance>>,
    /// The highest-priority origin that set the value at this node.
    pub origin: Arc<Origin>,
    /// Lower-priority origins whose values at this node lost to the value from a higher-priority origin.
    pub overridden: Vec<Arc<Origin>>,
}

impl Provenance {
    #[must_use]
    pub fn new(origin: Arc<Origin>) -> Self {
        Self {
            blocked: Vec::new(),
            children: None,
            origin,
            overridden: Vec::new(),
        }
    }

    /// Appends the losses at another provenance for the same node.
    pub fn append_losses(&mut self, other: &Provenance) {
        self.overridden.extend(other.overridden.iter().cloned());
        self.blocked.extend(other.blocked.iter().cloned());
    }

    /// Records that every origin in `defaults` was skipped because `frozen_by` froze this node.
    pub fn block(&mut self, defaults: &Provenance, frozen_by: &Arc<Origin>) {
        let mut sources = Vec::new();
        defaults.collect_origins(&mut sources);
        self.blocked
            .extend(sources.into_iter().map(|source| Blocked {
                frozen_by: frozen_by.clone(),
                source,
            }));
    }

    /// Gets the provenance for the value at `key`.
    #[must_use]
    pub fn child(&self, key: &str) -> Cow<'_, Provenance> {
        match self
            .children
            .as_ref()
            .and_then(|children| children.get(key))
        {
            Some(child) => Cow::Borrowed(child),
            None => Cow::Owned(Provenance::new(self.origin.clone())),
        }
    }

    /// Collects the origins that lost at this node, without its descendants.
    pub fn collect_losses(&self, origins: &mut Vec<Arc<Origin>>) {
        origins.extend(self.overridden.iter().cloned());
        origins.extend(self.blocked.iter().map(|blocked| blocked.source.clone()));
    }

    /// Collects every origin that contributed to or lost at this node or its descendants.
    pub fn collect_origins(&self, origins: &mut Vec<Arc<Origin>>) {
        origins.push(self.origin.clone());
        self.collect_losses(origins);
        if let Some(children) = &self.children {
            for child in children.values() {
                child.collect_origins(origins);
            }
        }
    }

    /// Splits the provenance for an entire object into a provenance per key so that keys can be attributed to different origins.
    pub fn expand(&mut self, map: &serde_json::Map<String, Value>) {
        if self.children.is_none() {
            self.children = Some(
                map.keys()
                    .map(|key| (key.clone(), Provenance::new(self.origin.clone())))
                    .collect(),
            );
        }
    }

    /// Records that every origin in `defaults` lost to the value at this node.
    pub fn override_with(&mut self, defaults: &Provenance) {
        defaults.collect_origins(&mut self.overridden);
    }
}
