//! Memory management utilities for FFI
//!
//! This module provides safe memory allocation and deallocation
//! functions for use across the FFI boundary.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use crate::types::{SpecadoResult, SpecadoString, SpecadoBuffer};

// Thread-local storage for last error message
thread_local! {
    static LAST_ERROR: Mutex<Option<CString>> = Mutex::new(None);
}

/// Set the last error message for the current thread
pub fn set_last_error<S: Into<String>>(err: S) {
    let error_string = CString::new(err.into()).unwrap_or_else(|_| {
        CString::new("Error message contained null byte").unwrap()
    });
    
    LAST_ERROR.with(|e| {
        *e.lock().unwrap() = Some(error_string);
    });
}

/// Clear the last error message
pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.lock().unwrap() = None;
    });
}

/// Allocate a new string for FFI return
/// 
/// # Safety
/// The caller must free this string using `specado_string_free`
pub unsafe fn allocate_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => {
            set_last_error("String contains null byte");
            ptr::null_mut()
        }
    }
}

/// Free a string allocated by Specado
///
/// # Safety
/// The pointer must have been allocated by `allocate_string` or similar
#[no_mangle]
pub unsafe extern "C" fn specado_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    
    // Reconstruct the CString and let it drop
    let _ = CString::from_raw(s);
}

/// Allocate a byte buffer
///
/// # Safety
/// The caller must free this buffer using `specado_buffer_free`
pub unsafe fn allocate_buffer(data: &[u8]) -> *mut SpecadoBuffer {
    let len = data.len();
    let capacity = len;
    
    // Allocate memory for the data
    let layout = std::alloc::Layout::array::<u8>(len).unwrap();
    let data_ptr = std::alloc::alloc(layout);
    
    if data_ptr.is_null() {
        set_last_error("Failed to allocate buffer");
        return ptr::null_mut();
    }
    
    // Copy the data
    std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, len);
    
    // Create the buffer struct
    let buffer = Box::new(SpecadoBuffer {
        data: data_ptr,
        len,
        owned: true,
    });
    
    Box::into_raw(buffer)
}

/// Free a buffer allocated by Specado
///
/// # Safety
/// The pointer must have been allocated by `allocate_buffer`
#[no_mangle]
pub unsafe extern "C" fn specado_buffer_free(buffer: *mut SpecadoBuffer) {
    if buffer.is_null() {
        return;
    }
    
    let buffer = Box::from_raw(buffer);
    
    if buffer.owned && !buffer.data.is_null() {
        let layout = std::alloc::Layout::array::<u8>(buffer.len).unwrap();
        std::alloc::dealloc(buffer.data as *mut u8, layout);
    }
    
    // Buffer struct itself is freed when Box is dropped
}

/// Convert a C string to a Rust string
///
/// # Safety
/// The pointer must be a valid null-terminated C string
pub unsafe fn c_str_to_string(s: *const c_char) -> Result<String, SpecadoResult> {
    if s.is_null() {
        return Err(SpecadoResult::NullPointer);
    }
    
    match CStr::from_ptr(s).to_str() {
        Ok(str) => Ok(str.to_string()),
        Err(_) => {
            set_last_error("Invalid UTF-8 in input string");
            Err(SpecadoResult::Utf8Error)
        }
    }
}

/// Get the last error message
///
/// # Safety
/// Returns a pointer that should NOT be freed by the caller
#[no_mangle]
pub unsafe extern "C" fn specado_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        match &*e.lock().unwrap() {
            Some(err) => err.as_ptr(),
            None => ptr::null(),
        }
    })
}

/// Clear the last error message
#[no_mangle]
pub extern "C" fn specado_clear_error() {
    clear_last_error();
}

/// Create a SpecadoString from a Rust string
pub fn create_specado_string(s: &str) -> SpecadoString {
    let c_string = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    let data = c_string.as_ptr();
    let len = s.len();
    let capacity = len;
    
    // Leak the CString so it won't be freed
    std::mem::forget(c_string);
    
    SpecadoString {
        data,
        len,
        capacity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_handling() {
        set_last_error("Test error");
        unsafe {
            let error = specado_get_last_error();
            assert!(!error.is_null());
            let error_str = CStr::from_ptr(error).to_str().unwrap();
            assert_eq!(error_str, "Test error");
        }
        
        clear_last_error();
        unsafe {
            let error = specado_get_last_error();
            assert!(error.is_null());
        }
    }
    
    #[test]
    fn test_string_allocation() {
        unsafe {
            let s = allocate_string("Hello, FFI!");
            assert!(!s.is_null());
            
            let c_str = CStr::from_ptr(s);
            assert_eq!(c_str.to_str().unwrap(), "Hello, FFI!");
            
            specado_string_free(s);
        }
    }
}