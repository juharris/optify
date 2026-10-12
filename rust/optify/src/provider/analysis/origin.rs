use super::OptionSource;

/// A source of values with its priority.
#[derive(Debug)]
pub(super) struct Origin {
    /// The position of the origin in the order that sources are merged.
    /// Lower ranks have higher priority.
    pub rank: usize,
    pub source: OptionSource,
}
