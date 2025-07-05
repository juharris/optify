use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct RegexWrapper(pub Regex);

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Regex::new(&s)
            .map(RegexWrapper)
            .map_err(serde::de::Error::custom)
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
pub enum OperatorValue {
    Equals { equals: serde_json::Value },
    Matches { matches: RegexWrapper },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub json_pointer: String,
    #[serde(flatten)]
    pub operator_value: OperatorValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConditionGroup {
    And { and: Vec<ConditionExpression> },
    Or { or: Vec<ConditionExpression> },
    Not { not: Box<ConditionExpression> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConditionExpression {
    Group(ConditionGroup),
    Condition(Condition),
}
