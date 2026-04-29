---
phase: 02-llm-service-layer
fixed_at: 2026-04-29T14:00:00Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T14:00:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: Missing `wiremock` dev-dependency prevents integration test compilation

**Files modified:** `Cargo.toml`
**Commit:** `2ec7ada`
**Applied fix:** Added `wiremock = "0.6"` to `[dev-dependencies]` section. Tests in `tests/llm_test.rs` use wiremock extensively through `Mock`, `MockServer`, `ResponseTemplate`, and matchers.

---

### CR-02: Flaky test due to CyclicResponder race with concurrent Semaphore-controlled requests

**Files modified:** `tests/llm_test.rs`
**Commit:** `e22a392`
**Applied fix:** Replaced `CyclicResponder` (backed by `AtomicUsize::fetch_add`) with a single fixed `ResponseTemplate` returning identical responses. Removed ordering assertions that were inherently racy. The test now verifies call count via `.expect(3)` and checks that all 3 results are non-empty.

---

### WR-01: Unused `&self` parameter on `build_text_messages`

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `9b089e2`
**Applied fix:** Removed `&self` from `build_text_messages` signature, making it a consistent associated function (matching `build_vision_messages`). Updated all three callers to use `Self::build_text_messages(...)`.

---

### WR-02: Overly broad heuristic substring matching in error classification

**Files modified:** `src/error.rs`
**Commit:** `5588379`
**Applied fix:** Replaced broad `"auth"` and `"key"` substring matching with targeted patterns: `"authentication"`, `"unauthorized"`, `"invalid api key"`, `"invalid_api_key"`, and `"invalid_token"`. Also added `"rate_limit"` to rate-limit detection.

---

### WR-03: JPEG integrity verification decodes full image, risking OOM under concurrency

**Files modified:** `src/llm/image_utils.rs`
**Commit:** `d82fa4a`
**Applied fix:** Removed the `image::load_from_memory(&raw_bytes)` integrity check from the JPEG fast path. Full decode of a 50 MB JPEG can allocate ~200 MB under concurrent preprocessing, creating OOM risk.

---

### WR-04: JSON fallback prompt uses Chinese, inconsistent with potential English system prompt

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `d4123c7`
**Applied fix:** Replaced Chinese JSON fallback instruction with English equivalent in both `generate_text_with_json_fallback` and `create_vision_chat_with_json_fallback`.

---

### IN-01: `response.choices.first()` called twice in analyze_images

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `c3aacbf`
**Applied fix:** Extracted `first_choice` into a local variable to avoid repeated slice access. Reused inside both the `and_then` for content extraction and the `ok_or_else` closure for finish_reason checking.

---

### IN-02: Double `.into_iter().collect()` chain reduces readability

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `9f9d82d`
**Applied fix:** Split the chained `.into_iter().collect()` sequence into three explicitly-typed intermediate variables. Added `use tokio::task::JoinHandle;` import.

---

## Skipped Issues

None -- all 8 findings were successfully fixed.

---

_Fixed: 2026-04-29T14:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
