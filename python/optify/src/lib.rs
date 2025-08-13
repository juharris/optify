use pyo3::prelude::*;

mod preferences;
use preferences::PyGetOptionsPreferences;

mod provider;
use provider::PyOptionsProvider;
use provider::PyOptionsProviderBuilder;

mod watcher;
use watcher::PyOptionsWatcher;
use watcher::PyOptionsWatcherBuilder;

#[pymodule(name = "optify")]
mod optify_python {
    #[pymodule_export]
    use super::PyGetOptionsPreferences;

    #[pymodule_export]
    use super::PyOptionsProviderBuilder;

    #[pymodule_export]
    use super::PyOptionsProvider;

    #[pymodule_export]
    use super::PyOptionsWatcherBuilder;

    #[pymodule_export]
    use super::PyOptionsWatcher;
}
