use crate::cli::{AuditTargetChoice, RuntimeOptions};
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;
use std::time::Duration;

#[cfg(feature = "audit-logging")]
use specado::audit::{AuditConfig, AuditContext, AuditTarget};
#[cfg(feature = "hot-reload")]
use specado::hot_reload::{set_global_config, HotReloadConfig};

#[cfg(feature = "hot-reload")]
pub fn apply_hot_reload_config(options: &RuntimeOptions, provider_path: &Path) {
    if !options.watch {
        return;
    }

    let mut paths = if options.watch_dirs.is_empty() {
        vec![provider_path.to_path_buf()]
    } else {
        options.watch_dirs.clone()
    };

    if paths.is_empty() {
        paths.push(provider_path.to_path_buf());
    }

    let config = HotReloadConfig::enabled(paths, Duration::from_millis(250));
    set_global_config(config);
    eprintln!(
        "{} Hot reload is experimental; no watcher is started until the feature is fully implemented.",
        "⚠".yellow()
    );
}

#[cfg(feature = "audit-logging")]
pub fn build_audit_context(options: &RuntimeOptions) -> Result<Option<AuditContext>> {
    if options.audit_file.is_some()
        && !matches!(options.audit_target, Some(AuditTargetChoice::File))
    {
        return Err(anyhow!(
            "--audit-file can only be used with --audit-target file"
        ));
    }

    let target = match &options.audit_target {
        None if options.audit_redact.is_empty() => return Ok(None),
        None => Some(AuditTarget::Stdout),
        Some(AuditTargetChoice::Stdout) => Some(AuditTarget::Stdout),
        Some(AuditTargetChoice::File) => {
            let path = options
                .audit_file
                .clone()
                .ok_or_else(|| anyhow!("--audit-file is required when --audit-target file"))?;
            Some(AuditTarget::File { path })
        }
    };

    let config = AuditConfig {
        target,
        redact: options.audit_redact.clone(),
    };

    if !config.is_enabled() {
        return Ok(None);
    }

    eprintln!(
        "{} Audit logging is experimental and currently writes JSONL synchronously.",
        "⚠".yellow()
    );

    Ok(Some(AuditContext::new(config)))
}
