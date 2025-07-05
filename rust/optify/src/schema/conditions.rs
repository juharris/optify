use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionOperator {
    Equals,
    Matches,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OperatorValue {
    Equals { equals: serde_json::Value },
    Matches { matches: String },
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
