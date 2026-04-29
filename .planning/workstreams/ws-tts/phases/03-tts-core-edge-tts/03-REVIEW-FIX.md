---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-29T14:45:00+08:00
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 2
findings_in_scope: 13
fixed: 12
skipped: 1
status: partial
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Fix Report

**Fixed at:** 2026-04-29T14:45:00+08:00
**Source review:** .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 13
- Fixed: 12
- Skipped: 1

## Fixed Issues

### WR-01: IPv6 代理地址无显式端口时失败

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 8d10910
**Applied fix:** 将 `rsplit_once("]:")` 的 `unwrap_or(Err(...))` 立即报错改为 `if let Some/else` 双路处理：有 `]:` 时解析端口，无端口时回退到 `default_proxy_port`（与 IPv4 行为一致）。

### WR-02: 代理 URL 中 `user:pass@` 凭据被静默丢弃

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** c32cf76
**Applied fix:** 在剥离 `user:pass@` 前缀前，通过 `rsplitn(2, '@').nth(1)` 提取凭据，用 `base64` 编码后以 `Proxy-Authorization: Basic` 标头加入 HTTP CONNECT 请求。

### WR-03: `String::from_utf8_lossy` 掩盖 JSON 数据损坏

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 7bb10f6
**Applied fix:** 将 wordboundary 和 turn.end 解析中的 `String::from_utf8_lossy()` 替换为 `std::str::from_utf8()`。UTF-8 非法序列时记录警告日志并返回空字符串，避免静默数据损坏。

### WR-04: turn.end JSON 解析失败导致 duration 静默为 0

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 304cf01
**Applied fix:** 在 `serde_json::from_str` 失败的 else 分支中添加 `tracing::warn!` 日志，记录 payload 字节数以便诊断。

### WR-05: 重试耗尽后残留输出文件未清理

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** e956b8e
**Applied fix:** 在 `synthesize_with_retry` 返回 `RetryExhausted` 前调用 `tokio::fs::remove_file(output_path)` 清理不完整/损坏的输出文件。

### IN-01: XML 1.0 受限字符 0x7F-0x9F 未在 SSML 中过滤

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 0bf0def
**Applied fix:** 扩展字符过滤谓词，在 `code >= 0x20` 基础上增加 `!(0x7F..=0x9F).contains(&code)` 排除 DEL 及 C1 控制字符。

### IN-03: 错误分类函数重复

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** eb616de
**Applied fix:** 合并 `classify_connection_error` 和 `classify_receive_error` 为泛型函数 `classify_error(err_msg, fallback)`，调用处传入不同的错误变体构造函数。

### IN-04: `wiremock` 开发依赖未使用

**Files modified:** `Cargo.toml`
**Commit:** d9acb9d
**Applied fix:** 从 `[dev-dependencies]` 中移除 `wiremock = "0.6"`。

### IN-05: `convert_rate_to_percent` 缺少边界值测试

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 875c43a
**Applied fix:** 添加了 `rate=0.0`（返回 `-100%`）、`NaN`（返回 `+0%`）、`-0.5`（返回 `+0%`）、`inf` 和 `-inf`（返回 `+0%`）的单元测试。

### IN-06: SOCKS5 代理处理方式存在误导

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 69dfb12
**Applied fix:** 将原本的 `"socks5" => 1080u16` 默认端口分支改为提前 `return Err(...)` 返回明确错误信息，指出本引擎不支持 SOCKS5。

### IN-07: `voice_name_to_lang` 对无法识别的音色名默认返回 "zh-CN"

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** b6d1e4b
**Applied fix:** 将 else 分支的默认返回值从 `"zh-CN"` 改为 `String::new()` 空字符串，避免非中文语音被错误标记为 `xml:lang="zh-CN"`。

### IN-08: `parse_edge_tts_binary` 没有单元测试

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 0f3bd82
**Applied fix:** 添加了 6 个单元测试：audio 消息、wordboundary 消息、turn.end 消息、缺失分隔符、空 payload、缺少 Path 头字段。

## Skipped Issues

### IN-02: `connect` 函数过长（约 180 行）

**File:** `src/tts/edge_tts.rs:119-299`
**Reason:** 拆分为 `connect_direct()` 和 `connect_via_proxy()` 是合理的重构建议，但涉及约 180 行网络/TLS/代理隧道逻辑，拆分风险较高。鉴于该发现为 Info 级别，且当前代码功能正确，跳过此重构以避免引入回归问题。
**Original issue:** `connect` 函数同时处理直连和代理两条路径，包含 HTTP CONNECT 协商、IPv4/IPv6 地址解析、TLS 升级等复杂逻辑，总长约 180 行。

---

_Fixed: 2026-04-29T14:45:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
