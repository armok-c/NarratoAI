---
phase: 12-additional-tts-engines
fixed_at: 2026-05-01T12:00:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-05-01T12:00:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Doubao TTS passes `pitch` value directly as `pitch_ratio` without semantic conversion

**Files modified:** `src/tts/doubaotts.rs`
**Commit:** 145454b
**Applied fix:** Changed `pitch: f64` to `_pitch: f64` in `synthesize_once` signature and hardcoded `"pitch_ratio": 1.0` in the request body. This follows the same pattern as SoulVoice/Qwen/IndexTTS2/Tencent which correctly ignore pitch (the Doubao/Volcengine API expects pitch_ratio as a multiplicative ratio where 1.0 = standard). Hardcoding 1.0 avoids sending out-of-range values when the trait contract provides pitch=0 (standard in Hz offset format).

### WR-01: Azure Speech SSML does not filter XML 1.0 disallowed control characters

**Files modified:** `src/tts/azure_speech.rs`
**Commit:** a5d3c34
**Applied fix:** Added the same `.chars().filter(...)` chain used in `edge_tts.rs:83-91` after the XML entity replacements (`&`, `<`, `>`). This filters out XML 1.0 disallowed control characters (U+0000-U+0008, U+000B-U+000C, U+000E-U+001F, U+007F-U+009F) that would otherwise cause the Azure Speech REST API to return a 400 Bad Request error.

### WR-02: Tencent TTS sends empty `X-TC-Token` header instead of omitting it

**Files modified:** `src/tts/tencent_tts.rs`
**Commit:** e99a572
**Applied fix:** Removed the `.header("X-TC-Token", "")` line from the TC3-HMAC-SHA256 signed request. Per the Tencent Cloud API specification, this header is only required for STS temporary credentials. For permanent credentials (SecretId + SecretKey), the header must be omitted entirely. Sending an empty string may cause some API gateway versions to reject the request.

### WR-03: `build_client` silently falls back to proxy-less client on builder failure

**Files modified:** `src/tts/common.rs`, `src/tts/doubaotts.rs`, `src/tts/azure_speech.rs`, `src/tts/tencent_tts.rs`, `src/tts/soulvoice.rs`, `src/tts/indextts2.rs`, `src/tts/qwen_tts.rs`
**Commit:** c6e5991
**Applied fix:** Changed `build_client()` to return `Result<reqwest::Client, TTSError>` instead of silently falling back to `reqwest::Client::new()`. Previously, a `Client::builder().build()` failure would discard the proxy configuration with only a `warn!`-level log. Now the function returns `Err(TTSError::ConnectionFailed(...))` with a descriptive message. All 6 engine constructors use `.expect()` with engine-specific messages to fail explicitly during startup if the HTTP client cannot be built.

---

*Fixed: 2026-05-01T12:00:00Z*
*Fixer: Claude (gsd-code-fixer)*
*Iteration: 1*
