use crate::schema::conditions::{ConditionExpression, ConditionGroup, OperatorValue};

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: serde_json::Value,
}

impl Constraints {
    pub fn evaluate(&self, conditions: &ConditionExpression) -> bool {
        match conditions {
            ConditionExpression::Group(group) => match group {
                ConditionGroup::And { and } => and.iter().all(|condition| self.evaluate(condition)),
                ConditionGroup::Or { or } => or.iter().any(|condition| self.evaluate(condition)),
                ConditionGroup::Not { not } => !self.evaluate(not),
            },
            ConditionExpression::Condition(condition) => match &condition.operator_value {
                OperatorValue::Equals { equals } => self
                    .constraints
                    .pointer(condition.json_pointer.as_str())
                    .map(|value| value == equals)
                    .unwrap_or(false),
                OperatorValue::Matches { matches } => self
                    .constraints
                    .pointer(condition.json_pointer.as_str())
                    .and_then(|value| value.as_str())
                    .map(|value| matches.0.is_match(value))
                    .unwrap_or(false),
            },
        }
    }
}
