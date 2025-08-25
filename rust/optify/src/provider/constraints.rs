use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: serde_json::Value,
}

impl Hash for Constraints {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the JSON value as a string for consistency
        self.constraints.to_string().hash(state);
    }
}

impl PartialEq for Constraints {
    fn eq(&self, other: &Self) -> bool {
        self.constraints == other.constraints
    }
}

impl Eq for Constraints {}
