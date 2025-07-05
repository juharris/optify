use crate::schema::conditions::ConditionExpression;

#[derive(Clone)]
pub struct Constraints {
    pub constraint: serde_json::Value,
}
impl Constraints {
    pub(crate) fn evaluate(&self, conditions: &ConditionExpression) -> bool {
        // todo!()
        return true;
    }
}
