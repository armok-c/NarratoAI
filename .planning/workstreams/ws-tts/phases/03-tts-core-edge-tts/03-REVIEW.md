---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T16:45:00Z
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
  warning: 2
  info: 8
  total: 11
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**审查时间:** 2026-04-29T16:45:00Z
**深度:** standard
**审查文件数:** 6
**状态:** issues_found

## 摘要

对 Phase 03（TTS 核心抽象 + Edge-TTS 引擎）的当前源代码进行了 standard 深度审查。项目范围包括 `src/tts/mod.rs`（TtsProvider trait、synthesize 路由器）、`src/tts/edge_tts.rs`（Edge-TTS WebSocket 引擎）、`src/error.rs`（TTSError 定义）、`src/lib.rs`（模块声明）、`Cargo.toml`（依赖管理）和 `tests/tts_test.rs`（集成测试）。

**总览：** 共发现 11 个问题：1 个 BLOCKER（编译错误）、2 个 WARNING（重试策略和超时缺失）、8 个 INFO。关键问题 CR-01 导致项目无法通过编译，必须优先修复。此前审查报告中的所有问题均未修复。

## Critical Issues

### CR-01: `Message::Text(stt_message)` 类型不匹配导致编译失败

**File:** `src/tts/edge_tts.rs:410`

**Issue:** `stt_message` 是 `String` 类型，但 `tungstenite 0.29.0` 的 `Message::Text` 变体接受 `Utf8Bytes` 而非 `String`。`Utf8Bytes` 虽然实现了 `From<String>`，但 Rust 不会在枚举变体构造函数参数中进行隐式转换，因此直接传入 `String` 会触发类型不匹配编译错误。错误信息如下：

```
error[E0308]: mismatched types
   --> src\tts\edge_tts.rs:410:33
    |
410 |             .send(Message::Text(stt_message))
    |                   ------------- ^^^^^^^^^^^ expected `Utf8Bytes`, found `String`
```

编译器已自动附着修复建议。`cargo check` 确认该错误存在，项目当前无法通过编译。

**Fix:**
```rust
ws_stream
    .send(Message::Text(stt_message.into()))
    .await
    .map_err(|e| TTSError::ConnectionFailed(format!("发送 SSML 失败: {}", e)))?;
```

## Warnings

### WR-01: 重试逻辑未排除不可重试错误类型

**File:** `src/tts/edge_tts.rs:357-388`

**Issue:** `synthesize_with_retry` 对 `synthesize_once` 返回的所有错误类型一律执行 3 次重试（每次间隔 1 秒），未利用 `classify_error`（第 345-352 行）已识别的错误分类信息。以下两类错误是不可重试的：

- `TTSError::AuthenticationFailed`：WebSocket 认证失败（`TrustedClientToken` 无效或过期）。重试不会改变凭据，每次重试浪费约 1+ 秒并产生无用网络流量。
- `TTSError::ConnectionFailed` 中由 URL 格式错误、TLS 配置错误等永久性缺陷引起的错误亦无重试意义。

虽然当前 `TrustedClientToken` 是硬编码且长期稳定的常量，认证失败概率低，但此设计缺陷在处理代理认证失败或将来支持可配置 token 时会导致不必要的延迟。

**Fix:** 在 `Err(e)` 匹配分支中对不可重试的错误短路：
```rust
Err(e) => {
    if matches!(&e, TTSError::AuthenticationFailed(_)) {
        return Err(e);
    }
    tracing::warn!("Edge-TTS 合成尝试 {} 失败: {}", attempt, e);
    last_error = Some(e);
    if attempt < max_attempts {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

### WR-02: `connect()` 函数缺少超时保护

**File:** `src/tts/edge_tts.rs:119-342`

**Issue:** `connect()` 函数在执行 HTTP CONNECT 隧道握手时没有任何超时保护。具体风险包括：

1. **CONNECT 响应读取循环（第 257-313 行）：** `tcp.read()` 在代理不回复或回复不完整时可能永久阻塞。当前仅 `synthesize_once` 中有 `WS_RECEIVE_TIMEOUT`（120 秒，第 11 行），但此超时只覆盖 WebSocket 建立后的消息接收阶段，不覆盖 CONNECT 握手阶段。
2. **TLS 握手（第 316-331 行）：** `native_tls::TlsConnector` 的握手操作也没有超时保护。如果代理通过 CONNECT 建立了隧道但对端 TLS 服务器无响应，`client_async_tls_with_config` 会无限阻塞。

虽然 `TcpStream::connect` 本身有操作系统级的 TCP 超时（通常 20-120 秒），但 CONNECT 阶段的 HTTP 响应读取和 TLS 握手不受此保护。

**Fix:** 在 `connect()` 整体或关键阶段添加超时：
```rust
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

async fn connect(&self) -> Result<..., TTSError> {
    tokio::time::timeout(CONNECT_TIMEOUT, async {
        // ... 现有 connect 逻辑 ...
    })
    .await
    .map_err(|_| TTSError::ConnectionFailed("代理 CONNECT 超时 (30s)".to_string()))?
}
```

## Info

### IN-01: `notify` 依赖为 RC 版本

**File:** `Cargo.toml:14`

**Issue:** `notify = "9.0.0-rc.3"` 为发布候选版本。`Cargo.lock` 将固定此 RC 版本，但正式版 9.0.0 发布后可能引入 API 破坏性变更。`Cargo.toml` 中已有注释说明此风险，建议在正式版发布后升级并验证兼容性。

### IN-02: `connect()` 函数复杂度过高

**File:** `src/tts/edge_tts.rs:119-342`（约 220 行）

**Issue:** `connect()` 函数包含以下可提取为独立函数的子逻辑：
- 代理 URL 解析（方案、主机、端口、凭据提取）—— 第 153-199 行
- 目标 WSS URL 解析（主机、端口）—— 第 201-210 行
- HTTP CONNECT 请求构建 —— 第 214-225 行
- CONNECT 响应读取与解析 —— 第 257-313 行

当前函数嵌套两层 `loop`（第 257、259 行），包含多处 `continue` 和条件分支，可维护性较低。提取为命名辅助函数后有利于单元测试和审查。

### IN-03: 签名测试使用 Unix 路径

**File:** `tests/tts_test.rs:113`

**Issue:** 编译期签名测试使用 `Path::new("/tmp/_narratoai_sig_check.mp3")`。当前代码通过 `let _` 丢弃返回的 Future 而不 poll，因此无实际文件操作。但如果未来有人在该测试上添加 `.await` 或将其改为集成测试，`/tmp/` 路径在 Windows 环境会失败。测试已有注释说明此约束，但路径本身仍然是平台假设。

### IN-04: 未处理的二进制消息路径被静默忽略

**File:** `src/tts/edge_tts.rs:432-496`

**Issue:** 在 `synthesize_once` 的二进制消息分发链中，`if-else if` 链覆盖了 `audio`、`turn.start`、`wordboundary`、`turn.end` 四个路径。Edge TTS 服务可能发送 `SentenceBoundary` 等其他路径类型（Python edge-tts 库的 `SubMaker` 会消费 `SentenceBoundary` 事件），当前代码让这些路径无声地通过整个 `if-else` 链。建议在链末尾添加兜底日志以捕获新增或未知路径：

```rust
} else {
    tracing::debug!("Edge-TTS 未处理的路径类型: {}", content.path);
}
```

### IN-05: 词边界时间单位测试为同义反复

**File:** `tests/tts_test.rs:80-91`

**Issue:** `test_word_boundary_time_unit_semantics` 测试声明常量 `50_000_000u64` 并验证其自减等于自身。此测试仅验证 Rust 编译器的 u64 算术运算正确性（编译器保证），有文档价值但无实际验证作用。可考虑替换为更有意义的测试（如时间单位转换的边界值验证）。

### IN-06: `voice_name_to_lang` 对缺少第二连字符的音色名返回空串

**File:** `src/tts/edge_tts.rs:61-71`

**Issue:** 当 `voice_name` 不符合 `xx-XX-VoiceName` 格式（少于两个连字符）时，`voice_name_to_lang` 返回空字符串。此空串被直接填入 SSML 的 `xml:lang=""` 属性。虽然 `synthesize` 已在第 593 行校验 `voice_name` 非空，但格式不合规的音色名（如 `"XiaoyiNeural"`）仍会通过校验并生成含 `xml:lang=""` 的 SSML。

标准 Edge TTS 音色名均遵循 `locale-SpeakerName` 格式（如 `zh-CN-XiaoyiNeural`），因此实际影响有限。降低为 INFO 级。

### IN-07: 重试耗尽后无条件删除输出文件

**File:** `src/tts/edge_tts.rs:381`

**Issue:** 在所有重试尝试失败后，`synthesize_with_retry` 无条件执行 `tokio::fs::remove_file(output_path)` 以清理可能的损坏输出文件：

```rust
let _ = tokio::fs::remove_file(output_path).await;
```

该操作未区分以下两种情况：
- 文件是由 TTS 合成的部分输出创建的（应清理）
- 文件在 TTS 调用之前就已存在（不应删除）

如果调用者传入一个已存在的重要文件路径作为 `output_path`，TTS 失败后该文件会被删除。虽然典型用法中调用者会传入新文件路径，且成功路径上 `tokio::fs::write` 也会覆盖该文件，但当前行为存在数据丢失风险。建议在移除前检查文件是否在本次调用中实际被创建，或由调用者负责清理。

### IN-08: rate 校验错误消息表述不精确

**File:** `src/tts/edge_tts.rs:597-598`

**Issue:** 参数校验中的错误消息为 `"rate 必须为有限正数: {}"`，但 `rate=0.0` 是允许值（通过 `rate < 0.0` 检查且 `is_finite()` 为真）。数学上 0 不是正数，消息应改为 `"rate 必须为有限非负数"` 以与校验逻辑一致。此问题不影响功能正确性。

---

**此前审查迭代（Iteration 2, 2026-04-29T16:30:00Z）报告的问题状态：**

以下问题在本次审查中确认仍未修复：

| ID | 标题 | 当前状态 |
|-----|-------|---------|
| CR-01 | `Message::Text` 类型不匹配 | 未修复（编译失败） |
| WR-01 | 重试逻辑未排除不可重试错误 | 未修复 |
| WR-02 | `connect()` 缺少超时 | 未修复 |
| IN-01 | `notify` RC 版本 | 未修复 |
| IN-02 | `connect()` 复杂度过高 | 未修复 |
| IN-03 | 签名测试 Unix 路径 | 未修复 |
| IN-04 | 未处理二进制消息路径 | 未修复 |
| IN-05 | 时间单位测试同义反复 | 未修复 |
| IN-06 | `voice_name_to_lang` 返回空串 | 未修复 |
| IN-07 | 重试耗尽后无条件删除文件 | 新增 |
| IN-08 | rate 校验错误消息不精确 | 新增 |

`src/error.rs` 中的 `From<OpenAIError>` 启发式分类已通过提交 `83d9027` 修复（`lower.contains("key")` 和 `lower.contains("auth")` 已替换为 `"authentication"`、`"unauthorized"`、`"invalid api key"` 等更精确的匹配），不再列为问题。

---

_审查时间: 2026-04-29T16:45:00Z_
_审查者: Claude (gsd-code-reviewer)_
_深度: standard_
