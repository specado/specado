# Specado GitHub Project Plan

## Epic Structure Based on Delivery Pyramid

### Epic 1: L1 - Contracts & Preview (Offline)
**Goal**: Establish schemas, validation, and preview capabilities
**Priority**: Critical
**Sprint**: Sprint 1-2
**Risk**: Low

#### Child Issues:

##### Issue 1.1: Define PromptSpec JSON Schema
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Complete JSON Schema draft 2020-12 for PromptSpec
  - [ ] Include all fields: model_class, messages, tools, sampling, limits, media
  - [ ] Add validation rules for required fields
  - [ ] Document in `/schemas/prompt-spec.schema.json`
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 1.2: Define ProviderSpec JSON Schema
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Complete JSON Schema for ProviderSpec
  - [ ] Include model definitions, endpoints, mappings, constraints
  - [ ] Add response normalization specifications
  - [ ] Document in `/schemas/provider-spec.schema.json`
- **Effort**: L
- **Time**: 6-8 hours

##### Issue 1.3: Implement Schema Loader and Validator
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Load and parse JSON/YAML specifications
  - [ ] Validate against JSON Schema draft 2020-12
  - [ ] Provide clear error messages for validation failures
  - [ ] Support environment variable expansion
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 1.4: Build Translation Engine Core
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Implement translate() function in Rust
  - [ ] Map uniform fields to provider format via JSONPath
  - [ ] Handle ranges, conflicts, and constraints
  - [ ] Generate TranslationResult with provider_request_json
- **Effort**: L
- **Time**: 8-10 hours

##### Issue 1.5: Implement Lossiness Reporting
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Define Lossiness struct with all codes
  - [ ] Track Clamp, Drop, Emulate, Conflict, Relocate operations
  - [ ] Generate comprehensive LossinessReport
  - [ ] Support Strict, Warn, Coerce modes
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 1.6: Create OpenAI Provider Spec
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Complete spec for GPT-5 model
  - [ ] Define endpoints, mappings, constraints
  - [ ] Support Responses API format
  - [ ] Document in `/providers/openai/gpt-5.yaml`
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 1.7: Create Anthropic Provider Spec
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Complete spec for Claude Opus 4.1
  - [ ] Handle system message relocation
  - [ ] Define mutual exclusivity rules
  - [ ] Document in `/providers/anthropic/claude-opus-4-1.yaml`
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 1.8: Implement CLI Preview Command
- **Type**: task
- **Acceptance Criteria**:
  - [ ] `specado preview` command implementation
  - [ ] Show translated request and lossiness report
  - [ ] Support --strict flag for mode selection
  - [ ] No network calls (offline operation)
- **Effort**: M
- **Time**: 3-4 hours

---

### Epic 2: L2 - Sync End-to-End
**Goal**: Execute requests and normalize responses synchronously
**Priority**: Critical
**Sprint**: Sprint 2-3
**Risk**: Medium

#### Child Issues:

##### Issue 2.1: Implement HTTP Client
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Support POST requests with headers
  - [ ] Handle authentication via environment variables
  - [ ] Implement retry logic with exponential backoff
  - [ ] Support TLS/HTTPS requirements
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 2.2: Build Response Normalization Engine
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Parse provider responses via JSONPath
  - [ ] Extract content, finish_reason, tool_calls
  - [ ] Map to UniformResponse structure
  - [ ] Handle error responses gracefully
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 2.3: Implement run() Function
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Execute HTTP request to provider
  - [ ] Normalize response to UniformResponse
  - [ ] Handle timeouts and network errors
  - [ ] Return structured errors
- **Effort**: M
- **Time**: 4-5 hours

##### Issue 2.4: Create CLI Run Command
- **Type**: task
- **Acceptance Criteria**:
  - [ ] `specado run` command implementation
  - [ ] Save normalized response to file
  - [ ] Display execution metrics
  - [ ] Support debug logging
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 2.5: Add Golden Tests for Sync
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Create test corpus of prompts
  - [ ] Snapshot provider requests
  - [ ] Validate normalized responses
  - [ ] Test both OpenAI and Anthropic
- **Effort**: M
- **Time**: 3-4 hours

---

### Epic 3: L3 - Streaming (Lite)
**Goal**: Basic streaming support with cancellation
**Priority**: High
**Sprint**: Sprint 3-4
**Risk**: Medium

#### Child Issues:

##### Issue 3.1: Implement SSE Parser
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Parse Server-Sent Events format
  - [ ] Handle data lines and event types
  - [ ] Support chunked transfer encoding
  - [ ] Buffer incomplete events
- **Effort**: M
- **Time**: 4-5 hours

##### Issue 3.2: Build Stream Handle System
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Create StreamHandle struct
  - [ ] Implement next_event() function
  - [ ] Support cancellation via handle drop
  - [ ] Handle connection timeouts
- **Effort**: M
- **Time**: 4-6 hours

##### Issue 3.3: Add Raw Stream Passthrough
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Pass raw provider events through
  - [ ] Maintain event ordering
  - [ ] Support both OpenAI and Anthropic formats
  - [ ] Handle connection drops
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 3.4: Implement CLI Stream Command
- **Type**: task
- **Acceptance Criteria**:
  - [ ] `specado stream --raw` command
  - [ ] Output NDJSON format
  - [ ] Support Ctrl+C cancellation
  - [ ] Display stream metrics
- **Effort**: S
- **Time**: 2-3 hours

---

### Epic 4: L4 - Streaming (Normalized)
**Goal**: Normalized stream events with proper routing
**Priority**: High
**Sprint**: Sprint 4-5
**Risk**: High

#### Child Issues:

##### Issue 4.1: Design Normalized Event Types
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Define start, delta, tool, stop events
  - [ ] Create uniform event structure
  - [ ] Document event ordering rules
  - [ ] Support partial content accumulation
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 4.2: Implement Stream Normalization
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Route provider events to normalized types
  - [ ] Use spec-driven event_selector
  - [ ] Handle out-of-order events
  - [ ] Maintain content continuity
- **Effort**: L
- **Time**: 6-8 hours

##### Issue 4.3: Add Stream State Management
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Track partial content accumulation
  - [ ] Handle tool call assembly
  - [ ] Manage finish reason detection
  - [ ] Support graceful stream termination
- **Effort**: M
- **Time**: 4-5 hours

##### Issue 4.4: Create Stream Golden Tests
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Test event sequences for both providers
  - [ ] Validate event ordering
  - [ ] Test cancellation scenarios
  - [ ] Fuzz test with malformed events
- **Effort**: M
- **Time**: 3-4 hours

---

### Epic 5: L5 - Tools, Structured Outputs & DX
**Goal**: Complete feature set with tools, JSON outputs, and bindings
**Priority**: High
**Sprint**: Sprint 5-6
**Risk**: High

#### Child Issues:

##### Issue 5.1: Implement Tool Support
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Native tool support where available
  - [ ] Tool serialization for providers without parallel support
  - [ ] Tool choice handling (auto, required, specific)
  - [ ] Tool call normalization in responses
- **Effort**: L
- **Time**: 6-8 hours

##### Issue 5.2: Add Structured Output Support
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Native JSON mode for OpenAI
  - [ ] Tool-based emulation for Anthropic
  - [ ] JSON Schema validation
  - [ ] Size constraint checking
- **Effort**: L
- **Time**: 6-8 hours

##### Issue 5.3: Implement Feature Flags
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Support reasoning, personality flags
  - [ ] Handle end_conversation_supported
  - [ ] Map to provider-specific parameters
  - [ ] Document in lossiness report
- **Effort**: M
- **Time**: 3-4 hours

##### Issue 5.4: Build Node.js Binding
- **Type**: task
- **Acceptance Criteria**:
  - [ ] NAPI-RS based binding
  - [ ] Async/await support
  - [ ] Stream as async iterable
  - [ ] JSON in/out interface
- **Effort**: L
- **Time**: 8-10 hours

##### Issue 5.5: Build Python Binding
- **Type**: task
- **Acceptance Criteria**:
  - [ ] PyO3 based binding
  - [ ] Generator for streaming
  - [ ] Type hints support
  - [ ] JSON serialization
- **Effort**: L
- **Time**: 8-10 hours

##### Issue 5.6: Create CLI Matrix Command
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Compare multiple providers/models
  - [ ] Generate comparison report
  - [ ] Show lossiness differences
  - [ ] Output as markdown
- **Effort**: M
- **Time**: 4-5 hours

##### Issue 5.7: Create CLI Diff Command
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Compare normalized responses
  - [ ] Structural diff analysis
  - [ ] Basic semantic comparison
  - [ ] Markdown output format
- **Effort**: M
- **Time**: 3-4 hours

##### Issue 5.8: Add Provider Discovery
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Search local provider directory
  - [ ] Check ~/.config/specado/providers
  - [ ] Support provider/model syntax
  - [ ] Clear error for missing specs
- **Effort**: S
- **Time**: 2-3 hours

##### Issue 5.9: Create Binding Parity Tests
- **Type**: task
- **Acceptance Criteria**:
  - [ ] Shared test corpus for all bindings
  - [ ] Validate identical behavior
  - [ ] Test error handling
  - [ ] Performance benchmarks
- **Effort**: M
- **Time**: 4-5 hours

##### Issue 5.10: Write Documentation
- **Type**: documentation
- **Acceptance Criteria**:
  - [ ] Complete CODES.md with all lossiness codes
  - [ ] API reference documentation
  - [ ] Provider spec authoring guide
  - [ ] Integration examples
- **Effort**: M
- **Time**: 4-6 hours

---

## GitHub Project Board Structure

### Board: Specado v1.0 Development

#### Columns:
1. **Backlog** - All issues start here
2. **Sprint Planning** - Issues selected for next sprint
3. **In Progress** - Active development
4. **In Review** - Code review/testing
5. **Done** - Completed and merged

#### Custom Fields:
- **Sprint**: Sprint 1, Sprint 2, Sprint 3, Sprint 4, Sprint 5, Sprint 6
- **Epic**: L1-Contracts, L2-Sync, L3-Streaming-Lite, L4-Streaming-Normalized, L5-Features
- **Effort**: XS, S, M, L, XL
- **Risk**: Low, Medium, High
- **Type**: task, bug, documentation, infrastructure

## Labels to Create:
- `epic` - Large feature or initiative
- `task` - Standard work item  
- `documentation` - Documentation only
- `infrastructure` - Build, CI/CD, tooling
- `priority:critical` - Must have for v1.0
- `priority:high` - Should have for v1.0
- `priority:medium` - Nice to have
- `blocked` - Cannot proceed
- `in-progress` - Active work
- `ready-for-review` - Awaiting review

## Milestones:
- **v1.0-preview** - L1 completion
- **v1.0-sync** - L2 completion
- **v1.0-streaming** - L3-L4 completion
- **v1.0-release** - L5 completion and release

## Sprint Planning:
- **Sprint 1** (Week 1-2): Schema definitions, validation, preview
- **Sprint 2** (Week 3-4): Translation engine, provider specs
- **Sprint 3** (Week 5-6): HTTP client, sync normalization
- **Sprint 4** (Week 7-8): Raw streaming support
- **Sprint 5** (Week 9-10): Normalized streaming
- **Sprint 6** (Week 11-12): Tools, structured outputs, bindings