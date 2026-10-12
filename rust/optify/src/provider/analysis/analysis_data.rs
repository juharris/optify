use crate::provider::{Imports, Sources};

/// Data retained when building a provider to support analyzing options.
#[derive(Clone)]
pub(crate) struct AnalysisData {
    /// The imports for each canonical feature name.
    pub imports: Imports,
    /// The options for each canonical feature name from its file, before merging imports.
    pub raw_sources: Sources,
}
