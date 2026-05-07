---
phase: 12
plan: 01
name: Shared utilities + SoulVoice + Doubao
subsystem: tts
tags: [tts, soulvoice, doubaotts, common, wave-1]
completed_date: 2026-04-30
duration: 0h 15m
requirements: [TTS-06, TTS-09]
requires: [03-tts-core-edge-tts]
provides: [common-utils, soulvoice-engine, doubao-engine]
affects: [src/tts/mod.rs]
key-files:
  created:
    - narratoai-core/src/tts/common.rs
    - narratoai-core/src/tts/soulvoice.rs
    - narratoai-core/src/tts/doubaotts.rs
  modified:
    - Cargo.toml
    - narratoai-core/src/tts/mod.rs
metrics:
  tasks_total: 3
  tasks_completed: 3
  tests_total: 49
  tests_passed: 49
  tests_ignored: 1
  lines_added: 747
commits:
  - 94fc93f: feat(12-01): create common.rs shared utility module and expand synthesize() signature
  - 8ddccf1: feat(12-01): implement SoulVoice TTS engine
  - 8818df3: feat(12-01): implement Doubao TTS engine
---

# Phase 12 Plan 01: Shared utilities + SoulVoice + Doubao Summary

## One-Liner

搭建共享工具模块（重试循环、代理配置、前缀解析、音频写入），实现 SoulVoice 和 Doubao 两个 REST API TTS 引擎，在 synthesize() 路由器中新增两个 match 分支。

## What Was Built

### Task 1: common.rs 共享工具模块

`src/tts/common.rs` 提供 5 个公开函数/结构体供所有 TTS 引擎复用：

- **`ProxyConfig`** 结构体 + `from_proxy()` / `apply_to_client()` — 从 `Option<&ProxySection>` 创建并应用到 reqwest ClientBuilder
- **`build_client()`** — 构建带代理配置的 reqwest Client
- **`retry_loop()`** — 通用重试循环（4 次尝试，认证错误直接返回，1 秒间隔）
- **`write_audio_bytes()`** — 异步写入音频文件
- **`parse_engine_prefix()`** — 从 voice_name 去除引擎名前缀（如 `soulvoice:`）

Cargo.toml 启用 reqwest `json` feature。synthesize() 签名新增 `app_config: Option<&AppConfig>` 参数。

### Task 2: SoulVoice TTS 引擎

`src/tts/soulvoice.rs` — 通过 REST API 调用 SoulVoice 服务：
- `POST` 到 `api_url`（默认 `https://tts.scsmtech.cn/tts`）
- `Authorization: Bearer {api_key}` + `Content-Type: application/json`
- 请求体 `{"text", "model", "voice", "speed"}`
- `parse_engine_prefix` 剥离 `soulvoice:` 前缀
- 5 个单元测试覆盖成功路径、空文本、HTTP 错误、前缀解析、引擎创建

### Task 3: Doubao TTS 引擎

`src/tts/doubaotts.rs` — 通过 REST API 调用火山引擎豆包语音：
- `POST` 到 `https://openspeech.bytedance.com/api/v1/tts`
- `Authorization: Bearer;{token}`（**分号**不是空格，对齐 Python 版）
- 嵌套 JSON payload（app/user/audio/request 四段）
- 响应格式 `{"code": 3000, "message": "Success", "data": "<base64>"}`
- 7 个单元测试覆盖引擎创建、空文本、配置缺失、auth 头格式、base64 解码、业务码检查、payload 结构

## Deviations from Plan

### Test approach adjustment for Doubao

- **Found during:** Task 3
- **Issue:** plan 指定 wiremock 测试 Doubao TTS 引擎的成功和业务错误路径，但 URL 在引擎中硬编码为 `https://openspeech.bytedance.com/api/v1/tts`，wiremock 无法拦截（wiremock 需要客户端连接到 `localhost:{port}`）
- **Made URL configurable?** No — 保持与 plan 一致（硬编码 URL），因为更改 URL 会引入不必要的可配置性
- **Alternative:** 创建了针对性单元测试验证引擎的各个逻辑组件：base64 解码、业务码 3000 检查、payload 结构、auth 头格式、空文本和配置缺失验证
- **Files modified:** `src/tts/doubaotts.rs`
- **Commit:** 8818df3

## Threat Flags

None. All new surface (SoulVoice and Doubao API endpoints) is covered by the plan's threat model — HTTPS transport, API keys from config, no PII in audio paths.

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| `duration: 0.0` | `src/tts/soulvoice.rs:312` | SoulVoice API 不返回时间长度信息 |
| `word_boundaries: vec![]` | `src/tts/soulvoice.rs:311` | SoulVoice 不支持词边界事件 |
| `duration: 0.0` | `src/tts/doubaotts.rs:110` | Doubao API 不返回时间长度信息 |
| `word_boundaries: vec![]` | `src/tts/doubaotts.rs:109` | Doubao 不支持词边界事件 |

These stubs match the Python behavior — only Edge-TTS supports word boundaries.

## Verification Results

- `cargo build` — success
- `cargo test --lib tts::common::tests` — 6/6 passed
- `cargo test --lib tts::soulvoice::tests` — 5/5 passed
- `cargo test --lib tts::doubaotts::tests` — 7/7 passed
- `cargo test --lib tts::tests` — 3/3 passed (existing router tests)
- `cargo test --lib tts` — 49/49 passed (1 ignored — integration test requiring network)

## Self-Check: PASSED

- [x] `Cargo.toml` reqwest features = ["json"]
- [x] `src/tts/common.rs` exists with ProxyConfig, parse_engine_prefix, retry_loop, build_client, write_audio_bytes
- [x] `src/tts/mod.rs` contains `mod common; mod soulvoice; mod doubaotts;`
- [x] synthesize() signature includes `app_config: Option<&AppConfig>`
- [x] synthesize() match has `"soulvoice" =>` and `"doubaotts" =>` branches
- [x] SoulVoice engine: struct + TtsProvider impl + Bearer auth + reqwest POST
- [x] Doubao engine: struct + TtsProvider impl + Bearer;{token} auth + base64 decode + code 3000 check
- [x] All unit tests pass
- [x] No shared orchestrator artifacts modified
