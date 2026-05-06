---
phase: 02-llm-service-layer
fixed_at: 2026-05-06T15:30:00Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 2
fix_scope: critical_warning
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 02: LLM Service Layer — Code Review Fix Report

**Fixed at:** 2026-05-06T15:30:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md (iteration 6)
**Fix scope:** critical + warning (--all not specified)
**Iteration:** 2

**Summary:**
- Findings in scope (critical + warning): 1
- Fixed: 1
- Skipped (info, out of scope): 3

## Fixed

| ID | Severity | Description | Fix |
|----|----------|-------------|-----|
| WR-01 | Warning | `analyze_images` unbounded concurrent image preprocessing — all base64 data URLs held in memory simultaneously | Restructured to interleave preprocessing with batch submission: images chunked first, each batch preprocesses only its own images before API call, preprocessing semaphore (max 4) bounds concurrent decodes |

### WR-01 Fix Detail

**Commit:** `cff8cca` — `fix(02): bound concurrent image preprocessing in analyze_images (WR-01)`

**Change:** `src/llm/openai_compatible.rs` (+22/-30)

**Before:** All images preprocessed upfront via `spawn_blocking`, all base64 data URLs collected into `Vec<String>` before any API call. Peak memory unbounded: N × ~67MB.

**After:** Images split into chunks matching `batch_size` first. Each batch task preprocesses only its own images inside the async task, bounded by a preprocessing semaphore (max 4 concurrent decodes). Base64 data for a batch is released after the API call completes. Peak memory bounded to: `batch_size × max_concurrency × ~67MB`.

**Verification:**
- `cargo check` passes (no new errors)
- 8/8 LLM tests pass, including `test_analyze_images_result_ordering`

## Skipped (info — not in default fix scope)

| ID | Description | File |
|----|-------------|------|
| IN-01 | Duplicate JSON fallback instruction string | `src/llm/openai_compatible.rs:230,283` |
| IN-02 | No test coverage for partial registration failure | `src/llm/register.rs:45` |
| IN-03 | Vision JSON fallback re-transmits all base64 image data | `src/llm/openai_compatible.rs:268-313` |

To include info-level issues, run: `/gsd-code-review-fix 02 --all`

---

_Fixed: 2026-05-06T15:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
