use once_cell::sync::Lazy;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pythonize::{depythonize, pythonize};
use specado_core::{
    execute, translate as core_translate, PromptSpec, ProviderSpec, UniformResponse,
};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

static RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .thread_name("specado-py")
            .build()
            .expect("Failed to create Tokio runtime"),
    )
});

#[pyclass]
struct Client {
    provider_path: String,
}

#[pymethods]
impl Client {
    #[new]
    fn new(provider_path: String) -> PyResult<Self> {
        Ok(Self { provider_path })
    }

    fn complete(&self, py: Python<'_>, prompt: &Bound<'_, PyDict>) -> PyResult<PyObject> {
        let prompt_json = depythonize::<serde_json::Value>(prompt)?;
        let prompt_spec: PromptSpec = serde_json::from_value(prompt_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid prompt spec: {e}")))?;
        let provider_path = self.provider_path.clone();

        let runtime = RUNTIME.clone();
        let response: UniformResponse = py
            .allow_threads(|| {
                runtime.block_on(async { execute(prompt_spec, &provider_path).await })
            })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let response_json = serde_json::to_value(&response)
            .map_err(|e| PyValueError::new_err(format!("Failed to encode response: {e}")))?;

        Ok(pythonize(py, &response_json)?.into_py(py))
    }
}

#[pyfunction]
fn translate(
    py: Python<'_>,
    prompt: &Bound<'_, PyDict>,
    provider: &Bound<'_, PyDict>,
) -> PyResult<(PyObject, PyObject)> {
    let prompt_json = depythonize::<serde_json::Value>(prompt)?;
    let provider_json = depythonize::<serde_json::Value>(provider)?;

    let prompt_spec: PromptSpec = serde_json::from_value(prompt_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid prompt spec: {e}")))?;
    let provider_spec: ProviderSpec = serde_json::from_value(provider_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid provider spec: {e}")))?;

    let (translated, lossiness) = core_translate(&prompt_spec, &provider_spec)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let translated_py = pythonize(py, &translated)?.into_py(py);
    let lossiness_py = pythonize(py, &lossiness)?.into_py(py);
    Ok((translated_py, lossiness_py))
}

#[pymodule]
fn specado(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_initializes_once() {
        let _runtime = RUNTIME.clone();
        assert!(tokio::runtime::Handle::try_current().is_err());
    }
}
