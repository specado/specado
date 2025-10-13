//! Public Specado API re-exported for downstream consumers.
//!
//! This crate wraps the internal `specado-core` implementation and exposes
//! the same types and helpers while reserving the `specado` name on crates.io.

#![allow(clippy::all)]

pub use specado_core::*;
