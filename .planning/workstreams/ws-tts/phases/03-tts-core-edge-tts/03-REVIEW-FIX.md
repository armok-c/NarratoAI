---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T09:55:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T09:55:00Z
**Source review:** `.planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (1 Critical + 6 Warnings)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: voice_name_to_lang 使用字符索引代替字节索引，在多字节字符语音名称上导致 panic

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** bbaaba9
**Applied fix:** Replaced `voice_name.chars().enumerate()` (char index) with `voice_name.char_indices()` (byte index). The `enumerate()` call returned the nth Unicode scalar value position, but `voice_name[..idx]` requires byte offsets. `char_indices()` yields `(byte_index, char)` tuples directly, ensuring slice indices land on valid UTF-8 boundaries. This was a latent bug for non-ASCII voice names (e.g., accented Latin or CJK characters).

### WR-01: TTSError::AuthenticationFailed 在 Edge-TTS 引擎中从未被使用

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 6413cac
**Applied fix:** Added authentication error detection in three WebSocket error mappings: the two connect paths (direct and proxy) in `connect()`, and the message receive path in `synthesize_once()`. When a tungstenite error string contains "401", "authentication", or "Unauthorized", the error is mapped to `TTSError::AuthenticationFailed(err_str)` instead of the generic `ConnectionFailed` or `SynthesisFailed`.

### WR-02: WebSocket 在 turn.end 前关闭导致 duration=0.0 被当作成功返回

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** ac579db
**Applied fix:** Changed the post-loop error handling to return `Err(TTSError::SynthesisFailed(...))` when `received_turn_end` is false (connection closed before receiving turn.end), instead of falling through to `Ok(TtsOutput { duration: 0.0, ... })`. The `duration == 0.0` / `received_turn_end` branching was replaced with: check `!received_turn_end` first (return error), then check `duration == 0.0` for the case where turn.end was received but audio_duration was missing/unparseable (keeps the warning log).

### WR-03: max_retries 变量命名与实际语义不符（误导维护者）

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 479062a
**Applied fix:** Renamed `max_retries` to `max_attempts` throughout `synthesize_with_retry()`. The variable holds the total attempt count (1 initial + 3 retries = 4). The previous name `max_retries` misleadingly suggested "maximum retry count" when the actual value included the initial attempt. The error message correctly uses `max_attempts - 1` to report "重试 3 次后仍失败".

### WR-04: 库测试与集成测试存在重复测试用例（翻倍维护成本）

**Files modified:** `tests/tts_test.rs`
**Commit:** fa67b5c
**Applied fix:** Removed the two duplicate test functions `test_synthesize_unknown_engine_error` and `test_synthesize_unknown_engine_message_contains_name` from `tests/tts_test.rs`. The equivalent tests in `src/tts/mod.rs` (`test_synthesize_unknown_engine` and `test_synthesize_unknown_engine_message`) provide identical coverage of the `synthesize()` router's unknown engine error path.

### WR-05: test_synthesize_function_signature 丢弃未 poll 的 async future（编译器警告）

**Files modified:** `tests/tts_test.rs`
**Commit:** 473e402
**Applied fix:** Added `#[allow(clippy::let_underscore_future)]` before the `#[test]` attribute on `test_synthesize_function_signature` to suppress the clippy warning about discarding an unpolled async future. The test intentionally creates and drops the future to verify the function signature at compile time.

### WR-06: EdgeTtsEngine 和 new() 的 pub 可见度过于宽松

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 8db599b
**Applied fix:** Changed the visibility of `EdgeTtsEngine` struct, all its fields (`proxy_enabled`, `proxy_http`, `proxy_https`), and the `new()` constructor from `pub` to `pub(super)`. This restricts direct instantiation to the `tts` module, ensuring all engine creation goes through the `tts::synthesize()` router and its proxy configuration logic.

---

## Skipped Issues

*None -- all 7 findings in scope (1 Critical + 6 Warnings) were successfully fixed.*

Info-level findings (IN-01 through IN-06) were excluded per `fix_scope: critical_warning`.

---

_Fixed: 2026-04-29T09:55:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
