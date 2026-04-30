# Phase 12: Additional TTS Engines - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

除 Edge-TTS（Phase 3 已完成）外的 6 个 TTS 引擎全部实现，达到与 Python 版一致的 TTS 功能覆盖。每个引擎实现 `TtsProvider` trait，接入 Phase 3 已建立的 `synthesize()` 路由器。

**覆盖需求：** TTS-04 (Azure), TTS-05 (Tencent), TTS-06 (SoulVoice), TTS-07 (Qwen), TTS-08 (IndexTTS2), TTS-09 (Doubao)

</domain>

<decisions>
## Implementation Decisions

### Azure Speech 路由策略
- **D-01:** 内部智能回退——`azure_speech` 引擎先检查 voice_name 是否符合 Azure Neural 格式（`xx-XX-NameNeural` 或 `-V2` 后缀）。匹配则走 Azure Cognitive Services REST API 调用，不匹配则回退到 Edge-TTS（调用 Phase 3 已实现的 EdgeTtsEngine）。与 Python 版 `should_use_azure_speech_services()` + 双分支行为完全对齐。
- **D-02:** 路由器内联分派——在 `synthesize()` 的 `match "azure_speech"` 分支中直接内联 V1/V2 判断逻辑。不创建额外的包装结构体或中间层。
- **D-03:** 硬编码音色列表——Azure Neural 支持的音色以 `&[&str]` 字符串数组硬编码在源文件中，提供公开查询函数 `get_azure_voices(filter_locale: Option<&str>) -> Vec<&str>`。对齐 Python 版 `get_all_azure_voices()`。

### HTTP 调用 vs 官方 SDK
- **D-04:** 统一 reqwest——所有 6 个引擎统一使用 `reqwest` 构造 HTTP 请求。不引入 `tencentcloud-sdk-rust`、`azure_cognitive_services` 或任何第三方 SDK。Azure 和 Tencent 手动实现 REST API 调用（含认证签名 TC3-HMAC-SHA256）。
- **D-05:** 统一 3 次重试——所有引擎 3 次重试，HTTP 超时 60s，失败时 `TTSError::RetryExhausted` 统一返回。重试间隔由共享模块统一管理。

### IndexTTS2 参考音频入参
- **D-06:** 前缀编码在 voice_name 内——`voice_name` 用 `"indextts2:/path/to/reference.wav"` 格式，解析提取参考音频路径。不改 `TtsProvider` trait 签名。其他引擎同样走前缀解析：
  - `soulvoice:speech:model:voice_id` → 提取实际 voice 字符串（去除 `soulvoice:` 前缀）
  - `tencent:101001` → 提取腾讯云音色代码
  - `qwen3:Cherry` → 提取 Qwen 音色名
  - 对齐 Python 版 `parse_*_voice()` 系列函数

### 引擎文件组织方式
- **D-07:** 每个引擎独立文件——`src/tts/azure_speech.rs`、`tencent_tts.rs`、`soulvoice.rs`、`qwen_tts.rs`、`indextts2.rs`、`doubaotts.rs`。与 Phase 3 `edge_tts.rs` 模式一致。
- **D-08:** 文件名对齐 Python 引擎名字符串——文件名与 `voice.py:tts()` 的 match 分支字符串一致。
- **D-09:** 提取共享模块——新建 `src/tts/common.rs`，提取以下共享逻辑供各引擎复用：
  - 代理配置应用（`apply_proxy(reqwest::ClientBuilder, &ProxySection) -> ClientBuilder`）
  - 重试循环（`async fn retry_loop<F>(f: F) -> Result<TtsOutput, TTSError>`）
  - 音频响应字节写入（`fn write_audio_bytes(path: &Path, data: &[u8])`）
  - HTTP 认证头构造辅助

### Claude's Discretion
- 各引擎的 HTTP 请求体构造细节（URL、header、payload 格式对齐 Python 版即可）
- reqwest Client 实例是全局复用还是每次创建（推荐复用，减少连接开销）
- 共享模块的具体函数签名和参数设计
- 是否需要为 IndexTTS2 延长 HTTP 超时（推理时间可能较长）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 TTS 参考（权威来源）
- `app/services/voice.py` — TTS 路由中心，包含所有 7 个引擎的 dispatch 逻辑和完整实现（azure_tts_v2、tencent_tts、soulvoice_tts、qwen3_tts、indextts2_tts、doubaotts_tts）
- `app/config/config.py` — TOML 配置加载，包含各 TTS 引擎的配置段读取逻辑

### Rust TTS 基础（Phase 3 已交付）
- `src/tts/mod.rs` — `TtsProvider` trait + `WordBoundary`/`TtsOutput` 类型 + `synthesize()` 路由器（已实现 edge_tts 分支，其余为 `_ => UnknownEngine`）
- `src/tts/edge_tts.rs` — EdgeTtsEngine 完整实现（模式参考：WebSocket 连接、SSML 构建、词边界解析、代理支持）
- `src/error.rs` — `TTSError` 枚举（5 变体：UnknownEngine、ConnectionFailed、AuthenticationFailed、SynthesisFailed、RetryExhausted）
- `src/config/types.rs` — 配置类型定义（需新增各 TTS 后端的配置结构体）

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（Rust 库 + Tauri 命令层、文件存储、不可变模式）
- `.planning/REQUIREMENTS.md` — Phase 12 覆盖需求：TTS-04 到 TTS-09
- `.planning/ROADMAP.md` — Phase 12 定义、成功标准、Phase 3 依赖
- `.planning/phases/03-tts-core-edge-tts/03-CONTEXT.md` — Phase 3 决策（TTS trait 签名、路由器设计、错误类型）

### 代码库映射
- `.planning/codebase/ARCHITECTURE.md` — Python 版 TTS 架构（voice.py routing hub）
- `.planning/codebase/INTEGRATIONS.md` — TTS 引擎列表和各配置段
- `.planning/codebase/STACK.md` — Python 版 TTS 依赖库（edge-tts、azure-cognitiveservices-speech、tencentcloud-sdk-python、dashscope）

### 配置模板参考
- `config.example.toml` — Python 版 `[azure]`、`[tencent]`、`[soulvoice]`、`[tts_qwen]`、`[indextts2]`、`[doubaotts]` 配置段模板

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/tts/mod.rs:synthesize()` — 路由器函数，Phase 12 只需在 match 中添加 6 个新分支，已有 proxy 参数传递
- `src/tts/edge_tts.rs:EdgeTtsEngine` — 引擎实现模板：struct 定义、TtsProvider impl、新建函数、SSML/请求构建、响应解析
- `src/tts/mod.rs:WordBoundary` + `TtsOutput` — 输出类型直接复用，各引擎填对应字段即可
- `src/error.rs:TTSError` — 5 变体覆盖所有引擎的错误场景，无需新增变体

### Established Patterns
- **TtsProvider trait 实现**：每个引擎 struct（含 config 字段）→ `#[async_trait] impl TtsProvider` → `async fn synthesize()` 实现
- **路由器 match 模式**：字符串匹配引擎名 → 解析配置 → 创建引擎实例 → 调用 synthesize()
- **异步运行时**：tokio + async_trait，引擎内部用 reqwest 异步请求
- **代理支持**：router 将 `Option<&ProxySection>` 传入各引擎的 `new()` 函数

### Integration Points
- `src/tts/mod.rs` — 新增 `mod azure_speech; mod tencent_tts; mod soulvoice; mod qwen_tts; mod indextts2; mod doubaotts; mod common;`
- `src/tts/common.rs` — 新文件，共享工具模块
- `src/config/types.rs` — 新增 AzureSection、TencentSection、SoulVoiceSection、TtsQwenSection、IndexTts2Section、DoubaoTtsSection 配置结构体
- `src/lib.rs` — 无需修改（tts 模块已在 Phase 3 导出）
- Phase 6/7/8 流水线 — 通过 router 字符串选择 TTS 引擎，Phase 12 无需改动调用方
- Phase 10 Tauri 命令层 — 暴露 TTS 引擎选择给前端

</code_context>

<specifics>
## Specific Ideas

- 6 个引擎的 Python 实现在 `app/services/voice.py` 中——研究者应以该文件为每个引擎的权威参考，逐函数对齐到 Rust
- 前缀解析模式（`engine:payload`）在 Python 版 4 个引擎中出现（soulvoice、tencent、qwen3、indextts2），Rust 应实现统一的 `parse_engine_prefix()` 辅助函数
- Doubao TTS 使用 `Bearer;{token}` 格式的 Authorization 头（注意是分号而不是空格），与标准 Bearer 不同——这是 Python 版的实际情况
- Azure Speech V2 需要 subscription_key + region 认证，REST API 端点格式为 `https://{region}.tts.speech.microsoft.com/cognitiveservices/v1`
- Tencent Cloud 需要 TC3-HMAC-SHA256 签名认证，实现复杂度高于其他引擎的简单 Bearer/API Key 认证
- IndexTTS2 是本地自托管服务，默认 API URL 为 `http://192.168.3.6:8081/tts`，需要 multipart/form-data 上传参考音频

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-additional-tts-engines*
*Context gathered: 2026-04-30*
