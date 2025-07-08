use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::provider::constraints::Constraints;

#[derive(Clone, Debug)]
pub(crate) struct RegexWrapper(Regex);

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pattern = String::deserialize(deserializer)?;
        match Regex::new(&pattern) {
            Ok(regex) => Ok(RegexWrapper(regex)),
            Err(e) => {
                let error_msg = format!("Invalid regex pattern '{pattern}': {e}");
                Err(serde::de::Error::custom(error_msg))
            }
        }
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
#[allow(private_interfaces)]
pub enum Predicate {
    Equals {
        equals: serde_json::Value,
    },
    #[doc(hidden)]
    Matches {
        matches: RegexWrapper,
    },
}

impl Predicate {
    pub fn equals(value: serde_json::Value) -> Self {
        Self::Equals { equals: value }
    }

    pub fn matches(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self::Matches {
            matches: RegexWrapper(Regex::new(pattern)?),
        })
    }

    pub fn evaluate(&self, value: &serde_json::Value) -> bool {
        match (self, value) {
            (Self::Equals { equals }, value) => value == equals,
            (Self::Matches { matches }, serde_json::Value::String(value)) => {
                matches.0.is_match(value)
            }
            (Self::Matches { matches }, value) => {
                matches.0.is_match(&serde_json::to_string(value).unwrap())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub json_pointer: String,
    #[serde(flatten)]
    pub operator_value: Predicate,
}

impl Condition {
    pub fn evaluate(&self, constraints: &Constraints) -> bool {
        constraints
            .constraints
            .pointer(&self.json_pointer)
            .is_some_and(|value| self.operator_value.evaluate(value))
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum ConditionExpression {
    Condition(Condition),
    And { and: Vec<Self> },
    Or { or: Vec<Self> },
    Not { not: Box<Self> },
}

// Implement a custom deserializer to ensure that errors, such as an invalid regex,
// are propagated up properly.
impl<'de> Deserialize<'de> for ConditionExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        // Try to deserialize as a raw Value first
        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(map) = &value {
            if map.contains_key("jsonPointer") {
                // Check for regex errors early to provide better error messages
                if let Some(serde_json::Value::String(pattern)) = map.get("matches") {
                    if let Err(e) = regex::Regex::new(pattern) {
                        return Err(D::Error::custom(e.to_string()));
                    }
                }

                match serde_json::from_value::<Condition>(value) {
                    Ok(condition) => return Ok(Self::Condition(condition)),
                    Err(e) => {
                        return Err(D::Error::custom(e.to_string()));
                    }
                }
            }

            // Try logical operators
            if let Some(and_val) = map.get("and") {
                if let Ok(vec) = serde_json::from_value::<Vec<ConditionExpression>>(and_val.clone())
                {
                    return Ok(Self::And { and: vec });
                }
            }

            if let Some(or_val) = map.get("or") {
                if let Ok(vec) = serde_json::from_value::<Vec<ConditionExpression>>(or_val.clone())
                {
                    return Ok(Self::Or { or: vec });
                }
            }

            if let Some(not_val) = map.get("not") {
                if let Ok(boxed) =
                    serde_json::from_value::<Box<ConditionExpression>>(not_val.clone())
                {
                    return Ok(Self::Not { not: boxed });
                }
            }
        }

        Err(D::Error::custom(
            "data did not match any variant of ConditionExpression",
        ))
    }
}

impl ConditionExpression {
    pub fn evaluate(&self, data: &Constraints) -> bool {
        match self {
            Self::Condition(condition) => condition.evaluate(data),
            Self::And { and } => and.iter().all(|expr| expr.evaluate(data)),
            Self::Or { or } => or.iter().any(|expr| expr.evaluate(data)),
            Self::Not { not } => !not.evaluate(data),
        }
    }

    pub fn evaluate_with(&self, data: &serde_json::Value) -> bool {
        self.evaluate(&Constraints {
            constraints: data.clone(),
        })
    }
}
