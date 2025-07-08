use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct RegexWrapper(pub Regex);

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map_err(|e| {
                eprintln!("Error deserializing pattern: {e}");
                serde::de::Error::custom(e.to_string())
            })
            .and_then(|s| {
                Regex::new(&s).map(RegexWrapper).map_err(|e| {
                    // It seems like this error is swallowed when deserializing, so we'll print and hope someone sees it.
                    eprintln!("Error compiling regex: {e}");
                    serde::de::Error::custom(e.to_string())
                })
            })
    }
}

impl Serialize for RegexWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Predicate {
    Equals { equals: serde_json::Value },
    Matches { matches: RegexWrapper },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub json_pointer: String,
    #[serde(flatten)]
    pub operator_value: Predicate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConditionExpression {
    Condition(Condition),
    And { and: Vec<Self> },
    Or { or: Vec<Self> },
    Not { not: Box<Self> },
}
