#![allow(clippy::useless_conversion)] // PyResult alias evaluates to PyErr; Clippy 1.80 flags false positives

use once_cell::sync::Lazy;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pythonize::{depythonize, pythonize};
#[cfg(feature = "audit-logging")]
use specado_core::audit::{AuditConfig, AuditContext};
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
    _watch_enabled: bool,
    #[cfg(feature = "audit-logging")]
    audit_config: Option<AuditConfig>,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (provider_path, watch=None, audit_config=None))]
    fn new(
        provider_path: String,
        watch: Option<bool>,
        audit_config: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let watch_enabled = watch.unwrap_or(false);

        #[cfg(feature = "audit-logging")]
        let audit_config = if let Some(cfg) = audit_config {
            let value = depythonize::<serde_json::Value>(cfg)?;
            let parsed = serde_json::from_value::<AuditConfig>(value)
                .map_err(|e| PyValueError::new_err(format!("Invalid audit configuration: {e}")))?;
            Some(parsed)
        } else {
            None
        };

        Ok(Self {
            provider_path,
            _watch_enabled: watch_enabled,
            #[cfg(feature = "audit-logging")]
            audit_config,
        })
    }

    #[allow(clippy::useless_conversion)] // PyResult alias raises false positive in Clippy 1.80
    fn complete(&self, py: Python<'_>, prompt: &Bound<'_, PyDict>) -> PyResult<PyObject> {
        let prompt_json = depythonize::<serde_json::Value>(prompt)?;
        let prompt_spec: PromptSpec = serde_json::from_value(prompt_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid prompt spec: {e}")))?;
        let provider_path = self.provider_path.clone();

        #[cfg(feature = "audit-logging")]
        let audit_context = self.audit_config.clone().map(AuditContext::new);

        let runtime = RUNTIME.clone();
        let response: UniformResponse = py
            .allow_threads(|| {
                runtime.block_on(async {
                    execute(
                        prompt_spec,
                        &provider_path,
                        #[cfg(feature = "audit-logging")]
                        audit_context,
                    )
                    .await
                })
            })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let response_json = serde_json::to_value(&response)
            .map_err(|e| PyValueError::new_err(format!("Failed to encode response: {e}")))?;

        Ok(pythonize(py, &response_json)?.into_py(py))
    }
}

#[pyfunction]
#[allow(clippy::useless_conversion)] // PyResult alias raises false positive in Clippy 1.80
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
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
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
