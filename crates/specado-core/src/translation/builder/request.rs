//! Incremental provider request JSON builder
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::TranslationResultBuilder;
use serde_json::Value;

/// Helper builder for incrementally constructing provider request JSON
pub struct ProviderRequestBuilder {
    builder: TranslationResultBuilder,
    request: Value,
}

impl ProviderRequestBuilder {
    pub(super) fn new(builder: TranslationResultBuilder) -> Self {
        Self {
            builder,
            request: serde_json::json!({}),
        }
    }

    /// Set a field in the provider request JSON
    pub fn set_field<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Value::Object(ref mut obj) = self.request {
            obj.insert(key.to_string(), serde_json::to_value(value).unwrap_or(Value::Null));
        }
        self
    }

    /// Set the model field
    pub fn set_model(self, model: &str) -> Self {
        self.set_field("model", model)
    }

    /// Set messages array
    pub fn set_messages(self, messages: Vec<Value>) -> Self {
        self.set_field("messages", messages)
    }

    /// Add a single message to the messages array
    pub fn add_message(mut self, message: Value) -> Self {
        if let Value::Object(ref mut obj) = self.request {
            let messages = obj.entry("messages".to_string())
                .or_insert_with(|| Value::Array(vec![]));
            
            if let Value::Array(ref mut arr) = messages {
                arr.push(message);
            }
        }
        self
    }

    /// Set tool-related fields
    pub fn set_tools(self, tools: Vec<Value>) -> Self {
        self.set_field("tools", tools)
    }

    /// Set tool choice
    pub fn set_tool_choice(self, tool_choice: Value) -> Self {
        self.set_field("tool_choice", tool_choice)
    }

    /// Set sampling parameters
    pub fn set_temperature(self, temperature: f64) -> Self {
        self.set_field("temperature", temperature)
    }

    /// Set top_p parameter
    pub fn set_top_p(self, top_p: f64) -> Self {
        self.set_field("top_p", top_p)
    }

    /// Set max_tokens parameter
    pub fn set_max_tokens(self, max_tokens: u32) -> Self {
        self.set_field("max_tokens", max_tokens)
    }

    /// Set response format
    pub fn set_response_format(self, format: Value) -> Self {
        self.set_field("response_format", format)
    }

    /// Merge another JSON object into the request
    pub fn merge_object(mut self, other: Value) -> Self {
        if let (Value::Object(ref mut base), Value::Object(other_obj)) = (&mut self.request, other) {
            for (key, value) in other_obj {
                base.insert(key, value);
            }
        }
        self
    }

    /// Complete the incremental building and return to the main builder
    pub fn done(mut self) -> TranslationResultBuilder {
        self.builder.provider_request_json = Some(self.request);
        self.builder.update_state();
        self.builder
    }
}