use std::fmt;

/// Error returned when a requester is not permitted to use a feature.
#[derive(Clone, Debug)]
pub struct PolicyDeniedError {
    pub feature_name: String,
    pub requester: String,
}

impl PolicyDeniedError {
    pub fn new(feature_name: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            feature_name: feature_name.into(),
            requester: requester.into(),
        }
    }
}

impl fmt::Display for PolicyDeniedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Requester \"{}\" is not permitted to use feature \"{}\".",
            self.requester, self.feature_name
        )
    }
}

impl std::error::Error for PolicyDeniedError {}
