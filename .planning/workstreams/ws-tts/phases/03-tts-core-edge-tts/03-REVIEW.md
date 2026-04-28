---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-28T08:00:00Z
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
  critical: 3
  warning: 2
  info: 2
  total: 7
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Report

**Reviewed:** 2026-04-28T08:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

审查了 TTS 核心抽象层（`TtsProvider` trait、路由器 `synthesize()`、数据类型）和 Edge-TTS 引擎的 Rust 实现。共发现 7 个问题，其中 3 个 BLOCKER。

最严重的问题是：**代理功能完全不可用**（`_proxy_url` 赋值后从未使用，`connect_async()` 直连），以及 **WordBoundary 事件在合成过程中未被解析**，导致 `word_boundaries` 始终为空。此外，路由器函数 `synthesize()` 硬编码了禁用代理的引擎实例，导致调用方无法启用代理。这些缺陷使分阶段交付的核心承诺（D-02、D-09、D-10）无法兑现。

## Critical Issues

### CR-01: 代理 URL 赋值但从未用于连接配置（代理功能完全不可用）

**File:** `src/tts/edge_tts.rs:103-113`
**Issue:** 当 `proxy_enabled = true` 时，变量 `_proxy_url` 被赋值但带有下划线前缀（抑制了未使用警告），然后 `connect_async(request)` 直接调用——这与直连分支的代码完全一致，没有任何代理配置生效。`tokio-tungstenite` 的 `connect_async` 不会自动读取系统代理或使用传入的代理地址；必须通过 `connect_async_with_config` 或自定义 `IntoClientRequest` 解析器来配置代理。当前代码使 D-02（WebSocket 连接支持代理）完全无法实现。

代理分支代码段：
```rust
let _proxy_url = if !self.proxy_https.is_empty() {
    &self.proxy_https  // 赋值但从未使用
} else { ... };

let (ws_stream, _) = connect_async(request)  // 直连，无视 proxy_url
    .await
    .map_err(|e| TTSError::ConnectionFailed(...))?;
```

**Fix:** 移除 `_` 前缀，并将代理地址传给 `tokio-tungstenite` 的代理连接器。示例修复方式（使用 `tungstenite` 的 ProxyConfig）：
```rust
use tokio_tungstenite::tungstenite::protocol::ProxyConfig;

let proxy_url = ...;
let mut request = ...;
// 方式一：通过 IntoClientRequest 的自定义实现或
// 方式二：使用 connect_async_with_config 传入 proxy
let config = tokio_tungstenite::Connector::new()
    .proxy(ProxyConfig::new_http(proxy_url));
let (ws_stream, _) = tokio_tungstenite::connect_async_with_config(request, Some(config))
    .await?;
```

---

### CR-02: WordBoundary 事件在合成循环中未被解析，word_boundaries 始终为空

**File:** `src/tts/edge_tts.rs:190-250`
**Issue:** `synthesize_once()` 的消息循环只处理了三种 `Path` 类型：`audio`、`turn.start`、`turn.end`。在实际的 Edge-TTS WebSocket 协议中，词边界事件的 `Path` 值为 `wordboundary`（小写），作为独立的二进制消息发送。当前代码将这些消息全部丢弃（落入 `_ => {}` 分支），导致 `word_boundaries` 向量始终为空。这意味着 D-09（词边界直接内嵌在 TtsOutput）和 D-10（流式接收 WordBoundary 事件）无法实现。Phase 6 的字幕生成将得不到任何词边界数据。

代码中只有 `turn.start` 处理分支尝试解析 wordboundary，但实际 Edge-TTS API 的 `turn.start` 元数据 JSON 中不包含完整的词边界列表——词边界是以独立消息流式推送的。

**Fix:** 添加对 `Path: wordboundary` 消息的处理：
```rust
} else if content.path == "wordboundary" {
    // 解析单个 WordBoundary JSON
    if let Ok(wb_json) = serde_json::from_str::<serde_json::Value>(
        &String::from_utf8_lossy(&content.payload),
    ) {
        if let (Some(offset), Some(duration), Some(text)) = (
            wb_json["offset"].as_u64(),
            wb_json["duration"].as_u64(),
            wb_json["text"].as_str(),
        ) {
            word_boundaries.push(WordBoundary {
                start_offset: offset,
                end_offset: offset + duration,
                text: text.to_string(),
            });
        }
    }
}
```
如有 `SentenceBoundary` 事件（`Path: sentenceboundary`），也应当类似处理。

---

### CR-03: `turn.end` 的 `audio_duration` 字段解析格式可能不匹配实际响应

**File:** `src/tts/edge_tts.rs:223-233`
**Issue:** 当前代码直接从 `turn.end` 的 JSON 中调用 `metadata["audio_duration"].as_f64()` 获取时长。但在 Edge-TTS 的实际响应中，`audio_duration` 字段可能以两种形式出现：
1. 字符串形式（如 `"281966000"`，单位为 100ns 刻度值）
2. 嵌套对象形式（如 `{"ticks": 281966000, "duration": 2.81966}`）

在这两种情况下，`as_f64()` 均返回 `None`，导致 `duration` 回退为 `0.0`。这意味着 `TtsOutput.duration` 将始终为 0.0，下游（如视频合成时计算音频时长）无法使用此值。

**Fix:** 检查实际响应格式，处理字符串和对象两种形态：
```rust
duration = if let Some(d) = metadata["audio_duration"].as_f64() {
    d
} else if let Some(s) = metadata["audio_duration"].as_str() {
    // 字符串 tick 值，转为秒
    s.parse::<f64>().unwrap_or(0.0) / 10_000_000.0
} else if let Some(ticks) = metadata["audio_duration"]["ticks"].as_u64() {
    ticks as f64 / 10_000_000.0
} else {
    0.0
};
```
注意：Edge-TTS 的 `audio_duration` 值单位是 100ns 刻度（ticks），需要除以 10,000,000 转换为秒。

## Warnings

### WR-01: 路由器函数 `synthesize()` 硬编码禁用代理

**File:** `src/tts/mod.rs:82-83`
**Issue:** 公有路由器函数硬编码了 `EdgeTtsEngine::new(false, String::new(), String::new())`，始终禁用代理。即使 CR-01 修复后，调用方通过 `tts::synthesize()` 也无法启用代理。D-02 明确要求"从 config.proxy 读取代理设置"，但路由器没有连接 Phase 1 的 `AppConfig.proxy`。

`src/config/types.rs` 中已存在 `ProxySection` 结构体（包含 `enabled`、`http`、`https` 字段），但 `synthesize()` 未导入任何配置。

**Fix:** 从调用方接收 `ProxySection` 或类似配置结构，或让 `synthesize()` 接受一个可选的代理参数：
```rust
pub async fn synthesize(
    engine: &str,
    text: &str,
    voice_name: &str,
    rate: f64,
    pitch: f64,
    output_path: &Path,
    proxy: Option<&crate::config::types::ProxySection>,  // 新增
) -> Result<TtsOutput, TTSError> {
    match engine {
        "edge_tts" => {
            let (proxy_enabled, proxy_http, proxy_https) = match proxy {
                Some(p) => (p.enabled, p.http.clone(), p.https.clone()),
                None => (false, String::new(), String::new()),
            };
            let edge_engine = EdgeTtsEngine::new(proxy_enabled, proxy_http, proxy_https);
            edge_engine.synthesize(text, voice_name, rate, pitch, output_path).await
        }
        _ => Err(TTSError::UnknownEngine { engine: engine.to_string() }),
    }
}
```

---

### WR-02: turn.start 收到后但未收到音频数据时，会写入空音频文件

**File:** `src/tts/edge_tts.rs:253-255`
**Issue:** 退出消息循环后的守卫条件是 `if audio_data.is_empty() && !is_turn_start`。如果收到 `turn.start`（`is_turn_start = true`）但之后未收到任何 `audio` 二进制消息（异常场景），条件判断为假，代码继续执行 `tokio::fs::write`，写入一个空文件，并返回 `Ok(TtsOutput{...})`。下游调用者会看到文件存在且状态为 OK，但音频文件时长为 0 且无内容。

**Fix:** 明确检查 `audio_data` 是否为空，并移除对 `is_turn_start` 的依赖：
```rust
if audio_data.is_empty() {
    return Err(TTSError::SynthesisFailed("未收到音频数据".to_string()));
}
```

## Info

### IN-01: 未使用变量 `_proxy_url`

**File:** `src/tts/edge_tts.rs:103`
**Issue:** `_proxy_url` 变量以下划线开头，虽然抑制了编译警告，但该变量从未在后续代码中使用。与 CR-01 关联，这是代理功能未实现残留在代码中的痕迹。同时导致同一分支中的 `self.proxy_http` 和 `self.proxy_https` 字段在整个函数中未被读取，任何 HTTP/HTTPS 代理配置都被忽略。

**Fix:** 在修复 CR-01 时移除该变量或正确使用它。

### IN-02: EdgeTtsEngine 所有字段为 `pub`

**File:** `src/tts/edge_tts.rs:60-62`
**Issue:** `proxy_enabled`、`proxy_http`、`proxy_https` 三个字段均为 `pub`，允许外部代码直接在构造后修改内部状态，破坏了封装性。在多线程上下文中，这种可变性可能导致意外行为或竞态。

**Fix:** 将字段设为 `pub(crate)` 或私有，通过 `new()` 构造器和 getter 方法访问：
```rust
pub struct EdgeTtsEngine {
    proxy_enabled: bool,
    proxy_http: String,
    proxy_https: String,
}
```

## Other Observations (No Action Required)

- `synthesize_once()` 方法中，`request_id` 使用 `uuid::Uuid::new_v4()` 生成。每次重试（包括相同 SSML 的重试尝试）都会生成新的 request ID，这是正确的，因为 Edge-TTS 服务端可能将 request ID 用于去重。
- `build_ssml()` 的 XML 转义正确处理了 `&`、`<`、`>`，但未转义 XML 中具有特殊意义的单引号 `'`（`&apos;`）和双引号 `"`（`&quot;`）。不过由于这些字符出现在 `text` 字符串中（而非 XML 属性值内），仅在 `prosody` 元素的内容体中出现，不进行转义仍可被 XML 解析器接受。这是可接受的简化。
- `src/error.rs` 中 `TTSError` 的 5 个变体和中文错误消息已正确实现，测试覆盖完整。
- `tests/tts_test.rs` 的集成测试覆盖了 `TtsOutput`、`WordBoundary`、错误消息及路由器函数签名，结构完整。

---

*Reviewed: 2026-04-28T08:00:00Z*
*Reviewer: Claude (gsd-code-reviewer)*
*Depth: standard*
