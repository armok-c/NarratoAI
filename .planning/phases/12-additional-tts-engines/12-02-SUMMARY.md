---
phase: 12-additional-tts-engines
plan: 02
subsystem: tts
tags: [qwen, indextts2, dashscope, multipart, wiremock]
requires:
  - phase: 12-01
    provides: common.rs infrastructure (build_client, retry_loop, parse_engine_prefix, write_audio_bytes)
provides:
  - Qwen TTS engine with double-download pattern (POST -> URL -> GET bytes)
  - IndexTTS2 engine with multipart/form-data file upload
affects: [13-tts-integration]
tech-stack:
  added: [reqwest multipart feature]
  patterns: [Secondary download pattern for audio URLs, Multipart file upload for reference audio]
key-files:
  created:
    - src/tts/qwen_tts.rs
    - src/tts/indextts2.rs
  modified:
    - Cargo.toml
    - src/config/types.rs
    - src/config/defaults.rs
    - src/tts/mod.rs
key-decisions:
  - "Added api_url field to TtsQwenSection for testability via wiremock (hardcoded URL prevents mock server interception)"
requirements-completed: [TTS-07, TTS-08]

# Metrics
duration: 12min
completed: 2026-04-30
---

# Phase 12 Plan 02: Qwen TTS + IndexTTS2 Engines Summary

**Qwen TTS (DashScope REST API with secondary audio URL download) and IndexTTS2 (multipart/form-data reference audio upload) engines implemented, tested with wiremock, and registered in synthesize() router**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-30T09:40:00Z
- **Completed:** 2026-04-30T09:52:33Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Qwen TTS engine with double-download pattern: POST to DashScope REST API receives audio URL, then GET downloads the actual audio bytes, with duration extracted from API response
- IndexTTS2 engine with multipart/form-data: reads reference audio file from disk, uploads as `prompt_audio` field alongside inference parameters (temperature, top_p, top_k, do_sample, num_beams, repetition_penalty), 120-second timeout for long inference
- Both engines registered in `synthesize()` router as `"qwen_tts"` and `"indextts2"` match branches
- Both engines use common.rs infrastructure: `build_client`, `retry_loop`, `parse_engine_prefix`, `write_audio_bytes`
- 10 unit tests total (5 per engine), all passing via wiremock

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Qwen TTS engine + update Cargo.toml** - `8fa8838` (feat)
2. **Task 2: Implement IndexTTS2 engine** - `7303380` (feat)

## Files Created/Modified

- `Cargo.toml` - Added `multipart` feature to reqwest dependency
- `src/config/types.rs` - Added `api_url` field to `TtsQwenSection`
- `src/config/defaults.rs` - Added `api_url` default to `TtsQwenSection`
- `src/tts/qwen_tts.rs` (NEW) - Qwen TTS engine with DashScope REST API + secondary audio download, 5 tests
- `src/tts/indextts2.rs` (NEW) - IndexTTS2 engine with multipart form upload, 5 tests
- `src/tts/mod.rs` - Added `mod qwen_tts`, `mod indextts2`, and both match branches in `synthesize()`

## Decisions Made

- Added `api_url` field to `TtsQwenSection` for testability. The plan's original code used a hardcoded DashScope URL constant, which prevents wiremock interception. Following the soulvoice pattern (configurable `api_url` with fallback default) enables both production use and unit testing.
- IndexTTS2 uses `reqwest::multipart::Part::bytes()` instead of `Part::file()` to avoid file path encoding issues and to be retry-safe (each retry re-reads the reference audio file).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added api_url field to TtsQwenSection for testability**
- **Found during:** Task 1 (Qwen TTS engine implementation)
- **Issue:** Plan specified a hardcoded DashScope API URL constant, but also required wiremock tests. A hardcoded URL prevents redirecting requests to the mock server.
- **Fix:** Added `api_url` field to `TtsQwenSection` (with `#[serde(default)]`), defaulting to empty string. The engine uses the config's `api_url` if non-empty, falling back to the hardcoded DashScope URL. This follows the same pattern as `SoulVoiceSection.api_url`.
- **Files modified:** `src/config/types.rs`, `src/config/defaults.rs`
- **Verification:** All 5 Qwen TTS tests pass (engine_new, success/wiremock, empty_text, no_audio_url, parse_prefix)
- **Committed in:** `8fa8838` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Necessary for testability. No scope creep.

## Issues Encountered

- None. Both engines implemented according to plan specification with minor testability enhancement.

## User Setup Required

None - no external service configuration required for this plan.

## Next Phase Readiness

- Both engines are fully implemented and tested, ready for integration into the TTS pipeline (Phase 13).
- `qwen3:` prefix is parsed to extract voice name; `indextts2:` prefix is parsed to extract reference audio path.
- All 59 TTS tests pass (Wave 1 + Wave 2).

---
*Phase: 12-additional-tts-engines*
*Completed: 2026-04-30*
