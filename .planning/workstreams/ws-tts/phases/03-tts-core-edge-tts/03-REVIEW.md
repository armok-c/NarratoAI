---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T14:30:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/lib.rs
  - src/tts/edge_tts.rs
  - src/tts/mod.rs
  - tests/tts_test.rs
findings:
  critical: 1
  warning: 6
  info: 5
  total: 12
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Re-review Report (Iteration 2)

**Reviewed:** 2026-04-29T14:30:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

对 Phase 03 的 6 个文件进行了标准深度再审查。

**关键发现：之前审查（03-REVIEW.md）报告的所有 7 个修复（1 个关键 + 6 个警告）均已被回退。** 提交 `879c980`（"docs(03): add code review fix report"）在添加固定报告文档的同时，对 `src/tts/edge_tts.rs` 和 `tests/tts_test.rs` 中的 85 行和 41 行分别进行了更改，实际上将所有 7 个修复恢复到了修复前的错误状态。当前工作树与修复提交（`bbaaba9`、`6413cac`、`ac579db`、`479062a`、`fa67b5c`、`473e402`、`8db599b`）之前的状态无法区分。

重新出现的错误包括：
1. 关键：`voice_name_to_lang()` 使用字符索引作为字节索引 -- 在非 ASCII 语音名称上出现恐慌（CR-01）
2. 警告：连接/合成中缺少认证错误检测（WR-01）
3. 警告：在 `turn.end` 之前关闭 WebSocket 返回 `Ok` 且 duration=0.0（WR-02）
4. 警告：`max_retries` 变量命名与语义不符（WR-03）
5. 警告：`tests/tts_test.rs` 中存在重复测试函数（WR-04）
6. 警告：`test_synthesize_function_signature` 中缺少 clippy lint 抑制（WR-05）
7. 警告：`EdgeTtsEngine` 公开可见性过于宽松（WR-06）

未发现新的问题。当前状态与上次审查完全相同。

## Critical Issues

### CR-01: voice_name_to_lang 使用字符索引代替字节索引，在多字节字符语音名称上导致 panic（重新出现）

**文件：** `src/tts/edge_tts.rs:50-61`
**根本原因：** 提交 `879c980` 回退了 `bbaaba9` 的修复。`voice_name_to_lang()` 仍然使用 `voice_name.chars().enumerate()` 返回字符索引，但随后使用 `voice_name[..idx]` 进行字节级别的字符串切片。当语音名称包含非 ASCII 字符时，这会导致运行时恐慌。
**当前代码：**
```rust
fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some(idx) = voice_name
        .chars()           // <-- 返回字符索引
        .enumerate()
        .filter(|(_, c)| *c == '-')
        .nth(1)
        .map(|(i, _)| i)   // <-- idx 是字符索引
    {
        voice_name[..idx]  // <-- 需要字节索引 -- 如果 idx 落在多字节字符中间，则会出现恐慌
            .to_string()
    } else {
        "zh-CN".to_string()
    }
}
```
**修复：** 使用 `char_indices()` 返回字节索引：
```rust
fn voice_name_to_lang(voice_name: &str) -> String {
    if let Some((idx, _)) = voice_name
        .char_indices()
        .filter(|(_, c)| *c == '-')
        .nth(1)
    {
        voice_name[..idx].to_string()
    } else {
        "zh-CN".to_string()
    }
}
```

## Warnings

### WR-01: TTSError::AuthenticationFailed 在 Edge-TTS 引擎中从未被使用（重新出现）

**文件：** `src/error.rs:57-58`，`src/tts/edge_tts.rs:109,174,186,188,195,224,235,253,258,319,329`
**根本原因：** 提交 `879c980` 回退了 `6413cac` 的修复。`connect()` 和 `synthesize_once()` 中的所有错误映射都将网络错误统一转换为 `ConnectionFailed(...)` 或 `SynthesisFailed(...)`，从不使用 `AuthenticationFailed` 变体。微软 TTS 返回的 401/403 错误无法诊断。
**修复：** 在 tungstenite 错误字符串包含 "401"、"authentication" 或 "Unauthorized" 的三个位置（直接连接路径、代理连接路径、消息接收路径）添加认证错误检测，并映射到 `TTSError::AuthenticationFailed(err_str)`。

### WR-02: WebSocket 在 turn.end 前关闭导致 duration=0.0 被当作成功返回（重新出现）

**文件：** `src/tts/edge_tts.rs:406-418`
**根本原因：** 提交 `879c980` 回退了 `ac579db` 的修复。当 `received_turn_end` 为 `false` 时，代码不会返回 `Err`，而是继续写入文件并返回 `Ok(TtsOutput { duration: 0.0, ... })`。调用方无法区分"完整成功"和"连接在 turn.end 之前关闭的音频不完整"。
**当前代码：**
```rust
if duration == 0.0 {
    if received_turn_end {
        tracing::warn!(...);
    } else {
        tracing::warn!(...);
    }
}
// 始终继续写入文件并返回 Ok
```
**修复：** 当 `received_turn_end` 为 `false` 时返回 `Err`，仅在 `received_turn_end` 为 `true` 但 `duration` 无法解析时保留 `duration == 0.0` 警告：
```rust
if !received_turn_end {
    return Err(TTSError::SynthesisFailed(
        "WebSocket 连接在收到 turn.end 前关闭，音频数据不完整".to_string(),
    ));
}
if duration == 0.0 {
    tracing::warn!(...);
}
```

### WR-03: max_retries 变量命名与实际语义不符（误导维护者）（重新出现）

**文件：** `src/tts/edge_tts.rs:272`
**根本原因：** 提交 `879c980` 回退了 `479062a` 的修复。变量名仍为 `max_retries`（暗示"最大重试次数"），而其值 4 包括首次尝试（1 次初始 + 3 次重试）。错误消息使用 `max_retries - 1`，表明开发者自己也意识到命名不一致。
**当前代码：**
```rust
let max_retries = 4; // 1 initial + 3 retries (D-03: 3 次重试)
```
**修复：** 重命名为 `max_attempts` 以清晰表明总尝试次数。

### WR-04: 库测试与集成测试存在重复测试用例（翻倍维护成本）（重新出现）

**文件：** `tests/tts_test.rs:77-114`
**根本原因：** 提交 `879c980` 回退了 `fa67b5c` 的修复。`test_synthesize_unknown_engine_error` 和 `test_synthesize_unknown_engine_message_contains_name` 已被放回 `tests/tts_test.rs`，与 `src/tts/mod.rs:136-172` 中的相同测试重复。逻辑完全相同，仅引擎名字面量不同。
**修复：** 从 `tests/tts_test.rs` 中移除这两个测试函数。`src/tts/mod.rs` 中的版本提供了相同的覆盖范围。

### WR-05: test_synthesize_function_signature 丢弃未 poll 的 async future（编译器警告）（重新出现）

**文件：** `tests/tts_test.rs:145-148`
**根本原因：** 提交 `879c980` 回退了 `473e402` 的修复。缺少 `#[allow(clippy::let_underscore_future)]` 属性。
**当前代码：**
```rust
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None);
}
```
**修复：** 添加 clippy 属性：
```rust
#[allow(clippy::let_underscore_future)]
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None);
}
```

### WR-06: EdgeTtsEngine 和 new() 的 pub 可见度过于宽松（重新出现）

**文件：** `src/tts/edge_tts.rs:83-96`
**根本原因：** 提交 `879c980` 回退了 `8db599b` 的修复。`EdgeTtsEngine` 结构体、其所有字段以及 `new()` 构造函数都是 `pub` 的，允许外部 crate 绕过 `tts::synthesize()` 路由器直接构造引擎实例。
**当前代码：**
```rust
pub struct EdgeTtsEngine {
    pub proxy_enabled: bool,
    pub proxy_http: String,
    pub proxy_https: String,
}

impl EdgeTtsEngine {
    pub fn new(proxy_enabled: bool, proxy_http: String, proxy_https: String) -> Self {
```
**修复：** 将可见度降低为 `pub(super)`，将实例化限制在 `tts` 模块内。

## Info

### IN-01: 多余的 `.into()` 转换

**文件：** `src/tts/edge_tts.rs:317`
**说明：** `stt_message` 已经是 `String`，`Message::Text` 接收 `String`，`.into()` 是空操作。

### IN-02: `and_then` 应改为 `map`

**文件：** `src/tts/edge_tts.rs:166-167`
**说明：** `Option::and_then(|(h, p)| Some(...))` 的闭包总是返回 `Some`，语义上等同于 `Option::map`。

### IN-03: EDGE_TTS_WSS_URL 硬编码公开令牌

**文件：** `src/tts/edge_tts.rs:14-15`
**说明：** URL 包含硬编码的 `TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4`。虽然这是公开令牌，但微软可能轮换此令牌，届时需要代码修改才能恢复服务。

### IN-04: `unwrap()` 无 `expect()` 说明安全性原因

**文件：** `src/tts/edge_tts.rs:113,119`
**说明：** 两处 `"string".parse().unwrap()` 调用使用 `.unwrap()` 但不附带任何解释。使用 `.expect("...")` 可以明确告知读者此 unwrap 不会失败。

### IN-05: 版本测试绑定硬编码字符串

**文件：** `src/lib.rs:17-19`
**说明：** `assert_eq!(version(), "0.1.0")` 在版本号变化时需要同步修改两个位置。可以对比编译时常量：

```rust
#[test]
fn test_version_matches_cargo_toml() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
```

## Appendix: Fix Status Audit

此再审查验证了之前审查（03-REVIEW.md）中报告的 7 个已识别修复的当前状态。

| ID | 修复提交 | 回退于提交 | 当前状态 |
|---|---|---|---|
| CR-01 | `bbaaba9` | `879c980` | **未修复 -- 回退** |
| WR-01 | `6413cac` | `879c980` | **未修复 -- 回退** |
| WR-02 | `ac579db` | `879c980` | **未修复 -- 回退** |
| WR-03 | `479062a` | `879c980` | **未修复 -- 回退** |
| WR-04 | `fa67b5c` | `879c980` | **未修复 -- 回退** |
| WR-05 | `473e402` | `879c980` | **未修复 -- 回退** |
| WR-06 | `8db599b` | `879c980` | **未修复 -- 回退** |

提交 `879c980`（"docs(03): add code review fix report"）在编辑固定报告文档的同时，对 `src/tts/edge_tts.rs`（85 行更改）和 `tests/tts_test.rs`（41 行更改）中的源代码进行了实质性更改。这些更改回退了先前 7 个修复提交的所有效果，使代码库恢复到修复前的错误状态。

---

_Reviewed: 2026-04-29T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
