//! Error handling for Python bindings
//!
//! This module defines custom exception types and error handling
//! for the Python bindings, mapping Rust errors to Python exceptions.

use pyo3::prelude::*;
use pyo3::exceptions::{PyException, PyValueError, PyRuntimeError};
use pyo3::create_exception;
use specado_ffi::SpecadoResult;
use std::fmt;

// Define Python exception types directly
create_exception!(specado, SpecadoError, PyException);
create_exception!(specado, TranslationError, PyException);
create_exception!(specado, ValidationError, PyException);
create_exception!(specado, ProviderError, PyException);
create_exception!(specado, TimeoutError, PyException);

/// Convert FFI result codes to Python exceptions
pub fn map_ffi_result_to_py_err(result: SpecadoResult, message: Option<String>) -> PyErr {
    let error_msg = message.unwrap_or_else(|| result.error_message().to_string());
    
    match result {
        SpecadoResult::Success => unreachable!("Success should not be converted to error"),
        SpecadoResult::InvalidInput => ValidationError::new_err(error_msg),
        SpecadoResult::JsonError => ValidationError::new_err(error_msg),
        SpecadoResult::ProviderNotFound => ProviderError::new_err(error_msg),
        SpecadoResult::ModelNotFound => ProviderError::new_err(error_msg),
        SpecadoResult::NetworkError => ProviderError::new_err(error_msg),
        SpecadoResult::AuthenticationError => ProviderError::new_err(error_msg),
        SpecadoResult::RateLimitError => ProviderError::new_err(error_msg),
        SpecadoResult::TimeoutError => TimeoutError::new_err(error_msg),
        SpecadoResult::InternalError => SpecadoError::new_err(error_msg),
        SpecadoResult::MemoryError => PyRuntimeError::new_err(error_msg),
        SpecadoResult::Utf8Error => PyValueError::new_err(error_msg),
        SpecadoResult::NullPointer => PyValueError::new_err(error_msg),
        SpecadoResult::Cancelled => SpecadoError::new_err(error_msg),
        SpecadoResult::NotImplemented => PyRuntimeError::new_err(error_msg),
        SpecadoResult::Unknown => SpecadoError::new_err(error_msg),
    }
}

/// Helper function to get last error from FFI layer
pub fn get_last_ffi_error() -> Option<String> {
    unsafe {
        let error_ptr = specado_ffi::specado_get_last_error();
        if error_ptr.is_null() {
            return None;
        }
        
        let c_str = std::ffi::CStr::from_ptr(error_ptr);
        c_str.to_str().ok().map(|s| s.to_string())
    }
}