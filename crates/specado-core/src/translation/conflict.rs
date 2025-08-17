//! Conflict resolution for mutually exclusive fields during translation
//!
//! This module handles conflicts between fields that cannot be used together
//! according to provider constraints, using strictness policies and lossiness
//! tracking to ensure proper resolution.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::{Error, Result, StrictMode, LossinessItem, LossinessCode, Severity};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;

use super::{
    TranslationContext,
    StrictnessPolicy,
    StrictnessAction,
    LossinessTracker,
    lossiness::{OperationType, TransformationRecord},
};

/// Represents a conflict between mutually exclusive fields
#[derive(Debug, Clone)]
pub struct FieldConflict {
    /// The group of mutually exclusive fields
    pub conflict_group: Vec<String>,
    
    /// Fields from the group that are actually present
    pub present_fields: Vec<String>,
    
    /// Values of the present fields
    pub field_values: HashMap<String, Value>,
    
    /// The field that will be kept after resolution
    pub winner: Option<String>,
    
    /// Fields that will be dropped
    pub losers: Vec<String>,
    
    /// Reason for the resolution decision
    pub resolution_reason: String,
    
    /// Whether this conflict was auto-resolved
    pub auto_resolved: bool,
}

/// Strategies for resolving conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionStrategy {
    /// Use provider's resolution preferences
    PreferenceOrder,
    
    /// Keep the first field in document order
    FirstWins,
    
    /// Keep the last field in document order
    LastWins,
    
    /// Keep the field with the most specific value
    MostSpecific,
    
    /// Fail on any conflict
    Fail,
    
    /// Custom resolution logic
    Custom(String),
}

/// Configuration for conflict resolution
#[derive(Debug, Clone)]
pub struct ConflictResolutionConfig {
    /// Strategy to use for resolution
    pub strategy: ResolutionStrategy,
    
    /// Whether to track dropped fields in lossiness
    pub track_lossiness: bool,
    
    /// Whether to emit warnings for auto-resolved conflicts
    pub warn_on_resolution: bool,
    
    /// Maximum number of conflicts to auto-resolve
    pub max_auto_resolutions: Option<usize>,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        Self {
            strategy: ResolutionStrategy::PreferenceOrder,
            track_lossiness: true,
            warn_on_resolution: true,
            max_auto_resolutions: None,
        }
    }
}

/// Main conflict resolver that handles mutually exclusive fields
pub struct ConflictResolver {
    context: TranslationContext,
    config: ConflictResolutionConfig,
    strictness_policy: StrictnessPolicy,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(context: TranslationContext) -> Self {
        let strictness_policy = StrictnessPolicy::new(context.clone());
        Self {
            context,
            config: ConflictResolutionConfig::default(),
            strictness_policy,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(context: TranslationContext, config: ConflictResolutionConfig) -> Self {
        let strictness_policy = StrictnessPolicy::new(context.clone());
        Self {
            context,
            config,
            strictness_policy,
        }
    }
    
    /// Detect conflicts in the current request
    pub fn detect_conflicts(&self, request: &Value) -> Vec<FieldConflict> {
        let mut conflicts = Vec::new();
        
        // Get the request as an object
        let request_obj = match request.as_object() {
            Some(obj) => obj,
            None => return conflicts,
        };
        
        // Check each mutually exclusive group
        for exclusive_group in &self.context.model_spec.constraints.mutually_exclusive {
            let mut present_fields = Vec::new();
            let mut field_values = HashMap::new();
            
            // Check which fields from the group are present
            for field in exclusive_group {
                if let Some(value) = self.get_field_value(request_obj, field) {
                    present_fields.push(field.clone());
                    field_values.insert(field.clone(), value);
                }
            }
            
            // If more than one field is present, we have a conflict
            if present_fields.len() > 1 {
                conflicts.push(FieldConflict {
                    conflict_group: exclusive_group.clone(),
                    present_fields: present_fields.clone(),
                    field_values,
                    winner: None,
                    losers: vec![],
                    resolution_reason: String::new(),
                    auto_resolved: false,
                });
            }
        }
        
        conflicts
    }
    
    /// Resolve all detected conflicts
    pub fn resolve_conflicts(
        &self,
        request: &mut Value,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<Vec<FieldConflict>> {
        let mut conflicts = self.detect_conflicts(request);
        
        // If there are no conflicts, return early
        if conflicts.is_empty() {
            return Ok(conflicts);
        }
        
        // Check if we should fail on conflicts based on strict mode
        if self.context.strict_mode == StrictMode::Strict && self.config.strategy == ResolutionStrategy::Fail {
            return Err(Error::Translation {
                message: format!(
                    "Found {} field conflicts in strict mode with fail strategy",
                    conflicts.len()
                ),
                context: Some(format!("Conflicts: {:?}", conflicts)),
            });
        }
        
        // Resolve each conflict
        for conflict in &mut conflicts {
            self.resolve_single_conflict(conflict, request, lossiness_tracker)?;
        }
        
        Ok(conflicts)
    }
    
    /// Resolve a single conflict
    fn resolve_single_conflict(
        &self,
        conflict: &mut FieldConflict,
        request: &mut Value,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<()> {
        // Determine which field to keep based on strategy
        let winner = self.determine_winner(conflict)?;
        
        // Set the winner and losers
        conflict.winner = Some(winner.clone());
        conflict.losers = conflict.present_fields
            .iter()
            .filter(|f| **f != winner)
            .cloned()
            .collect();
        
        // Use strictness policy to evaluate the conflict resolution
        let dropped_values: HashMap<String, Value> = conflict.losers
            .iter()
            .filter_map(|field| {
                conflict.field_values.get(field).map(|v| (field.clone(), v.clone()))
            })
            .collect();
        
        let policy_result = self.strictness_policy.evaluate_conflict_resolution(
            &winner,
            &conflict.losers,
            &conflict.resolution_reason,
            Some(serde_json::json!(dropped_values)),
        );
        
        // Handle the policy action
        match policy_result.action {
            StrictnessAction::Fail { error } => return Err(error),
            StrictnessAction::Warn { message } => {
                if self.config.warn_on_resolution {
                    log::warn!("{}", message);
                }
            }
            StrictnessAction::Proceed | StrictnessAction::Coerce { .. } => {
                // Continue with resolution
            }
        }
        
        // Track lossiness if configured
        if self.config.track_lossiness {
            if let Some(tracker) = lossiness_tracker {
                if let Ok(mut tracker) = tracker.lock() {
                    // Add policy lossiness item if present
                    if let Some(lossiness_item) = policy_result.lossiness_item {
                        tracker.add_item(lossiness_item);
                    }
                    
                    // Track each dropped field
                    for loser in &conflict.losers {
                        let mut metadata = HashMap::new();
                        metadata.insert("conflict_group".to_string(), format!("{:?}", conflict.conflict_group));
                        metadata.insert("winner".to_string(), winner.clone());
                        metadata.insert("strategy".to_string(), format!("{:?}", self.config.strategy));
                        
                        tracker.track_transformation(
                            loser,
                            OperationType::Dropped,
                            conflict.field_values.get(loser).cloned(),
                            None,
                            &format!(
                                "Dropped due to conflict with '{}': {}",
                                winner, conflict.resolution_reason
                            ),
                            Some(self.context.provider_name().to_string()),
                            metadata,
                        );
                    }
                }
            }
        }
        
        // Remove the losing fields from the request
        if let Some(obj) = request.as_object_mut() {
            for loser in &conflict.losers {
                self.remove_field(obj, loser);
            }
        }
        
        conflict.auto_resolved = true;
        
        Ok(())
    }
    
    /// Determine which field should win based on the resolution strategy
    fn determine_winner(&self, conflict: &FieldConflict) -> Result<String> {
        let winner = match &self.config.strategy {
            ResolutionStrategy::PreferenceOrder => {
                self.resolve_by_preference(conflict)?
            }
            ResolutionStrategy::FirstWins => {
                conflict.present_fields.first()
                    .cloned()
                    .ok_or_else(|| Error::Translation {
                        message: "No fields present in conflict".to_string(),
                        context: None,
                    })?
            }
            ResolutionStrategy::LastWins => {
                conflict.present_fields.last()
                    .cloned()
                    .ok_or_else(|| Error::Translation {
                        message: "No fields present in conflict".to_string(),
                        context: None,
                    })?
            }
            ResolutionStrategy::MostSpecific => {
                self.resolve_by_specificity(conflict)?
            }
            ResolutionStrategy::Fail => {
                return Err(Error::Translation {
                    message: format!(
                        "Conflict between mutually exclusive fields: {:?}",
                        conflict.present_fields
                    ),
                    context: Some(format!("Conflict group: {:?}", conflict.conflict_group)),
                });
            }
            ResolutionStrategy::Custom(logic) => {
                self.resolve_by_custom_logic(conflict, logic)?
            }
        };
        
        Ok(winner)
    }
    
    /// Resolve conflict using provider's resolution preferences
    fn resolve_by_preference(&self, conflict: &FieldConflict) -> Result<String> {
        let preferences = &self.context.model_spec.constraints.resolution_preferences;
        
        // If no preferences defined, fall back to first wins
        if preferences.is_empty() {
            return conflict.present_fields.first()
                .cloned()
                .ok_or_else(|| Error::Translation {
                    message: "No fields present in conflict".to_string(),
                    context: None,
                });
        }
        
        // Find the first field in preferences that is present
        for pref_field in preferences {
            if conflict.present_fields.contains(pref_field) {
                return Ok(pref_field.clone());
            }
        }
        
        // If none of the present fields are in preferences, use first present
        conflict.present_fields.first()
            .cloned()
            .ok_or_else(|| Error::Translation {
                message: "No fields present in conflict".to_string(),
                context: None,
            })
    }
    
    /// Resolve conflict by choosing the most specific value
    fn resolve_by_specificity(&self, conflict: &FieldConflict) -> Result<String> {
        let mut most_specific = None;
        let mut highest_score = 0;
        
        for field in &conflict.present_fields {
            let score = self.calculate_specificity_score(
                conflict.field_values.get(field)
            );
            if score > highest_score {
                highest_score = score;
                most_specific = Some(field.clone());
            }
        }
        
        most_specific.ok_or_else(|| Error::Translation {
            message: "Could not determine most specific field".to_string(),
            context: None,
        })
    }
    
    /// Calculate how specific a value is (higher = more specific)
    fn calculate_specificity_score(&self, value: Option<&Value>) -> usize {
        match value {
            None => 0,
            Some(Value::Null) => 1,
            Some(Value::Bool(_)) => 2,
            Some(Value::Number(_)) => 3,
            Some(Value::String(s)) => 4 + s.len(),
            Some(Value::Array(a)) => 5 + a.len() * 10,
            Some(Value::Object(o)) => 6 + o.len() * 20,
        }
    }
    
    /// Resolve using custom logic (placeholder for extensibility)
    fn resolve_by_custom_logic(&self, conflict: &FieldConflict, logic: &str) -> Result<String> {
        // This could be extended to support custom resolution patterns
        // For now, just return the first field with a note
        log::warn!("Custom resolution logic '{}' not implemented, using first field", logic);
        conflict.present_fields.first()
            .cloned()
            .ok_or_else(|| Error::Translation {
                message: "No fields present in conflict".to_string(),
                context: None,
            })
    }
    
    /// Get a field value from the request, handling nested paths
    fn get_field_value(&self, obj: &serde_json::Map<String, Value>, field: &str) -> Option<Value> {
        // Handle simple field names
        if !field.contains('.') {
            return obj.get(field).cloned();
        }
        
        // Handle nested paths (e.g., "sampling.temperature")
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = Value::Object(obj.clone());
        
        for part in parts {
            match current {
                Value::Object(ref obj) => {
                    current = obj.get(part)?.clone();
                }
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    /// Remove a field from the request, handling nested paths
    fn remove_field(&self, obj: &mut serde_json::Map<String, Value>, field: &str) {
        // Handle simple field names
        if !field.contains('.') {
            obj.remove(field);
            return;
        }
        
        // Handle nested paths
        let parts: Vec<&str> = field.split('.').collect();
        if parts.is_empty() {
            return;
        }
        
        // Navigate to the parent object
        let (last, parents) = parts.split_last().unwrap();
        let mut current = obj;
        
        for part in parents {
            match current.get_mut(*part) {
                Some(Value::Object(ref mut nested)) => {
                    current = nested;
                }
                _ => return,
            }
        }
        
        // Remove the field
        current.remove(*last);
    }
}

/// Extension trait for StrictnessPolicy to handle conflict resolution
impl StrictnessPolicy {
    /// Evaluate a conflict resolution decision
    pub fn evaluate_conflict_resolution(
        &self,
        winner: &str,
        losers: &[String],
        reason: &str,
        dropped_values: Option<Value>,
    ) -> super::PolicyResult {
        let message = format!(
            "Resolved conflict: keeping '{}', dropping {:?}. Reason: {}",
            winner, losers, reason
        );
        
        match self.context().strict_mode {
            StrictMode::Strict => {
                // In strict mode, conflicts might be errors
                if losers.len() > 1 {
                    super::PolicyResult {
                        action: StrictnessAction::Fail {
                            error: Error::Translation {
                                message: format!("Multiple field conflicts in strict mode: {:?}", losers),
                                context: Some(message),
                            },
                        },
                        lossiness_item: None,
                    }
                } else {
                    // Single conflict can be resolved with warning
                    super::PolicyResult {
                        action: StrictnessAction::Warn { message: message.clone() },
                        lossiness_item: Some(LossinessItem {
                            code: LossinessCode::Conflict,
                            path: losers.join(", "),
                            message,
                            severity: Severity::Warning,
                            before: dropped_values.clone(),
                            after: None,
                        }),
                    }
                }
            }
            StrictMode::Warn => {
                super::PolicyResult {
                    action: StrictnessAction::Warn { message: message.clone() },
                    lossiness_item: Some(LossinessItem {
                        code: LossinessCode::Drop,
                        path: losers.join(", "),
                        message,
                        severity: Severity::Info,
                        before: dropped_values.clone(),
                        after: None,
                    }),
                }
            }
            StrictMode::Coerce => {
                super::PolicyResult {
                    action: StrictnessAction::Proceed,
                    lossiness_item: Some(LossinessItem {
                        code: LossinessCode::Drop,
                        path: losers.join(", "),
                        message,
                        severity: Severity::Info,
                        before: dropped_values.clone(),
                        after: None,
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PromptSpec, ProviderSpec, ModelSpec, ProviderInfo, MessageRole, Message};
    use crate::{Endpoints, EndpointConfig, InputModes, ToolingConfig, JsonOutputConfig};
    use crate::{Constraints, ConstraintLimits, Mappings, ResponseNormalization};
    use crate::{SyncNormalization, StreamNormalization, EventSelector};
    
    fn create_test_context() -> TranslationContext {
        let prompt = PromptSpec {
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
            strict_mode: StrictMode::Warn,
        };
        
        let provider = ProviderSpec {
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
                    tools_supported: false,
                    parallel_tool_calls_default: false,
                    can_disable_parallel_tool_calls: false,
                    disable_switch: None,
                },
                json_output: JsonOutputConfig {
                    native_param: false,
                    strategy: "none".to_string(),
                },
                parameters: serde_json::json!({}),
                constraints: Constraints {
                    system_prompt_location: "first".to_string(),
                    forbid_unknown_top_level_fields: false,
                    mutually_exclusive: vec![
                        vec!["temperature".to_string(), "top_k".to_string()],
                        vec!["stream".to_string(), "stream_options".to_string()],
                    ],
                    resolution_preferences: vec![
                        "temperature".to_string(),
                        "stream_options".to_string(),
                    ],
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
                        event_selector: EventSelector {
                            type_path: "type".to_string(),
                            routes: vec![],
                        },
                    },
                },
            }],
        };
        
        let model = provider.models[0].clone();
        TranslationContext::new(
            prompt,
            provider,
            model,
            StrictMode::Warn,
        )
    }
    
    #[test]
    fn test_detect_conflicts() {
        let context = create_test_context();
        let resolver = ConflictResolver::new(context);
        
        let request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,
            "top_k": 40,  // Conflicts with temperature
            "stream": true,
            "stream_options": {"include_usage": true},  // Conflicts with stream
        });
        
        let conflicts = resolver.detect_conflicts(&request);
        assert_eq!(conflicts.len(), 2);
        
        // Check first conflict (temperature vs top_k)
        assert!(conflicts[0].present_fields.contains(&"temperature".to_string()));
        assert!(conflicts[0].present_fields.contains(&"top_k".to_string()));
        
        // Check second conflict (stream vs stream_options)
        assert!(conflicts[1].present_fields.contains(&"stream".to_string()));
        assert!(conflicts[1].present_fields.contains(&"stream_options".to_string()));
    }
    
    #[test]
    fn test_resolve_by_preference() {
        let context = create_test_context();
        let resolver = ConflictResolver::new(context);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,
            "top_k": 40,
        });
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        let conflicts = resolver.resolve_conflicts(&mut request, Some(&tracker)).unwrap();
        
        // Temperature should win based on resolution_preferences
        assert_eq!(conflicts[0].winner, Some("temperature".to_string()));
        assert_eq!(conflicts[0].losers, vec!["top_k".to_string()]);
        
        // top_k should be removed from request
        assert!(request["temperature"].is_number());
        assert!(request.get("top_k").is_none());
        
        // Check lossiness tracking
        let tracker = tracker.lock().unwrap();
        let stats = tracker.get_summary_statistics();
        assert!(stats.total_transformations > 0);
    }
    
    #[test]
    fn test_resolve_first_wins_strategy() {
        let mut context = create_test_context();
        // Clear resolution preferences to test FirstWins strategy
        context.model_spec.constraints.resolution_preferences.clear();
        
        let config = ConflictResolutionConfig {
            strategy: ResolutionStrategy::FirstWins,
            ..Default::default()
        };
        let resolver = ConflictResolver::with_config(context, config);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,  // First in conflict group
            "top_k": 40,
        });
        
        let conflicts = resolver.resolve_conflicts(&mut request, None).unwrap();
        
        // First field (temperature) should win
        assert_eq!(conflicts[0].winner, Some("temperature".to_string()));
        assert!(request["temperature"].is_number());
        assert!(request.get("top_k").is_none());
    }
    
    #[test]
    fn test_resolve_most_specific_strategy() {
        let context = create_test_context();
        let config = ConflictResolutionConfig {
            strategy: ResolutionStrategy::MostSpecific,
            ..Default::default()
        };
        let resolver = ConflictResolver::with_config(context, config);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "stream": true,  // Simple boolean
            "stream_options": {  // More specific object
                "include_usage": true,
                "chunk_size": 1024
            },
        });
        
        let conflicts = resolver.resolve_conflicts(&mut request, None).unwrap();
        
        // stream_options should win as it's more specific (object vs bool)
        assert_eq!(conflicts[0].winner, Some("stream_options".to_string()));
        assert!(request["stream_options"].is_object());
        assert!(request.get("stream").is_none());
    }
    
    #[test]
    fn test_strict_mode_conflict_handling() {
        let mut context = create_test_context();
        context.strict_mode = StrictMode::Strict;
        
        let config = ConflictResolutionConfig {
            strategy: ResolutionStrategy::Fail,
            ..Default::default()
        };
        let resolver = ConflictResolver::with_config(context, config);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,
            "top_k": 40,
        });
        
        // Should fail in strict mode with Fail strategy
        let result = resolver.resolve_conflicts(&mut request, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_no_conflicts() {
        let context = create_test_context();
        let resolver = ConflictResolver::new(context);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,
            "max_tokens": 100,
        });
        
        let conflicts = resolver.resolve_conflicts(&mut request, None).unwrap();
        assert_eq!(conflicts.len(), 0);
        
        // Request should be unchanged
        assert!(request["temperature"].is_number());
        assert!(request["max_tokens"].is_number());
    }
    
    #[test]
    fn test_nested_field_conflict() {
        let mut context = create_test_context();
        // Add a nested field conflict
        context.model_spec.constraints.mutually_exclusive.push(
            vec!["sampling.temperature".to_string(), "sampling.top_k".to_string()]
        );
        
        let resolver = ConflictResolver::new(context);
        
        let request = serde_json::json!({
            "model": "test-model",
            "sampling": {
                "temperature": 0.7,
                "top_k": 40,
            }
        });
        
        let conflicts = resolver.detect_conflicts(&request);
        assert!(conflicts.len() > 0);
        
        // Should detect the nested conflict
        let nested_conflict = conflicts.iter()
            .find(|c| c.present_fields.contains(&"sampling.temperature".to_string()))
            .expect("Should find nested conflict");
        
        assert!(nested_conflict.present_fields.contains(&"sampling.top_k".to_string()));
    }
    
    #[test]
    fn test_comprehensive_tracking() {
        let context = create_test_context();
        let resolver = ConflictResolver::new(context);
        
        let mut request = serde_json::json!({
            "model": "test-model",
            "temperature": 0.7,
            "top_k": 40,
            "stream": true,
            "stream_options": {"include_usage": true},
        });
        
        let tracker = Arc::new(Mutex::new(LossinessTracker::new(StrictMode::Warn)));
        let conflicts = resolver.resolve_conflicts(&mut request, Some(&tracker)).unwrap();
        
        assert_eq!(conflicts.len(), 2);
        
        // Get the audit trail
        let tracker = tracker.lock().unwrap();
        let audit = tracker.generate_audit_report();
        
        // Should have transformation records for dropped fields
        assert!(audit.contains("top_k"));
        assert!(audit.contains("stream"));
        assert!(audit.contains("conflict"));
        // The word "Dropped" appears in the operation type
        assert!(audit.contains("Dropped") || audit.contains("dropped"));
    }
}
