# Specado PRD Documentation Index

## 📚 Document Overview
**Product**: Specado v1.0 - Spec-driven LLM Translation Engine  
**Document Type**: Product Requirements Document (PRD)  
**Version**: 1.0.0  
**Status**: Final  
**Date**: 2025-08-16  
**Purpose**: Define the complete v1.0 specification for Specado, a library and CLI for universal LLM provider interoperability  

## 🎯 Executive Summary

Specado is a **spec-driven translation and normalization engine** that:
- Compiles uniform LLM prompts into provider-native requests using declarative specifications
- Normalizes provider responses bidirectionally to a common format
- Provides transparent **lossiness reports** detailing all transformations
- Enables seamless multi-provider workflows without hardcoded adapters

### Core Value Proposition
> "All provider behavior is defined in declarative specification files - zero hardcoded adapters in the engine."

## 📑 Table of Contents

### [📋 Product Definition](#product-definition)
- [One-Line Summary](#one-line-summary)
- [Goals & Non-Goals](#goals-non-goals)
- [Success Metrics](#success-metrics)

### [👥 Users & Use Cases](#users-use-cases)
- [Personas](#personas)
- [Core Use Cases](#core-use-cases)
- [User Journeys](#user-journeys)

### [🏗️ System Architecture](#system-architecture)
- [System Overview](#system-overview)
- [Translation Pipeline](#translation-pipeline)
- [Normalization Flow](#normalization-flow)

### [🔌 Public APIs](#public-apis)
- [Rust Core Library](#rust-core-library)
- [Command-Line Interface](#cli-interface)
- [Language Bindings](#language-bindings)

### [📝 Specifications](#specifications)
- [PromptSpec Schema](#promptspec-schema)
- [ProviderSpec Schema](#providerspec-schema)
- [UniformResponse Format](#uniform-response)
- [LossinessReport Model](#lossiness-model)

### [🚀 Delivery Plan](#delivery-plan)
- [The Pyramid - 5 Levels](#pyramid-levels)
- [L1: Contracts & Preview](#level-1)
- [L2: Sync End-to-End](#level-2)
- [L3: Streaming Lite](#level-3)
- [L4: Streaming Normalized](#level-4)
- [L5: Complete Features](#level-5)

### [🧪 Quality & Testing](#quality-testing)
- [Testing Strategy](#testing-strategy)
- [Golden Tests](#golden-tests)
- [Binding Parity](#binding-parity)

### [📦 Project Structure](#project-structure)
- [Crate Layout](#crate-layout)
- [Repository Structure](#repository-structure)
- [Versioning Strategy](#versioning-strategy)

### [📊 Appendices](#appendices)
- [Provider Spec Examples](#provider-examples)
- [Lossiness Codes](#lossiness-codes)
- [Example Workflows](#example-workflows)

## 🔍 Quick Navigation

### By Component

#### **Core Engine**
- [Translation Logic](#translation-pipeline)
- [Normalization Logic](#normalization-flow)
- [Lossiness Reporting](#lossiness-model)
- [Streaming Support](#streaming-architecture)

#### **Specifications**
- [PromptSpec Fields](#promptspec-fields)
- [ProviderSpec Structure](#providerspec-structure)
- [Mapping Rules](#mapping-rules)
- [Validation Requirements](#validation-requirements)

#### **CLI Commands**
- [validate](#cli-validate)
- [preview](#cli-preview)
- [run](#cli-run)
- [stream](#cli-stream)
- [matrix](#cli-matrix)
- [diff](#cli-diff)

#### **Bindings**
- [Node.js API](#nodejs-binding)
- [Python API](#python-binding)
- [WASM Interface](#wasm-binding)
- [FFI Boundary](#ffi-boundary)

### By Task

#### **Getting Started**
1. [Environment Setup](#environment-setup)
2. [First Translation](#first-translation)
3. [Understanding Lossiness](#understanding-lossiness)
4. [Provider Configuration](#provider-configuration)

#### **Integration**
1. [Adding New Providers](#adding-providers)
2. [Custom Specifications](#custom-specs)
3. [Handling Responses](#response-handling)
4. [Stream Processing](#stream-processing)

#### **Operations**
1. [Monitoring Lossiness](#monitoring-lossiness)
2. [Performance Tuning](#performance-tuning)
3. [Error Handling](#error-handling)
4. [Debug Logging](#debug-logging)

## 🎯 Key Concepts

### Translation Pipeline
```
UniformPrompt → Validate → Map → Transform → ProviderRequest
                    ↓                              ↓
             PromptSpec Schema            LossinessReport
```

### Normalization Pipeline
```
ProviderResponse → Parse → Extract → Normalize → UniformResponse
                      ↓                                ↓
                Stream Events                   Structured Output
```

### Lossiness Model
```yaml
Categories:
  - Clamp: Value adjusted to range
  - Drop: Unsupported field removed
  - Emulate: Non-native implementation
  - Conflict: Mutual exclusivity resolved
  - Relocate: Field position changed
  - Unsupported: Capability unavailable
  - MapFallback: Alternative mapping used
  - PerformanceImpact: Quality/latency risk
```

### Strictness Policies
```yaml
Strict: Fail on any lossiness
Warn: Proceed with warnings
Coerce: Auto-adjust values
```

## 📊 Command Reference

### Core Commands
```bash
# Validation & Preview
specado validate --prompt p.json --provider spec.yaml --model gpt-5
specado preview --prompt p.json --provider spec.yaml --strict warn

# Execution
specado run --prompt p.json --provider spec.yaml --out response.json
specado stream --prompt p.json --provider spec.yaml

# Analysis
specado matrix --prompt p.json --models "openai/gpt-5 anthropic/opus"
specado diff --left response1.json --right response2.json
```

### API Examples
```rust
// Rust
let result = translate(prompt_json, provider_spec, "gpt-5", StrictMode::Warn)?;

// Node.js
const result = await specado.translate(prompt, spec, 'gpt-5', 'warn');

// Python
result = specado.translate(prompt, spec, 'gpt-5', strictness='warn')
```

## 📈 Implementation Roadmap

### Phase 1: Foundation (L1-L2)
- **Week 1-2**: Schema definitions and validation
- **Week 3-4**: Translation engine and preview
- **Week 5-6**: HTTP client and sync normalization

### Phase 2: Streaming (L3-L4)
- **Week 7-8**: Raw streaming support
- **Week 9-10**: Normalized stream events

### Phase 3: Features (L5)
- **Week 11-12**: Tools and structured outputs
- **Week 13-14**: Feature flags and emulation
- **Week 15-16**: Bindings and CLI polish

## 🚨 Critical Requirements

### Must Have (v1.0)
- ✅ Pure spec-driven behavior (no hardcoded adapters)
- ✅ Complete lossiness transparency
- ✅ OpenAI & Anthropic support
- ✅ JSON Schema validation
- ✅ Streaming with cancellation
- ✅ Node.js & Python bindings

### Won't Have (v1.0)
- ❌ Orchestration or agent loops
- ❌ UI beyond CLI
- ❌ Auto-scraping provider docs
- ❌ Cost estimation
- ❌ Request queuing

## 📊 Success Criteria

### Technical Success
```yaml
Zero Provider Code: No provider-specific logic in engine
Spec Coverage: 100% behavior in specifications
Translation Speed: <100ms overhead
Stream Latency: <50ms per event
```

### User Success
```yaml
Provider Switch: Single config change
Lossiness Visibility: Complete transparency
API Consistency: Identical across bindings
Documentation: Comprehensive and clear
```

## 🔗 Related Documentation

### Internal References
- [JSON Schemas](/schemas/) - Authoritative contract definitions
- [Provider Specs](/providers/) - Curated provider specifications
- [CODES.md](/docs/CODES.md) - Lossiness code documentation

### External Standards
- [JSON Schema Draft 2020-12](https://json-schema.org/draft/2020-12/json-schema-core.html)
- [JSONPath Specification](https://goessner.net/articles/JsonPath/)
- [Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html)

## 📝 Document Structure

### Section Organization
1. **Executive Overview** - Product vision and value
2. **Technical Specification** - Detailed requirements
3. **Implementation Plan** - Phased delivery approach
4. **Quality Standards** - Testing and validation
5. **Appendices** - Examples and references

### Version History
- **v1.0.0** (2025-08-16) - Final PRD release
- Future: v1.1 (Additional providers), v1.2 (New modalities), v2.0 (Policy-as-code)

## 🎓 Learning Path

### For Product Managers
1. Read [Executive Summary](#executive-summary)
2. Review [Goals & Non-Goals](#goals-non-goals)
3. Understand [Personas & Use Cases](#users-use-cases)
4. Check [Success Metrics](#success-metrics)

### For Engineers
1. Study [System Architecture](#system-architecture)
2. Review [Specifications](#specifications)
3. Understand [Delivery Plan](#delivery-plan)
4. Implement per [Pyramid Levels](#pyramid-levels)

### For Integrators
1. Review [Public APIs](#public-apis)
2. Understand [Lossiness Model](#lossiness-model)
3. Study [Provider Examples](#provider-examples)
4. Test with [CLI Commands](#cli-interface)

## 🔍 Quick Lookup

### Key Files
```
/schemas/prompt-spec.schema.json    # Uniform request schema
/schemas/provider-spec.schema.json  # Provider capability schema
/providers/openai/gpt-5.yaml       # OpenAI specification
/providers/anthropic/opus.yaml     # Anthropic specification
```

### Environment Variables
```bash
OPENAI_API_KEY      # Required for OpenAI
ANTHROPIC_API_KEY   # Required for Anthropic
```

### Core Types
```rust
StrictMode { Strict, Warn, Coerce }
Lossiness { code, path, message, before, after, severity }
LossinessReport { items: Vec<Lossiness> }
TranslationResult { provider_request_json, lossiness }
UniformResponse { model, content, finish_reason, tool_calls }
```

## 📈 Metrics Dashboard

### Development Progress
- **L1**: Contracts & Preview ⬜
- **L2**: Sync End-to-End ⬜
- **L3**: Streaming Lite ⬜
- **L4**: Streaming Normalized ⬜
- **L5**: Full Features ⬜

### Quality Gates
- Schema Validation ✓
- Translation Accuracy ✓
- Normalization Correctness ✓
- Streaming Reliability ✓
- Binding Parity ✓

---

*This index provides comprehensive navigation for the Specado PRD. Use the quick navigation sections to efficiently access specific information about the spec-driven LLM translation engine.*