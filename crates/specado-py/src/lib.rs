#![allow(clippy::useless_conversion)] // PyResult alias evaluates to PyErr; Clippy 1.80 flags false positives

use once_cell::sync::Lazy;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pythonize::{depythonize, pythonize};
#[cfg(feature = "audit-logging")]
use specado::audit::{AuditConfig, AuditContext};
use specado::{
    create_prompt as core_create_prompt, execute, load_prompt_from_path,
    simple_prompt as core_simple_prompt, translate as core_translate, ExecuteOptions,
    PromptBuilder, PromptSpec, ProviderSpec, SimplePromptOptions, UniformResponse,
};
use std::path::PathBuf;
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
    provider: String,
    model: Option<String>,
    providers_dir: Option<PathBuf>,
    _watch_enabled: bool,
    #[cfg(feature = "audit-logging")]
    audit_config: Option<AuditConfig>,
}

#[pymethods]
impl Client {
    #[new]
    #[pyo3(signature = (provider, *, model=None, providers_dir=None, watch=None, audit_config=None))]
    fn new(
        provider: String,
        model: Option<String>,
        providers_dir: Option<String>,
        watch: Option<bool>,
        audit_config: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let watch_enabled = watch.unwrap_or(false);
        let providers_dir = providers_dir.map(PathBuf::from);

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
            provider,
            model,
            providers_dir,
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
        self.execute_to_py(py, prompt_spec)
    }

    fn complete_file(&self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        let prompt =
            load_prompt_from_path(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        self.execute_to_py(py, prompt)
    }

    #[pyo3(signature = (message, *, options = None))]
    fn complete_text(
        &self,
        py: Python<'_>,
        message: &str,
        options: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyObject> {
        let mut parsed = if let Some(opts) = options {
            let opts_json = depythonize::<serde_json::Value>(opts)?;
            serde_json::from_value::<SimplePromptOptions>(opts_json)
                .map_err(|e| PyValueError::new_err(format!("Invalid prompt options: {e}")))?
        } else {
            SimplePromptOptions::default()
        };
        if parsed.message.is_none() && parsed.user.is_none() {
            parsed.message = Some(message.to_string());
        }

        let prompt =
            core_simple_prompt(parsed).map_err(|e| PyValueError::new_err(e.to_string()))?;
        self.execute_to_py(py, prompt)
    }
}

impl Client {
    fn execute_prompt(&self, py: Python<'_>, prompt_spec: PromptSpec) -> PyResult<UniformResponse> {
        let provider = self.provider.clone();

        let mut options = ExecuteOptions::default();
        if let Some(model) = self.model.as_ref() {
            options.model = Some(model.clone());
        }
        if let Some(dir) = self.providers_dir.as_ref() {
            options.providers_dir = Some(dir.clone());
        }

        #[cfg(feature = "audit-logging")]
        let audit_context = self.audit_config.clone().map(AuditContext::new);

        let runtime = RUNTIME.clone();
        py.allow_threads(move || {
            runtime.block_on(async {
                execute(
                    prompt_spec,
                    &provider,
                    options,
                    #[cfg(feature = "audit-logging")]
                    audit_context,
                )
                .await
            })
        })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn execute_to_py(&self, py: Python<'_>, prompt_spec: PromptSpec) -> PyResult<PyObject> {
        let response = self.execute_prompt(py, prompt_spec)?;
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

#[pyfunction]
fn load_prompt(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    let prompt = load_prompt_from_path(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let prompt_json = serde_json::to_value(&prompt)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to encode prompt: {e}")))?;
    Ok(pythonize(py, &prompt_json)?.into_py(py))
}

#[pyfunction]
fn create_prompt(py: Python<'_>, options: &Bound<'_, PyDict>) -> PyResult<PyObject> {
    let options_json = depythonize::<serde_json::Value>(options)?;
    let builder: PromptBuilder = serde_json::from_value(options_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid prompt options: {e}")))?;
    let prompt = core_create_prompt(builder);
    let prompt_json = serde_json::to_value(&prompt)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to encode prompt: {e}")))?;
    Ok(pythonize(py, &prompt_json)?.into_py(py))
}

#[pyfunction]
#[pyo3(signature = (options = None))]
fn simple_prompt(py: Python<'_>, options: Option<&Bound<'_, PyDict>>) -> PyResult<PyObject> {
    let parsed = if let Some(opts) = options {
        let options_json = depythonize::<serde_json::Value>(opts)?;
        serde_json::from_value::<SimplePromptOptions>(options_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid prompt options: {e}")))?
    } else {
        SimplePromptOptions::default()
    };

    let prompt = core_simple_prompt(parsed).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let prompt_json = serde_json::to_value(&prompt)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to encode prompt: {e}")))?;
    Ok(pythonize(py, &prompt_json)?.into_py(py))
}

#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    m.add_function(wrap_pyfunction!(load_prompt, m)?)?;
    m.add_function(wrap_pyfunction!(create_prompt, m)?)?;
    m.add_function(wrap_pyfunction!(simple_prompt, m)?)?;
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
