use serde::{Deserialize, Serialize};

use crate::{
    provider::SourceValue,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
    Policies,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feature {
    pub conditions: Option<ConditionExpression>,
    pub imports: Option<Vec<String>>,
    pub metadata: OptionsMetadata,
    pub policies: Option<Policies>,
    pub options: SourceValue,
}
