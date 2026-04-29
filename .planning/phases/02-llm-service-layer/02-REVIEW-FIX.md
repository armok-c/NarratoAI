---
phase: 02-llm-service-layer
fixed_at: 2026-04-29T13:30:00Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 1
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T13:30:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 9
- Fixed: 9
- Skipped: 0

## Fixed Issues

### CR-01: `test_analyze_images_result_ordering` deterministically fails due to wiremock mock registration collision

**Files modified:** `tests/llm_test.rs`
**Commit:** 370df96
**Applied fix:** Added `.up_to_n_times(1)` to each mock builder chain in the `for i in 0..3` loop to enforce each mock stops accepting matches after one request, preventing mock 0 from consuming all 3 requests.

### WR-01: Integer division by zero panic in `image_to_base64_data_url` when image dimensions are 0

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 68bfce0
**Applied fix:** Added guard condition `if w == 0 || h == 0` before the resize computation, returning an `LLMError::General` early instead of panicking on division by zero.

### WR-02: `analyze_images` cannot receive system-level instructions

**Files modified:** `src/llm/provider.rs`, `src/llm/openai_compatible.rs`, `tests/llm_test.rs`
**Commit:** 8f6e848
**Applied fix:** Added `system_prompt: Option<&str>` parameter to the `analyze_images` trait method in `LlmProvider`, used it to push a `System` message before the `User` message in the vision request construction. Updated both test call sites to pass `None` for the new parameter.

### WR-03: No file size validation in image preprocessing (potential OOM)

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 5b1b913
**Applied fix:** Added `std::fs::metadata()` check before `image::open()` with a 50 MB limit. Files exceeding the limit return `LLMError::General` before any decoding occurs.

### IN-01: `Registry::list_providers` returns keys in arbitrary order

**Files modified:** `src/llm/registry.rs`
**Commit:** 52d678c
**Applied fix:** Changed `list_providers` to collect keys into a `Vec<String>`, sort them with `.sort()`, then return. Callers now get deterministic ordering.

### IN-02: Missing `#[must_use]` on `Registry::get`

**Files modified:** `src/llm/registry.rs`
**Commit:** f52725d
**Applied fix:** Added `#[must_use]` attribute on `Registry::get` so that discarding the `Result` value triggers a compiler warning.

### IN-03: `OpenAiCompatibleProvider::new` has 7 constructor parameters

**Files modified:** `src/llm/openai_compatible.rs`, `src/llm/register.rs`, `tests/llm_test.rs`
**Commit:** 1f6ded8
**Applied fix:** Introduced a `ProviderConfig` struct with named fields (`api_key`, `model_name`, `base_url`, `max_retries`, `timeout_secs`, `proxy_http`, `proxy_https`). Changed `OpenAiCompatibleProvider::new` to accept `config: ProviderConfig` instead of 7 positional parameters. Updated all call sites in `register.rs` and `tests/llm_test.rs`.

### IN-04: `image_to_base64_data_url` always re-encodes as JPEG regardless of input format

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 81f0413
**Applied fix:** Added JPEG passthrough optimization: read raw file bytes first, check for JPEG magic bytes (`0xFF, 0xD8, 0xFF`), and if matched, directly base64-encode the raw bytes without decode-re-encode. Non-JPEG inputs continue through the existing decode/resize/encode pipeline.

### IN-05: `generate_text_with_json_fallback` rebuilds messages from scratch

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 4054d95
**Applied fix:** Added documentation comment noting that `generate_text_with_json_fallback` only supports single-turn system+user requests, and that extending message types requires updating both `build_text_messages` and the fallback path.

## Skipped Issues

None -- all 9 findings were successfully fixed.

---

_Fixed: 2026-04-29T13:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
