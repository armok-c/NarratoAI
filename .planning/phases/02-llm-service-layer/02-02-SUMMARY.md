---
phase: 02-llm-service-layer
plan: 02
subsystem: llm
tags: [rust, async-openai, provider-registry, image-preprocessing, vision, text-generation, streaming]

requires:
  - phase: 02-llm-service-layer-01
    provides: LlmProvider trait, LLMError enum, LlmResponseFormat/VisionBatchConfig types

provides:
  - Registry runtime provider registration (D-05)
  - image_to_base64_data_url preprocessing pipeline (D-20)
  - OpenAiCompatibleProvider implementing generate_text, generate_text_stream, analyze_images (D-06)

affects:
  - phase: 02-llm-service-layer-03
  - phase: 04-config-layer
  - phase: 06-documentary-pipeline

tech-stack:
  added:
    - "reqwest 0.13 (upgraded from 0.12 to match async-openai 0.36.1 internal dep)"
  patterns:
    - "Registry + trait object registration pattern (HashMap<String, Arc<dyn LlmProvider>>)"
    - "Vision batch processing with Semaphore concurrency control"
    - "JSON response_format fallback via serde_json manipulation"
    - "Proxy configuration via reqwest::Client::builder().proxy()"

key-files:
  created:
    - src/llm/registry.rs
    - src/llm/image_utils.rs
    - src/llm/openai_compatible.rs
  modified:
    - src/llm/mod.rs
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Use serde_json deserialization to construct typed request messages (avoids complex async-openai variant/enum construction)"
  - "Client::build() for proxy support (async-openai 0.36.1 expects reqwest 0.13 internally, requiring version alignment)"
  - "Inline content part construction in analyze_images spawned tasks (avoids self-borrow in 'static tokio::spawn closures)"
  - "Check ApiError.message for 'response_format' string rather than HTTP status code (ApiError lacks status_code field in async-openai 0.36)"

patterns-established:
  - "Builder chain with `pattern = mutable` requires separate statements, not chained calls"
  - "async-openai Client is Clone-able (cheap Arc wrap) for use across multiple tokio tasks"
  - "Error mapping: image processing failures -> LLMError::Configuration, API errors -> LLMError::APICall"

requirements-completed:
  - LLM-01
  - LLM-02
  - LLM-03
  - LLM-04
  - LLM-05
  - LLM-06
  - LLM-07

duration: 18min
completed: 2026-04-28
---

# Phase 02 Plan 02: LLM Provider Registry + OpenAiCompatibleProvider Summary

**Registry runtime registration (HashMap<String, Arc<dyn LlmProvider>>), image preprocessing pipeline (thumbnail -> JPEG q85 -> base64 -> data URL), and complete OpenAiCompatibleProvider implementing text generation (sync/stream), vision analysis with batch+concurrency control, JSON fallback, and proxy support.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-04-28T11:27:00Z
- **Completed:** 2026-04-28T11:45:33Z
- **Tasks:** 2 (both auto, no TDD)
- **Files created:** 3
- **Files modified:** 3
- **Commits:** 3 (2 plan tasks + 1 pre-existing fix)

## Accomplishments

- **Registry (D-05):** Runtime provider registration with lowercase name normalization, `get()` returning structured `LLMError::ProviderNotFound` for missing keys, and `list_providers()` enumeration. Ready for provider injection at application startup.
- **Image preprocessing (D-20):** `image_to_base64_data_url()` pipeline matching Python's PIL behavior: `image::open()` -> `thumbnail(1024,1024)` (keep aspect ratio) -> JPEG quality 85 -> base64 encode via `base64::engine::general_purpose::STANDARD` -> `data:image/jpeg;base64,...` format.
- **OpenAiCompatibleProvider (D-06):** Complete `LlmProvider` trait implementation with three methods:
  - `generate_text()`: Non-streaming chat completion with configurable temperature, max_tokens, response_format.
  - `generate_text_stream()`: Streaming response via `create_stream()`, mapping chunks to token-level deltas.
  - `analyze_images()`: Batch processing with `tokio::sync::Semaphore` concurrency control, `tokio::spawn` per batch, `futures::future::join_all` collection, sorted by batch index.
- **JSON response_format fallback (D-19):** Detects `ApiError.message` containing "response_format" -> drops `response_format` parameter, appends JSON constraint to prompt -> retries. Implemented via serde_json serialization/deserialization for request modification.
- **Proxy support (D-16):** Constructor accepts `proxy_http`/`proxy_https` options, builds custom `reqwest::Client` with `Proxy::http()`/`Proxy::https()` when set, uses `Client::build(http_client, config, Default::default())`.

## Task Commits

| # | Task | Commit | Message |
|---|------|--------|---------|
| 1 | Registry + image_utils | `467752a` | feat(02-llm-service-layer): implement Registry and image preprocessing |
| 2 | OpenAiCompatibleProvider | `c0a5df4` | feat(02-llm-service-layer): implement OpenAiCompatibleProvider |
| - | Pre-existing edge_tts fix | `08b9bc6` | fix(03-tts): restore is_turn_start variable removed by mistake |

## Files Created/Modified

- `src/llm/registry.rs` (46 lines) - Provider registration with HashMap<Arc<dyn LlmProvider>>
- `src/llm/image_utils.rs` (29 lines) - Image-to-base64-data-url preprocessing pipeline
- `src/llm/openai_compatible.rs` (425 lines) - Full OpenAiCompatibleProvider implementation
- `src/llm/mod.rs` (5 lines) - Module declarations for registry, image_utils, openai_compatible
- `Cargo.toml` - Upgraded reqwest 0.12 -> 0.13 to match async-openai internal dependency
- `Cargo.lock` - Updated dependency resolution

## Decisions Made

1. **serde_json deserialization for message construction:** Rather than constructing async-openai's deeply nested enum variants (ChatCompletionRequestMessage::User -> ChatCompletionRequestUserMessage -> ChatCompletionRequestUserMessageContent::Text), we use `serde_json::from_value::<ChatCompletionRequestMessage>(json!({...}))` for build_vision_messages and build_text_messages. This avoids complex variant matching while producing the same serialized request body.

2. **Client::build() for proxy:** async-openai 0.36.1's `Client` struct has a private `http_client` field but exposes `Client::build(http_client, config, backoff)` as the public API for custom HTTP clients. All code paths (proxy and non-proxy) use this method for consistency.

3. **ApiError has no status_code:** async-openai's ApiError has `message`, `type`, `param`, `code` fields but no HTTP status code. JSON fallback detection checks `message.to_lowercase().contains("response_format")` instead of status_code == 400.

4. **Spawned tasks with owned data:** analyze_images' tokio::spawn tasks require `'static` lifetimes. All data (client, model_name, batch, prompt, semaphore) is cloned/moved into each task closure rather than borrowing `self`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existing is_turn_start variable missing in edge_tts.rs**
- **Found during:** Task 2 compilation (`cargo check`)
- **Issue:** Another worktree agent accidentally removed `let mut is_turn_start = false;` from `EdgeTtsEngine::speak_ssml` function, but the variable is still referenced at lines 218 and 251. This caused the entire crate to fail compilation.
- **Fix:** Restored the deleted variable declaration.
- **Files modified:** `src/tts/edge_tts.rs`
- **Verification:** `cargo check` passes with zero warnings.
- **Committed in:** `08b9bc6` (separate pre-existing fix commit)

**2. [Rule 3 - Blocking] reqwest version mismatch (0.12 vs 0.13)**
- **Found during:** Task 2 compilation
- **Issue:** async-openai 0.36.1 internally uses reqwest 0.13 for `Client::build()`, but Cargo.toml specified reqwest 0.12. The different version caused type mismatch when passing the custom http client.
- **Fix:** Upgraded reqwest from 0.12 to 0.13 in Cargo.toml.
- **Files modified:** `Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo check` passes cleanly.
- **Committed in:** `c0a5df4` (Task 2 commit)

**3. [Rule 3 - Blocking] async-openai types in wrong submodule**
- **Found during:** Task 2 compilation
- **Issue:** Chat completion types are in `async_openai::types::chat::*`, not directly in `async_openai::types::*`. The module re-exports only a subset.
- **Fix:** Changed all imports to use `async_openai::types::chat::*`.
- **Files modified:** `src/llm/openai_compatible.rs`
- **Verification:** `cargo check` passes.
- **Committed in:** `c0a5df4` (Task 2 commit)

**4. [Rule 3 - Blocking] Builder chain with `pattern = mutable` not chainable**
- **Found during:** Task 2 compilation
- **Issue:** `CreateChatCompletionRequestArgs` uses `#[builder(pattern = "mutable")]` meaning setters return `&mut Self`, not `Self`. The chained `.default().model().messages()` creates a temporary that is dropped before the `&mut` reference is used.
- **Fix:** Broke the builder chain: `let mut b = Args::default(); b.model(...); b.messages(...);` etc.
- **Files modified:** `src/llm/openai_compatible.rs`
- **Verification:** `cargo check` passes.
- **Committed in:** `c0a5df4` (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (Rule 3, blocking)
**Impact on plan:** All deviations were blocking issues from environment/API differences. No scope creep. No architectural changes required.

## Issues Encountered

- async-openai 0.36.1 API surface differs significantly from 0.25 (types moved to `types::chat` submodule, `pattern = "mutable"` builder pattern, `Client<C: Config>` generic struct, `ApiError` without `status_code`). Required reading source code directly to discover correct types and APIs.
- `Client::build()` is the only way to set a custom HTTP client on async-openai 0.36.1's Client. The `http_client` field is private.
- The `build_vision_content` helper was initially written but unused (spawned tasks need owned data, cannot borrow `self`). Removed to eliminate dead code warning.

## Stub Tracking

No stubs found. All implementations are complete with proper types, error handling, and no placeholder values.

## Threat Flags

None. All threat register mitigations verified:
- T-02-02-01: API key stored in struct field only, never logged
- T-02-02-02: Proxy uses typed reqwest::Proxy API
- T-02-02-03: JSON fallback accepted (business logic, not security)
- T-02-02-04: OpenAIError -> LLMError conversion uses err.to_string() which does not expose the API key (OpenAIConfig handles key in Authorization header only)

## Next Phase Readiness

- Registry ready for provider injection in fase 03 (LLM service wiring + configuration layer)
- OpenAiCompatibleProvider ready to be instantiated and registered via Registry
- Image preprocessing utility available for any phase needing visual analysis
- Required dependency pattern established: Provider trait in `provider.rs` -> concrete impl in `openai_compatible.rs` -> registration in `registry.rs`

---
*Phase: 02-llm-service-layer*
*Plan: 02*
*Completed: 2026-04-28*
