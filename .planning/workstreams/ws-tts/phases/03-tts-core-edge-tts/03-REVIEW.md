---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-28T14:30:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/tts/mod.rs
  - src/error.rs
  - src/lib.rs
  - Cargo.toml
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
findings:
  critical: 1
  warning: 6
  info: 5
  total: 12
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine - Code Review Report

**Reviewed:** 2026-04-28T14:30:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

对 Phase 03 的 6 个文件进行了标准深度审查。审查范围包括 TTS 核心模块（`WordBoundary`、`TtsOutput`、`TtsProvider` trait、`synthesize()` 路由器）、Edge-TTS 引擎实现（WebSocket 连接、SSML 生成、代理支持、二进制消息解析、3 次重试逻辑）、错误类型定义和集成测试。

上一轮审查报告的 8 个警告（WR-01 至 WR-08）已被确认修复，包括：代理 CONNECT 的 interim 1xx 支持、重试语义修正、pitch 函数重命名、turn.end/duration 区分、WebSocket 超时保护、代理认证凭据支持、xml:lang 动态提取和 saturating_add 溢出保护。

本次审查发现 **1 个关键问题**和 **6 个警告**。关键问题为 `voice_name_to_lang()` 函数中字符索引与字节索引混用，对包含多字节 UTF-8 字符的语音名称会导致运行时 panic。警告包括未使用的错误变体、提前关闭返回 duration=0.0、变量命名误导、测试代码重复和可见度过宽等问题。

## Critical Issues

### CR-01: voice_name_to_lang 使用字符索引代替字节索引，在多字节字符语音名称上导致 panic

**File:** `src/tts/edge_tts.rs:50-61`
**Issue:** `voice_name_to_lang()` 使用 `voice_name.chars().enumerate()` 遍历字符，`enumerate()` 返回的是**字符索引**（第几个 Unicode 标量值）。但后续使用 `voice_name[..idx]` 进行**字节索引**字符串切片。Rust 的字符串切片要求索引必须落在 UTF-8 字符边界上。

当语音名称包含非 ASCII 字符时（如中文语音名称或带重音符号的拉丁字母），字符索引与字节索引不一致。导致：
1. **运行时 panic**：如果 `idx` 落在多字节字符中间，Rust 字符串切片直接 panic。
2. **错误结果**：即使不 panic，切片也切出错误内容（缺少或多余字符）。

示例：假设语音名称为 `"cafe-FR-HenriNeural"`（法语语音），使用 `é`（2 字节）替换第一个 `e`。字符序列为 `c(0) a(1) f(2) e-acute(3) -(4) F(5) R(6) -(7)`，第二个 `-` 的字符索引为 7。但字节序列为 `c(0) a(1) f(2) e-acute-b1(3) e-acute-b2(4) -(5) F(6) R(7) -(8)`，第二个 `-` 的字节索引为 8。`voice_name[..7]` 切到字节 7（`R` 的内部），产生 panic 或错误的 `"cafe-FR"`（缺少第二个 `-`）。

当前标准 Edge-TTS 语音名称（`zh-CN-XiaoyiNeural`、`en-US-JennyNeural`、`ja-JP-NanamiNeural`）均为纯 ASCII，因此该 bug 在当前输入下**潜伏未触发**。但一旦遇到非 ASCII 语音名称（如未来新增的法语 `fr-FR-DeniseNeural` 等）就会崩溃。

**Fix:** 使用 `char_indices()` 替代 `chars().enumerate()`。`char_indices()` 直接返回 `(byte_index, char)` 元组，确保 `idx` 是合法的字节索引。

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

### WR-01: TTSError::AuthenticationFailed 在 Edge-TTS 引擎中从未被使用

**File:** `src/error.rs:57-58`
**Issue:** `TTSError` 定义了 `AuthenticationFailed(String)` 变体，但 Edge-TTS 引擎在 `connect()` 和 `synthesize_once()` 中将所有网络相关错误都映射为 `ConnectionFailed(...)`。即使微软 TTS 服务返回 HTTP 401（认证失败）或 403（禁止访问），调用方也只能看到统一的"连接失败"消息，无法区分是网络问题还是认证问题。这降低了错误诊断能力。

在 `Cargo.lock` 中确认 tokio-tungstenite 0.29.0 的 `Connector::NativeTls` 接受 `native_tls_crate::TlsConnector`（即 `native_tls::TlsConnector`），类型匹配正确。TLS 握手失败时 tungstenite 会返回 `Error::Tls`，带有 `native_tls::Error` 内部错误，但代码将其统一映射为 `ConnectionFailed`。

**Fix:** 在 `connect()` 和 `synthesize_once()` 中，当 tungstenite 返回的错误可以识别为认证错误时，映射到 `TTSError::AuthenticationFailed(...)`：

```rust
// 在 synthesize_once 中：
.map_err(|e| {
    let err_str = e.to_string();
    if err_str.contains("401") || err_str.contains("authentication") || err_str.contains("Unauthorized") {
        TTSError::AuthenticationFailed(err_str)
    } else {
        TTSError::SynthesisFailed(format!("接收消息失败: {}", e))
    }
})?
```

### WR-02: WebSocket 在 turn.end 前关闭导致 duration=0.0 被当作成功返回

**File:** `src/tts/edge_tts.rs:326-431`
**Issue:** `synthesize_once()` 的消息接收循环在两种情况下退出：
1. 正常：收到 `turn.end` 消息后 `break`，`duration` 正常解析
2. 异常：`ws_stream.next()` 返回 `None`（连接关闭）或超时

在异常情况下，`received_turn_end` 保持 `false`，`duration` 保持 `0.0`（默认值）。但代码在第 420-424 行仍然写入已收到的部分音频数据，并在第 426 行返回 `Ok(TtsOutput { duration: 0.0, ... })`。这导致：
- 调用方收到 `Ok` 但 `duration` 为 0，如果下游用 `duration` 做除数或计算比特率，会产生无穷大/NaN。
- 调用方无法区分"完整成功"和"部分音频+连接中断"。

当前第 406-418 行的警告日志只输出 warning 不改变控制流。`received_turn_end` 标志位虽然在第 364 行设为 true，但检查在第 407 行 `duration == 0.0` 时只在日志中区分场景，仍然返回 `Ok`。

**Fix:** 当 `received_turn_end` 为 `false` 时，应返回 `Err` 而非 `Ok`：

```rust
// 替代第 406-418 行的 duration 检查
if !received_turn_end {
    return Err(TTSError::SynthesisFailed(
        "WebSocket 连接在收到 turn.end 前关闭，音频数据不完整".to_string(),
    ));
}
// duration == 0.0 但 received_turn_end 为 true 的情况可以保留 warning
```

### WR-03: max_retries 变量命名与实际语义不符（误导维护者）

**File:** `src/tts/edge_tts.rs:272`
**Issue:** `let max_retries = 4;  // 1 initial + 3 retries (D-03: 3 次重试)` 变量名 `max_retries` 暗示"最大重试次数为 4"，但实际语义是"总尝试次数为 4"（1 次首次尝试 + 3 次重试）。第 291 行的错误消息用 `max_retries - 1` 来计算实际重试次数，说明代码作者也知道命名不一致。

这种命名在将来修改时容易引入 off-by-one bug：假设有人看到 `let max_retries = 3` 以为"只重试 3 次"，但循环 `1..=3` 只做 2 次重试（1 次初始 + 2 次重试）。

**Fix:** 将变量重命名以清晰表达语义：

```rust
let max_attempts = 4; // 1 initial + 3 retries
for attempt in 1..=max_attempts {
    // ...
    if attempt < max_attempts {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
Err(TTSError::RetryExhausted(format!(
    "Edge-TTS 重试 {} 次后仍失败: {}",
    max_attempts - 1,
    last_error.map_or("未知错误".to_string(), |e| e.to_string())
)))
```

### WR-04: 库测试与集成测试存在重复测试用例（翻倍维护成本）

**Files:**
- `src/tts/mod.rs:136-172`（`test_synthesize_unknown_engine`、`test_synthesize_unknown_engine_message`）
- `tests/tts_test.rs:79-114`（`test_synthesize_unknown_engine_error`、`test_synthesize_unknown_engine_message_contains_name`）

**Issue:** 两组测试对 `tts::synthesize()` 路由器的未知引擎分支执行完全相同的验证：断言返回 `Err(TTSError::UnknownEngine {..})`，断言错误消息包含引擎名。差异仅仅是使用的引擎名字面量不同（`"nonexistent_engine"` vs `"invalid_engine"`、`"bad_engine"` vs `"bad_engine_name"`），验证逻辑和断言完全相同。

当路由器的错误处理逻辑改变时（例如修改错误消息格式），两组测试需要同步修改。这增加了维护成本且无额外覆盖收益。

**Fix:** 移除 `tests/tts_test.rs` 中的 `test_synthesize_unknown_engine_error` 和 `test_synthesize_unknown_engine_message_contains_name`，保留 `src/tts/mod.rs` 中的版本即可覆盖。`tts_test.rs` 专注测试集成级别的场景。

### WR-05: test_synthesize_function_signature 丢弃未 poll 的 async future（编译器警告）

**File:** `tests/tts_test.rs:146-148`
**Issue:** `let _ = tts::synthesize(...)` 创建了 `synthesize()` 返回的 async future 但立即通过 `let _ =` 丢弃，从未被 `.await` 或 poll。Rust 中 `Future` 实现了 `#[must_use]`，编译器会对丢弃未 poll 的 future 产生警告。clippy 的 `let_underscore_future` lint 也会报告此问题。

未来如果在 `synthesize()` 中添加了带副作用的初始化逻辑（例如创建临时文件），该测试不会捕获这些副作用，但仍然"通过"编译。

**Fix:** 用类型检查闭包替代函数调用，避免实际创建 future：

```rust
#[test]
fn test_synthesize_function_signature() {
    fn assert_send<T: Send>(_t: T) {}
    // 仅验证函数签名（编译时检查），不创建 future
    assert_send(
        tts::synthesize("", "", "", 1.0, 0.0, std::path::Path::new(""), None)
    );
}
```

或者保留原有形式但显式消音 clippy 警告：

```rust
#[allow(clippy::let_underscore_future)]
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None);
}
```

### WR-06: EdgeTtsEngine 和 new() 的 pub 可见度过于宽松

**File:** `src/tts/edge_tts.rs:83-87, 90`
**Issue:** `EdgeTtsEngine` 结构体及其 `new()` 方法被标记为 `pub`，允许外部 crate 直接构造引擎实例。这绕过了 `tts::synthesize()` 路由器的代理配置逻辑和引擎名校验。随着 Phase 3+ 新增更多 TTS 引擎（Azure、Tencent 等），外部代码直接实例化引擎将破坏路由器的一致性。

**Fix:** 将可见度降级为 `pub(super)`（仅对父模块 `tts` 可见）：

```rust
pub(super) struct EdgeTtsEngine {
    pub(super) proxy_enabled: bool,
    pub(super) proxy_http: String,
    pub(super) proxy_https: String,
}

impl EdgeTtsEngine {
    pub(super) fn new(...) -> Self { ... }
}
```

## Info

### IN-01: 多余的 `.into()` 转换

**File:** `src/tts/edge_tts.rs:317`
**Issue:** `stt_message` 已经是 `String`，`Message::Text` 接收 `String`，`.into()` 是空操作（`String` -> `String` 的 identity 转换）。

### IN-02: `and_then` 应改为 `map`

**File:** `src/tts/edge_tts.rs:166-167`
**Issue:** `Option::and_then(|(h, p)| Some(...))` 的闭包总是返回 `Some`，语义上等同于 `Option::map`。使用 `map` 更清晰地表达了"转换内部值"的意图。

### IN-03: EDGE_TTS_WSS_URL 硬编码公开令牌

**File:** `src/tts/edge_tts.rs:14-15`
**Issue:** `EDGE_TTS_WSS_URL` 包含硬编码的 `TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4`。虽然这是公开令牌（来自 Edge 浏览器扩展，广泛公开），但微软可能轮换此令牌，届时需要代码修改才能恢复服务。建议提取为常量并添加来源说明，或在未来通过配置开放。

### IN-04: `unwrap()` 无 `expect()` 说明安全性原因

**File:** `src/tts/edge_tts.rs:113-122`
**Issue:** 两处 `"string".parse().unwrap()` 调用使用 `.unwrap()` 但不附带任何解释。虽然硬编码的字符串字面量保证有效，但使用 `.expect("...")` 可以明确告知读者"此 unwrap 不会失败"的原因。

### IN-05: 版本测试绑定硬编码字符串

**File:** `src/lib.rs:17-19`
**Issue:** `assert_eq!(version(), "0.1.0")` 在版本号变化时需要同步修改两个位置。可以直接与编译时常量对比以避免遗忘：

```rust
#[test]
fn test_version_matches_cargo_toml() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}
```

### IN-06: TTSError 中文消息测试在集成测试中与单元测试重叠

**Files:** `tests/tts_test.rs:57-71` vs `src/error.rs:189-221`
**Issue:** `test_tts_error_all_variants_chinese` 验证所有 `TTSError` 变体的 Display 包含中文。`src/error.rs` 中已有 5 个独立的单元测试对每个变体逐一验证。集成测试的合并遍历没有增加覆盖价值。

---

_Reviewed: 2026-04-28T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

## Appendix: Previous Review Resolution Status

| Previous ID | Issue | Fixed in current code? |
|---|---|---|
| WR-01 | Proxy CONNECT no 1xx support | 已修复（第 193-230 行） |
| WR-02 | Retry semantics 3 vs 2 | 已修复（max_retries=4） |
| WR-03 | convert_pitch_to_percent naming | 已修复（函数已重命名） |
| WR-04 | turn.end vs duration log confusion | 已修复（received_turn_end 标志位） |
| WR-05 | No timeout on WS receive | 已修复（WS_RECEIVE_TIMEOUT） |
| WR-06 | Proxy URL with credentials | 已修复（rsplit('@') 处理） |
| WR-07 | xml:lang hardcoded to zh-CN | 已修复（voice_name_to_lang 函数） |
| WR-08 | u64 overflow on word boundary | 已修复（saturating_add） |
