---
phase: 02-llm-service-layer
plan: 01
subsystem: llm
tags: [rust, async-openai, llm, provider-trait, error-types]

requires:
  - phase: 01-foundation
    provides: Cargo.toml project structure, src/error.rs Error enum pattern (ConfigError/FFmpegError/TTSError with thiserror + Chinese messages), src/lib.rs module declarations

provides:
  - LLMError enum (9 variants) with Chinese error messages and From<OpenAIError> conversion
  - LlmResponseFormat enum (Text/Json) and VisionBatchConfig struct
  - LlmProvider trait with generate_text, generate_text_stream, analyze_images methods

affects: [02-llm-service-layer plans 02 and 03]

tech-stack:
  added: [async-openai, image, base64, futures, reqwest, wiremock]
  patterns:
    - "Error enum with #[derive(Error, Debug)] + Chinese #[error] messages (aligned with existing Phase 1 patterns)"
    - "Trait definition with #[async_trait], Send + Sync bounds for Arc<dyn LlmProvider> dynamic dispatch"
    - "Stream return type via Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>"

key-files:
  created:
    - narratoai-core/src/llm/mod.rs - llm module root
    - narratoai-core/src/llm/types.rs - LlmResponseFormat and VisionBatchConfig type definitions
    - narratoai-core/src/llm/provider.rs - LlmProvider trait definition
  modified:
    - Cargo.toml - added 6 [dependencies] + 1 [dev-dependencies]
    - narratoai-core/src/error.rs - added LLMError enum (9 variants) + From impl + 9 tests
    - narratoai-core/src/lib.rs - added pub mod llm

key-decisions:
  - "VisionBatchConfig fields (batch_size/max_concurrency) used as individual Option params in trait instead of struct — keeps trait signature flexible for implementors"

patterns-established:
  - "Error module: each domain error enum in src/error.rs with Chinese Display messages + From impls + test cases"
  - "Module structure: src/llm/mod.rs re-exports types and provider submodules"

requirements-completed: [LLM-07]

duration: 15min
completed: 2026-04-28
---

# Phase 02 Plan 01: LLM Service Core Infrastructure Summary

**LLMError enum (9 variants), LlmResponseFormat/VisionBatchConfig types, and LlmProvider trait with async text generation, streaming, and image analysis methods**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-28T11:10:00Z
- **Completed:** 2026-04-28T11:25:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added 6 production dependencies (async-openai, async-trait, image, base64, futures, reqwest) and 1 dev dependency (wiremock) to Cargo.toml
- Added `LLMError` enum with 9 Chinese-message variants (General, ProviderNotFound, Configuration, APICall, Validation, ModelNotSupported, RateLimit, Authentication, ContentFilter) plus `From<OpenAIError>` impl
- Added 9 test cases verifying Chinese error messages for each LLMError variant
- Created `LlmResponseFormat` (Text/Json) and `VisionBatchConfig` (batch_size=10, max_concurrency=1) types
- Created `LlmProvider` trait with `generate_text`, `generate_text_stream`, `analyze_images` methods — all async, Send+Sync, with proper Stream return types
- Registered `pub mod llm` in src/lib.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Cargo.toml dependencies and LLMError enum** - `0eb6353` (feat)
2. **Task 2: Create src/llm/types.rs and src/llm/provider.rs** - `f2310cf` (feat)

**Plan metadata:** Pending (docs: complete 02-01 plan)

## Files Created/Modified

- `Cargo.toml` - Added async-openai, image, base64, futures, reqwest to [dependencies]; wiremock to [dev-dependencies]
- `src/error.rs` - Added LLMError enum (9 variants), From<OpenAIError> impl, 9 test functions
- `src/lib.rs` - Added `pub mod llm` declaration
- `src/llm/mod.rs` - Module root re-exporting types and provider submodules
- `src/llm/types.rs` - LlmResponseFormat enum and VisionBatchConfig struct
- `src/llm/provider.rs` - LlmProvider trait with 3 methods

## Decisions Made

- Used individual `Option<usize>` params for `batch_size`/`max_concurrency` in `analyze_images` instead of passing `VisionBatchConfig` as a whole — keeps the trait interface more flexible for callers who may not need the full config object
- Kept `types.rs` dependency-free from async-openai types to avoid upstream API breakage
- Used `#[async_trait::async_trait]` (full path) instead of `use async_trait::async_trait` import for clarity

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing src/llm/mod.rs module declaration**
- **Found during:** Task 2 (cargo check)
- **Issue:** Rust requires a `mod.rs` inside the `src/llm/` directory for the module system to recognize it; plan only specified creating `types.rs` and `provider.rs`
- **Fix:** Created `src/llm/mod.rs` with `pub mod types; pub mod provider;`
- **Files modified:** src/llm/mod.rs
- **Verification:** cargo check passes
- **Committed in:** f2310cf (Task 2 commit)

**2. [Rule 2 - Cleanup] Removed unused import VisionBatchConfig from provider.rs**
- **Found during:** Task 2 (cargo check warning)
- **Issue:** Plan included `use crate::llm::types::VisionBatchConfig` in provider.rs but the trait signature uses individual `Option<usize>` params, making the import unused
- **Fix:** Removed the unused import
- **Files modified:** src/llm/provider.rs
- **Verification:** cargo check passes with 0 warnings
- **Committed in:** f2310cf (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 cleanup)
**Impact on plan:** Both auto-fixes necessary for correct Rust module compilation. No scope creep.

## Issues Encountered

- **Pre-existing test failure:** `ffmpeg::hwaccel::tests::test_detect_encoders_format` fails because FFmpeg is not installed on this system. This is not related to this plan's changes and is a pre-existing issue.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- LLM service core infrastructure is complete: error types, data types, and trait contract are defined
- Ready for Plan 02 (OpenAI-compatible Provider implementation) and Plan 03 (Provider Registry + Service Manager)
- Next plans can implement `LlmProvider` trait for specific LLM providers

---
*Phase: 02-llm-service-layer*
*Completed: 2026-04-28*
