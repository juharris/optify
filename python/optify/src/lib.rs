use pyo3::prelude::*;
use pyo3::types::PyType;

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::provider::{GetOptionsPreferences, OptionsProvider, OptionsRegistry};

#[pyclass(name = "GetOptionsPreferences")]
struct PyGetOptionsPreferences(GetOptionsPreferences);

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

#[pyclass(name = "OptionsProviderBuilder")]
// TODO Try to use inheritance, maybe?
struct PyOptionsProviderBuilder(OptionsProviderBuilder);

#[pyclass(name = "OptionsProvider")]
struct PyOptionsProvider(OptionsProvider);

#[pymethods]
impl PyOptionsProvider {
    #[classmethod]
    fn build(_cls: &Bound<'_, PyType>, directory: &str) -> PyResult<PyOptionsProvider> {
        match OptionsProvider::build(directory) {
            Ok(provider) => Ok(PyOptionsProvider(provider)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    #[classmethod]
    fn build_from_directories(
        _cls: &Bound<'_, PyType>,
        directories: Vec<String>,
    ) -> PyResult<PyOptionsProvider> {
        match OptionsProvider::build_from_directories(&directories) {
            Ok(provider) => Ok(PyOptionsProvider(provider)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    /// @return All of the canonical feature names.
    fn features(&self) -> Vec<String> {
        self.0
            .get_features()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn get_canonical_feature_name(&self, feature_name: &str) -> String {
        self.0
            .get_canonical_feature_name(feature_name)
            .expect("feature name should be valid")
    }

    fn get_canonical_feature_names(&self, feature_names: Vec<String>) -> Vec<String> {
        self.0
            .get_canonical_feature_names(&feature_names)
            .expect("feature names should be valid")
    }

    fn get_options_json(&self, key: &str, feature_names: Vec<String>) -> PyResult<String> {
        self.get_options_json_with_preferences(key, feature_names, None)
    }

    fn get_options_json_with_preferences(
        &self,
        key: &str,
        feature_names: Vec<String>,
        preferences: Option<&PyGetOptionsPreferences>,
    ) -> PyResult<String> {
        let preferences = preferences.map(|p| &p.0);
        let result = &self
            .0
            .get_options_with_preferences(key, &feature_names, None, preferences)
            .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
        Ok(result.to_string())
    }
}

#[pymethods]
impl PyOptionsProviderBuilder {
    #[new]
    fn new() -> Self {
        Self(OptionsProviderBuilder::new())
    }

    fn add_directory(&mut self, directory: &str) -> PyResult<Self> {
        let path = std::path::Path::new(&directory);
        match self.0.add_directory(path) {
            Ok(_) => Ok(Self(self.0.clone())),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    fn build(&mut self) -> PyResult<PyOptionsProvider> {
        match self.0.build() {
            Ok(provider) => Ok(PyOptionsProvider(provider)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }
}

#[pymodule(name = "optify")]
mod optify_python {
    #[pymodule_export]
    use super::PyGetOptionsPreferences;

    #[pymodule_export]
    use super::PyOptionsProviderBuilder;

    #[pymodule_export]
    use super::PyOptionsProvider;
}
