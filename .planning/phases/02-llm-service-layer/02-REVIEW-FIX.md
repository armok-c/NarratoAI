---
phase: 02-llm-service-layer
fixed_at: '2026-04-29T06:50:00Z'
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T06:50:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: Deadlock when `analyze_images` receives `max_concurrency = Some(0)`

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 64c0576
**Applied fix:** Added `.max(1)` guard to the `bounded_concurrency` initialization. Without this guard, `Some(0).unwrap_or(1)` returns 0, creating a `Semaphore::new(0)` that causes every spawned task to wait forever in `acquire_owned().await`, hanging `analyze_images` permanently.

### WR-01: `max_retries` config parameter silently ignored

**Files modified:** `src/llm/openai_compatible.rs`, `Cargo.toml`
**Commit:** 33082fd
**Applied fix:** Wired `ProviderConfig.max_retries` into the async-openai `Client` by constructing a custom `backoff::ExponentialBackoff` with `max_elapsed_time` derived from the retry count. When `max_retries = 0`, sets `max_elapsed_time` to `Duration::ZERO` to disable retries entirely. For non-zero values, allocates a reasonable elapsed time budget. Replaced the stale comment acknowledging the gap with the actual wiring. Added `backoff = "0.4"` as a direct dependency.

### WR-02: `test_openai_error_mapping` is `#[ignore]`d, leaving error mapping uncovered

**Files modified:** `tests/llm_test.rs`
**Commit:** ca87242
**Applied fix:** Changed `create_test_provider` to use `max_retries: 0` (now wired by WR-01 to actually disable retries) and removed the `#[ignore]` attribute from the error mapping test. The HTTP error responses propagate immediately without hanging because async-openai's `ExponentialBackoff` will not retry when `max_elapsed_time` is `Duration::ZERO`.

### WR-03: `create_vision_chat_with_json_fallback` duplicates message construction logic

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 7fbd676
**Applied fix:** Extracted the duplicated vision message construction (text + image URL parts wrapping + system prompt handling) from both `analyze_images` main path and `create_vision_chat_with_json_fallback` fallback path into a single `build_vision_messages` associated function. The main path now calls `Self::build_vision_messages(&prompt_owned, system_prompt_owned.as_deref(), &batch)`, and the fallback path calls `Self::build_vision_messages(&json_prompt, system_prompt, batch)`. Any future change to the vision message format now needs to be applied in only one place.

### IN-01: Full JPEG decode contradicts "lightweight verification" comment

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 8f1c735
**Applied fix:** Updated the comment to accurately describe the tradeoff: the decode is a full decompression to RGBA (not lightweight), with memory proportional to image size. Replaced "轻量验证 JPEG 完整性" with an explanation of the actual behavior and memory cost.

### IN-02: TOCTOU window between file size check and file open

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 0d8d704
**Applied fix:** Changed the order of operations to open the file first (`std::fs::File::open`), then query metadata through the already-open file handle (`file.metadata()`) instead of calling `std::fs::metadata(path)` as a separate syscall. This eliminates the race window where a racer could replace the file with a larger one between the two operations.

### IN-03: Blocking file I/O inside async context without `spawn_blocking`

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** c25cbb7
**Applied fix:** Offloaded the synchronous `image_to_base64_data_url` calls (which perform file I/O, JPEG decode/encode, and base64 encoding) to `tokio::task::spawn_blocking`, collected with `futures::future::join_all`. This prevents tokio worker threads from being blocked by CPU-intensive image processing, avoiding starvation of other async tasks on the same runtime.

## Notes

- The pre-existing compilation error in `src/tts/edge_tts.rs:375` (tungstenite `Utf8Bytes` type mismatch) is unrelated to these fixes and continues to exist after all LLM service layer changes.
- Each fix was verified via `cargo check --lib` before committing.

---

_Fixed: 2026-04-29T06:50:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
