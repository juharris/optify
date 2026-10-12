use std::sync::Arc;

use serde_json::Value;

use super::origin::Origin;
use super::projection::project;
use super::provenance::Provenance;
use super::tracked_merge::{merge_json_with_defaults_tracked, TrackedFrozenPaths};
use super::{AnalysisData, OptionSource};

/// The result of replaying the merges used to build options for a request.
pub(super) struct Replayed {
    /// Origins whose own values have a value at the key path.
    pub defining_origins: Vec<Arc<Origin>>,
    /// The merged value projected onto the key path and its provenance.
    /// `None` when there are no features or overrides.
    pub merged: Option<(Value, Provenance)>,
}

/// Replays the merges used to build options for the canonical `feature_names` while tracking where values came from.
pub(super) fn replay(
    data: &AnalysisData,
    key_path: &str,
    segments: &[String],
    feature_names: &[String],
    overrides: Option<&Value>,
) -> Result<Replayed, String> {
    let mut replayer = Replayer {
        data,
        defining_origins: Vec::new(),
        key_path,
        next_rank: 0,
        segments,
    };
    let merged = replayer.replay_request(feature_names, overrides)?;
    Ok(Replayed {
        defining_origins: replayer.defining_origins,
        merged,
    })
}

struct Replayer<'a> {
    data: &'a AnalysisData,
    defining_origins: Vec<Arc<Origin>>,
    key_path: &'a str,
    /// Assigned in the order that sources are merged, which is from highest to lowest priority.
    next_rank: usize,
    segments: &'a [String],
}

impl Replayer<'_> {
    fn create_origin(&mut self, source: OptionSource, raw_value: &Value) -> Arc<Origin> {
        let origin = Arc::new(Origin {
            rank: self.next_rank,
            source,
        });
        self.next_rank += 1;
        if raw_value.pointer(self.key_path).is_some() {
            self.defining_origins.push(origin.clone());
        }
        origin
    }

    /// Merges like `resolve_imports` when building the provider.
    fn replay_feature(
        &mut self,
        canonical_feature_name: &str,
        import_chain: &[String],
    ) -> Result<(Value, Provenance), String> {
        let data = self.data;
        let raw_source = data
            .raw_sources
            .get(canonical_feature_name)
            .ok_or_else(|| {
                format!("Feature name {canonical_feature_name:?} is not a known feature.")
            })?;
        let mut value = project(raw_source, self.segments);
        let origin = self.create_origin(
            OptionSource::Feature {
                import_chain: import_chain.to_vec(),
            },
            raw_source,
        );
        let mut provenance = Provenance::new(origin);

        if let Some(imports) = data.imports.get(canonical_feature_name) {
            let mut frozen_paths = TrackedFrozenPaths::new();
            for import in imports.iter().rev() {
                let mut chain_for_import = import_chain.to_vec();
                chain_for_import.push(import.clone());
                let (import_value, import_provenance) =
                    self.replay_feature(import, &chain_for_import)?;
                merge_json_with_defaults_tracked(
                    &mut value,
                    &mut provenance,
                    &import_value,
                    &import_provenance,
                    &mut frozen_paths,
                );
            }
        }

        Ok((value, provenance))
    }

    /// Merges like `OptionsProvider::get_entire_config`.
    fn replay_request(
        &mut self,
        feature_names: &[String],
        overrides: Option<&Value>,
    ) -> Result<Option<(Value, Provenance)>, String> {
        let mut result = overrides.map(|overrides| {
            let origin = self.create_origin(OptionSource::Overrides, overrides);
            (project(overrides, self.segments), Provenance::new(origin))
        });

        let mut frozen_paths = TrackedFrozenPaths::new();
        for canonical_feature_name in feature_names.iter().rev() {
            let (value, provenance) = self.replay_feature(
                canonical_feature_name,
                std::slice::from_ref(canonical_feature_name),
            )?;
            match &mut result {
                Some((result_value, result_provenance)) => merge_json_with_defaults_tracked(
                    result_value,
                    result_provenance,
                    &value,
                    &provenance,
                    &mut frozen_paths,
                ),
                None => result = Some((value, provenance)),
            }
        }

        Ok(result)
    }
}
