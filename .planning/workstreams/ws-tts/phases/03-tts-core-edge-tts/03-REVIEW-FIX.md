---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T13:50:00+08:00
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS - Code Review Fix Report

**Fixed at:** 2026-04-29T13:50:00+08:00
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: 代理 CONNECT 响应读取器无法处理分开发送的 interim 1xx 响应

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 36c16e2
**Applied fix:** 将原来的"单次读取+单次解析"模式重构为两层嵌套循环：
- 外层循环：每次读取一个完整响应（直到 `\r\n\r\n`），然后解析该响应
- 内层循环：从 TCP 读取直到 `\r\n\r\n`
- 如果状态码是 2xx 则跳出外层循环（CONNECT 成功）
- 如果状态码是 1xx 则继续外层循环，读取下一个响应
- 如果状态码是其他值则返回错误
- 这修复了当代理将 interim 1xx（如 `100 Continue`）和最终 2xx 用独立 TCP segment 发送时的 bug

### WR-01: build_ssml 中 XML 控制字符未过滤

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 62d2df0
**Applied fix:** 在 `.replace('>', "&gt;")` 链后增加 `.chars().filter()` 调用，过滤掉 XML 1.0 禁止的控制字符（U+0000-U+0008、U+000B、U+000C、U+000E-U+001F，除 `\t`、`\n`、`\r` 外）。`escaped_text` 从隐式 `&str` 改为显式 `String` 类型标注以支持 `.collect()`。

### WR-02: 认证/连接错误分类逻辑在三个位置重复

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 4fea468
**Applied fix:** 提取了两个 `EdgeTtsEngine` 私有静态方法用于错误分类：
- `classify_connection_error()` -- 用于连接相关错误（映射到 `ConnectionFailed`）
- `classify_receive_error()` -- 用于消息接收错误（映射到 `SynthesisFailed`）
- 三个重复位置（connect() 代理路径、connect() 直连路径、synthesize_once() 接收路径）全部替换为方法调用

### IN-01: stt_message.into() 是多余的空操作

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 50ab532
**Applied fix:** 将 `.send(Message::Text(stt_message.into()))` 简化为 `.send(Message::Text(stt_message))`。`stt_message` 已经是 `String`，`Message::Text` 接受 `String`，`.into()` 是 identity 转换。

### IN-02: 测试模块中存在多余的类型导入

**Files modified:** `src/tts/mod.rs`
**Commit:** 80878cc
**Applied fix:** 移除了 `src/tts/mod.rs` 测试模块中的 `use std::path::Path;`。该导入已被 `use super::*;` 覆盖，后者从父模块导入了相同的 `use std::path::Path;`。

## Skipped Issues

None -- all 5 findings were fixed.

---

_Fixed: 2026-04-29T13:50:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
