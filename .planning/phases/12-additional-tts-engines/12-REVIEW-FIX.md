---
phase: 12-additional-tts-engines
fixed_at: 2026-05-01T22:00:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-05-01T22:00:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 3 (Critical + Warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: build_client returns Result but all callers use .expect() -- panics in production on TLS failure

**Files modified:** `src/tts/soulvoice.rs`, `src/tts/doubaotts.rs`, `src/tts/qwen_tts.rs`, `src/tts/indextts2.rs`, `src/tts/azure_speech.rs`, `src/tts/tencent_tts.rs`, `src/tts/mod.rs`
**Commit:** a4b0738
**Applied fix:** Changed all 6 engine `new()` constructors from `fn new(...) -> Self` to `fn new(...) -> Result<Self, TTSError>`, replacing `.expect()` with `?` for error propagation. Updated `mod.rs` router to handle `Result` with `.map_err(|e| TTSError::ConnectionFailed(...))`. All test code updated to use `.expect()` on construction (acceptable in test context). Now TLS initialization failures return a graceful `TTSError::ConnectionFailed` to the caller instead of panicking.

### WR-02: Tencent TTS subtitle timestamp cast from i64 to u64 silently wraps negative values

**Files modified:** `src/tts/tencent_tts.rs`
**Commit:** a4b0738
**Applied fix:** Added guard clauses before the `as u64` cast: `if start_ms >= 0 { (start_ms as u64) * 10000 } else { 0 }` and same for `end_ms`. Negative timestamps from the Tencent API (which should never occur in practice) now clamp to 0 instead of wrapping to near-u64::MAX.

### WR-03: Doubao TTS test test_doubaotts_payload_structure asserts pitch_ratio == 1.2 but production code hardcodes 1.0

**Files modified:** `src/tts/doubaotts.rs`
**Commit:** a4b0738
**Applied fix:** Updated test payload construction to use `"pitch_ratio": 1.0` (matching production code hardcoded in `synthesize_once` after the prior CR-01 fix) and changed assertion from `assert_eq!(payload["audio"]["pitch_ratio"], 1.2)` to `assert_eq!(payload["audio"]["pitch_ratio"], 1.0)`.

## Verification

- `cargo check` passes (only pre-existing dead_code warning for `get_azure_voices`)
- `cargo test --lib`: 367 passed, 1 failed (pre-existing `ffmpeg::hwaccel::tests::test_detect_encoders_format` unrelated to this fix)
- All 36 TTS-related tests pass

---
*Fixed: 2026-05-01T22:00:00Z*
*Fixer: Claude (gsd-code-fixer)*
*Iteration: 2*
