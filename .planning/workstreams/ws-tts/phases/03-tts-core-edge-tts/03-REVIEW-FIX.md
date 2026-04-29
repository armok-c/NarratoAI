---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T22:30:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T22:30:00Z
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Word boundary offsets silently dropped when Edge TTS returns float-valued JSON

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 39b2d1c
**Applied fix:** Changed word boundary offset/duration extraction from exclusive `as_u64()` to try `as_u64()` first with `as_f64()` fallback (converting to `u64`). This matches the defensive parsing pattern already used in `turn.end` duration processing. Backward-compatible: integer JSON numbers still go through `as_u64()`; float-valued JSON numbers are now handled instead of being silently dropped into the warning path.

### WR-02: `voice_name_to_lang` returns empty string for non-standard voice names, producing invalid SSML

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 4ef96df
**Applied fix:** Changed the `else` branch in `voice_name_to_lang` from `String::new()` to `"en-US".to_string()` with a `tracing::warn!()` log when the second hyphen is not found. This prevents invalid `<speak ... xml:lang="">` in SSML output for non-standard voice names.

## Skipped Issues

None -- all 2 findings in scope were successfully fixed.

---

_Fixed: 2026-04-29T22:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
