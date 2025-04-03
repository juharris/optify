use pyo3::prelude::*;

use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;

#[pyclass(name = "OptionsProviderBuilder")]
// TODO Try to use inheritance, maybe?
struct PyOptionsProviderBuilder(OptionsProviderBuilder);

#[pyclass(name = "OptionsProvider")]
struct PyOptionsProvider(OptionsProvider);

#[pymethods]
impl PyOptionsProvider {
    fn features(&self) -> Vec<String> {
        self.0.get_features()
    }

    fn get_options_json(&self, key: &str, feature_names: Vec<String>) -> String {
        self.0
            .get_options(key, &feature_names)
            .expect("Failed to get options")
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
            .expect("Failed to add the directory");
        // TODO Try to avoid cloning
        Self(self.0.clone())
    }

    fn build(&mut self) -> PyOptionsProvider {
        PyOptionsProvider(
            self.0
                .build()
                .expect("Failed to build the options provider"),
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
