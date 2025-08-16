# PromptSpec

Uniform prompt specification for LLM providers

---

## Table of Contents

- [Overview](#overview)
- [Properties](#properties)
- [Complete Example](#complete-example)
- [Validation Rules](#validation-rules)

## Overview

**Schema Version:** `https://json-schema.org/draft/2020-12/schema`

**ID:** `https://specado.com/schemas/prompt-spec.schema.json`

**Root Type:** `object`

## Properties

### `messages`

**Type:** `array<reference>` **[Required]**

Conversation messages

**Constraints:**

- Required
- Min items: 1

### `model_class`

**Type:** `string` **[Required]**

Model capability class determining behavior

**Allowed Values:**

- `"Chat"`
- `"ReasoningChat"`
- `"VisionChat"`
- `"AudioChat"`
- `"MultimodalChat"`
- `"RAGChat"`
- `"Completion"`
- `"Embedding"`

**Constraints:**

- Required

### `strict_mode`

**Type:** `string` **[Required]**

Strictness policy for translation

**Default:** `"Warn"`

**Allowed Values:**

- `"Strict"`
- `"Warn"`
- `"Coerce"`

**Constraints:**

- Required

### `conversation`

**Type:** `object`

Conversation management settings

##### Properties

##### `branch_from`

**Type:** `string`

Message ID to branch conversation from

##### `conversation_id`

**Type:** `string`

Persistent conversation identifier

##### `parent_message_id`

**Type:** `string`

Parent message ID for conversation branching

##### `preserve_history`

**Type:** `boolean`

Whether to preserve conversation history

**Default:** `true`

### `limits`

**Type:** `object`

Token and output limits

##### Properties

##### `max_output_tokens`

**Type:** `integer`

Maximum tokens in response

**Constraints:**

- Min: 1

##### `max_prompt_tokens`

**Type:** `integer`

Maximum tokens in prompt

**Constraints:**

- Min: 1

##### `reasoning_tokens`

**Type:** `integer`

Tokens for reasoning (ReasoningChat models)

**Constraints:**

- Min: 1

### `media`

**Type:** `object`

Media inputs and outputs

##### Properties

##### `input_audio`

**Type:** `object`

Audio input configuration

####### Properties

###### `data`

**Type:** `string`

Base64 encoded audio data

###### `format`

**Type:** `string`

Audio format (e.g., mp3, wav, m4a)

###### `url`

**Type:** `string`

Audio file URL

##### `input_documents`

**Type:** `array<object>`

Documents to be included directly in the prompt's context (not for retrieval)

##### `input_images`

**Type:** `array<oneOf>`

Images to include in prompt

##### `input_video`

**Type:** `object`

Video input configuration

####### Properties

###### `data`

**Type:** `string`

Base64 encoded video data

###### `duration_seconds`

**Type:** `number`

Video duration in seconds

**Constraints:**

- Min: 0

###### `format`

**Type:** `string`

Video format (e.g., mp4, webm, mov)

###### `url`

**Type:** `string`

Video file URL

##### `output_audio`

**Type:** `object`

Audio output configuration

### `preferences`

**Type:** `object`

User preferences for translation behavior

##### Properties

##### `citation_format`

**Type:** `string`

Preferred citation format for RAG responses

**Default:** `"inline"`

**Allowed Values:**

- `"inline"`
- `"footnote"`
- `"bibliography"`

##### `fallback_behavior`

**Type:** `string`

How to handle unsupported features

**Default:** `"adaptive"`

**Allowed Values:**

- `"strict"`
- `"adaptive"`
- `"permissive"`

##### `parallel_tool_calls`

**Type:** `boolean`

Preference for parallel tool execution

**Default:** `true`

##### `prompt_truncation`

**Type:** `string`

How to handle prompts that exceed token limits

**Default:** `"AUTO"`

**Allowed Values:**

- `"OFF"`
- `"AUTO"`
- `"AUTO_PRESERVE_ORDER"`

### `rag`

**Type:** `object`

Retrieval-augmented generation configuration

##### Properties

##### `citations_required`

**Type:** `boolean`

Whether to include citations in response

**Default:** `false`

##### `connectors`

**Type:** `array<string>`

RAG connectors to use

##### `documents`

**Type:** `array<object>`

A corpus of documents for the model to retrieve from during RAG operations

##### `max_results`

**Type:** `integer`

Maximum number of results to retrieve

**Constraints:**

- Min: 1

##### `search_queries`

**Type:** `array<string>`

Search queries for retrieval

### `response_format`

**Type:** `oneOf`

Expected response format

### `sampling`

**Type:** `object`

Sampling parameters for generation

##### Properties

##### `frequency_penalty`

**Type:** `number`

Penalize frequently used tokens

**Constraints:**

- Min: -2
- Max: 2

##### `presence_penalty`

**Type:** `number`

Penalize tokens that have appeared

**Constraints:**

- Min: -2
- Max: 2

##### `temperature`

**Type:** `number`

Controls randomness (0=deterministic, 2=very random)

**Constraints:**

- Min: 0
- Max: 2

##### `top_k`

**Type:** `integer`

Top-k sampling parameter

**Constraints:**

- Min: 1

##### `top_p`

**Type:** `number`

Nucleus sampling threshold

**Constraints:**

- Min: 0
- Max: 1

### `tool_choice`

**Type:** `oneOf`

Tool selection strategy

### `tools`

**Type:** `array<reference>`

Available tools for the model to use

## Complete Example

```json
{
  "conversation": {
    "branch_from": "example_branch_from",
    "parent_message_id": "example_parent_message_id",
    "preserve_history": true
  },
  "messages": [
    null
  ],
  "model_class": "Chat",
  "preferences": {
    "citation_format": "inline",
    "fallback_behavior": "strict",
    "parallel_tool_calls": true
  },
  "strict_mode": "Strict",
  "tools": [
    null
  ]
}
```

## Validation Rules

- **PromptSpec.preferences.citation_format**: Value must be one of the allowed values
- **PromptSpec.preferences.fallback_behavior**: Value must be one of the allowed values
- **PromptSpec.preferences.prompt_truncation**: Value must be one of the allowed values
- **PromptSpec.model_class**: Value must be one of the allowed values
- **PromptSpec.sampling.presence_penalty**: Value must be between -2 and 2
- **PromptSpec.sampling.top_k**: Minimum value: 1
- **PromptSpec.sampling.temperature**: Value must be between 0 and 2
- **PromptSpec.sampling.top_p**: Value must be between 0 and 1
- **PromptSpec.sampling.frequency_penalty**: Value must be between -2 and 2
- **PromptSpec.media.input_video.duration_seconds**: Minimum value: 0
- **PromptSpec.rag.max_results**: Minimum value: 1
- **PromptSpec.strict_mode**: Value must be one of the allowed values
- **PromptSpec.limits.max_prompt_tokens**: Minimum value: 1
- **PromptSpec.limits.reasoning_tokens**: Minimum value: 1
- **PromptSpec.limits.max_output_tokens**: Minimum value: 1
- **PromptSpec.messages**: Minimum 1 items required


---

*Generated by Specado Schema Documentation Generator*
