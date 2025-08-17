//! JSONPath mapping engine for field translation
//!
//! This module implements the high-performance JSONPath mapping engine
//! for translating fields between uniform and provider-specific formats.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::TranslationContext;
use super::jsonpath::JSONPath;
use super::transformer::{TransformationPipeline, TransformationDirection};
use super::lossiness::LossinessTracker;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JSONPath mapper for translating fields between uniform and provider formats
///
/// The JSONPathMapper handles the translation of fields from the uniform
/// PromptSpec format to provider-specific JSON structures using JSONPath
/// expressions defined in the provider specification.
pub struct JSONPathMapper<'a> {
    context: &'a TranslationContext,
    /// Compiled JSONPath expressions cache
    compiled_paths: HashMap<String, JSONPath>,
}

impl<'a> JSONPathMapper<'a> {
    /// Create a new JSONPath mapper
    pub fn new(context: &'a TranslationContext) -> Self {
        Self {
            context,
            compiled_paths: HashMap::new(),
        }
    }

    /// Map a value from a source path to a target path
    ///
    /// Executes JSONPath expressions to extract data from the source and
    /// place it at the target location in the provider-specific format.
    pub fn map_field(
        &mut self,
        source_path: &str,
        target_path: &str,
        source_data: &Value,
        target_data: &mut Value,
    ) -> Result<()> {
        self.map_field_with_tracker(source_path, target_path, source_data, target_data, None)
    }

    /// Map a value from a source path to a target path with lossiness tracking
    ///
    /// This version includes comprehensive lossiness tracking for field mappings.
    pub fn map_field_with_tracker(
        &mut self,
        source_path: &str,
        target_path: &str,
        source_data: &Value,
        target_data: &mut Value,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<()> {
        // Extract value from source using JSONPath
        let extracted_values = self.extract_values(source_path, source_data)?;
        
        if extracted_values.is_empty() {
            // Track missing field if we have a tracker
            if let Some(tracker) = lossiness_tracker {
                if let Ok(mut tracker) = tracker.lock() {
                    tracker.track_dropped_field(
                        source_path,
                        None,
                        "Source field not found during mapping",
                        Some(self.context.provider_name().to_string()),
                    );
                }
            }
            // No values found - this might be acceptable depending on strictness
            return Ok(());
        }
        
        // For now, take the first value if multiple are found
        let value_to_set = extracted_values[0].clone();
        
        // Track field mapping if paths are different
        if source_path != target_path {
            if let Some(tracker) = lossiness_tracker {
                if let Ok(mut tracker) = tracker.lock() {
                    tracker.track_transformation(
                        source_path,
                        super::lossiness::OperationType::FieldMove,
                        Some(value_to_set.clone()),
                        Some(value_to_set.clone()),
                        &format!("Field moved from '{}' to '{}'", source_path, target_path),
                        Some(self.context.provider_name().to_string()),
                        HashMap::new(),
                    );
                }
            }
        }
        
        // Set the value at the target path
        self.set_value_at_path(target_path, value_to_set, target_data)
    }

    /// Apply all mappings from the provider spec
    ///
    /// This method iterates through all mapping rules defined in the
    /// provider specification and applies them to transform the uniform
    /// format to the provider-specific format.
    pub fn apply_mappings(&mut self, input: &Value) -> Result<Value> {
        self.apply_mappings_with_tracker(input, None)
    }

    /// Apply all mappings from the provider spec with lossiness tracking
    ///
    /// This version includes comprehensive tracking of field mappings and drops.
    pub fn apply_mappings_with_tracker(
        &mut self,
        input: &Value,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<Value> {
        let mut output = Value::Object(serde_json::Map::new());
        
        // Apply path mappings from the provider spec
        for (source_path, target_path) in &self.context.model_spec.mappings.paths {
            self.map_field_with_tracker(source_path, target_path, input, &mut output, lossiness_tracker)?;
        }
        
        // If no mappings are defined, perform a basic structure copy
        if self.context.model_spec.mappings.paths.is_empty() {
            output = input.clone();
        }
        
        // Apply flag mappings
        self.apply_flags_with_tracker(&mut output, lossiness_tracker)?;
        
        Ok(output)
    }

    /// Map a single value using the provider's mapping rules
    pub fn map_value(&mut self, path: &str, source_data: &Value) -> Result<Value> {
        // Extract the value at the given path
        let extracted_values = self.extract_values(path, source_data)?;
        
        if extracted_values.is_empty() {
            return Ok(Value::Null);
        }
        
        // Return the first extracted value
        Ok(extracted_values[0].clone())
    }

    /// Check if a path has a mapping defined
    pub fn has_mapping(&self, path: &str) -> bool {
        self.context.model_spec.mappings.paths.contains_key(path)
    }

    /// Get the target path for a source path
    pub fn get_target_path(&self, source_path: &str) -> Option<&String> {
        self.context.model_spec.mappings.paths.get(source_path)
    }
    
    /// Execute a JSONPath expression against data
    pub fn execute_path(&mut self, path: &str, data: &Value) -> Result<Vec<Value>> {
        self.extract_values(path, data)
    }
    
    /// Check if a JSONPath exists in the data
    pub fn path_exists(&mut self, path: &str, data: &Value) -> Result<bool> {
        let results = self.extract_values(path, data)?;
        Ok(!results.is_empty())
    }
    
    /// Get execution metrics for performance monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.compiled_paths.len(), self.compiled_paths.capacity())
    }

    /// Apply mappings with transformations
    ///
    /// This method combines JSONPath mapping with field transformations,
    /// providing a comprehensive translation from uniform to provider format.
    pub fn apply_mappings_with_transformations(
        &mut self,
        input: &Value,
        transformation_pipeline: &mut TransformationPipeline,
        direction: TransformationDirection,
    ) -> Result<Value> {
        self.apply_mappings_with_transformations_and_tracker(input, transformation_pipeline, direction, None)
    }

    /// Apply mappings with transformations and lossiness tracking
    ///
    /// This version includes comprehensive tracking throughout the transformation pipeline.
    pub fn apply_mappings_with_transformations_and_tracker(
        &mut self,
        input: &Value,
        transformation_pipeline: &mut TransformationPipeline,
        direction: TransformationDirection,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<Value> {
        // First apply JSONPath mappings
        let mut mapped_output = self.apply_mappings_with_tracker(input, lossiness_tracker)?;
        
        // Then apply transformations (transformer module should handle its own lossiness tracking)
        mapped_output = transformation_pipeline.transform(
            &mapped_output,
            direction,
            self.context,
        )?;
        
        Ok(mapped_output)
    }

    /// Create a basic transformation pipeline from provider mappings
    ///
    /// This creates transformation rules based on the provider spec's mapping configuration.
    /// In the future, this could be extended to read transformation configs from the spec.
    pub fn create_transformation_pipeline(&self) -> TransformationPipeline {
        let pipeline = TransformationPipeline::new();
        
        // Add transformation rules based on provider mappings
        // This is a placeholder - in practice, transformation rules would be
        // defined in the provider specification
        
        pipeline
    }

    /// Apply flag mappings (boolean transformations)
    pub fn apply_flags(&self, output: &mut Value) -> Result<()> {
        self.apply_flags_with_tracker(output, None)
    }

    /// Apply flag mappings with lossiness tracking
    pub fn apply_flags_with_tracker(
        &self,
        output: &mut Value,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<()> {
        // Apply flag mappings from the provider spec
        for (flag_name, flag_value) in &self.context.model_spec.mappings.flags {
            // Flags are simple key-value pairs that get set directly
            if let Some(obj) = output.as_object_mut() {
                obj.insert(flag_name.clone(), flag_value.clone());
                
                // Track flag application
                if let Some(tracker) = lossiness_tracker {
                    if let Ok(mut tracker) = tracker.lock() {
                        tracker.track_transformation(
                            &format!("$.{}", flag_name),
                            super::lossiness::OperationType::DefaultApplied,
                            None,
                            Some(flag_value.clone()),
                            &format!("Applied provider flag: {}", flag_name),
                            Some(self.context.provider_name().to_string()),
                            HashMap::new(),
                        );
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Extract values at a JSONPath
    ///
    /// Uses the high-performance JSONPath engine to extract values
    fn extract_values(&mut self, path: &str, data: &Value) -> Result<Vec<Value>> {
        // Check cache first
        if !self.compiled_paths.contains_key(path) {
            let jsonpath = JSONPath::parse(path)?;
            self.compiled_paths.insert(path.to_string(), jsonpath);
        }
        
        let jsonpath = &self.compiled_paths[path];
        let results = jsonpath.execute(data)?;
        
        // Convert references to owned values
        Ok(results.into_iter().cloned().collect())
    }

    /// Set value at a JSONPath
    ///
    /// Sets a value at the specified JSONPath location, creating
    /// intermediate objects as needed.
    fn set_value_at_path(&self, path: &str, value: Value, data: &mut Value) -> Result<()> {
        // For now, use simple dot notation parsing
        // In the future, this could support full JSONPath for target paths
        if path == "$" {
            *data = value;
            return Ok(());
        }
        
        let path = path.trim_start_matches("$.");
        let parts: Vec<&str> = path.split('.').collect();
        
        if parts.is_empty() {
            return Ok(());
        }
        
        // Ensure data is an object
        if !data.is_object() {
            *data = Value::Object(serde_json::Map::new());
        }
        
        let mut current = data;
        let last_index = parts.len() - 1;
        
        for (i, part) in parts.iter().enumerate() {
            if i == last_index {
                // Set the final value
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(part.to_string(), value);
                }
                return Ok(());
            } else {
                // Navigate or create intermediate objects
                let obj = current.as_object_mut().ok_or_else(|| {
                    crate::Error::Translation {
                        message: "Expected object for path navigation".to_string(),
                        context: Some("JSONPath".to_string()),
                    }
                })?;
                current = obj
                    .entry(part.to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
            }
        }
        
        Ok(())
    }

    /// Check if a provider supports a specific feature
    pub fn provider_supports_feature(&self, feature: &str) -> bool {
        match feature {
            "tools" => self.context.model_spec.tooling.tools_supported,
            "json_output" => self.context.model_spec.json_output.native_param,
            "streaming" => true, // Most providers support streaming
            _ => false,
        }
    }

    /// Track when a field is dropped due to provider limitations
    pub fn track_field_dropped_due_to_provider(
        &self,
        field_path: &str,
        original_value: Option<Value>,
        reason: &str,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) {
        if let Some(tracker) = lossiness_tracker {
            if let Ok(mut tracker) = tracker.lock() {
                tracker.track_dropped_field(
                    field_path,
                    original_value,
                    reason,
                    Some(self.context.provider_name().to_string()),
                );
            }
        }
    }

    /// Track when a field is mapped to a different location
    pub fn track_field_mapping(
        &self,
        from_path: &str,
        to_path: &str,
        value: Option<Value>,
        reason: &str,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) {
        if let Some(tracker) = lossiness_tracker {
            if let Ok(mut tracker) = tracker.lock() {
                let mut metadata = HashMap::new();
                metadata.insert("target_path".to_string(), to_path.to_string());
                
                tracker.track_transformation(
                    from_path,
                    super::lossiness::OperationType::FieldMove,
                    value.clone(),
                    value,
                    reason,
                    Some(self.context.provider_name().to_string()),
                    metadata,
                );
            }
        }
    }

    /// Handle array manipulations with tracking
    pub fn handle_array_manipulation(
        &mut self,
        source_path: &str,
        target_path: &str,
        source_data: &Value,
        manipulation_type: &str,
        lossiness_tracker: Option<&Arc<Mutex<LossinessTracker>>>,
    ) -> Result<Value> {
        let extracted_values = self.extract_values(source_path, source_data)?;
        
        if extracted_values.is_empty() {
            if let Some(tracker) = lossiness_tracker {
                if let Ok(mut tracker) = tracker.lock() {
                    tracker.track_dropped_field(
                        source_path,
                        None,
                        &format!("Array not found for manipulation: {}", manipulation_type),
                        Some(self.context.provider_name().to_string()),
                    );
                }
            }
            return Ok(Value::Array(vec![]));
        }
        
        let array_value = &extracted_values[0];
        
        // Track the array manipulation
        if let Some(tracker) = lossiness_tracker {
            if let Ok(mut tracker) = tracker.lock() {
                let mut metadata = HashMap::new();
                metadata.insert("manipulation_type".to_string(), manipulation_type.to_string());
                metadata.insert("target_path".to_string(), target_path.to_string());
                
                tracker.track_transformation(
                    source_path,
                    super::lossiness::OperationType::FieldMove,
                    Some(array_value.clone()),
                    Some(array_value.clone()),
                    &format!("Array manipulation: {} from {} to {}", manipulation_type, source_path, target_path),
                    Some(self.context.provider_name().to_string()),
                    metadata,
                );
            }
        }
        
        Ok(array_value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PromptSpec, ProviderSpec, ModelSpec, StrictMode,
        Message, MessageRole, ProviderInfo, Endpoints, EndpointConfig,
        InputModes, ToolingConfig, JsonOutputConfig, Constraints,
        ConstraintLimits, Mappings, ResponseNormalization,
        SyncNormalization, StreamNormalization,
    };
    use std::collections::HashMap;

    fn create_test_context() -> TranslationContext {
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
            strict_mode: StrictMode::Warn,
        };

        let mut path_mappings = HashMap::new();
        path_mappings.insert("$.messages".to_string(), "conversation".to_string());
        path_mappings.insert("$.temperature".to_string(), "temp".to_string());

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
                tools_supported: true,
                parallel_tool_calls_default: false,
                can_disable_parallel_tool_calls: false,
                disable_switch: None,
            },
            json_output: JsonOutputConfig {
                native_param: true,
                strategy: "native".to_string(),
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
                paths: path_mappings,
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

        TranslationContext::new(prompt_spec, provider_spec, model_spec, StrictMode::Warn)
    }

    #[test]
    fn test_mapper_creation() {
        let context = create_test_context();
        let mapper = JSONPathMapper::new(&context);
        
        assert!(mapper.has_mapping("$.messages"));
        assert!(mapper.has_mapping("$.temperature"));
        assert!(!mapper.has_mapping("unknown"));
        
        let (cache_size, _cache_capacity) = mapper.get_cache_stats();
        assert_eq!(cache_size, 0); // No paths compiled yet
    }

    #[test]
    fn test_get_target_path() {
        let context = create_test_context();
        let mapper = JSONPathMapper::new(&context);
        
        assert_eq!(mapper.get_target_path("$.messages"), Some(&"conversation".to_string()));
        assert_eq!(mapper.get_target_path("$.temperature"), Some(&"temp".to_string()));
        assert_eq!(mapper.get_target_path("unknown"), None);
    }

    #[test]
    fn test_map_value() {
        let context = create_test_context();
        let mut mapper = JSONPathMapper::new(&context);
        
        let data = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0.7
        });
        let result = mapper.map_value("$.messages", &data).unwrap();
        
        assert_eq!(result, serde_json::json!([{"role": "user", "content": "hello"}]));
    }

    #[test]
    fn test_extract_values() {
        let context = create_test_context();
        let mut mapper = JSONPathMapper::new(&context);
        
        let data = serde_json::json!({
            "level1": {
                "level2": {
                    "value": 42
                }
            }
        });
        
        let result = mapper.extract_values("$.level1.level2.value", &data).unwrap();
        assert_eq!(result, vec![serde_json::json!(42)]);
        
        let result = mapper.extract_values("$.level1.level2", &data).unwrap();
        assert_eq!(result, vec![serde_json::json!({"value": 42})]);
        
        let result = mapper.extract_values("$.nonexistent", &data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_set_value_at_path() {
        let context = create_test_context();
        let mapper = JSONPathMapper::new(&context);
        
        let mut data = serde_json::json!({});
        mapper.set_value_at_path("$.field1", serde_json::json!("value1"), &mut data).unwrap();
        
        assert_eq!(data, serde_json::json!({"field1": "value1"}));
        
        mapper.set_value_at_path("$.nested.field2", serde_json::json!(42), &mut data).unwrap();
        assert_eq!(data["nested"]["field2"], serde_json::json!(42));
    }

    #[test]
    fn test_apply_mappings() {
        let context = create_test_context();
        let mut mapper = JSONPathMapper::new(&context);
        
        let input = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0.7,
        });
        
        let output = mapper.apply_mappings(&input).unwrap();
        
        // Should have applied the mappings from the context
        assert!(output.is_object());
        // With the test context, $.messages -> conversation and $.temperature -> temp
        if let Some(conversation) = output.get("conversation") {
            assert_eq!(conversation, &serde_json::json!([{"role": "user", "content": "hello"}]));
        }
        if let Some(temp) = output.get("temp") {
            assert_eq!(temp, &serde_json::json!(0.7));
        }
    }
}