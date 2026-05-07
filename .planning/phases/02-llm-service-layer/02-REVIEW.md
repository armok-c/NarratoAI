---
phase: 02-llm-service-layer
reviewed: 2026-05-06T16:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/lib.rs
  - src/llm/mod.rs
  - src/llm/types.rs
  - src/llm/provider.rs
  - src/llm/registry.rs
  - src/llm/image_utils.rs
  - src/llm/openai_compatible.rs
  - src/llm/register.rs
  - src/llm/test_utils.rs
  - tests/llm_test.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
iteration: 7
previous_iteration: 6
---

# Phase 02: LLM Service Layer — Code Review Report

**Reviewed:** 2026-05-06T16:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found (0 critical, 0 warning, 3 info)
**Iteration:** 7

## Summary

Re-reviewed phase 02 after WR-01 fix (commit `cff8cca` — bounded concurrent image preprocessing). `cargo check` passes, 8/8 tests pass including previously-ignored `test_openai_error_mapping`.

No new issues found. WR-01 fix is correctly implemented: images are chunked first, each batch preprocesses only its own images inside the spawned task, and a preprocessing semaphore (max 4 concurrent decodes) bounds peak memory.

Three info-level issues from iteration 6 remain valid and are carried forward.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo check` | Pass (9 pre-existing warnings, none from Phase 02) |
| `cargo test --test llm_test` | 8/8 passed, 0 ignored |
| `cargo test --test config_test` | 6/6 passed (Phase 01 regression) |

---

## Previously Fixed (verified in earlier iterations)

| ID | Description | Iteration Fixed |
|----|-------------|----------------|
| CR-01 | Missing `wiremock` dev-dependency | 1 |
| CR-02 | Flaky CyclicResponder race in test | 2 |
| WR-01 | Unused `&self` on `build_text_messages` | 2 |
| WR-02 | Overly broad error classification | 3 |
| WR-03 | JPEG integrity decode OOM risk | 4 |
| WR-04 | Chinese JSON fallback prompt | 5 |
| IN-01 (prev) | Duplicate `response.choices.first()` calls | 2 |
| IN-02 (prev) | Double `into_iter().collect()` readability | 2 |
| IN-03 (prev) | `cargo check` blocked by TTS module error | 5 |
| WR-01 (iter 6) | Unbounded concurrent image preprocessing | 6 (fix commit `cff8cca`) |

---

## Info

### IN-01: Duplicate JSON fallback instruction string

**File:** `src/llm/openai_compatible.rs:229` and `src/llm/openai_compatible.rs:283`
**Severity:** Info (carried from iteration 5)

The JSON fallback instruction `"IMPORTANT: You MUST respond with ONLY valid JSON. Do not include any explanatory text, markdown formatting, or code blocks."` appears identically in both `generate_text_with_json_fallback` and `create_vision_chat_with_json_fallback`. Extracting to a `const` would prevent drift between the two paths.

### IN-02: No test coverage for partial registration failure

**File:** `src/llm/register.rs:45-116`
**Severity:** Info (carried from iteration 5)

`register_all_providers()` documents partial-failure behavior (successful providers remain registered even when others fail), but no test verifies this. A test with one valid and one invalid provider config would cover the partial-failure contract and prevent regressions.

### IN-03: Vision JSON fallback re-transmits all base64 image data

**File:** `src/llm/openai_compatible.rs:267-312`
**Severity:** Info (carried from iteration 6)

When `create_vision_chat_with_json_fallback` detects a `response_format` rejection, it rebuilds the request with the same `batch` (containing full base64 data URLs) and retransmits everything. For large batches (e.g., 10 images × ~67MB base64 each), this doubles the outbound bandwidth on fallback. This is inherent to the stateless API design and documented as a known characteristic.

---
_Reviewed: 2026-05-06T16:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 7_
