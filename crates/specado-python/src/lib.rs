//! Specado Python Bindings
//!
//! This crate provides Python bindings for Specado using PyO3, enabling
//! Python applications to translate prompts, validate specifications,
//! and execute provider requests.

#![allow(non_local_definitions)] // PyO3 macros generate non-local impl blocks

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod types;
mod translate;
mod validate;
mod run;
mod error;

use types::*;
use error::*;

/// Specado Python module
#[pymodule]
fn _specado(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Add exception types
    m.add("SpecadoError", _py.get_type::<SpecadoError>())?;
    m.add("TranslationError", _py.get_type::<TranslationError>())?;
    m.add("ValidationError", _py.get_type::<ValidationError>())?;
    m.add("ProviderError", _py.get_type::<ProviderError>())?;
    m.add("TimeoutError", _py.get_type::<TimeoutError>())?;
    
    // Add core functions
    m.add_function(wrap_pyfunction!(translate::translate, m)?)?;
    m.add_function(wrap_pyfunction!(validate::validate, m)?)?;
    m.add_function(wrap_pyfunction!(run::run_async, m)?)?;
    m.add_function(wrap_pyfunction!(run::run_sync, m)?)?;
    m.add_function(wrap_pyfunction!(run::create_provider_request, m)?)?;
    
    // Add utility functions
    m.add_function(wrap_pyfunction!(version, m)?)?;
    
    // Add type classes
    m.add_class::<PyPromptSpec>()?;
    m.add_class::<PyProviderSpec>()?;
    m.add_class::<PyTranslationResult>()?;
    m.add_class::<PyUniformResponse>()?;
    m.add_class::<PyValidationResult>()?;
    
    Ok(())
}

/// Get the version of the Specado library
#[pyfunction]
fn version() -> PyResult<String> {
    Ok(format!("{} {}", 
        env!("CARGO_PKG_NAME"), 
        env!("CARGO_PKG_VERSION")
    ))
}