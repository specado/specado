//! Command handlers for CLI subcommands
//!
//! This module contains the implementation logic for each CLI subcommand,
//! organized into focused submodules for better maintainability.

// Re-export all handler functions
pub use validate::handle_validate;
pub use preview::handle_preview;
pub use translate::handle_translate;
pub use run::handle_run;
pub use config::handle_config;
pub use completions::handle_completions;

// Module declarations
mod validate;
mod preview;
mod translate;
mod run;
mod config;
mod completions;
mod utils;