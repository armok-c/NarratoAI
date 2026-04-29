---
phase: "03"
fixed_at: 2026-04-29T11:00:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 2
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-04-29T11:00:00Z
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 7 (1 Critical + 6 Warnings)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: voice_name_to_lang 使用字符索引代替字节索引，在多字节字符语音名称上导致 panic

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** aeb2bd1
**Applied fix:** 将 `voice_name.chars().enumerate()` 替换为 `voice_name.char_indices()`，同时将模式匹配从 `.map(|(i, _)| i)` 改为直接解构 `Some((idx, _))`。`char_indices()` 返回字节索引，与后续的 `voice_name[..idx]` 字节切片兼容，避免了多字节 UTF-8 语音名称上的运行时 panic。

### WR-01: TTSError::AuthenticationFailed 在 Edge-TTS 引擎中从未被使用

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** c0e31d6
**Applied fix:** 在三处 `map_err` 调用中添加认证错误检测，当错误字符串包含 "401"、"authentication" 或 "unauthorized" 时返回 `TTSError::AuthenticationFailed` 而非 `ConnectionFailed`/`SynthesisFailed`：
1. 直连路径（`connect_async` 失败）
2. 代理连接路径（`client_async_tls_with_config` 失败）
3. 消息接收路径（`ws_stream.next()` 失败）
**Status:** fixed: requires human verification -- 字符串匹配可能无法覆盖所有认证错误变体，建议人工验证认证失败场景的错误映射是否正确。

### WR-02: WebSocket 在 turn.end 前关闭导致 duration=0.0 被当作成功返回

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 3a3a703
**Applied fix:** 在写入音频文件和 `duration == 0.0` 检查之前添加了 `if !received_turn_end { return Err(...) }` 提前返回。当连接在收到 `turn.end` 消息前关闭时，返回 `Err(TTSError::SynthesisFailed(...))` 而非 `Ok(TtsOutput { duration: 0.0, ... })`。同时简化了 `duration == 0.0` 分支，移除了 `received_turn_end` 为 false 时的死代码分支。
**Status:** fixed: requires human verification -- 语义逻辑变更，建议人工确认"连接在 turn.end 前关闭即视为错误"的策略是否符合预期行为。

### WR-03: max_retries 变量命名与实际语义不符（误导维护者）

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 28b8fa6
**Applied fix:** 将 `max_retries` 重命名为 `max_attempts`，更新所有 5 处引用（声明、循环条件、info 日志、sleep 条件、错误消息）。变量值保持 4（1 次初始 + 3 次重试），命名现在清晰反映"总尝试次数"语义。

### WR-04: 库测试与集成测试存在重复测试用例（翻倍维护成本）

**Files modified:** `tests/tts_test.rs`
**Commit:** d525dc9
**Applied fix:** 从 `tests/tts_test.rs` 中移除重复的 `test_synthesize_unknown_engine_error` 和 `test_synthesize_unknown_engine_message_contains_name` 测试函数（共 38 行）。这两个测试已经存在于 `src/tts/mod.rs` 中，提供相同的覆盖范围。同时删除了对应的" synthesize 路由器集成测试"章节注释。

### WR-05: test_synthesize_function_signature 丢弃未 poll 的 async future（编译器警告）

**Files modified:** `tests/tts_test.rs`
**Commit:** a9a169a
**Applied fix:** 在 `test_synthesize_function_signature` 函数的 `#[test]` 属性之前添加 `#[allow(clippy::let_underscore_future)]`，抑制因有意丢弃未 poll 的 async future 而产生的 clippy 警告。

### WR-06: EdgeTtsEngine 和 new() 的 pub 可见度过于宽松

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 33c5fc5
**Applied fix:** 将 `EdgeTtsEngine` 结构体声明、`proxy_enabled`、`proxy_http`、`proxy_https` 字段以及 `new()` 构造函数的可见度从 `pub` 降为 `pub(super)`，将实例化限制在 `tts` 模块内部，防止外部 crate 绕过 `tts::synthesize()` 路由器直接构造引擎实例。

## Skipped Issues

无 -- 所有 7 个发现（1 个关键 + 6 个警告）均已成功修复。

Info 级别发现（IN-01 至 IN-05）已按 `fix_scope: critical_warning` 排除。

---

_Fixed: 2026-04-29T11:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
