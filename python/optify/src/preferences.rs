use pyo3::prelude::*;

use optify::provider::GetOptionsPreferences;

#[pyclass(name = "GetOptionsPreferences")]
pub struct PyGetOptionsPreferences(pub(crate) GetOptionsPreferences);

#[pymethods]
impl PyGetOptionsPreferences {
    #[new]
    fn new() -> Self {
        Self(GetOptionsPreferences::new())
    }

    /// Indicates if configurable strings are enabled.
    pub fn are_configurable_strings_enabled(&self) -> bool {
        self.0.are_configurable_strings_enabled
    }

    /// Enables configurable strings which are disabled by default.
    pub fn enable_configurable_strings(&mut self) {
        self.0.are_configurable_strings_enabled = true;
    }

    /// Disables configurable strings which are disabled by default.
    pub fn disable_configurable_strings(&mut self) {
        self.0.are_configurable_strings_enabled = false;
    }

    fn set_constraints_json(&mut self, constraints_json: Option<String>) {
        self.0.set_constraints_json(constraints_json.as_deref());
    }
}
