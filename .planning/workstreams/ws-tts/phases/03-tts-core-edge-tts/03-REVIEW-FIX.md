---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-28T15:23:21Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Fix Report (Iteration 1)

**Fixed at:** 2026-04-28T15:23:21Z
**Source review:** `.planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (all Warnings)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### WR-01: Proxy CONNECT handler does not support interim 1xx responses

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `55cb0cd`

**Applied fix:** Replaced the single `read_line` + `contains("200")` check with a loop that properly parses HTTP status codes. 2xx responses are accepted as success, interim 1xx responses and their trailing headers are skipped, and all other status codes (or empty responses) are treated as permanent failure. This complies with RFC 7231 Section 4.3.6.

### WR-02: Retry semantics do not match documentation

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `eab8ae2`

**Applied fix:** Changed `max_retries` from 3 to 4 with loop `1..=max_retries`, yielding 4 total attempts = 1 initial + 3 retries (matching D-03 requirement). Error message uses `max_retries - 1` to correctly display "3 retries".

### WR-03: `convert_pitch_to_percent` function name is misleading

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `9498e22`

**Applied fix:** Renamed function to `convert_pitch_to_hz` to match its actual output format (`+XHz` / `-XHz`), consistent with `convert_rate_to_percent` which correctly outputs percent format. All references (call site in `build_ssml` and 3 test functions) updated via `replace_all`.

### WR-04: Warning log misleading when turn.end received with missing audio_duration

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `4f9e359`

**Applied fix:** Added `received_turn_end` boolean flag, set to `true` in the `turn.end` handler. The post-loop `duration == 0.0` warning now distinguishes between "turn.end received but audio_duration missing/unparseable" and "connection closed before turn.end", providing accurate diagnostic context.

### WR-05: No timeout on WebSocket message receive

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `a11af01`

**Applied fix:** Added `use tokio::time::timeout` import and `WS_RECEIVE_TIMEOUT` constant (120 seconds). Wrapped `ws_stream.next().await` with `timeout()`, returning `TTSError::SynthesisFailed` with a descriptive message if the server does not respond within the timeout window.

### WR-06: Proxy URL with authentication credentials is not supported

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `6ffbe27`

**Applied fix:** Replaced simple `split_once(':')` on the raw proxy address with a two-step parse: (1) strip `user:pass@` prefix using `rsplit('@')`, then (2) parse `host:port` from the remaining string. Default port handling also updated to use `u16` directly. Error message hints at unsupported credentials format.

### WR-07: `xml:lang="zh-CN"` hardcoded in SSML template

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `021de1b`

**Applied fix:** Added `voice_name_to_lang()` helper that extracts the language tag from the voice name (e.g., `"en-US-JennyNeural"` yields `"en-US"`) by locating the second hyphen. The SSML template now uses `{}` format for `xml:lang` with this derived value. Falls back to `"zh-CN"` if extraction fails.

### WR-08: Word boundary offset addition can overflow `u64`

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `2811a55`

**Applied fix:** Changed `offset + duration_100ns` to `offset.saturating_add(duration_100ns)` to prevent debug panics and release wraparound on u64 overflow from untrusted server response data.

---

## Skipped Issues

*None -- all 8 findings in scope were successfully fixed.*

---

_Fixed: 2026-04-28T15:23:21Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
