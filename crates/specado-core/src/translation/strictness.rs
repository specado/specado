//! Strictness Policy Engine for the Specado translation system
//!
//! This module implements a comprehensive policy engine that evaluates how StrictMode
//! (Strict/Warn/Coerce) affects translation behavior. The engine works with the
//! LossinessTracker to make decisions based on severity levels and strictness policies.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{
    Error, LossinessCode, LossinessItem, Result, Severity, StrictMode,
};
use super::{LossinessTracker, TranslationContext};
use serde_json::Value;
use std::collections::HashMap;

/// Action to take based on strictness policy evaluation
#[derive(Debug)]
pub enum StrictnessAction {
    /// Continue processing without modification
    Proceed,
    /// Log a warning and continue
    Warn { message: String },
    /// Fail immediately with error
    Fail { error: Error },
    /// Auto-adjust the value and continue
    Coerce { adjusted_value: Value, reason: String },
}

/// Policy evaluation result containing action and any lossiness to track
#[derive(Debug)]
pub struct PolicyResult {
    /// The action to take
    pub action: StrictnessAction,
    /// Optional lossiness item to add to tracker
    pub lossiness_item: Option<LossinessItem>,
}

/// Strictness Policy Engine that evaluates translation decisions
///
/// The StrictnessPolicy engine is responsible for determining how the translation
/// engine should respond to various issues based on the current strictness mode.
/// It works closely with the LossinessTracker to make informed decisions about
/// whether to proceed, warn, fail, or coerce values during translation.
#[derive(Debug)]
pub struct StrictnessPolicy {
    /// The current strictness mode
    mode: StrictMode,
    /// Reference to the translation context
    context: TranslationContext,
    /// Custom policy overrides - simplified to just store the override mode
    policy_overrides: HashMap<String, StrictMode>,
}

impl StrictnessPolicy {
    /// Create a new strictness policy engine
    pub fn new(context: TranslationContext) -> Self {
        let mode = context.strict_mode;
        Self {
            mode,
            context,
            policy_overrides: HashMap::new(),
        }
    }

    /// Add a custom policy override for a specific path
    /// 
    /// This allows overriding the default strictness mode for specific paths.
    pub fn add_override(&mut self, path: String, override_mode: StrictMode) {
        self.policy_overrides.insert(path, override_mode);
    }

    /// Evaluate whether to proceed with translation based on current lossiness
    ///
    /// This method examines the current state of the lossiness tracker and
    /// determines if the translation should continue based on the strictness mode.
    pub fn evaluate_proceeding(&self, tracker: &LossinessTracker) -> Result<()> {
        match self.mode {
            StrictMode::Strict => {
                if tracker.has_critical_issues() {
                    return Err(Error::StrictnessViolation {
                        message: "Critical lossiness issues detected in strict mode".to_string(),
                        mode: self.mode,
                        severity: Severity::Critical,
                    });
                }
                if tracker.has_errors() {
                    return Err(Error::StrictnessViolation {
                        message: "Error-level lossiness issues detected in strict mode".to_string(),
                        mode: self.mode,
                        severity: Severity::Error,
                    });
                }
            }
            StrictMode::Warn => {
                if tracker.has_critical_issues() {
                    return Err(Error::StrictnessViolation {
                        message: "Critical lossiness issues detected".to_string(),
                        mode: self.mode,
                        severity: Severity::Critical,
                    });
                }
                // In warn mode, errors are allowed but should be logged
            }
            StrictMode::Coerce => {
                if tracker.has_critical_issues() {
                    return Err(Error::StrictnessViolation {
                        message: "Critical lossiness issues cannot be coerced".to_string(),
                        mode: self.mode,
                        severity: Severity::Critical,
                    });
                }
                // In coerce mode, most issues are auto-adjusted
            }
        }
        Ok(())
    }

    /// Evaluate policy for an unsupported feature
    pub fn evaluate_unsupported_feature(
        &self,
        path: &str,
        feature_name: &str,
        value: Option<Value>,
    ) -> PolicyResult {
        // Check for custom override first
        let effective_mode = self.policy_overrides.get(path).unwrap_or(&self.mode);

        let message = format!("Feature '{}' is not supported by provider", feature_name);

        match effective_mode {
            StrictMode::Strict => PolicyResult {
                action: StrictnessAction::Fail {
                    error: Error::StrictnessViolation {
                        message: message.clone(),
                        mode: *effective_mode,
                        severity: Severity::Critical,
                    },
                },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Unsupported,
                    path: path.to_string(),
                    message,
                    severity: Severity::Critical,
                    before: value,
                    after: None,
                }),
            },
            StrictMode::Warn => PolicyResult {
                action: StrictnessAction::Warn { message: message.clone() },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Drop,
                    path: path.to_string(),
                    message: format!("Dropped unsupported feature '{}' in warn mode", feature_name),
                    severity: Severity::Warning,
                    before: value,
                    after: None,
                }),
            },
            StrictMode::Coerce => {
                // In coerce mode, we drop the unsupported feature
                PolicyResult {
                    action: StrictnessAction::Proceed,
                    lossiness_item: Some(LossinessItem {
                        code: LossinessCode::Drop,
                        path: path.to_string(),
                        message: format!("Dropped unsupported feature '{}' in coerce mode", feature_name),
                        severity: Severity::Warning,
                        before: value,
                        after: None,
                    }),
                }
            }
        }
    }

    /// Evaluate policy for a value that needs clamping
    pub fn evaluate_value_clamping(
        &self,
        path: &str,
        original_value: Value,
        min_value: f64,
        max_value: f64,
        provider_name: &str,
    ) -> PolicyResult {
        // Check for custom override first
        let effective_mode = self.policy_overrides.get(path).unwrap_or(&self.mode);

        if let Some(num) = original_value.as_f64() {
            if num < min_value || num > max_value {
                let clamped_value = num.max(min_value).min(max_value);
                let clamped_json = Value::from(clamped_value);
                let message = format!(
                    "Value {} clamped to {}'s supported range [{}, {}]",
                    num, provider_name, min_value, max_value
                );

                match effective_mode {
                    StrictMode::Strict => PolicyResult {
                        action: StrictnessAction::Warn { message: message.clone() },
                        lossiness_item: Some(LossinessItem {
                            code: LossinessCode::Clamp,
                            path: path.to_string(),
                            message,
                            severity: Severity::Warning,
                            before: Some(original_value),
                            after: Some(clamped_json),
                        }),
                    },
                    StrictMode::Warn => PolicyResult {
                        action: StrictnessAction::Warn { message: message.clone() },
                        lossiness_item: Some(LossinessItem {
                            code: LossinessCode::Clamp,
                            path: path.to_string(),
                            message,
                            severity: Severity::Info,
                            before: Some(original_value),
                            after: Some(clamped_json),
                        }),
                    },
                    StrictMode::Coerce => PolicyResult {
                        action: StrictnessAction::Coerce {
                            adjusted_value: clamped_json.clone(),
                            reason: message.clone(),
                        },
                        lossiness_item: Some(LossinessItem {
                            code: LossinessCode::Clamp,
                            path: path.to_string(),
                            message,
                            severity: Severity::Info,
                            before: Some(original_value),
                            after: Some(clamped_json),
                        }),
                    },
                }
            } else {
                // Value is within range, no action needed
                PolicyResult {
                    action: StrictnessAction::Proceed,
                    lossiness_item: None,
                }
            }
        } else {
            // Non-numeric value, cannot clamp
            let message = format!("Cannot clamp non-numeric value at path '{}'", path);
            PolicyResult {
                action: StrictnessAction::Fail {
                    error: Error::Validation {
                        field: path.to_string(),
                        message: message.clone(),
                        expected: Some("numeric value".to_string()),
                    },
                },
                lossiness_item: None,
            }
        }
    }

    /// Evaluate policy for conflicting field values
    pub fn evaluate_field_conflict(
        &self,
        path: &str,
        field1: &str,
        field2: &str,
        value1: Value,
        value2: Value,
        resolution_preference: Option<&str>,
    ) -> PolicyResult {
        // Check for custom override first
        let effective_mode = self.policy_overrides.get(path).unwrap_or(&self.mode);

        let message = format!("Conflicting values for fields '{}' and '{}'", field1, field2);

        // Determine which value to use based on resolution preference
        let (chosen_value, chosen_field) = if let Some(pref) = resolution_preference {
            if pref == field1 {
                (value1.clone(), field1)
            } else {
                (value2.clone(), field2)
            }
        } else {
            // Default to first field if no preference
            (value1.clone(), field1)
        };

        let conflict_data = serde_json::json!({
            field1: value1,
            field2: value2
        });

        match effective_mode {
            StrictMode::Strict => PolicyResult {
                action: StrictnessAction::Fail {
                    error: Error::StrictnessViolation {
                        message: message.clone(),
                        mode: *effective_mode,
                        severity: Severity::Error,
                    },
                },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Conflict,
                    path: path.to_string(),
                    message,
                    severity: Severity::Error,
                    before: Some(conflict_data),
                    after: Some(chosen_value),
                }),
            },
            StrictMode::Warn => PolicyResult {
                action: StrictnessAction::Warn {
                    message: format!("{}, using value from '{}'", message, chosen_field),
                },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Conflict,
                    path: path.to_string(),
                    message: format!("Resolved conflict by choosing '{}'", chosen_field),
                    severity: Severity::Warning,
                    before: Some(conflict_data),
                    after: Some(chosen_value),
                }),
            },
            StrictMode::Coerce => PolicyResult {
                action: StrictnessAction::Coerce {
                    adjusted_value: chosen_value.clone(),
                    reason: format!("Auto-resolved conflict by choosing '{}'", chosen_field),
                },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Conflict,
                    path: path.to_string(),
                    message: format!("Auto-resolved conflict by choosing '{}'", chosen_field),
                    severity: Severity::Info,
                    before: Some(conflict_data),
                    after: Some(chosen_value),
                }),
            },
        }
    }

    /// Evaluate policy for field relocation
    pub fn evaluate_field_relocation(
        &self,
        original_path: &str,
        new_path: &str,
        value: Value,
    ) -> PolicyResult {
        // Check for custom override first - but relocation is generally acceptable in all modes
        let _effective_mode = self.policy_overrides.get(original_path).unwrap_or(&self.mode);

        let message = format!("Field relocated from '{}' to '{}'", original_path, new_path);

        // Field relocation is generally acceptable in all modes
        PolicyResult {
            action: StrictnessAction::Proceed,
            lossiness_item: Some(LossinessItem {
                code: LossinessCode::Relocate,
                path: original_path.to_string(),
                message,
                severity: Severity::Info,
                before: Some(value.clone()),
                after: Some(value),
            }),
        }
    }

    /// Evaluate policy for performance impact scenarios
    pub fn evaluate_performance_impact(
        &self,
        path: &str,
        impact_description: &str,
        affected_value: Option<Value>,
    ) -> PolicyResult {
        // Check for custom override first
        let effective_mode = self.policy_overrides.get(path).unwrap_or(&self.mode);

        let message = format!("Performance impact: {}", impact_description);

        match effective_mode {
            StrictMode::Strict => PolicyResult {
                action: StrictnessAction::Warn { message: message.clone() },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::PerformanceImpact,
                    path: path.to_string(),
                    message,
                    severity: Severity::Warning,
                    before: affected_value,
                    after: None,
                }),
            },
            StrictMode::Warn | StrictMode::Coerce => PolicyResult {
                action: StrictnessAction::Warn { message: message.clone() },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::PerformanceImpact,
                    path: path.to_string(),
                    message,
                    severity: Severity::Warning,
                    before: affected_value,
                    after: None,
                }),
            },
        }
    }

    /// Evaluate policy for feature emulation
    pub fn evaluate_feature_emulation(
        &self,
        path: &str,
        feature_name: &str,
        emulation_method: &str,
        original_value: Option<Value>,
    ) -> PolicyResult {
        // Check for custom override first
        let effective_mode = self.policy_overrides.get(path).unwrap_or(&self.mode);

        let message = format!(
            "Feature '{}' emulated via {}",
            feature_name, emulation_method
        );

        match effective_mode {
            StrictMode::Strict => PolicyResult {
                action: StrictnessAction::Warn { message: message.clone() },
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Emulate,
                    path: path.to_string(),
                    message,
                    severity: Severity::Warning,
                    before: original_value,
                    after: None,
                }),
            },
            StrictMode::Warn | StrictMode::Coerce => PolicyResult {
                action: StrictnessAction::Proceed,
                lossiness_item: Some(LossinessItem {
                    code: LossinessCode::Emulate,
                    path: path.to_string(),
                    message,
                    severity: Severity::Warning,
                    before: original_value,
                    after: None,
                }),
            },
        }
    }

    /// Get the current strictness mode
    pub fn mode(&self) -> StrictMode {
        self.mode
    }

    /// Get a reference to the translation context
    pub fn context(&self) -> &TranslationContext {
        &self.context
    }

    /// Check if the policy should fail fast on any error
    pub fn should_fail_fast(&self) -> bool {
        matches!(self.mode, StrictMode::Strict)
    }

    /// Check if warnings should be logged
    pub fn should_log_warnings(&self) -> bool {
        matches!(self.mode, StrictMode::Warn | StrictMode::Strict)
    }

    /// Check if auto-coercion is enabled
    pub fn auto_coercion_enabled(&self) -> bool {
        matches!(self.mode, StrictMode::Coerce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Constraints, ConstraintLimits, EndpointConfig, Endpoints, InputModes, JsonOutputConfig,
        Mappings, Message, MessageRole, ModelSpec, PromptSpec, ProviderInfo, ProviderSpec,
        ResponseNormalization, SamplingParams, StreamNormalization, SyncNormalization, ToolingConfig,
    };
    use std::collections::HashMap;

    fn create_test_context(strict_mode: StrictMode) -> TranslationContext {
        let prompt_spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Test".to_string(),
                name: None,
                metadata: None,
            }],
            tools: None,
            tool_choice: None,
            response_format: None,
            sampling: None,
            limits: None,
            media: None,
            strict_mode,
        };

        let provider_spec = ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: HashMap::new(),
            },
            models: vec![],
        };

        let model_spec = ModelSpec {
            id: "test-model".to_string(),
            aliases: None,
            family: "test".to_string(),
            endpoints: Endpoints {
                chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
                streaming_chat_completion: EndpointConfig {
                    method: "POST".to_string(),
                    path: "/chat".to_string(),
                    protocol: "https".to_string(),
                    query: None,
                    headers: None,
                },
            },
            input_modes: InputModes {
                messages: true,
                single_text: false,
                images: false,
            },
            tooling: ToolingConfig {
                tools_supported: false,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: false,
                strategy: "system_prompt".to_string(),
            },
            parameters: serde_json::json!({}),
            constraints: Constraints {
                system_prompt_location: "first".to_string(),
                forbid_unknown_top_level_fields: false,
                mutually_exclusive: vec![],
                resolution_preferences: vec![],
                limits: ConstraintLimits {
                    max_tool_schema_bytes: 100000,
                    max_system_prompt_bytes: 10000,
                },
            },
            mappings: Mappings {
                paths: HashMap::new(),
                flags: HashMap::new(),
            },
            response_normalization: ResponseNormalization {
                sync: SyncNormalization {
                    content_path: "content".to_string(),
                    finish_reason_path: "finish".to_string(),
                    finish_reason_map: HashMap::new(),
                },
                stream: StreamNormalization {
                    protocol: "sse".to_string(),
                    event_selector: crate::EventSelector {
                        type_path: "type".to_string(),
                        routes: vec![],
                    },
                },
            },
        };

        TranslationContext::new(prompt_spec, provider_spec, model_spec, strict_mode)
    }

    #[test]
    fn test_policy_creation() {
        let context = create_test_context(StrictMode::Strict);
        let policy = StrictnessPolicy::new(context);
        assert_eq!(policy.mode(), StrictMode::Strict);
        assert!(policy.should_fail_fast());
        assert!(policy.should_log_warnings());
        assert!(!policy.auto_coercion_enabled());
    }

    #[test]
    fn test_evaluate_proceeding_strict_mode() {
        let context = create_test_context(StrictMode::Strict);
        let policy = StrictnessPolicy::new(context);

        // Test with clean tracker
        let tracker = LossinessTracker::new(StrictMode::Strict);
        assert!(policy.evaluate_proceeding(&tracker).is_ok());

        // Test with critical issues
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        tracker.add_unsupported("tools", "Not supported", None);
        assert!(policy.evaluate_proceeding(&tracker).is_err());

        // Test with errors
        let mut tracker = LossinessTracker::new(StrictMode::Strict);
        tracker.add_dropped("field", "Dropped", None);
        assert!(policy.evaluate_proceeding(&tracker).is_err());
    }

    #[test]
    fn test_evaluate_proceeding_warn_mode() {
        let context = create_test_context(StrictMode::Warn);
        let policy = StrictnessPolicy::new(context);

        // Test with errors (should be ok in warn mode)
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        tracker.add_dropped("field", "Dropped", None);
        assert!(policy.evaluate_proceeding(&tracker).is_ok());

        // Test with critical issues (should fail even in warn mode)
        let mut tracker = LossinessTracker::new(StrictMode::Warn);
        tracker.add_unsupported("tools", "Not supported", None);
        assert!(policy.evaluate_proceeding(&tracker).is_err());
    }

    #[test]
    fn test_evaluate_proceeding_coerce_mode() {
        let context = create_test_context(StrictMode::Coerce);
        let policy = StrictnessPolicy::new(context);

        // Test with errors (should be ok in coerce mode)
        let mut tracker = LossinessTracker::new(StrictMode::Coerce);
        tracker.add_dropped("field", "Dropped", None);
        assert!(policy.evaluate_proceeding(&tracker).is_ok());

        // Test with critical issues (should fail even in coerce mode)
        let mut tracker = LossinessTracker::new(StrictMode::Coerce);
        tracker.add_unsupported("tools", "Not supported", None);
        assert!(policy.evaluate_proceeding(&tracker).is_err());
    }

    #[test]
    fn test_evaluate_unsupported_feature_strict() {
        let context = create_test_context(StrictMode::Strict);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_unsupported_feature(
            "tools",
            "function_calling",
            Some(serde_json::json!([])),
        );

        match result.action {
            StrictnessAction::Fail { .. } => (),
            _ => panic!("Expected Fail action in strict mode"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_evaluate_unsupported_feature_warn() {
        let context = create_test_context(StrictMode::Warn);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_unsupported_feature(
            "tools",
            "function_calling",
            Some(serde_json::json!([])),
        );

        match result.action {
            StrictnessAction::Warn { .. } => (),
            _ => panic!("Expected Warn action in warn mode"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn test_evaluate_unsupported_feature_coerce() {
        let context = create_test_context(StrictMode::Coerce);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_unsupported_feature(
            "tools",
            "function_calling",
            Some(serde_json::json!([])),
        );

        match result.action {
            StrictnessAction::Proceed => (),
            _ => panic!("Expected Proceed action in coerce mode"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().code, LossinessCode::Drop);
    }

    #[test]
    fn test_evaluate_value_clamping() {
        let context = create_test_context(StrictMode::Coerce);
        let policy = StrictnessPolicy::new(context);

        // Test value that needs clamping
        let result = policy.evaluate_value_clamping(
            "temperature",
            serde_json::json!(2.5),
            0.0,
            2.0,
            "test-provider",
        );

        match result.action {
            StrictnessAction::Coerce { adjusted_value, .. } => {
                assert_eq!(adjusted_value, serde_json::json!(2.0));
            }
            _ => panic!("Expected Coerce action for clamping"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().code, LossinessCode::Clamp);

        // Test value that doesn't need clamping
        let result = policy.evaluate_value_clamping(
            "temperature",
            serde_json::json!(1.0),
            0.0,
            2.0,
            "test-provider",
        );

        match result.action {
            StrictnessAction::Proceed => (),
            _ => panic!("Expected Proceed action for value in range"),
        }
        assert!(result.lossiness_item.is_none());
    }

    #[test]
    fn test_evaluate_field_conflict() {
        let context = create_test_context(StrictMode::Warn);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_field_conflict(
            "sampling",
            "temperature",
            "temp",
            serde_json::json!(0.7),
            serde_json::json!(0.8),
            Some("temperature"),
        );

        match result.action {
            StrictnessAction::Warn { .. } => (),
            _ => panic!("Expected Warn action for field conflict"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().code, LossinessCode::Conflict);
    }

    #[test]
    fn test_evaluate_field_relocation() {
        let context = create_test_context(StrictMode::Strict);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_field_relocation(
            "max_tokens",
            "max_completion_tokens",
            serde_json::json!(100),
        );

        match result.action {
            StrictnessAction::Proceed => (),
            _ => panic!("Expected Proceed action for field relocation"),
        }
        assert!(result.lossiness_item.is_some());
        let lossiness_item = result.lossiness_item.unwrap();
        assert_eq!(lossiness_item.code, LossinessCode::Relocate);
        assert_eq!(lossiness_item.severity, Severity::Info);
    }

    #[test]
    fn test_evaluate_performance_impact() {
        let context = create_test_context(StrictMode::Warn);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_performance_impact(
            "response_format",
            "JSON mode emulated via system prompt",
            Some(serde_json::json!({"type": "json_object"})),
        );

        match result.action {
            StrictnessAction::Warn { .. } => (),
            _ => panic!("Expected Warn action for performance impact"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().code, LossinessCode::PerformanceImpact);
    }

    #[test]
    fn test_evaluate_feature_emulation() {
        let context = create_test_context(StrictMode::Coerce);
        let policy = StrictnessPolicy::new(context);

        let result = policy.evaluate_feature_emulation(
            "json_mode",
            "structured_output",
            "system_prompt",
            Some(serde_json::json!({"type": "json_object"})),
        );

        match result.action {
            StrictnessAction::Proceed => (),
            _ => panic!("Expected Proceed action for feature emulation in coerce mode"),
        }
        assert!(result.lossiness_item.is_some());
        assert_eq!(result.lossiness_item.unwrap().code, LossinessCode::Emulate);
    }

    #[test]
    fn test_policy_overrides() {
        let context = create_test_context(StrictMode::Strict);
        let mut policy = StrictnessPolicy::new(context);

        // Add override to use warn mode for tools instead of strict
        policy.add_override("tools".to_string(), StrictMode::Warn);

        let result = policy.evaluate_unsupported_feature(
            "tools",
            "function_calling",
            Some(serde_json::json!([])),
        );

        // Should use warn mode instead of default strict behavior
        match result.action {
            StrictnessAction::Warn { .. } => (),
            _ => panic!("Expected warn action due to override"),
        }
        
        // Should have warning severity since warn mode drops unsupported features
        assert_eq!(result.lossiness_item.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn test_integration_with_translate() {
        // This test demonstrates how the strictness policy engine integrates with
        // the main translate function
        let prompt_spec = PromptSpec {
            model_class: "Chat".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Test message".to_string(),
                name: None,
                metadata: None,
            }],
            tools: Some(vec![]), // Unsupported tools
            tool_choice: None,
            response_format: None,
            sampling: Some(SamplingParams {
                temperature: Some(2.5), // Out of range temperature
                top_p: None,
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
            }),
            limits: None,
            media: None,
            strict_mode: StrictMode::Warn,
        };

        let provider_spec = ProviderSpec {
            spec_version: "1.0.0".to_string(),
            provider: ProviderInfo {
                name: "test-provider".to_string(),
                base_url: "https://api.test.com".to_string(),
                headers: HashMap::new(),
            },
            models: vec![ModelSpec {
                id: "test-model".to_string(),
                aliases: None,
                family: "test".to_string(),
                endpoints: Endpoints {
                    chat_completion: EndpointConfig {
                        method: "POST".to_string(),
                        path: "/chat".to_string(),
                        protocol: "https".to_string(),
                        query: None,
                        headers: None,
                    },
                    streaming_chat_completion: EndpointConfig {
                        method: "POST".to_string(),
                        path: "/chat".to_string(),
                        protocol: "https".to_string(),
                        query: None,
                        headers: None,
                    },
                },
                input_modes: InputModes {
                    messages: true,
                    single_text: false,
                    images: false,
                },
                tooling: ToolingConfig {
                    tools_supported: false, // Tools not supported
                    parallel_tool_calls_default: false,
                    can_disable_parallel_tool_calls: false,
                    disable_switch: None,
                },
                json_output: JsonOutputConfig {
                    native_param: false,
                    strategy: "system_prompt".to_string(),
                },
                parameters: serde_json::json!({}),
                constraints: Constraints {
                    system_prompt_location: "first".to_string(),
                    forbid_unknown_top_level_fields: false,
                    mutually_exclusive: vec![],
                    resolution_preferences: vec![],
                    limits: ConstraintLimits {
                        max_tool_schema_bytes: 100000,
                        max_system_prompt_bytes: 10000,
                    },
                },
                mappings: Mappings {
                    paths: HashMap::new(),
                    flags: HashMap::new(),
                },
                response_normalization: ResponseNormalization {
                    sync: SyncNormalization {
                        content_path: "content".to_string(),
                        finish_reason_path: "finish".to_string(),
                        finish_reason_map: HashMap::new(),
                    },
                    stream: StreamNormalization {
                        protocol: "sse".to_string(),
                        event_selector: crate::EventSelector {
                            type_path: "type".to_string(),
                            routes: vec![],
                        },
                    },
                },
            }],
        };

        // Test with warn mode - should succeed but create lossiness report
        let result = crate::translate(&prompt_spec, &provider_spec, "test-model", StrictMode::Warn);
        assert!(result.is_ok());
        
        let translation_result = result.unwrap();
        
        // Should have lossiness items for unsupported tools and clamped temperature
        assert!(!translation_result.lossiness.items.is_empty());
        
        // Should contain warning about tools being dropped
        let has_tool_warning = translation_result.lossiness.items.iter()
            .any(|item| item.code == LossinessCode::Drop && item.path == "tools");
        assert!(has_tool_warning);
        
        // Should contain clamp warning for temperature
        let has_temp_clamp = translation_result.lossiness.items.iter()
            .any(|item| item.code == LossinessCode::Clamp && item.path == "temperature");
        assert!(has_temp_clamp);
    }

    #[test]
    fn test_mode_specific_behaviors() {
        // Test strict mode
        let context = create_test_context(StrictMode::Strict);
        let policy = StrictnessPolicy::new(context);
        assert!(policy.should_fail_fast());
        assert!(policy.should_log_warnings());
        assert!(!policy.auto_coercion_enabled());

        // Test warn mode
        let context = create_test_context(StrictMode::Warn);
        let policy = StrictnessPolicy::new(context);
        assert!(!policy.should_fail_fast());
        assert!(policy.should_log_warnings());
        assert!(!policy.auto_coercion_enabled());

        // Test coerce mode
        let context = create_test_context(StrictMode::Coerce);
        let policy = StrictnessPolicy::new(context);
        assert!(!policy.should_fail_fast());
        assert!(!policy.should_log_warnings());
        assert!(policy.auto_coercion_enabled());
    }
}
