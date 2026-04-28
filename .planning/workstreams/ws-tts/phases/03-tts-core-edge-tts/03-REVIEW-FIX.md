---
phase: 03-tts-core-edge-tts
fixed_at: 2026-04-28T12:10:00Z
review_path: .planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS -- Code Review Fix Report

**Fixed at:** 2026-04-28T12:10:00Z
**Source review:** `.planning/workstreams/ws-tts/phases/03-tts-core-edge-tts/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: 代理 URL 赋值但从未用于连接配置

**Files modified:** `Cargo.toml`, `src/tts/edge_tts.rs`
**Commits:** `f7be7a9`, `fa39ff1`
**Applied fix:** 将 `connect()` 方法中的代理分支从直连重写为 HTTP CONNECT 隧道实现：
1. 手动解析代理 URL（去除 `http://`/`https://` 前缀，分割 host:port）
2. 手动解析目标 WSS URL（去除 `wss://` 前缀，分割 host:port）
3. 通过 `tokio::net::TcpStream::connect` 连接到代理服务器
4. 发送 `CONNECT target_host:target_port HTTP/1.1` 请求
5. 读取并验证 `200 Connection established` 响应
6. 使用 `tokio_tungstenite::client_async_tls_with_config` 在隧道连接上执行 TLS + WebSocket 握手

同时为 `tokio-tungstenite` 启用了 `native-tls` 特性并添加 `native-tls` 依赖（Cargo.toml）。

### CR-02: WordBoundary 事件在合成循环中未被解析

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `f7be7a9`
**Applied fix:** 在 `synthesize_once()` 的二进制消息处理链中新增 `Path: wordboundary` 分支。从 `turn.start` 中移除错误的 `metadata["wordboundary"]` 解析逻辑。新增的 `wordboundary` 路径处理器解析每个独立的 wordboundary JSON 消息中的 `offset`、`duration` 和 `text` 字段，构造 `WordBoundary` 并推入 `word_boundaries` 向量。

### CR-03: turn.end 的 audio_duration 字段解析格式可能不匹配

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `f7be7a9`
**Applied fix:** 将 `turn.end` 处理分支中的 `audio_duration` 解析从单一的 `as_f64()` 扩展为三种格式：
- 直接 f64 值
- 字符串 tick 值（100ns 单位，除以 10_000_000 转为秒）
- 嵌套对象 `{"ticks": ..., "duration": ...}`（提取 ticks 字段）

### WR-01: 路由器函数 synthesize() 硬编码禁用代理

**Files modified:** `src/tts/mod.rs`, `tests/tts_test.rs`
**Commits:** `f7be7a9`, `f803d08`
**Applied fix:** 为公有路由器 `synthesize()` 新增 `proxy: Option<&crate::config::types::ProxySection>` 参数。当传入 `Some(p)` 时，从 `ProxySection` 中提取 `enabled`、`http`、`https` 字段传给 `EdgeTtsEngine::new()`。传入 `None` 时保持原有行为（禁用代理）。同步更新了所有测试调用点（内部测试和集成测试）。

### WR-02: turn.start 收到后但未收到音频数据时，会写入空音频文件

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** `f7be7a9`
**Applied fix:** 将后置守卫条件从 `audio_data.is_empty() && !is_turn_start` 简化为 `audio_data.is_empty()`。移除了不再使用的 `is_turn_start` 变量及其声明和赋值。

## Verification

- `cargo check --lib`: PASS (编译通过)
- `cargo test tts`: SKIPPED (LLM 模块预存编译错误阻止了测试编译，与本次修复无关)
- All 5 findings in scope resolved

---
_Fixed: 2026-04-28T12:10:00Z_
_Fixer: Claude (manual — agent produced falsified report)_
_Iteration: 1_
