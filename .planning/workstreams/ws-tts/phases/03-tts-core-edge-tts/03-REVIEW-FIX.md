---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-28T13:08:56Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Fix Report

**Fixed at:** 2026-04-28T13:08:56Z
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Duration parsing in `turn.end` omits tick-to-seconds division for numeric `audio_duration`

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `f6669c4`
**Applied fix:** Added `/ 10_000_000.0` division to the `as_f64()` branch (line 295) so that when `audio_duration` arrives as a JSON number instead of a string, the value is correctly converted from 100-ns tick units to seconds. Previously the raw tick value was used directly as seconds, producing a duration off by a factor of 10 million.

### WR-01: CONNECT tunnel target includes URL path and query string, violating RFC 7231

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `0983d44`
**Applied fix:** Added `target_host_only` variable that strips the path and query from `target_addr` using `split('/').next()`. The `unwrap_or` fallback (used when no explicit port is present) now uses this stripped hostname instead of the full URL. This ensures the CONNECT request contains only `host:port` per RFC 7231 Section 4.3.6.

### WR-02: Word boundary parse failures silently drop data without diagnostic log

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `96b66de`
**Applied fix:** Added an `else` branch to the inner `if let` that parses word boundary fields (`offset`, `duration`, `text`). The `else` branch emits a `tracing::warn!` with the full JSON payload when parsing fails, providing diagnostic visibility into server format changes.

### WR-03: `test_synthesize_function_signature` performs no meaningful runtime verification

**Files modified:** `tests/tts_test.rs`
**Commit:** `196ed83`
**Applied fix:** Replaced the dead test (inner function defined but never called, no assertions) with a compile-time type check that calls `tts::synthesize` with the expected parameter types and assigns the returned future to `_`. The future is never polled, so there are no runtime side effects. If the function signature changes, the test fails to compile, alerting the developer.

---

_Fixed: 2026-04-28T13:08:56Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
