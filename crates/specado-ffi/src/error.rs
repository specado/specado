//! Error handling for FFI boundary
//!
//! This module provides utilities for safely propagating errors
//! across the FFI boundary without panics or undefined behavior.

use std::panic;
use std::any::Any;

use crate::types::SpecadoResult;
use crate::memory::set_last_error;

/// Convert a Specado core error to an FFI result code
pub fn map_core_error(error: specado_core::Error) -> SpecadoResult {
    use specado_core::Error;
    
    match error {
        Error::Json { .. } => {
            set_last_error(format!("JSON error: {}", error));
            SpecadoResult::JsonError
        }
        Error::Http { status_code, .. } => {
            set_last_error(format!("HTTP error: {}", error));
            match status_code {
                Some(401) | Some(403) => SpecadoResult::AuthenticationError,
                Some(429) => SpecadoResult::RateLimitError,
                _ => SpecadoResult::NetworkError,
            }
        }
        Error::HttpWithDiagnostics { diagnostics, .. } => {
            set_last_error(format!("{}", diagnostics.format_display(false)));
            match diagnostics.classification.as_str() {
                "AuthenticationError" => SpecadoResult::AuthenticationError,
                "RateLimitError" => SpecadoResult::RateLimitError,
                _ => SpecadoResult::NetworkError,
            }
        }
        Error::Provider { .. } => {
            set_last_error(format!("Provider error: {}", error));
            SpecadoResult::ProviderNotFound
        }
        Error::Validation { .. } => {
            set_last_error(format!("Validation error: {}", error));
            SpecadoResult::InvalidInput
        }
        Error::Configuration { .. } => {
            set_last_error(format!("Configuration error: {}", error));
            SpecadoResult::InvalidInput
        }
        Error::Unsupported { .. } => {
            set_last_error(format!("Unsupported: {}", error));
            SpecadoResult::NotImplemented
        }
        Error::Io { .. } => {
            set_last_error(format!("IO error: {}", error));
            SpecadoResult::InternalError
        }
        _ => {
            set_last_error(format!("Internal error: {}", error));
            SpecadoResult::InternalError
        }
    }
}

/// Safely execute a closure that might panic
/// 
/// This function catches any panics and converts them to appropriate
/// error codes, preventing undefined behavior at the FFI boundary.
pub fn catch_panic<F, R>(f: F) -> Result<R, SpecadoResult>
where
    F: FnOnce() -> Result<R, SpecadoResult> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => result,
        Err(panic_info) => {
            let msg = get_panic_message(&panic_info);
            set_last_error(format!("Panic occurred: {}", msg));
            Err(SpecadoResult::InternalError)
        }
    }
}

/// Extract a message from panic info
fn get_panic_message(panic_info: &Box<dyn Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    }
}

/// Macro for safely executing FFI functions
#[macro_export]
macro_rules! ffi_boundary {
    ($body:expr) => {{
        match $crate::error::catch_panic(|| $body) {
            Ok(result) => result,
            Err(code) => return code,
        }
    }};
}

/// Validate that a pointer is not null
pub fn validate_ptr<T>(ptr: *const T, name: &str) -> Result<(), SpecadoResult> {
    if ptr.is_null() {
        set_last_error(format!("{} is null", name));
        Err(SpecadoResult::NullPointer)
    } else {
        Ok(())
    }
}

/// Validate that a mutable pointer is not null
pub fn validate_mut_ptr<T>(ptr: *mut T, name: &str) -> Result<(), SpecadoResult> {
    if ptr.is_null() {
        set_last_error(format!("{} is null", name));
        Err(SpecadoResult::NullPointer)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panic_catching() {
        let result = catch_panic(|| {
            panic!("Test panic");
            #[allow(unreachable_code)]
            Ok(42)
        });
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpecadoResult::InternalError);
    }
    
    #[test]
    fn test_ptr_validation() {
        let value = 42;
        let ptr = &value as *const i32;
        
        assert!(validate_ptr(ptr, "test_ptr").is_ok());
        assert!(validate_ptr(std::ptr::null::<i32>(), "null_ptr").is_err());
    }
}