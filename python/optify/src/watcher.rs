use pyo3::prelude::*;
use pyo3::types::PyType;

use optify::builder::{OptionsRegistryBuilder, OptionsWatcherBuilder};
use optify::provider::{OptionsRegistry, OptionsWatcher};

use crate::preferences::PyGetOptionsPreferences;

#[pyclass(name = "OptionsWatcherBuilder")]
pub struct PyOptionsWatcherBuilder(OptionsWatcherBuilder);

#[pyclass(name = "OptionsWatcher")]
pub struct PyOptionsWatcher(OptionsWatcher);

#[pymethods]
impl PyOptionsWatcher {
    #[classmethod]
    fn build(_cls: &Bound<'_, PyType>, directory: &str) -> PyResult<PyOptionsWatcher> {
        match OptionsWatcher::build(directory) {
            Ok(provider) => Ok(PyOptionsWatcher(provider)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    #[classmethod]
    fn build_from_directories(
        _cls: &Bound<'_, PyType>,
        directories: Vec<String>,
    ) -> PyResult<PyOptionsWatcher> {
        match OptionsWatcher::build_from_directories(&directories) {
            Ok(watcher) => Ok(PyOptionsWatcher(watcher)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    /// All of the canonical feature names.
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
impl PyOptionsWatcherBuilder {
    #[new]
    fn new() -> Self {
        Self(OptionsWatcherBuilder::new())
    }

    fn add_directory(&mut self, directory: &str) -> PyResult<Self> {
        let path = std::path::Path::new(&directory);
        match self.0.add_directory(path) {
            Ok(_) => Ok(Self(self.0.clone())),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }

    fn build(&mut self) -> PyResult<PyOptionsWatcher> {
        match self.0.build() {
            Ok(watcher) => Ok(PyOptionsWatcher(watcher)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }
}
