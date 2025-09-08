use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ReplacementValue {
    String(String),
    Object(HashMap<String, Value>),
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ConfigurableString {
    pub template: String,
    pub replacements: HashMap<String, ReplacementValue>,
}

impl ConfigurableString {
    pub fn build(&self) -> String {
        // TODO Use Liquid to replace the values.
        let mut result = self.template.clone();
        // for (key, value) in self.replacements.iter() {
        //     result = result.replace(key, &value.to_string());
        // }
        result
    }
}
