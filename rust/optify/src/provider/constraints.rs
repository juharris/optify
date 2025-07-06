use crate::schema::conditions::{ConditionExpression, OperatorValue};

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: serde_json::Value,
}

impl Constraints {
    pub fn evaluate(&self, conditions: &ConditionExpression) -> bool {
        match conditions {
            ConditionExpression::And { and } => {
                and.iter().all(|condition| self.evaluate(condition))
            }
            ConditionExpression::Or { or } => or.iter().any(|condition| self.evaluate(condition)),
            ConditionExpression::Not { not } => !self.evaluate(not),
            ConditionExpression::Condition(condition) => match &condition.operator_value {
                OperatorValue::Equals { equals } => self
                    .constraints
                    .pointer(condition.json_pointer.as_str())
                    .map(|value| value == equals)
                    .unwrap_or(false),
                OperatorValue::Matches { matches } => self
                    .constraints
                    .pointer(condition.json_pointer.as_str())
                    .map(|value| match value {
                        serde_json::Value::String(value) => matches.0.is_match(value),
                        _ => matches.0.is_match(&serde_json::to_string(value).unwrap()),
                    })
                    .unwrap_or(false),
            },
        }
    }
}
