use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionOperator {
    Equals,
    Regex,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub json_path: Option<String>,
    pub operator: ConditionOperator,
    // TODO Think of a name other than "value".
    pub value: Option<serde_json::Value>,
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
