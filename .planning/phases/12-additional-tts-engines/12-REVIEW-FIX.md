---
phase: 12-additional-tts-engines
fixed_at: 2026-04-30T00:00:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 1
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-04-30T00:00:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 9
- Fixed: 9
- Skipped: 0

## Fixed Issues

### WR-01: Tencent TTS `speed_value` always computed as 0.0 (rate parameter ignored)

**Files modified:** `src/tts/tencent_tts.rs`
**Commit:** 50b7429

**Applied fix:**
- Added `rate: f64` parameter to `synthesize_at_url()` and `synthesize_once()`
- Changed speed computation from `0.0_f64.clamp(-2.0, 2.0)` to `((rate - 1.0) * 2.0).clamp(-2.0, 2.0)`
- Updated `synthesize()` to pass `rate` instead of using `_rate` as a discarded parameter
- Updated all test calls to pass rate parameter (1.0)

### WR-02: Azure Speech SSML omits prosody rate and pitch

**Files modified:** `src/tts/azure_speech.rs`
**Commit:** 2e1593f

**Applied fix:**
- Added `convert_rate_to_percent()` helper method (1.0 → "+0%", 1.5 → "+50%", 0.5 → "-50%")
- Added `convert_pitch_to_hz()` helper method (0 → "+0Hz", 50 → "+50Hz", -10 → "-10Hz")
- Updated SSML template to wrap voice text in `<prosody rate="{}" pitch="{}">` element
- Changed `synthesize()` to pass `rate` and `pitch` (no longer underscore-prefixed)
- Updated `synthesize_once()` to accept and use rate/pitch parameters
- Added `xmlns:mstts` namespace to SSML declaration

### WR-03: Whitespace-only text bypasses empty-text validation

**Files modified:** `src/tts/soulvoice.rs`, `src/tts/doubaotts.rs`, `src/tts/qwen_tts.rs`, `src/tts/indextts2.rs`, `src/tts/tencent_tts.rs`, `src/tts/azure_speech.rs`
**Commit:** 7b0bac1

**Applied fix:**
- Changed all 6 engine `synthesize()` methods from `if text.is_empty()` to `if text.trim().is_empty()`, preventing whitespace-only strings (e.g., `"  "`) from bypassing validation and being sent as empty text to upstream APIs.

### WR-04: `should_use_azure_services` recompiles regex on each invocation

**Files modified:** `src/tts/azure_speech.rs`
**Commit:** 340aef2

**Applied fix:**
- Added `azure_voice_regex()` function using `std::sync::OnceLock` to compile the Azure Neural voice regex once and reuse it
- Changed `should_use_azure_services()` to call the cached regex instead of `Regex::new()` on every invocation
- Used `.expect()` instead of `.unwrap_or(false)` — regex compilation failure is a programming error that should panic, not silently return false

### WR-05: `build_client` silently falls back to unproxied client on error

**Files modified:** `src/tts/common.rs`
**Commit:** 9751569

**Applied fix:**
- Added `tracing::warn!` logging in the `unwrap_or_else` fallback path, so proxy configuration errors are logged with the error details instead of being silently swallowed

### WR-06: Dead config field `SoulVoiceSection::voice_uri`

**Files modified:** `src/config/types.rs`, `src/config/defaults.rs`, `src/tts/soulvoice.rs`
**Commit:** e0036c1

**Applied fix:**
- Removed `voice_uri` field from `SoulVoiceSection` struct
- Removed `voice_uri: String::new()` from `Default` impl and all test constructors (4 occurrences in soulvoice.rs)
- Removed `voice_uri = "speech:test"` from test TOML (would have caused `deny_unknown_fields` rejection)

### WR-07: Dead config fields `DoubaoTTSSection::ak` and `sk`

**Files modified:** `src/config/types.rs`, `src/config/defaults.rs`, `src/tts/doubaotts.rs`
**Commit:** 515c538

**Applied fix:**
- Removed `ak` and `sk` fields from `DoubaoTTSSection` struct (unused — authentication uses only `token` field)
- Removed from `Default` impl and all test constructors (4 occurrences in doubaotts.rs)
- Removed `ak = "db-ak"` and `sk = "db-sk"` from test TOML

### WR-08: Dead config fields `UiSection::doubaotts_voice_type` and `doubaotts_rate`

**Files modified:** `src/config/types.rs`, `src/config/defaults.rs`
**Commit:** 908a04b

**Applied fix:**
- Removed `doubaotts_voice_type` and `doubaotts_rate` fields from `UiSection` struct (unused — voice type comes from `voice_name` parameter flow, rate is passed via the `rate` parameter)
- Removed from `Default` impl
- Removed from test TOML

### WR-09: `retry_loop` callers do not clean up corrupted output files after retry exhaustion

**Files modified:** `src/tts/soulvoice.rs`, `src/tts/doubaotts.rs`, `src/tts/qwen_tts.rs`, `src/tts/indextts2.rs`, `src/tts/tencent_tts.rs`, `src/tts/azure_speech.rs`
**Commit:** 0b7d086

**Applied fix:**
- Added `.map_err(|e| { let _ = std::fs::remove_file(output_path); e })` after each `retry_loop()` call across all 6 engines, ensuring partial/corrupted output files are cleaned up when all retry attempts are exhausted (matching the existing pattern in `edge_tts.rs:402`).

---

_Fixed: 2026-04-30T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
