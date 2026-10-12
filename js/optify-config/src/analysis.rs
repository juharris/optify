/// Explains where the value at a path in the options came from.
#[napi(object)]
pub struct OptionAnalysis {
  /// Set when features define a value at the path, but it is not in the final options because of a non-object value at an ancestor of the path.
  /// `path` is the ancestor, `source` set the non-object value, and `overridden` lists the sources whose values at the path were blocked.
  pub blocker: Option<OptionOrigin>,
  /// Where the value at the path came from.
  /// There is exactly one entry when one origin supplied the entire value at the path.
  /// When the value is an object merged from several origins, there is one entry per highest-level sub-path with a single origin, sorted by path.
  /// Empty when `blocker` is set.
  pub origins: Vec<OptionOrigin>,
}

/// Where a value in the options came from.
#[napi(object)]
pub struct OptionOrigin {
  /// Lower-priority sources that also set a value at the path (or at an ancestor on the way to the path) and lost, highest priority first.
  pub overridden: Vec<OptionSource>,
  /// A JSON pointer equal to the requested path, a descendant of it, or an ancestor of it for `blocker`.
  pub path: String,
  pub source: OptionSource,
}

/// A source of values in the options.
#[napi(object)]
pub struct OptionSource {
  /// For a feature, the canonical feature names from the requested feature to the feature that defined the value.
  pub lineage: Option<Vec<String>>,
  #[napi(ts_type = "'feature' | 'overrides'")]
  pub kind: String,
}

impl From<optify::provider::OptionAnalysis> for OptionAnalysis {
  fn from(analysis: optify::provider::OptionAnalysis) -> Self {
    Self {
      blocker: analysis.blocker.map(OptionOrigin::from),
      origins: analysis
        .origins
        .into_iter()
        .map(OptionOrigin::from)
        .collect(),
    }
  }
}

impl From<optify::provider::OptionOrigin> for OptionOrigin {
  fn from(origin: optify::provider::OptionOrigin) -> Self {
    Self {
      overridden: origin
        .overridden
        .into_iter()
        .map(OptionSource::from)
        .collect(),
      path: origin.path,
      source: origin.source.into(),
    }
  }
}

impl From<optify::provider::OptionSource> for OptionSource {
  fn from(source: optify::provider::OptionSource) -> Self {
    match source {
      optify::provider::OptionSource::Feature { lineage } => Self {
        lineage: Some(lineage),
        kind: "feature".to_owned(),
      },
      optify::provider::OptionSource::Overrides => Self {
        lineage: None,
        kind: "overrides".to_owned(),
      },
    }
  }
}
