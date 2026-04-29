---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T17:30:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T17:30:00Z
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (Critical + Warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: `Message::Text(stt_message)` 类型不匹配导致编译失败

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** a43d796
**Applied fix:** Changed `Message::Text(stt_message)` to `Message::Text(stt_message.into())`. tungstenite 0.29 requires `Utf8Bytes` for `Message::Text` instead of `String`, and `Utf8Bytes` implements `From<String>`, so `.into()` performs the conversion.

### WR-01: 重试逻辑未排除不可重试错误类型

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** d7b9eb9
**Applied fix:** Added `AuthenticationFailed` check at the top of the `Err(e)` branch in `synthesize_with_retry()`. If the error is `TTSError::AuthenticationFailed`, the function returns immediately instead of retrying, saving ~1+ second per wasted retry on invalid credentials.

### WR-02: `connect()` 函数缺少超时保护

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 34118ab
**Applied fix:** Wrapped the entire `connect()` body in `tokio::time::timeout(CONNECT_TIMEOUT, async move { ... })` where `CONNECT_TIMEOUT = Duration::from_secs(30)`. On timeout, returns `TTSError::ConnectionFailed("Edge-TTS CONNECT 超时 (30s)")`, preventing infinite blocking during CONNECT response reading or TLS handshake.

## Skipped Issues

None -- all 3 findings in scope (1 Critical + 2 Warnings) were successfully fixed.

---

_Fixed: 2026-04-29T17:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
