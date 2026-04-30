---
phase: 12
plan: 03
subsystem: tts
tags: [azure-speech, tencent-tts, tc3-hmac-sha256, wave-3]
requires: [12-01, 12-02]
provides: [azure_speech, tencent_tts]
affects: [mod.rs, Cargo.toml, synthesizer router]
tech-stack:
  added:
    - hmac = "0.13" (TC3-HMAC-SHA256 签名)
    - sha2 = "0.11" (SHA-256 哈希)
    - hex = "0.4" (hex 编码)
    - chrono = "0.4" (时间戳生成)
  patterns:
    - Azure REST API via SSML + Ocp-Apim-Subscription-Key header
    - TC3-HMAC-SHA256 手动签名（无腾讯云 SDK）
    - endpoint_override 字段支持 wiremock 测试
    - Should_use_azure_services 智能回退内联分支
key-files:
  created:
    - src/tts/azure_speech.rs (467 行)
    - src/tts/tencent_tts.rs (427 行)
  modified:
    - src/tts/mod.rs (+azure_speech/tencent_tts mod + match 分支)
    - Cargo.toml (+hmac/sha2/hex/chrono)
    - Cargo.lock (自动更新)
metrics:
  duration: ~25 min
  commits: 3
  tests-added: 19 (12 azure + 7 tencent)
  completed: 2026-04-30
---

# Phase 12 Plan 03: Azure Speech + Tencent TTS 引擎实现

实现 Phase 12 最复杂的两个 TTS 引擎——Azure Speech（REST API + SSML，含 Edge-TTS 智能回退）和 Tencent TTS（手动 TC3-HMAC-SHA256 签名认证）。至此 Phase 12 全部 6 个引擎（SoulVoice, Doubao, Qwen, IndexTTS2, Azure, Tencent）实现完毕。

## 任务执行

### Task 1: Azure Speech TTS 引擎 + 智能回退查询函数

**文件:** `src/tts/azure_speech.rs` (NEW), `src/tts/mod.rs`

- 实现 `should_use_azure_services()`：检查 -V2 后缀或 `xx-XX-NameNeural` 正则格式，对齐 Python 版 `voice.py:1098-1116`
- 实现 `get_azure_voices()`：返回 250+ 硬编码 Azure Neural 音色列表
- 实现 `AzureSpeechEngine`：
  - REST API 端点 POST `https://{region}.tts.speech.microsoft.com/cognitiveservices/v1`
  - 认证：`Ocp-Apim-Subscription-Key` header
  - 请求体：SSML XML (`<speak><voice>` + XML 转义文本)
  - 语言标签从 voice_name 提取（如 `zh-CN-YunzeNeural` -> `zh-CN`）
  - -V2 后缀在 SSML 构建前剥离（防 400 错误）
- 输出格式：`audio-48khz-192kbitrate-mono-mp3`
- `mod.rs` 添加 `mod azure_speech;` + match 分支，含智能回退条件判断
- **测试:** 12 个单元测试全部通过

**Commit:** `8032999`

### Task 2: Tencent TTS 引擎 + TC3-HMAC-SHA256 签名

**文件:** `Cargo.toml`, `src/tts/tencent_tts.rs` (NEW), `src/tts/mod.rs`

- 新增依赖：`hmac = "0.13"`, `sha2 = "0.11"`, `hex = "0.4"`, `chrono = "0.4"`
- 实现 `TencentTtsEngine`：
  - API 端点 POST `https://tts.tencentcloudapi.com`
  - 完整 TC3-HMAC-SHA256 手动签名（无 SDK）：
    - SecretKey -> SecretDate -> SecretService -> SecretSigning 派生链
    - `content-type;host` 签名头部
    - `X-TC-*` headers (Action/Timestamp/Version/Region/Token)
  - 请求体：JSON (`Text`, `VoiceType`, `Speed`, `SampleRate`, `Codec`, `EnableSubtitle`...)
  - 响应解析：`Response.Audio` (base64) + `Response.Subtitles` (词边界)
  - 词边界单位转换：ms -> 100ns (x10000)
  - 时间戳和日期在同一 `chrono::Utc::now()` 调用中生成（防时间偏差 Pitfall 3）
- `mod.rs` 添加 `mod tencent_tts;` + match 分支
- **测试:** 7 个单元测试全部通过（含 wiremock 验证签名+音频+词边界）

**Commit:** `7657ebd`

## 验证结果

- `cargo build` -- 编译成功 (0 errors, 1 warning: `get_azure_voices` 暂未调用)
- `cargo test --lib tts` -- 78 passed, 1 ignored (edge_tts 集成测试), 0 failed
- 所有 6 个引擎的测试均通过：Edge-TTS, SoulVoice, Doubao, Qwen, IndexTTS2, Azure, Tencent
- 文件行数：azure_speech.rs 467 行 (>= 150), tencent_tts.rs 427 行 (>= 200)
- mod.rs 包含 "azure_speech" 和 "tencent_tts" 模式

## 与计划的偏差

### 规则 2 - 添加缺失的关键功能

1. **端点可测试性 (endpoint_override 字段)**
   - **发现于:** Task 1 + Task 2 的 wiremock 测试实现
   - **原因:** 两个引擎均使用硬编码的生产 URL（Azure: `{region}.tts.speech.microsoft.com`; Tencent: `tts.tencentcloudapi.com`），wiremock 测试需要将请求路由到本地 mock server。TencentSection 和 AzureSection 均无可配置的 api_url 字段。
   - **修复:** 在两个引擎 struct 中添加 `endpoint_override: Option<String>` 字段（pub(super)），`synthesize_once()` 中优先使用覆盖 URL。
   - **文件:** `src/tts/azure_speech.rs:44`, `src/tts/tencent_tts.rs:33`

### 依赖版本兼容性调整

1. **sha2 从 0.10 升到 0.11**
   - **原因:** `hmac = "0.13"` 依赖 `digest = "0.11"`，而 `sha2 = "0.10"` 依赖 `digest = "0.10"`，两者不兼容（`Hmac<Sha256>` 的 `EagerHash` trait bound 无法满足）。
   - **修复:** 将 `Cargo.toml` 中的 `sha2` 版本从 `"0.10"` 改为 `"0.11"`，同时添加 `use sha2::Digest;` 和 `use hmac::KeyInit;` 导入。

## Success Criteria

- [x] Azure Speech 引擎通过 `"azure_speech"` 路由可调用，含 `should_use_azure_services()` 智能回退
- [x] Tencent TTS 引擎通过 `"tencent_tts"` 路由可调用，含完整 TC3-HMAC-SHA256 签名
- [x] Tencent TTS 从 Subtitles 数组正确生成 WordBoundary 列表
- [x] `get_azure_voices()` 返回硬编码音色列表
- [x] 所有 6 个引擎（SoulVoice, Doubao, Qwen, IndexTTS2, Azure, Tencent）的 synthesize() 路由分支全部就位
- [x] 所有单元测试通过，cargo build 成功

## Threat Surface 扫描

| 标志 | 文件 | 描述 |
|------|------|------|
| threat_flag: new_api_endpoint | `src/tts/azure_speech.rs` | 新增 Azure Speech REST API（HTTPS, 有 auth） |
| threat_flag: new_api_endpoint | `src/tts/tencent_tts.rs` | 新增 Tencent Cloud API（HTTPS, TC3 签名） |

两个端点均在计划的 threat_model 覆盖范围内（T-12-09 至 T-12-12），无需额外缓解措施。

## Self-Check

- [x] `src/tts/azure_speech.rs` 存在 (467 行)
- [x] `src/tts/tencent_tts.rs` 存在 (427 行)
- [x] `12-03-SUMMARY.md` 存在
- [x] Commit `8032999` 存在 (Azure Speech)
- [x] Commit `7657ebd` 存在 (Tencent TTS)
- [x] Commit `7f1829c` 存在 (Cargo.lock)
- [x] `cargo build` 编译通过
- [x] `cargo test --lib tts` 78 passed / 0 failed

## Self-Check: PASSED
