---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T12:45:00+08:00
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T12:45:00+08:00
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: undefined variable `connect_req` in HTTP CONNECT proxy tunnel -- compile error

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 29777ec, 4cd2ee9
**Applied fix:** Constructed the HTTP CONNECT request string using `target_host` and `target_port` before sending it over the proxy TCP connection. Also fixed pre-existing `let tcp` to `let mut tcp` to allow mutable borrows for `write_all`, `flush`, and `read` calls in the proxy tunnel. The `target_host` and `target_port` variables (previously dead code, IN-01) are now consumed by `connect_req`.

### CR-02: undefined behavior on NaN/infinity rate or pitch in conversion functions

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 2f173cf
**Applied fix:** Added `!rate.is_finite() || rate < 0.0` guard in `convert_rate_to_percent` and `!pitch.is_finite()` guard in `convert_pitch_to_hz`. Both return safe fallback values (`+0%` and `+0Hz` respectively) when input is non-finite. This prevents the undefined behavior that occurs in Rust when casting NaN/infinity `f64` values to `i32`.

### WR-01: no input validation before establishing WebSocket connection

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 07d0dff
**Applied fix:** Added input validation at the start of `EdgeTtsEngine::synthesize()`: empty `text` returns `Err(TTSError::SynthesisFailed)`; non-finite or negative `rate` returns `Err(TTSError::SynthesisFailed)`; non-finite `pitch` returns `Err(TTSError::SynthesisFailed)`. Validation happens before SSML construction and WebSocket connection, avoiding wasted connections.

### WR-02: binary message parse failure silently discarded -- no diagnostic

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 68ca0a7
**Applied fix:** Added `else` branch to the `if let Some(content) = parse_edge_tts_binary(&data)` block. When parsing fails (returns `None`), a `tracing::warn!` is emitted with the raw bytes truncated to 256 bytes for diagnostic visibility.

### WR-03: fragile proxy URL parsing -- uppercase scheme, missing scheme, unknown scheme

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** d9d29b3
**Applied fix:** Replaced case-sensitive `starts_with("https://")` and `trim_start_matches("http://")`/`trim_start_matches("https://")` with case-insensitive `split_once("://")` + `to_lowercase()` scheme matching. Added explicit port defaults for `wss` (443) and `socks5` (1080) schemes.

### IN-01: unused variables `target_host` and `target_port`

**Status:** automatically resolved by CR-01. These variables are now consumed by the `connect_req` construction. No separate commit needed.

### IN-02: pre-release dependency `notify = "9.0.0-rc.3"`

**Files modified:** `Cargo.toml`
**Commit:** d6922d0
**Applied fix:** Added inline comment documenting the known risk: `# 已知风险: RC 版 API 不稳定，9.0.0 正式版可能破坏兼容性`. Documents the risk without changing the pin.

### IN-03: signature test uses opaque empty-string parameters

**Files modified:** `tests/tts_test.rs`
**Commit:** 1a79f81
**Applied fix:** Changed `tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None)` to use descriptive variable bindings (`engine`, `text`, `voice_name`) so readers can identify which empty-string maps to which parameter.

## Verification Notes

- All 8 in-scope findings addressed (7 findings requiring explicit changes, 1 auto-resolved by CR-01).
- `cargo check` confirms all modified files compile correctly. Pre-existing errors in `image_utils.rs` (missing `GenericImageView` import) and `openai_compatible.rs` (`with_max_retries` method not found) are unrelated to TTS changes and were present before this fix iteration.
- The `let mut tcp` fix was added as an auxiliary change to CR-01 since the proxy tunnel code requires mutable access to the TCP stream for `write_all`, `flush`, and `read`. This was a pre-existing bug hidden by the CR-01 compile error.

## Skipped Issues

None -- all 8 findings were fixed.

---

_Fixed: 2026-04-29T12:45:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
