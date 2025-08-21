//! Builder for creating transformation rules
//!
//! This module provides a fluent builder API for constructing transformation rules
//! with optional configuration and validation.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use crate::Result;
use super::super::lossiness::LossinessTracker;
use super::types::{TransformationError, TransformationType, TransformationDirection, TransformationRule};
use std::sync::{Arc, Mutex};

/// Builder for creating transformation rules
pub struct TransformationRuleBuilder {
    id: String,
    source_path: String,
    target_path: Option<String>,
    transformation: Option<TransformationType>,
    direction: TransformationDirection,
    priority: i32,
    optional: bool,
    tracker: Option<Arc<Mutex<LossinessTracker>>>,
}

impl TransformationRuleBuilder {
    /// Create a new rule builder
    pub fn new(id: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_path: source_path.into(),
            target_path: None,
            transformation: None,
            direction: TransformationDirection::Forward,
            priority: 0,
            optional: false,
            tracker: None,
        }
    }

    /// Set the target path
    pub fn target_path(mut self, path: impl Into<String>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// Set the transformation type
    pub fn transformation(mut self, transformation: TransformationType) -> Self {
        self.transformation = Some(transformation);
        self
    }

    /// Set the direction
    pub fn direction(mut self, direction: TransformationDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Make the rule optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Add a lossiness tracker to this rule
    pub fn with_tracker(mut self, tracker: Arc<Mutex<LossinessTracker>>) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Build the transformation rule
    pub fn build(self) -> Result<TransformationRule> {
        let transformation = self.transformation.ok_or_else(|| {
            TransformationError::Configuration {
                message: "Transformation type is required".to_string(),
                rule_id: Some(self.id.clone()),
            }
        })?;

        Ok(TransformationRule {
            id: self.id,
            source_path: self.source_path,
            target_path: self.target_path,
            transformation,
            direction: self.direction,
            priority: self.priority,
            optional: self.optional,
        })
    }
}