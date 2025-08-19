//! Python type wrappers for Specado core types
//!
//! This module provides Python-compatible wrappers for Specado's core
//! data structures, with proper serialization and type hints.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::exceptions::PyValueError;
use serde_json::Value;
use specado_core::types::*;

/// Python wrapper for PromptSpec
#[pyclass(name = "PromptSpec")]
#[derive(Debug, Clone)]
pub struct PyPromptSpec {
    pub inner: PromptSpec,
}

#[pymethods]
impl PyPromptSpec {
    #[new]
    #[pyo3(signature = (model_class, messages, tools=None, tool_choice=None, response_format=None, sampling=None, limits=None, media=None, strict_mode="warn"))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        model_class: String,
        messages: Vec<PyMessage>,
        tools: Option<Vec<PyTool>>,
        tool_choice: Option<PyToolChoice>,
        response_format: Option<PyResponseFormat>,
        sampling: Option<PySamplingParams>,
        limits: Option<PyLimits>,
        media: Option<PyMediaConfig>,
        strict_mode: &str,
    ) -> PyResult<Self> {
        let strict_mode = match strict_mode {
            "warn" => StrictMode::Warn,
            "strict" => StrictMode::Strict,
            _ => return Err(PyValueError::new_err("strict_mode must be 'warn' or 'strict'")),
        };

        Ok(PyPromptSpec {
            inner: PromptSpec {
                model_class,
                messages: messages.into_iter().map(|m| m.inner).collect(),
                tools: tools.map(|t| t.into_iter().map(|tool| tool.inner).collect()),
                tool_choice: tool_choice.map(|tc| tc.inner),
                response_format: response_format.map(|rf| rf.inner),
                sampling: sampling.map(|s| s.inner),
                limits: limits.map(|l| l.inner),
                media: media.map(|m| m.inner),
                strict_mode,
            },
        })
    }

    #[getter]
    fn model_class(&self) -> String {
        self.inner.model_class.clone()
    }

    #[getter]
    fn messages(&self) -> Vec<PyMessage> {
        self.inner.messages.iter().map(|m| PyMessage { inner: m.clone() }).collect()
    }

    #[getter]
    fn strict_mode(&self) -> String {
        match self.inner.strict_mode {
            StrictMode::Warn => "warn".to_string(),
            StrictMode::Strict => "strict".to_string(),
            StrictMode::Coerce => "coerce".to_string(),
        }
    }

    fn to_dict(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    #[classmethod]
    fn from_dict(_cls: &PyType, py: Python<'_>, data: &PyDict) -> PyResult<Self> {
        let json_value = py_to_json(py, data.as_ref())?;
        let inner: PromptSpec = serde_json::from_value(json_value)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyPromptSpec { inner })
    }

    fn __repr__(&self) -> String {
        format!("PromptSpec(model_class='{}', messages=[...], strict_mode='{}')", 
                self.inner.model_class, self.strict_mode())
    }
}

/// Python wrapper for ProviderSpec
#[pyclass(name = "ProviderSpec")]
#[derive(Debug, Clone)]
pub struct PyProviderSpec {
    pub inner: ProviderSpec,
}

#[pymethods]
impl PyProviderSpec {
    #[new]
    fn new(spec_version: String, provider: PyProviderInfo, models: Vec<PyModelSpec>) -> Self {
        PyProviderSpec {
            inner: ProviderSpec {
                spec_version,
                provider: provider.inner,
                models: models.into_iter().map(|m| m.inner).collect(),
            },
        }
    }

    #[getter]
    fn spec_version(&self) -> String {
        self.inner.spec_version.clone()
    }

    #[getter]
    fn provider(&self) -> PyProviderInfo {
        PyProviderInfo { inner: self.inner.provider.clone() }
    }

    #[getter]
    fn models(&self) -> Vec<PyModelSpec> {
        self.inner.models.iter().map(|m| PyModelSpec { inner: m.clone() }).collect()
    }

    fn to_dict(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    #[classmethod]
    fn from_dict(_cls: &PyType, py: Python<'_>, data: &PyDict) -> PyResult<Self> {
        let json_value = py_to_json(py, data.as_ref())?;
        let inner: ProviderSpec = serde_json::from_value(json_value)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyProviderSpec { inner })
    }

    fn __repr__(&self) -> String {
        format!("ProviderSpec(provider='{}', models=[...] ({} models))", 
                self.inner.provider.name, self.inner.models.len())
    }
}

/// Python wrapper for Message
#[pyclass(name = "Message")]
#[derive(Debug, Clone)]
pub struct PyMessage {
    pub inner: Message,
}

#[pymethods]
impl PyMessage {
    #[new]
    #[pyo3(signature = (role, content, name=None, metadata=None))]
    fn new(role: &str, content: String, name: Option<String>, metadata: Option<PyObject>) -> PyResult<Self> {
        let role = match role {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            _ => return Err(PyValueError::new_err("role must be 'system', 'user', or 'assistant'")),
        };

        let metadata = if let Some(meta) = metadata {
            Python::with_gil(|py| {
                py_to_json(py, meta.as_ref(py)).ok()
            })
        } else {
            None
        };

        Ok(PyMessage {
            inner: Message {
                role,
                content,
                name,
                metadata,
            },
        })
    }

    #[getter]
    fn role(&self) -> String {
        match self.inner.role {
            MessageRole::System => "system".to_string(),
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
        }
    }

    #[getter]
    fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    fn __repr__(&self) -> String {
        format!("Message(role='{}', content='{}...')", 
                self.role(), 
                if self.inner.content.len() > 50 { 
                    format!("{}...", &self.inner.content[..50]) 
                } else { 
                    self.inner.content.clone() 
                })
    }
}

// Additional wrapper types for completeness
#[pyclass(name = "Tool")]
#[derive(Debug, Clone)]
pub struct PyTool {
    pub inner: Tool,
}

#[pyclass(name = "ToolChoice")]
#[derive(Debug, Clone)]
pub struct PyToolChoice {
    pub inner: ToolChoice,
}

#[pyclass(name = "ResponseFormat")]
#[derive(Debug, Clone)]
pub struct PyResponseFormat {
    pub inner: ResponseFormat,
}

#[pyclass(name = "SamplingParams")]
#[derive(Debug, Clone)]
pub struct PySamplingParams {
    pub inner: SamplingParams,
}

#[pyclass(name = "Limits")]
#[derive(Debug, Clone)]
pub struct PyLimits {
    pub inner: Limits,
}

#[pyclass(name = "MediaConfig")]
#[derive(Debug, Clone)]
pub struct PyMediaConfig {
    pub inner: MediaConfig,
}

#[pyclass(name = "ProviderInfo")]
#[derive(Debug, Clone)]
pub struct PyProviderInfo {
    pub inner: ProviderInfo,
}

#[pyclass(name = "ModelSpec")]
#[derive(Debug, Clone)]
pub struct PyModelSpec {
    pub inner: ModelSpec,
}

/// Python wrapper for TranslationResult
#[pyclass(name = "TranslationResult")]
#[derive(Debug, Clone)]
pub struct PyTranslationResult {
    pub inner: TranslationResult,
}

#[pymethods]
impl PyTranslationResult {
    #[getter]
    fn provider_request_json(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            json_to_py(py, &self.inner.provider_request_json)
        })
    }

    #[getter]
    fn has_lossiness(&self) -> bool {
        self.inner.has_lossiness()
    }

    #[getter]
    fn lossiness(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner.lossiness)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    #[getter]
    fn metadata(&self) -> PyResult<Option<PyObject>> {
        if let Some(metadata) = &self.inner.metadata {
            Python::with_gil(|py| {
                let json_str = serde_json::to_string(metadata)
                    .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
                let json_value: Value = serde_json::from_str(&json_str)
                    .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
                Ok(Some(json_to_py(py, &json_value)?))
            })
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn lossiness_summary(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let summary = &self.inner.lossiness.summary;
            let json_str = serde_json::to_string(summary)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    #[getter]
    fn max_severity(&self) -> String {
        format!("{:?}", self.inner.lossiness.max_severity)
    }

    fn to_dict(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    fn __repr__(&self) -> String {
        format!("TranslationResult(has_lossiness={}, max_severity={:?})", 
                self.has_lossiness(), self.inner.lossiness.max_severity)
    }
}

/// Python wrapper for UniformResponse
#[pyclass(name = "UniformResponse")]
#[derive(Debug, Clone)]
pub struct PyUniformResponse {
    pub inner: UniformResponse,
}

#[pymethods]
impl PyUniformResponse {
    #[getter]
    fn model(&self) -> String {
        self.inner.model.clone()
    }

    #[getter]
    fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[getter]
    fn finish_reason(&self) -> String {
        match self.inner.finish_reason {
            FinishReason::Stop => "stop".to_string(),
            FinishReason::Length => "length".to_string(),
            FinishReason::ToolCall => "tool_call".to_string(),
            FinishReason::EndConversation => "end_conversation".to_string(),
            FinishReason::Other => "other".to_string(),
        }
    }

    fn to_dict(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&self.inner)
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            let json_value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyValueError::new_err(format!("JSON error: {}", e)))?;
            json_to_py(py, &json_value)
        })
    }

    fn __repr__(&self) -> String {
        format!("UniformResponse(model='{}', finish_reason='{}')", 
                self.inner.model, self.finish_reason())
    }
}

/// Python wrapper for ValidationResult
#[pyclass(name = "ValidationResult")]
#[derive(Debug, Clone)]
pub struct PyValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

#[pymethods]
impl PyValidationResult {
    #[new]
    pub fn new(is_valid: bool, errors: Vec<String>) -> Self {
        PyValidationResult { is_valid, errors }
    }

    #[getter]
    fn is_valid(&self) -> bool {
        self.is_valid
    }

    #[getter]
    fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    fn __repr__(&self) -> String {
        format!("ValidationResult(is_valid={}, errors={})", 
                self.is_valid, self.errors.len())
    }
}

// Helper functions for JSON conversion
pub fn json_to_py(py: Python<'_>, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Ok(n.to_string().to_object(py))
            }
        }
        Value::String(s) => Ok(s.to_object(py)),
        Value::Array(arr) => {
            let py_list = pyo3::types::PyList::empty(py);
            for item in arr {
                py_list.append(json_to_py(py, item)?)?;
            }
            Ok(py_list.to_object(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
pub fn py_to_json(py: Python<'_>, obj: &PyAny) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into())))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(list) = obj.downcast::<pyo3::types::PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json(py, item)?);
        }
        Ok(Value::Array(arr))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key = k.extract::<String>()?;
            map.insert(key, py_to_json(py, v)?);
        }
        Ok(Value::Object(map))
    } else {
        Err(PyValueError::new_err("Unsupported Python type for JSON conversion"))
    }
}