use pyo3::prelude::*;

use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;

/*
/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn optify(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
*/


#[pyclass(name = "OptionsProviderBuilder")]
// TODO Try to use inheritance, maybe?
struct PyOptionsProviderBuilder(OptionsProviderBuilder);

#[pyclass(name = "OptionsProvider")]
struct PyOptionsProvider(OptionsProvider);

#[pymethods]
impl PyOptionsProvider{
    fn get_options_json(&self, key: &str, feature_names: Vec<String>) -> String {
        self.0.get_options(&key, &feature_names).unwrap().to_string()
    }
}

#[pymethods]
impl PyOptionsProviderBuilder{
    #[new]
    fn new() -> Self {
        Self(OptionsProviderBuilder::new())
    }

    fn add_directory(&mut self, directory: &str) -> Self {
        let path = std::path::Path::new(&directory);
        self.0.add_directory(path).unwrap();
        // TODO Try to avoid cloning
        Self(self.0.clone())
    }

    fn build(&mut self) -> PyOptionsProvider {
        PyOptionsProvider(self.0.build().unwrap())
    }
}

#[pymodule(name="optify")]
mod optify_python {
    // Probably not needed.
    // use super::*;

    #[pymodule_export]
    use super::PyOptionsProviderBuilder;

    #[pymodule_export]
    use super::PyOptionsProvider;
}
