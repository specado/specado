# Specado Comprehensive GitHub Issue Plan

## Structure Overview

- **Milestones**: L1-L5 pyramid levels (already created)
- **Epics**: Major work areas within each level (regular issues with 'epic' label)
- **Tasks**: Detailed implementation work (regular issues with 'task' label)
- **Relationships**: Parent-child using GitHub's sub-issue feature

---

## L1 - Contracts & Preview (Milestone)

### Epic: Schema Infrastructure
**Description**: Define and implement JSON Schema specifications for PromptSpec and ProviderSpec

#### Child Issues:
1. **Design PromptSpec JSON Schema structure**
   - Define all fields: model_class, messages, tools, sampling, limits, media
   - Establish validation rules and required fields
   - Document field constraints and relationships
   
2. **Implement PromptSpec validation logic**
   - JSON Schema draft 2020-12 validation
   - Custom validation for field relationships
   - Error message formatting and clarity

3. **Design ProviderSpec JSON Schema structure**
   - Model definitions, endpoints, mappings
   - Capability declarations and constraints
   - Response normalization specifications

4. **Implement ProviderSpec validation logic**
   - Schema validation with version checking
   - Constraint validation (mutually exclusive, limits)
   - Mapping path validation

5. **Create schema loader with YAML/JSON support**
   - Parse both YAML and JSON formats
   - Handle file includes and references
   - Error handling for malformed files

6. **Add environment variable expansion**
   - Support ${ENV:VAR} syntax in specs
   - Secure handling of sensitive values
   - Validation of required environment variables

7. **Implement schema versioning and compatibility**
   - Semver version handling
   - Compatibility range checking
   - Migration support between versions

8. **Create schema documentation generator**
   - Auto-generate docs from schemas
   - Include examples and constraints
   - Markdown output format

### Epic: Translation Engine Core
**Description**: Build the core translation engine that converts uniform prompts to provider-specific requests

#### Child Issues:
1. **Implement translate() function interface**
   - Define function signature and return types
   - Error handling structure
   - StrictMode enum implementation

2. **Build JSONPath mapping engine**
   - JSONPath evaluation and mapping
   - Create missing parent nodes
   - Handle complex path expressions

3. **Implement pre-validation logic**
   - Range validation (temperature, top_p)
   - Mutual exclusivity checking
   - Capability mismatch detection

4. **Create field transformation system**
   - Relocate fields (e.g., system prompt)
   - Transform value formats
   - Handle type conversions

5. **Build lossiness tracking infrastructure**
   - Track all transformations
   - Categorize by lossiness codes
   - Severity assignment logic

6. **Implement strictness policy engine**
   - Strict mode (fail fast)
   - Warn mode (proceed with warnings)
   - Coerce mode (auto-adjust values)

7. **Add conflict resolution logic**
   - Handle mutually exclusive fields
   - Apply resolution preferences
   - Document resolution decisions

8. **Create TranslationResult builder**
   - Assemble provider request JSON
   - Compile lossiness report
   - Include metadata

### Epic: Provider Specifications
**Description**: Create comprehensive provider specification files for OpenAI and Anthropic

#### Child Issues:
1. **Create OpenAI GPT-5 base specification**
   - Define endpoints and protocols
   - Map all parameters and constraints
   - Response normalization rules

2. **Add OpenAI tool support specification**
   - Parallel tool calling configuration
   - Tool choice handling
   - Tool response mapping

3. **Create Anthropic Claude Opus 4.1 specification**
   - Messages API configuration
   - System prompt relocation rules
   - Stream event mapping

4. **Add Anthropic-specific constraints**
   - Temperature/top_p mutual exclusivity
   - Field size limits
   - Forbidden top-level fields

5. **Implement provider discovery logic**
   - Local provider directory search
   - User config directory support
   - Provider/model syntax parsing

6. **Create provider spec validation tests**
   - Validate against ProviderSpec schema
   - Test mapping paths
   - Verify constraint definitions

### Epic: CLI Foundation
**Description**: Implement core CLI commands for validation and preview

#### Child Issues:
1. **Create CLI argument parser**
   - Define command structure
   - Parse flags and options
   - Help text generation

2. **Implement validate command**
   - Schema validation
   - Spec file checking
   - Error reporting

3. **Implement preview command**
   - Translation preview without HTTP
   - Display lossiness report
   - Show provider request JSON

4. **Add CLI configuration management**
   - Config file support
   - Default values
   - Environment variable integration

5. **Create CLI output formatting**
   - JSON output mode
   - Human-readable format
   - Error formatting

6. **Implement debug logging**
   - Log levels and filtering
   - Redact sensitive information
   - Request ID tracking

### Epic: Testing Framework Foundation
**Description**: Establish testing infrastructure for L1 components

#### Child Issues:
1. **Create golden test infrastructure**
   - Snapshot testing setup
   - Corpus management
   - Comparison logic

2. **Add property-based testing**
   - Range validation tests
   - Invariant checking
   - Random input generation

3. **Implement unit test suite for schemas**
   - Schema validation tests
   - Edge case coverage
   - Error condition tests

4. **Create integration tests for translation**
   - End-to-end translation tests
   - Multiple provider tests
   - Lossiness validation

5. **Add fuzzing for JSONPath mapping**
   - Invalid path handling
   - Missing parent nodes
   - Malformed expressions

---

## L2 - Sync End-to-End (Milestone)

### Epic: HTTP Client Infrastructure
**Description**: Build robust HTTP client for provider communication

#### Child Issues:
1. **Implement HTTP request builder**
   - Method, path, headers construction
   - Query parameter handling
   - Body serialization

2. **Add authentication handling**
   - API key management
   - Header injection
   - Environment variable support

3. **Implement retry logic with exponential backoff**
   - Configurable retry attempts
   - Backoff calculation
   - Jitter implementation

4. **Add timeout configuration**
   - Connection timeouts
   - Read timeouts
   - Total request timeout

5. **Create rate limiting support**
   - Rate limit detection
   - Throttling logic
   - Queue management

6. **Implement TLS/HTTPS support**
   - Certificate validation
   - TLS version configuration
   - Security best practices

### Epic: Response Normalization
**Description**: Normalize provider responses to UniformResponse format

#### Child Issues:
1. **Define UniformResponse structure**
   - Standard fields definition
   - Optional fields handling
   - Metadata structure

2. **Implement sync response parser**
   - JSONPath extraction
   - Field mapping
   - Type conversion

3. **Add finish reason mapping**
   - Provider-specific mappings
   - Standard reason codes
   - Fallback handling

4. **Create tool call normalization**
   - Extract tool calls
   - Normalize arguments
   - Handle parallel calls

5. **Implement error response handling**
   - Error detection
   - Error normalization
   - Retry decision logic

6. **Add response validation**
   - Schema validation
   - Required field checking
   - Type validation

### Epic: CLI Run Command
**Description**: Implement the run command for synchronous execution

#### Child Issues:
1. **Implement run command handler**
   - Command parsing
   - Execution flow
   - Result handling

2. **Add output file support**
   - File writing
   - Format selection
   - Overwrite protection

3. **Create execution metrics**
   - Timing information
   - Token counting
   - Cost estimation

4. **Add progress indicators**
   - Execution status
   - Progress bars
   - Status messages

---

## L3 - Streaming Lite (Milestone)

### Epic: SSE/Stream Infrastructure
**Description**: Basic streaming support implementation

#### Child Issues:
1. **Implement SSE parser**
   - Event parsing
   - Data extraction
   - Connection handling

2. **Create StreamHandle abstraction**
   - Handle lifecycle
   - Event buffering
   - State management

3. **Add chunked transfer support**
   - Chunk parsing
   - Buffer management
   - Stream reassembly

4. **Implement connection management**
   - Keep-alive handling
   - Reconnection logic
   - Connection pooling

### Epic: Cancellation Support
**Description**: Handle stream cancellation and cleanup

#### Child Issues:
1. **Implement handle drop cancellation**
   - Drop trait implementation
   - Resource cleanup
   - Connection termination

2. **Add signal handling**
   - Ctrl+C support
   - Graceful shutdown
   - State preservation

3. **Create cancellation propagation**
   - Cancel tokens
   - Propagation chain
   - Cleanup coordination

### Epic: Raw Stream CLI
**Description**: CLI support for raw streaming

#### Child Issues:
1. **Implement stream command with --raw**
   - Raw event output
   - NDJSON formatting
   - Real-time display

2. **Add stream debugging support**
   - Event inspection
   - Connection diagnostics
   - Performance metrics

---

## L4 - Streaming Normalized (Milestone)

### Epic: Event Normalization
**Description**: Normalize stream events to standard format

#### Child Issues:
1. **Define normalized event types**
   - start, delta, tool, stop events
   - Event structure
   - Metadata fields

2. **Implement event routing logic**
   - Provider event mapping
   - Route selection
   - Event transformation

3. **Add content accumulation**
   - Partial content tracking
   - Buffer management
   - Content assembly

4. **Create tool event handling**
   - Tool call detection
   - Argument accumulation
   - Call completion

5. **Implement stop event generation**
   - Completion detection
   - Finish reason extraction
   - Final state assembly

### Epic: Stream State Management
**Description**: Manage streaming state and continuity

#### Child Issues:
1. **Create stream state tracker**
   - State machine implementation
   - Transition validation
   - State persistence

2. **Add partial content management**
   - Content buffering
   - Fragment assembly
   - Overflow handling

3. **Implement error recovery**
   - Partial failure handling
   - State recovery
   - Continuation support

### Epic: Provider-Specific Routing
**Description**: Handle provider-specific event formats

#### Child Issues:
1. **Implement OpenAI event routing**
   - Event type mapping
   - Field extraction
   - Special case handling

2. **Implement Anthropic event routing**
   - Message events
   - Content blocks
   - Tool use events

3. **Add fallback routing**
   - Unknown event handling
   - Generic routing
   - Error events

---

## L5 - Tools & Features (Milestone)

### Epic: Tool Support
**Description**: Comprehensive tool calling implementation

#### Child Issues:
1. **Implement native tool support**
   - Tool schema handling
   - Call generation
   - Response processing

2. **Add tool serialization**
   - Serial execution
   - Order preservation
   - State tracking

3. **Create tool emulation**
   - Prompt-based tools
   - Response parsing
   - Validation logic

4. **Implement tool_choice handling**
   - Auto selection
   - Required tools
   - Specific tool selection

5. **Add parallel tool support**
   - Parallel execution
   - Result aggregation
   - Error handling

### Epic: Structured Outputs
**Description**: JSON and structured output support

#### Child Issues:
1. **Implement native JSON mode**
   - OpenAI JSON mode
   - Schema enforcement
   - Validation

2. **Add tool-based JSON emulation**
   - Tool wrapper for JSON
   - Response extraction
   - Schema validation

3. **Create prompt-based JSON emulation**
   - Prompt engineering
   - Response parsing
   - Retry logic

4. **Implement schema size validation**
   - Size calculation
   - Limit checking
   - Warning generation

5. **Add structured output tests**
   - Schema compliance
   - Edge cases
   - Provider differences

### Epic: Feature Flags
**Description**: Provider-specific feature support

#### Child Issues:
1. **Implement reasoning flag**
   - OpenAI reasoning
   - Anthropic thinking
   - Mapping logic

2. **Add personality features**
   - Warmth parameters
   - Style control
   - Provider mapping

3. **Create end_conversation support**
   - Conversation ending
   - State cleanup
   - Provider differences

4. **Implement custom feature flags**
   - Extensible system
   - Provider-specific flags
   - Default handling

### Epic: Node.js Binding
**Description**: Complete Node.js integration via NAPI-RS

#### Child Issues:
1. **Setup NAPI-RS infrastructure**
   - Build configuration
   - TypeScript definitions
   - Package structure

2. **Implement translate binding**
   - Function wrapper
   - Type conversion
   - Error handling

3. **Add run binding**
   - Async execution
   - Promise support
   - Result conversion

4. **Create stream binding**
   - Async iterator
   - Event emission
   - Cancellation

5. **Add TypeScript types**
   - Type definitions
   - Documentation
   - Examples

6. **Create Node.js tests**
   - Unit tests
   - Integration tests
   - Parity validation

### Epic: Python Binding
**Description**: Complete Python integration via PyO3

#### Child Issues:
1. **Setup PyO3 infrastructure**
   - Build configuration
   - Module structure
   - Package setup

2. **Implement translate binding**
   - Function wrapper
   - Type conversion
   - Exception handling

3. **Add run binding**
   - Async support
   - Result conversion
   - Error mapping

4. **Create stream binding**
   - Generator protocol
   - Iteration support
   - Cancellation

5. **Add type hints**
   - Type stubs
   - Documentation
   - Examples

6. **Create Python tests**
   - pytest suite
   - Integration tests
   - Parity validation

### Epic: CLI DX Features
**Description**: Developer experience improvements

#### Child Issues:
1. **Implement matrix command**
   - Multi-provider comparison
   - Report generation
   - Markdown output

2. **Add diff command**
   - Response comparison
   - Structural diff
   - Semantic analysis

3. **Create provider discovery**
   - Auto-discovery
   - Path resolution
   - Listing command

4. **Add shell completions**
   - Bash completion
   - Zsh completion
   - Fish completion

5. **Implement config management**
   - Global config
   - Project config
   - Precedence rules

6. **Create interactive mode**
   - REPL interface
   - Command history
   - Tab completion

---

## Cross-Cutting Epics

### Epic: CI/CD Pipeline
**Description**: Build, test, and release automation

#### Child Issues:
1. **Setup GitHub Actions workflow**
2. **Add multi-platform builds**
3. **Create release automation**
4. **Implement dependency caching**
5. **Add security scanning**

### Epic: Documentation
**Description**: Comprehensive project documentation

#### Child Issues:
1. **Write CODES.md**
2. **Create API reference**
3. **Add usage examples**
4. **Write provider authoring guide**
5. **Create troubleshooting guide**

### Epic: Performance Optimization
**Description**: Performance testing and optimization

#### Child Issues:
1. **Add performance benchmarks**
2. **Implement profiling**
3. **Optimize hot paths**
4. **Add caching layers**
5. **Create performance tests**

---

## Summary Statistics

- **Total Epics**: 25
- **Total Issues**: ~150
- **L1**: 5 epics, 35 issues
- **L2**: 3 epics, 16 issues  
- **L3**: 3 epics, 11 issues
- **L4**: 3 epics, 11 issues
- **L5**: 8 epics, 41 issues
- **Cross-cutting**: 3 epics, 15 issues

Each issue will include:
- Comprehensive description
- Acceptance criteria
- Technical details
- Dependencies
- Testing requirements
- Effort estimates