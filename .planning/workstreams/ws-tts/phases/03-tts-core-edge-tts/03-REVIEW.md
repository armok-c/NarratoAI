---
phase: 03-tts-core-edge-tts
reviewed: 2026-04-29T12:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/tts/mod.rs
  - src/tts/edge_tts.rs
  - tests/tts_test.rs
  - src/error.rs
  - src/lib.rs
  - Cargo.toml
findings:
  critical: 1
  warning: 2
  info: 2
  total: 5
status: issues_found
---

# Phase 03: TTS Core + Edge-TTS - Code Review Report

**Reviewed:** 2026-04-29T12:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

审查了 Phase 03 的 6 个源文件：TTS 核心抽象（`tts/mod.rs`）、Edge-TTS WebSocket 引擎（`tts/edge_tts.rs`）、集成测试（`tests/tts_test.rs`）、错误类型（`error.rs`）、库入口（`lib.rs`）和依赖声明（`Cargo.toml`）。

此前审查的 8 个发现**已经全部修复**（已验证：commit `4cd2ee9` 修复了 proxy tunnel 的 `connect_req` 未定义问题，commit `ec9c5bf` 和 `507f24b` 分别处理了余下的 8/8 和 9/9 个问题）。NaN/Infinity 的 UB 转换已通过 `is_finite()` 守卫修复，proxy URL 解析已改为 `split_once` + `to_lowercase` 的大小写不敏感方式。

本审查发现了 **1 个 BLOCKER**、**2 个 WARNING** 和 **2 个 INFO** 级别问题。

**BLOCKER 总结：** 代理 CONNECT 响应读取器存在逻辑缺陷——读取循环在遇到第一个 `\r\n\r\n` 后就停止，如果代理将 interim 1xx 响应（如 `100 Continue`）和最终 2xx 响应用**独立 TCP segment** 发送（这是 RFC 7231 允许的常规行为），代码只读到 1xx，无法获取 2xx，导致代理 TTS 功能对大量真实 HTTP 代理完全不可用。

## Critical Issues

### CR-01: 代理 CONNECT 响应读取器无法处理分开发送的 interim 1xx 响应

**File:** `src/tts/edge_tts.rs:219-276`
**Issue:** 代理 CONNECT 的响应读取循环（第 219-233 行）在读到第一个 `\r\n\r\n` 序列后即停止读取。解析循环（第 239-276 行）处理该缓冲区中的响应。

当代理在独立 TCP segment 中先发送 interim 1xx（如 `HTTP/1.1 100 Continue`）再发送 `HTTP/1.1 200 Connection Established` 时——这是一个常见行为，符合 RFC 7231 Section 4.3.6——读取循环只读到了 1xx 响应末尾的 `\r\n\r\n` 就退出。解析循环处理完 `100` 状态行、跳过其尾部头部后，调用 `lines.next()` 获取下一行时发现无数据。此时 `success` 仍为 `false`，函数最终返回错误：

```
代理 CONNECT 未收到 2xx 响应
```

**复现条件：** 任何在发送最终 2xx 之前先发送 interim 1xx 响应的 HTTP 代理都会触发此问题。这是非常多代理的标准行为。Proxyrack、Squid 等主流代理在特定配置下会发送 `100 Continue`。

**影响：** 代理 TTS 功能对大量 HTTP 代理完全不可用。

**Fix:** 需要在内层 CONNECT 响应解析循环中，处理完一个 1xx interim 响应后回到外层的 TCP 读取循环获取更多数据。有两种实现策略：

**方案 A（推荐——重构成两层循环）：**

```rust
// 取代现有的单次读取+解析，改为循环：读取一个完整响应 -> 解析 -> 决定是否继续
loop {
    // 1. 读取直到 \r\n\r\n
    let mut response_buf = vec![0u8; 4096];
    let mut total_read = 0usize;
    loop {
        let n = tcp.read(&mut response_buf[total_read..]).await
            .map_err(|e| TTSError::ConnectionFailed(format!("读取代理响应失败: {}", e)))?;
        if n == 0 {
            return Err(TTSError::ConnectionFailed("代理连接提前关闭".to_string()));
        }
        total_read += n;
        if response_buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if total_read >= response_buf.len() {
            return Err(TTSError::ConnectionFailed("代理响应头过大".to_string()));
        }
    }

    // 2. 解析这个响应
    let response_str = std::str::from_utf8(&response_buf[..total_read])
        .map_err(|_| TTSError::ConnectionFailed("代理响应包含无效 UTF-8".to_string()))?;

    let status_line = response_str.lines().next()
        .ok_or_else(|| TTSError::ConnectionFailed("代理响应为空".to_string()))?;
    let status_code: u16 = status_line
        .trim()
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| TTSError::ConnectionFailed(format!("无法解析状态码: {}", status_line)))?;

    if (200..300).contains(&status_code) {
        break; // 成功
    } else if (100..200).contains(&status_code) {
        // 1xx interim — 继续读取下一个响应
        continue;
    } else {
        return Err(TTSError::ConnectionFailed(format!(
            "代理 CONNECT 失败: {}",
            status_line.trim()
        )));
    }
}
```

**方案 B（最小改动——在现有结构中修复）：**

在当前 interim 1xx 处理分支（第 270-275 行）之后，不继续内层 `while let Some(line)` 循环，而是跳出它并返回到外层的 TCP 读取循环（标记跳转或重构）。但此方案需要仔细控制控制流，不如方案 A 清晰。

## Warnings

### WR-01: build_ssml 中 XML 控制字符未过滤

**File:** `src/tts/edge_tts.rs:78-81`
**Issue:** `build_ssml` 函数只转义了 `&`、`<`、`>` 三个 XML 特殊字符，但没有移除 XML 1.0 禁止的控制字符（U+0000-U+0008、U+000B、U+000C、U+000E-U+001F，除 `\t`、`\n`、`\r` 外）。如果输入文本包含这些字符（例如从外部 API 获取的文本包含不可见控制字符），生成的 SSML 是语法无效的 XML，Edge TTS 服务端会拒绝处理。

虽然纪录片解说文案通常不包含控制字符，但此函数是 `pub` 模块的公开 API 的一部分，未来复用边界可能扩大。在无过滤的情况下，对含控制字符的文本进行 TTS 合成会失败，且错误信息指向 WebSocket 层，排查困难。

**Fix:** 在 `replace` 链后增加 `.chars().filter(...)` 过滤掉 XML 禁止的控制字符：

```rust
fn build_ssml(text: &str, voice_name: &str, rate: f64, pitch: f64) -> String {
    // ... 现有代码 ...

    let escaped_text: String = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .chars()
        .filter(|&c| {
            // 保留标准可见字符、回车、换行、制表符
            c == '\t' || c == '\n' || c == '\r' || c as u32 >= 0x20
        })
        .collect();

    // ... format! 不变 ...
}
```

### WR-02: 认证/连接错误分类逻辑在三个位置重复

**File:** `src/tts/edge_tts.rs:297-301`, `310-313`, `389-393`
**Issue:** 以下三段代码（分别位于 `connect()` 的代理路径、`connect()` 的直连路径、`synthesize_once()` 的消息接收路径）完全重复了相同的错误分类启发式逻辑：

```rust
let lower = err_msg.to_lowercase();
if err_msg.contains("401") || lower.contains("authentication") || lower.contains("unauthorized") {
    TTSError::AuthenticationFailed(err_msg)
} else {
    TTSError::ConnectionFailed(err_msg)  // 或 SynthesisFailed
}
```

如果未来需要调整分类逻辑（例如增加 403 状态码的判断、增加对 "403" 的字面匹配），三处必须同步修改。此外第三处返回 `SynthesisFailed` 而非 `ConnectionFailed`，容易在复制粘贴时被忽略。

**Fix:** 提取为一个 `EdgeTtsEngine` 的私有静态方法：

```rust
impl EdgeTtsEngine {
    /// 根据错误消息启发式判断是否是认证错误
    fn classify_connection_error(err_msg: String) -> TTSError {
        let lower = err_msg.to_lowercase();
        if err_msg.contains("401") || lower.contains("authentication") || lower.contains("unauthorized") {
            TTSError::AuthenticationFailed(err_msg)
        } else {
            TTSError::ConnectionFailed(err_msg)
        }
    }

    /// 消息接收错误的分类（返回 SynthesisFailed 而非 ConnectionFailed）
    fn classify_receive_error(err_msg: String) -> TTSError {
        let lower = err_msg.to_lowercase();
        if err_msg.contains("401") || lower.contains("authentication") || lower.contains("unauthorized") {
            TTSError::AuthenticationFailed(err_msg)
        } else {
            TTSError::SynthesisFailed(err_msg)
        }
    }
}
```

然后在三处分别调用 `Self::classify_connection_error(err_msg)` 或 `Self::classify_receive_error(err_msg)`。

## Info

### IN-01: stt_message.into() 是多余的空操作

**File:** `src/tts/edge_tts.rs:373`
**Issue:** `Message::Text(stt_message.into())` — `stt_message` 已经是 `String` 类型，`Message::Text` 正好接受 `String`。末尾的 `.into()` 是一个 identity 转换（`String -> String`），不产生任何效果但增加了语义噪音。

**Fix:** 简化为 `Message::Text(stt_message)`。

### IN-02: 测试模块中存在多余的类型导入

**File:** `src/tts/mod.rs:100`
**Issue:** 内联测试模块同时导入了 `use super::*;` 和 `use std::path::Path;`。由于 `super::*` 已包含父模块中的 `use std::path::Path`，第二行是多余的。

**Fix:** 删除 `use std::path::Path;`。

---

_Reviewed: 2026-04-29T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
