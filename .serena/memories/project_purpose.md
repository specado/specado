# Specado Project Purpose

Specado is a spec-driven LLM translation engine that compiles uniform prompts into provider-native requests with transparent lossiness reporting.

## Core Functionality
- Translates uniform PromptSpec format to provider-specific API formats
- Provides transparent lossiness reporting when features are not supported
- Supports multiple LLM providers (OpenAI, Anthropic, etc.)
- Handles validation, conflict resolution, and transformation pipelines

## Key Components
- **Translation Engine**: Core functionality for converting prompts
- **Lossiness Tracking**: Reports deviations and limitations during translation
- **Strictness Policies**: Configurable handling of unsupported features
- **JSONPath Mapping**: Field mapping between formats
- **Conflict Resolution**: Handles mutually exclusive fields
- **Transformation Pipeline**: Applies value transformations

## Project Goals
- Provide a uniform interface for multiple LLM providers
- Maintain transparency about feature limitations
- Enable strict validation and flexible configuration
- Support provider-specific optimizations