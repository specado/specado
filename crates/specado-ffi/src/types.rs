//! FFI-safe type definitions
//! 
//! All types in this module are designed to be safely passed across
//! the FFI boundary with C ABI compatibility.

use std::os::raw::{c_char, c_int};

/// Result codes for FFI operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecadoResult {
    /// Operation completed successfully
    Success = 0,
    /// Invalid input parameters
    InvalidInput = -1,
    /// JSON parsing error
    JsonError = -2,
    /// Provider not found
    ProviderNotFound = -3,
    /// Model not found
    ModelNotFound = -4,
    /// Network error during operation
    NetworkError = -5,
    /// Authentication failure
    AuthenticationError = -6,
    /// Rate limit exceeded
    RateLimitError = -7,
    /// Timeout occurred
    TimeoutError = -8,
    /// Internal error
    InternalError = -9,
    /// Memory allocation failure
    MemoryError = -10,
    /// Invalid UTF-8 string
    Utf8Error = -11,
    /// Null pointer provided
    NullPointer = -12,
    /// Operation cancelled
    Cancelled = -13,
    /// Not implemented
    NotImplemented = -14,
    /// Unknown error
    Unknown = -99,
}

/// Opaque handle for a Specado context
#[repr(C)]
pub struct SpecadoContext {
    _private: [u8; 0],
}

/// Opaque handle for a translation result
#[repr(C)]
pub struct TranslationHandle {
    _private: [u8; 0],
}

/// Opaque handle for a response
#[repr(C)]
pub struct ResponseHandle {
    _private: [u8; 0],
}

/// FFI-safe string wrapper
#[repr(C)]
pub struct SpecadoString {
    /// Pointer to UTF-8 string data
    pub data: *const c_char,
    /// Length of the string in bytes
    pub len: usize,
    /// Capacity (for owned strings)
    pub capacity: usize,
}

/// FFI-safe byte buffer
#[repr(C)]
pub struct SpecadoBuffer {
    /// Pointer to byte data
    pub data: *const u8,
    /// Length of the buffer
    pub len: usize,
    /// Whether this buffer owns its data
    pub owned: bool,
}

impl SpecadoResult {
    /// Convert from a Rust Result
    pub fn from_result<T, E>(result: Result<T, E>) -> (Self, Option<T>) {
        match result {
            Ok(value) => (SpecadoResult::Success, Some(value)),
            Err(_) => (SpecadoResult::InternalError, None),
        }
    }
    
    /// Check if the result indicates success
    pub fn is_success(self) -> bool {
        self == SpecadoResult::Success
    }
    
    /// Get a human-readable error message
    pub fn error_message(self) -> &'static str {
        match self {
            SpecadoResult::Success => "Success",
            SpecadoResult::InvalidInput => "Invalid input parameters",
            SpecadoResult::JsonError => "JSON parsing error",
            SpecadoResult::ProviderNotFound => "Provider not found",
            SpecadoResult::ModelNotFound => "Model not found",
            SpecadoResult::NetworkError => "Network error",
            SpecadoResult::AuthenticationError => "Authentication failed",
            SpecadoResult::RateLimitError => "Rate limit exceeded",
            SpecadoResult::TimeoutError => "Operation timed out",
            SpecadoResult::InternalError => "Internal error",
            SpecadoResult::MemoryError => "Memory allocation failed",
            SpecadoResult::Utf8Error => "Invalid UTF-8 string",
            SpecadoResult::NullPointer => "Null pointer provided",
            SpecadoResult::Cancelled => "Operation cancelled",
            SpecadoResult::NotImplemented => "Not implemented",
            SpecadoResult::Unknown => "Unknown error",
        }
    }
}