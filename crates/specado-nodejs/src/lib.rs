//! Specado Node.js Bindings
//!
//! This crate provides Node.js bindings for Specado using NAPI-RS,
//! enabling JavaScript and TypeScript applications to use Specado's
//! prompt translation and execution capabilities.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod error;
mod translate;
mod validate;
mod run;
mod types;

// Re-export modules
pub use error::*;
pub use translate::*;
pub use validate::*;
pub use run::*;
pub use types::*;

/// Initialize the Specado module
#[napi]
pub fn init() -> Result<String> {
    Ok("Specado Node.js bindings initialized".to_string())
}

/// Get the version of the Specado library
#[napi]
pub fn get_version() -> Result<String> {
    Ok(format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
}

/// Get detailed version information including core library version
#[napi]
pub fn get_version_info() -> Result<VersionInfo> {
    Ok(VersionInfo {
        nodejs_binding: env!("CARGO_PKG_VERSION").to_string(),
        core_library: "0.1.0".to_string(), // Would use specado_core::version() when available
        build_timestamp: option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown").to_string(),
        git_commit: option_env!("VERGEN_GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

/// Version information structure
#[napi(object)]
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Node.js binding version
    pub nodejs_binding: String,
    /// Core library version
    pub core_library: String,
    /// Build timestamp
    pub build_timestamp: String,
    /// Git commit hash
    pub git_commit: String,
}