# Interface Taxonomy

Specado uses neutral interface hints to route provider specs without hardcoding vendor names. Each hint follows the pattern `domain.action[.mode]`.

## Conversational / Text

| Hint | Description |
| --- | --- |
| `conversational.generate` | Multi-turn chat (tool use allowed) |
| `conversational.stream` | Streaming chat responses |
| `text.generate` | Single-turn completion |
| `text.extract` | Structured/JSON output |
| `text.moderate` | Safety/policy classification |

## Tooling

| Hint | Description |
| --- | --- |
| `tools.call` | Function/tool orchestration |

## Embeddings / Search

| Hint | Description |
| --- | --- |
| `embeddings.generate` | Generate vector embeddings |
| `search.rerank` | Rank documents for a query |

## Vision / Images / Video

| Hint | Description |
| --- | --- |
| `vision.describe` | Caption/OCR multimodal input |
| `image.generate` | Create images |
| `image.edit` | Edit an existing image |
| `image.variations` | Produce variations of an image |
| `video.generate` | Generate video content |
| `video.transcribe` | Transcribe video |

## Audio / Speech

| Hint | Description |
| --- | --- |
| `speech.synthesize` | Text to speech |
| `audio.transcribe` | Speech to text |
| `audio.translate` | Speech translation |

## Files & Batch (reserved)

| Hint | Description |
| --- | --- |
| `files.upload` | Upload provider files |
| `files.retrieve` | Retrieve provider files |
| `batch.submit` | Submit batch job |
| `batch.status` | Query batch job |

Providers may also use experimental hints starting with `x_`.

## Adapter Precedence

The adapter registry evaluates hints in this order:

1. Direct `interface` match.
2. Endpoint URL domain/path.
3. Provider name heuristics.
4. Default to chat completions.

## Overlays

Overlays live in `overlays/<provider>.<adapter>.yaml` and declare `overlay_for` metadata. Each overlay is applied after inheritance using precedence `spec < overlay < runtime overrides`.
