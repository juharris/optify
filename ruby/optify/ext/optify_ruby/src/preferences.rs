use magnus::wrap;
use optify::provider::GetOptionsPreferences;
use std::cell::RefCell;

pub fn convert_preferences(
    preferences: &MutGetOptionsPreferences,
) -> std::cell::Ref<'_, GetOptionsPreferences> {
    preferences.0.borrow()
}

#[derive(Clone)]
#[wrap(class = "Optify::GetOptionsPreferences")]
pub struct MutGetOptionsPreferences(RefCell<GetOptionsPreferences>);

impl MutGetOptionsPreferences {
    pub fn new() -> Self {
        Self(RefCell::new(GetOptionsPreferences::new()))
    }

    // Configurable Strings Section
    pub fn are_configurable_strings_enabled(&self) -> bool {
        self.0.borrow().are_configurable_strings_enabled
    }

    pub fn disable_configurable_strings(&self) {
        self.0.borrow_mut().are_configurable_strings_enabled = false;
    }

    pub fn enable_configurable_strings(&self) {
        self.0.borrow_mut().are_configurable_strings_enabled = true;
    }

    // Constraints Section
    pub fn set_constraints_json(&self, constraints_json: Option<String>) {
        self.0
            .borrow_mut()
            .set_constraints_json(constraints_json.as_deref());
    }

    pub fn get_constraints_json(&self) -> Option<String> {
        self.0
            .borrow()
            .constraints
            .as_ref()
            .map(|c| serde_json::to_string(&c.constraints).unwrap())
    }

    // Overrides Section
    pub fn has_overrides(&self) -> bool {
        self.0.borrow().overrides_json.is_some()
    }

    pub fn set_overrides_json(&self, overrides: Option<String>) {
        self.0.borrow_mut().overrides_json = overrides;
    }

    pub fn get_overrides_json(&self) -> Option<String> {
        self.0.borrow().overrides_json.clone()
    }

    // Skip Feature Name Conversion Section
    pub fn set_skip_feature_name_conversion(&self, value: bool) {
        self.0.borrow_mut().skip_feature_name_conversion = value;
    }

    pub fn skip_feature_name_conversion(&self) -> bool {
        self.0.borrow().skip_feature_name_conversion
    }
}
