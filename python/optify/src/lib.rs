use pyo3::prelude::*;

use optify::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use optify::convert_to_str_slice;
use optify::provider::{OptionsProvider, OptionsRegistry};

#[pyclass(name = "OptionsProviderBuilder")]
// TODO Try to use inheritance, maybe?
struct PyOptionsProviderBuilder(OptionsProviderBuilder);

#[pyclass(name = "OptionsProvider")]
struct PyOptionsProvider(OptionsProvider);

#[pymethods]
impl PyOptionsProvider {
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
        let _feature_names = convert_to_str_slice!(feature_names);
        self.0
            .get_canonical_feature_names(&_feature_names)
            .expect("feature names should be valid")
    }

    fn get_options_json(&self, key: &str, feature_names: Vec<String>) -> String {
        let _feature_names = convert_to_str_slice!(feature_names);
        self.0
            .get_options(key, &_feature_names)
            .expect("key and feature names should be valid")
            .to_string()
    }
}

#[pymethods]
impl PyOptionsProviderBuilder {
    #[new]
    fn new() -> Self {
        Self(OptionsProviderBuilder::new())
    }

    fn add_directory(&mut self, directory: &str) -> Self {
        let path = std::path::Path::new(&directory);
        self.0
            .add_directory(path)
            .expect("directory contents should be valid");
        // TODO Try to avoid cloning
        Self(self.0.clone())
    }

    fn build(&mut self) -> PyOptionsProvider {
        PyOptionsProvider(
            self.0
                .build()
                .expect("OptionsProvider should be built successfully"),
        )
    }
}

#[pymodule(name = "optify")]
mod optify_python {
    #[pymodule_export]
    use super::PyOptionsProviderBuilder;

    #[pymodule_export]
    use super::PyOptionsProvider;
}
