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

    fn set_constraints_json(&mut self, constraints_json: Option<String>) {
        self.0.set_constraints_json(constraints_json.as_deref());
    }
}
