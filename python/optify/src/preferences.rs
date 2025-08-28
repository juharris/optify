use pyo3::{prelude::*, types::PyDict};

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

    fn set_constraints(&mut self, constraints: Option<&PyDict>) {
        match constraints {
            None => self.0.set_constraints(None),
            Some(constraints) => {
                let serde_value = serde_pyobject::from_pyobject(constraints)?;
                self.0.set_constraints(Some(serde_value));
            }
        }
    }
}
