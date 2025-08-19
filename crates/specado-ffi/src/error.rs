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
        Error::Json { message, .. } => {
            set_last_error(format!("JSON error: {}", message));
            SpecadoResult::JsonError
        }
        Error::Http { status_code, message, .. } => {
            set_last_error(format!("HTTP error ({}): {}", status_code.unwrap_or(0), message));
            match status_code {
                Some(401) | Some(403) => SpecadoResult::AuthenticationError,
                Some(429) => SpecadoResult::RateLimitError,
                Some(404) => SpecadoResult::ProviderNotFound,
                Some(400) => SpecadoResult::InvalidInput,
                Some(408) => SpecadoResult::TimeoutError,
                _ => SpecadoResult::NetworkError,
            }
        }
        Error::HttpWithDiagnostics { diagnostics, .. } => {
            set_last_error(diagnostics.format_display(false).to_string());
            match diagnostics.classification.as_str() {
                "AuthenticationError" => SpecadoResult::AuthenticationError,
                "RateLimitError" => SpecadoResult::RateLimitError,
                "ValidationError" => SpecadoResult::InvalidInput,
                "TimeoutError" => SpecadoResult::TimeoutError,
                _ => SpecadoResult::NetworkError,
            }
        }
        Error::Provider { provider, message, .. } => {
            set_last_error(format!("Provider '{}' error: {}", provider, message));
            SpecadoResult::ProviderNotFound
        }
        Error::Validation { field, message, expected } => {
            let full_message = if let Some(exp) = expected {
                format!("Validation error in field '{}': {}. Expected: {}", field, message, exp)
            } else {
                format!("Validation error in field '{}': {}", field, message)
            };
            set_last_error(full_message);
            SpecadoResult::InvalidInput
        }
        Error::Configuration { message, .. } => {
            set_last_error(format!("Configuration error: {}", message));
            SpecadoResult::InvalidInput
        }
        Error::Unsupported { message, feature } => {
            let full_message = if let Some(feat) = feature {
                format!("Unsupported feature '{}': {}", feat, message)
            } else {
                format!("Unsupported: {}", message)
            };
            set_last_error(full_message);
            SpecadoResult::NotImplemented
        }
        Error::Io { message, .. } => {
            set_last_error(format!("IO error: {}", message));
            SpecadoResult::InternalError
        }
        Error::Translation { message, context } => {
            let full_message = if let Some(ctx) = context {
                format!("Translation error in {}: {}", ctx, message)
            } else {
                format!("Translation error: {}", message)
            };
            set_last_error(full_message);
            SpecadoResult::InternalError
        }
        Error::Timeout { message, timeout_duration } => {
            set_last_error(format!("Timeout: {} (after {:?})", message, timeout_duration));
            SpecadoResult::TimeoutError
        }
        Error::RateLimit { message, .. } => {
            set_last_error(format!("Rate limit exceeded: {}", message));
            SpecadoResult::RateLimitError
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
#[allow(dead_code)]
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