//! Specado FFI - Foreign Function Interface for cross-language bindings
//!
//! This crate provides a C-compatible FFI layer for Specado, enabling
//! integration with languages like Python, Node.js, Go, and others.
//!
//! # Safety
//! 
//! All FFI functions are marked `unsafe` as they deal with raw pointers
//! and cross-language boundaries. Users must ensure:
//! - Proper memory management (free allocated strings/buffers)
//! - Valid UTF-8 strings
//! - Non-null pointers where required
//! - Thread safety for concurrent calls

#![warn(missing_docs)]

#[macro_use]
mod error;
mod api;
mod memory;
mod run;
mod translate;
mod types;

// Re-export public API
pub use api::*;
pub use memory::{specado_string_free, specado_buffer_free, specado_get_last_error, specado_clear_error};
pub use types::{SpecadoResult, SpecadoContext, TranslationHandle, ResponseHandle};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        unsafe {
            let version = specado_version();
            assert!(!version.is_null());
        }
    }
}