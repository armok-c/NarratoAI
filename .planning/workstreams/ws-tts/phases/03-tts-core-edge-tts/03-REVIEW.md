---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T16:30:00Z
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
  info: 5
  total: 8
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Report

**审查时间:** 2026-04-29T16:30:00Z
**深度:** standard
**审查文件数:** 6
**状态:** issues_found

## 摘要

对 Phase 03（TTS 核心抽象 + Edge-TTS 引擎）的当前源代码进行了标准深度审查。此前审查迭代（Iteration 2, 03-REVIEW.md 旧版）报告的 13 个问题中，绝大多数已通过 `git log` 确认的修复提交得到解决，包括 WR-06（CONNECT 响应数据丢失）、WR-07（Path 头大小写匹配）、WR-08（voice_name 空值校验）以及多项 INFO 级问题均已修复。

本报告基于 **当前代码状态** 进行审查，仅报告当前代码中仍然存在的问题。

**核心发现：** 项目存在 1 个 BLOCKER 编译错误——`Message::Text(stt_message)` 在 tungstenite 0.29.0 中存在类型不匹配，导致整个项目无法通过编译。此外存在 2 个 WARNING（重试策略未排除不可重试错误、`connect()` 缺少超时机制）和 5 个 INFO 级问题。

## Critical Issues

### CR-01: `Message::Text` 类型不匹配导致编译失败

**File:** `src/tts/edge_tts.rs:410`
**Issue:** 将 `stt_message`（`String` 类型）直接传入 `Message::Text()`，但 `tungstenite 0.29.0` 的 `Message::Text` 变体期望 `Utf8Bytes` 而非 `String`。这是 tungstenite 从 0.28.x 升级到 0.29.0 时的 API 变更。编译错误信息如下：

```
error[E0308]: mismatched types
   --> src\tts\edge_tts.rs:410:33
    |
410 |             .send(Message::Text(stt_message))
    |                   ------------- ^^^^^^^^^^^ expected `Utf8Bytes`, found `String`
```

编译器已自动附着修复建议。

**Fix:** 在第 410 行添加 `.into()` 转换：

```rust
ws_stream
    .send(Message::Text(stt_message.into()))
    .await
    .map_err(|e| TTSError::ConnectionFailed(format!("发送 SSML 失败: {}", e)))?;
```

## Warnings

### WR-01: 重试逻辑未排除不可重试错误类型

**File:** `src/tts/edge_tts.rs:357-388`
**Issue:** `synthesize_with_retry` 对 `synthesize_once` 返回的 **所有** 错误一律执行 3 次重试（每次间隔 1 秒）。但 `TTSError` 中包含不可重试的错误变体：

- `AuthenticationFailed`：认证凭据无效时，无论如何重试都会失败
- 每当尝试建立 WebSocket 连接时都会重新触发认证失败，每次尝试浪费数秒

虽然当前 `TrustedClientToken` 是硬编码且长期稳定的，但此设计缺陷会在将来支持可配置 token 时显现。错误分类函数 `classify_error`（第 345 行）已能识别认证错误，但重试逻辑未利用此信息。

**Fix:** 在匹配 `Err(e)` 时，对不可重试的错误类型短路返回：

```rust
Err(e) => {
    if matches!(&e, TTSError::AuthenticationFailed(_) | TTSError::UnknownEngine { .. }) {
        return Err(e);
    }
    tracing::warn!("Edge-TTS 合成尝试 {} 失败: {}", attempt, e);
    last_error = Some(e);
    if attempt < max_attempts {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

### WR-02: `connect()` 函数缺少超时机制

**File:** `src/tts/edge_tts.rs:119-342`（整体 `connect` 函数，~220 行）
**Issue:** `connect()` 函数在执行代理 CONNECT 握手时没有任何超时保护。以下场景会导致函数永久阻塞：

1. 代理服务器在 CONNECT 阶段无响应（TCP 连接成功但不回复数据）
2. 代理发送不完整的 HTTP 响应头（缺少 `\r\n\r\n` 终止符），读循环无限等待
3. DNS 解析和 TCP 连接本身有系统级超时，但 CONNECT 阶段不受此保护

当前仅 `synthesize_once` 中有 `WS_RECEIVE_TIMEOUT`（120秒，第 11 行），此超时仅作用于 **已建立的 WebSocket** 的消息接收，而不作用于 **CONNECT 握手阶段**。

**Fix:** 在 `connect()` 函数外层添加超时：

```rust
use tokio::time::timeout;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

async fn connect(&self) -> Result<..., TTSError> {
    timeout(CONNECT_TIMEOUT, async {
        // ... 现有 connect 逻辑 ...
    }).await
    .map_err(|_| TTSError::ConnectionFailed("代理 CONNECT 超时 (30s)".to_string()))?
}
```

## Info

### IN-01: `notify` 依赖为 RC 版本

**File:** `Cargo.toml:14`
**Issue:** `notify = "9.0.0-rc.3"` 为发布候选版本，正式版 9.0.0 可能引入 API 破坏性变更。`Cargo.toml` 中已有注释说明此风险，但建议在 9.0.0 正式版发布后升级并进行集成验证。

### IN-02: `connect()` 函数复杂度过高

**File:** `src/tts/edge_tts.rs:119-342`
**Issue:** `connect()` 函数约 220 行，包含 IPv6 代理地址解析、HTTP CONNECT 隧道构建、1xx/2xx 响应合并处理等复杂逻辑。建议将以下子逻辑提取为独立的辅助函数以提升可维护性：

- 代理 URL 解析（主机、端口、凭据提取）
- 目标 WSS URL 解析（主机、端口）
- HTTP CONNECT 请求构建
- CONNECT 响应读取与解析

当前代码虽然逻辑正确，但函数长度和嵌套深度（涉及 `loop` 套 `loop` 加 `continue`）增加了未来维护的理解成本。

### IN-03: 签名测试使用 Unix 路径

**File:** `tests/tts_test.rs:113`
**Issue:** 编译期签名测试使用 `Path::new("/tmp/_narratoai_sig_check.mp3")`。该路径 `let _` 丢弃未来而不 poll，因此当前无实际文件操作。但如果未来有人在该测试上添加 `.await` 或将其改为集成测试，路径在 Windows 上会失败。安全注释已说明此约束。

### IN-04: 未处理的二进制消息路径被静默忽略

**File:** `src/tts/edge_tts.rs:464-495`
**Issue:** 在 `synthesize_once` 的二进制消息分发链中，`SentenceBoundary` 等非核心路径类型会无任何日志地通过整个 if-else 链。如果 Edge TTS 服务未来新增消息类型或改变行为，静默忽略会掩盖问题。建议在 if-else 链末尾添加兜底日志：

```rust
} else {
    tracing::debug!("Edge-TTS 未处理的路径类型: {}", content.path);
}
```

### IN-05: 词边界时间单位测试为同义反复

**File:** `tests/tts_test.rs:80-91`
**Issue:** `test_word_boundary_time_unit_semantics` 测试声明常量 `50_000_000u64` 并验证其自减等于自身。此测试仅验证 Rust 编译器的 u64 算数运算正确性（编译器保证），作为文档有价值但无实际验证作用。

### IN-06: `voice_name_to_lang` 对缺少第二连字符的音色名返回空串

**File:** `src/tts/edge_tts.rs:61-71`
**Issue:** 当 `voice_name` 不符合 `xx-XX-VoiceName` 格式（少于两个连字符）时，`voice_name_to_lang` 返回空字符串。此空串被直接填入 SSML 的 `xml:lang=""` 属性。虽然 `synthesize` 已在第 593 行校验 `voice_name` 非空，但格式不合规的音色名（如 `"XiaoyiNeural"`、`"CustomVoice"`）仍会通过校验并生成含 `xml:lang=""` 的 SSML。降低为 INFO 级别，因为标准 Edge TTS 音色名均符合此格式，该问题仅影响自定义/非标准音色名。

---

**此前审查迭代已修复的问题（不再列为当前问题）：**

此前 03-REVIEW.md（Iteration 2）报告的 13 个问题中，12 个已有明确的修复提交：

| 状态 | ID | 修复提交 |
|------|-----|---------|
| 已修复 | WR-06 (旧) CONNECT 1xx+2xx 数据丢失 | `44d4b95` |
| 已修复 | WR-07 (旧) Path 头大小写敏感 | `1c5e5b1` |
| 已修复 | WR-08 (旧) voice_name 空值未校验 | `345719a` |
| 已修复 | IN-09 (旧) expect() 潜在 panic | `02070c1` |
| 已修复 | IN-10 (旧) 空路径签名测试 | `7396104` |
| 已修复 | WR-01 (旧) IPv6 代理 | `44d4b95` |
| 已修复 | WR-02 (旧) 代理凭据丢失 | `44d4b95` |
| 已修复 | WR-03 (旧) from_utf8_lossy | `44d4b95` |
| 已修复 | IN-03 (旧) 重复错误分类函数 | `237894b` |
| 已修复 | IN-04 (旧) 未使用 wiremock | 已移除 |
| 已修复 | IN-05 (旧) 边界值测试缺失 | `97fc2ea` |
| 已修复 | IN-06 (旧) SOCKS5 处理 | `ce32853` |
| 已修复 | IN-08 (旧) parse_edge_tts_binary 无单元测试 | `b1fdfbe` |
| 未修复 | WR-04 (旧) turn.end 解析失败静默设 duration=0 | 仍在代码中（见第 486/491 行），但不影响正确性 |

---

_审查时间: 2026-04-29T16:30:00Z_
_审查者: Claude (gsd-code-reviewer)_
_深度: standard_
_基准: 当前代码状态（git HEAD: 58227cd）_
