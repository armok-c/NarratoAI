# Phase 12: Additional TTS Engines - Research

**Researched:** 2026-04-30
**Domain:** TTS (Text-to-Speech) engine integration via Rust + reqwest HTTP
**Confidence:** HIGH

## Summary

Phase 12 实现除 Edge-TTS 外的 6 个 TTS 引擎，使 Rust 版的 TTS 功能覆盖与 Python 版完全对齐。每个引擎实现 `TtsProvider` trait，通过 `synthesize()` 路由器中的新 `match` 分支接入。

**技术核心：** 6 个引擎统一使用 `reqwest` 构造 HTTP 请求（D-04），不使用任何第三方 SDK。Azure Speech 实现 REST API + 内部智能回退到 Edge-TTS。Tencent TTS 手动实现 TC3-HMAC-SHA256 签名。IndexTTS2 使用 multipart/form-data 上传参考音频。SoulVoice、Qwen、Doubao 为简单 REST API 调用。

**架构新增：** `src/tts/common.rs` 共享模块（重试循环、代理配置、音频写入、前缀解析），6 个独立引擎文件，`src/config/types.rs` 已有完整配置结构体（无需修改）。`src/error.rs` 无需新增变体。

**复杂度排序（从简到繁）：** SoulVoice < Doubao < Qwen < IndexTTS2 (multipart) < Azure Speech (智能回退) < Tencent (TC3 签名)

**Primary recommendation:** 按 3 个 Wave 分组实现，Wave 1 交付 common.rs + SoulVoice + Doubao（快速验证模式），Wave 2 交付 Qwen + IndexTTS2，Wave 3 交付 Azure Speech + Tencent（最复杂）。

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TTS-04 | Azure Speech TTS 引擎实现（REST API） | Azure REST API 端点 `{region}.tts.speech.microsoft.com/cognitiveservices/v1`，`Ocp-Apim-Subscription-Key` 认证，SSML 请求体，返回原始音频字节。Python 版 V2 使用 SDK，Rust 版需手动实现 REST API。智能回退逻辑与 Python 版 `should_use_azure_speech_services()` 对齐 |
| TTS-05 | Tencent TTS 引擎实现 | 端点 `tts.tencentcloudapi.com`，TC3-HMAC-SHA256 签名（需 `hmac` + `sha2` + `hex` crate），请求体含 Text/SessionId/VoiceType/Speed/SampleRate/Codec，响应含 base64 Audio 和 Subtitles 数组。最复杂的引擎 |
| TTS-06 | SoulVoice TTS 引擎实现 | 最简单引擎。POST `{api_url}`（默认 `https://tts.scsmtech.cn/tts`），`Authorization: Bearer {api_key}`，JSON body {text, model, voice, speed}，返回原始音频字节 |
| TTS-07 | Qwen TTS 引擎实现 | DashScope REST API。端点 `https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization` 或 `https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation`，`Authorization: Bearer {api_key}`，返回 JSON 含 `output.audio.url`，需二次 GET 下载音频 |
| TTS-08 | IndexTTS2 语音克隆引擎实现 | 本地自托管服务。POST `{api_url}`（默认 `http://127.0.0.1:8081/tts`），multipart/form-data 含 `prompt_audio` 文件 + 文本参数，返回原始音频字节。需 `reqwest` 的 `multipart` feature。超时 120s |
| TTS-09 | Doubao TTS 引擎实现 | 端点 `https://openspeech.bytedance.com/api/v1/tts`，`Authorization: Bearer;{token}`（注意分号！），JSON body 含 app/user/audio/request 四段，响应 JSON 含 base64 编码 `data` 字段。需 base64 解码 |
</phase_requirements>

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Azure Speech 内部智能回退——检查 voice_name 格式，匹配 Azure Neural 则走 REST API，否则回退 Edge-TTS
- **D-02:** 路由器内联分派——在 `synthesize()` 的 `match "azure_speech"` 分支中直接内联 V1/V2 判断
- **D-03:** 硬编码音色列表——Azure Neural 音色以 `&[&str]` 硬编码，提供 `get_azure_voices()` 查询函数
- **D-04:** 统一 reqwest 无第三方 SDK——手动实现 REST API 调用（含 TC3-HMAC-SHA256）
- **D-05:** 统一 3 次重试，HTTP 超时 60s，`TTSError::RetryExhausted`
- **D-06:** 前缀编码在 voice_name 内——`engine:payload` 格式，不改 `TtsProvider` trait
- **D-07:** 每个引擎独立文件——与 edge_tts.rs 模式一致
- **D-08:** 文件名对齐 Python 引擎名字符串
- **D-09:** 提取 `src/tts/common.rs` 共享模块（代理、重试、音频写入、HTTP 认证辅助）

### Claude's Discretion

- 各引擎的 HTTP 请求体构造细节（URL、header、payload 格式对齐 Python 版即可）
- reqwest Client 实例是全局复用还是每次创建（推荐复用）
- 共享模块的具体函数签名和参数设计
- 是否需要为 IndexTTS2 延长 HTTP 超时（推理时间可能较长）

### Deferred Ideas (OUT OF SCOPE)

None
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Azure Speech TTS | API / Backend | — | REST API 调用 Azure Cognitive Services，无浏览器参与 |
| Tencent TTS | API / Backend | — | REST API 调用腾讯云 TTS，含 TC3-HMAC-SHA256 签名 |
| SoulVoice TTS | API / Backend | — | REST API 调用第三方 TTS 服务 |
| Qwen TTS | API / Backend | — | REST API 调用阿里云 DashScope |
| IndexTTS2 | API / Backend | — | HTTP 调用本地自托管服务，multipart 上传 |
| Doubao TTS | API / Backend | — | REST API 调用火山引擎 |
| TTS 路由器 | API / Backend | — | 按引擎名字符串分发的纯函数，Phase 3 已有架构 |
| 引擎前缀解析 | API / Backend | — | voice_name 字符串处理辅助函数，所有引擎可复用 |

所有 6 个引擎都是纯后端 API 调用，不涉及浏览器、前端服务器、CDN 或数据库层级。`src/tts/mod.rs` 中的 `synthesize()` 路由器是唯一的编排点。

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.13 | HTTP 客户端 | 所有 6 个引擎的 HTTP 请求统一使用。需启用 `json` + `multipart` features |
| serde_json | 1.0 | JSON 序列化/反序列化 | 请求体构造和响应解析（Doubao、Qwen、Tencent 等） |
| base64 | 0.22 | Base64 编解码 | Doubao TTS 响应中 audio data 为 base64 编码；Tencent TTS 响应中 Audio 字段也为 base64 |
| tracing | 0.1 | 日志 | 所有引擎共享日志基础设施（与 Phase 3 一致） |

### New Dependencies Required
| Library | Version | Purpose | Engine |
|---------|---------|---------|--------|
| hmac | 0.13 | HMAC-SHA256 计算 | Tencent TTS TC3 签名 |
| sha2 | 0.10 | SHA-256 哈希 | Tencent TTS TC3 签名（CanonicalRequest 和 StringToSign 哈希） |
| hex | 0.4 | 十六进制编码 | Tencent TTS TC3 签名（输出签名和哈希的 hex 字符串） |
| chrono | 0.4 | 日期格式化 | Tencent TTS CredentialScope 日期（`YYYY-MM-DD` 格式） |

### Cargo.toml Changes
```toml
# 现有 reqwest 需添加 features
reqwest = { version = "0.13", features = ["json", "multipart"] }

# 新增依赖
hmac = "0.13"
sha2 = "0.10"
hex = "0.4"
chrono = "0.4"
```

### No Changes Required
| Library | Reason |
|---------|--------|
| tokio-tungstenite | 仅 Edge-TTS 使用，新引擎不需要 |
| uuid | Phase 3 已引入，Tencent 引擎的 SessionId 可复用 |
| uuid | Phase 3 已引入，Doubao 引擎的 reqid 可复用 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| 手动 reqwest | tencentcloud-sdk-rust | D-04 明确禁止第三方 SDK |
| 手动 reqwest | azure_cognitive_services | D-04 明确禁止 |
| 手动 reqwest | dashscope SDK for Rust | D-04 明确禁止 |

## Architecture Patterns

### System Architecture Diagram
```
Voice name (含前缀) + 文本 + 参数
                |
                v
    +---------------------------+
    | synthesize() 路由器       |
    | src/tts/mod.rs            |
    +---------------------------+
                |
     match engine string
                |
    +---+---+---+---+---+---+--+
    |   |   |   |   |   |   |  |
    v   v   v   v   v   v   v  v
  edge azure tencent soul qwen idx2 doubao
    |   |   |   |   |   |   |
    |   +- 智能回退? -+   |   |
    |   |   是 | 否      |   |
    |   v     v          |   |
    | REST   ↓ Edge-TTS  |   |
    | API   (Phase 3)    |   |
    |                     |   |
    +----+------+--------+---+
         |      |           |
    +----v------v-----------v---+
    | common.rs 共享模块        |
    | - retry_loop()           |
    | - apply_proxy()          |
    | - write_audio_bytes()    |
    | - parse_engine_prefix()  |
    +---------------------------+
                |
                v
      TtsOutput { audio_file_path,
                  word_boundaries,
                  duration }
```

### Recommended Project Structure
```
src/tts/
├── mod.rs              # TtsProvider trait + synthesize() 路由器（已存在，需添加 mod 声明 + match 分支）
├── common.rs           # NEW: 共享工具模块（重试循环、代理应用、音频写入、前缀解析）
├── edge_tts.rs         # 已存在（Phase 3）
├── azure_speech.rs     # NEW: Azure Speech REST API + 智能回退
├── tencent_tts.rs      # NEW: Tencent TTS + TC3-HMAC-SHA256 签名
├── soulvoice.rs        # NEW: SoulVoice TTS
├── qwen_tts.rs         # NEW: Qwen TTS via DashScope
├── indextts2.rs        # NEW: IndexTTS2 语音克隆（multipart）
└── doubaotts.rs        # NEW: Doubao TTS
```

### Pattern 1: 引擎实现模板（与 EdgeTtsEngine 对齐）

每个引擎遵循以下统一结构：

```rust
// src/tts/azure_speech.rs

use crate::error::TTSError;
use crate::tts::{TtsOutput, TtsProvider};
use crate::config::types::AzureSection;
use async_trait::async_trait;
use std::path::Path;

pub(super) struct AzureSpeechEngine {
    config: AzureSection,
    proxy: Option<super::common::ProxyConfig>,
}

impl AzureSpeechEngine {
    pub(super) fn new(config: AzureSection, proxy: Option<super::common::ProxyConfig>) -> Self {
        Self { config, proxy }
    }

    /// 内部合成逻辑（不含重试）
    async fn synthesize_once(&self, text: &str, voice_name: &str, output_path: &Path) -> Result<TtsOutput, TTSError> {
        // 1. 构建 HTTP 请求
        // 2. 发送请求
        // 3. 解析响应
        // 4. 写入音频文件
        // 5. 返回 TtsOutput
    }
}

#[async_trait]
impl TtsProvider for AzureSpeechEngine {
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        _rate: f64,
        _pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError> {
        if text.is_empty() {
            return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
        }
        // 使用 common.rs 的 retry_loop 包装内部调用
        super::common::retry_loop(
            || self.synthesize_once(text, voice_name, output_path),
        ).await
    }
}
```

### Pattern 2: 前缀解析模式（统一辅助函数）

4 个引擎使用 `engine:payload` 前缀格式。在 `common.rs` 中提供统一解析：

```rust
/// 解析引擎前缀，返回去除前缀后的 voice name 子串
///
/// # 示例
/// ```
/// assert_eq!(parse_engine_prefix("soulvoice:speech:model:voice:id", &["soulvoice:"]), "speech:model:voice:id");
/// assert_eq!(parse_engine_prefix("tencent:101001", &["tencent:"]), "101001");
/// assert_eq!(parse_engine_prefix("zh-CN-XiaoyiNeural", &["soulvoice:", "tencent:", "qwen3:", "indextts2:"]), "zh-CN-XiaoyiNeural");
/// ```
pub fn parse_engine_prefix<'a>(voice_name: &'a str, prefixes: &[&str]) -> &'a str {
    for prefix in prefixes {
        if let Some(stripped) = voice_name.strip_prefix(prefix) {
            return stripped;
        }
    }
    voice_name
}
```

### Pattern 3: Azure Speech 智能回退

```rust
/// 判断是否应使用 Azure Speech Services REST API（V2）
///
/// 对齐 Python 版 `should_use_azure_speech_services()`：
/// - voice_name 以 `-V2` 结尾 → V2
/// - voice_name 匹配 Azure Neural 格式 `xx-XX-NameNeural` → V2
/// - 否则 → 回退 Edge-TTS（V1）
fn should_use_azure_services(voice_name: &str) -> bool {
    let name = parse_voice_name(voice_name);
    if name.ends_with("-V2") {
        return true;
    }
    // 正则匹配: xx-XX-NameNeural
    regex_lite::Regex::new(r"^[a-z]{2}-[A-Z]{2}-\w+Neural$")
        .map(|re| re.is_match(&name))
        .unwrap_or(false)
}

/// 去除 voice name 中的 Gender 后缀（如 "-Female"、"-Male"）
/// 对齐 Python 版 `parse_voice_name()`
fn parse_voice_name(name: &str) -> String {
    name.replace("-Female", "").replace("-Male", "").trim().to_string()
}
```

路由器中的 azure_speech 分支：

```rust
"azure_speech" => {
    if should_use_azure_services(voice_name) {
        // V2: Azure Speech REST API
        let engine = AzureSpeechEngine::new(config.azure.clone(), proxy_config);
        engine.synthesize(text, voice_name, rate, pitch, output_path).await
    } else {
        // V1: 回退 Edge-TTS
        let (proxy_enabled, proxy_http, proxy_https) = /* 从 proxy 解构 */;
        let edge = EdgeTtsEngine::new(proxy_enabled, proxy_http, proxy_https);
        edge.synthesize(text, voice_name, rate, pitch, output_path).await
    }
}
```

### Pattern 4: 共享 retry_loop

```rust
/// 统一重试循环
/// D-05: 3 次重试，间隔 1 秒
pub async fn retry_loop<F, Fut>(f: F) -> Result<TtsOutput, TTSError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<TtsOutput, TTSError>>,
{
    let max_attempts = 4; // 1 initial + 3 retries
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match f().await {
            Ok(output) => return Ok(output),
            Err(e) => {
                if matches!(&e, TTSError::AuthenticationFailed(_)) {
                    return Err(e);
                }
                tracing::warn!("TTS 合成尝试 {} 失败: {}", attempt, e);
                last_error = Some(e);
                if attempt < max_attempts {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(TTSError::RetryExhausted(format!(
        "TTS 重试 3 次后仍失败: {}",
        last_error.map_or("未知错误".to_string(), |e| e.to_string())
    )))
}
```

### Anti-Patterns to Avoid
- **在引擎内直接 panic/unwrap：** 所有 HTTP 请求解析必须使用 `?` 或 `map_err` 转换到 `TTSError`
- **为每个请求创建新的 reqwest::Client：** 应在引擎 struct 中持有复用 Client 实例（`Arc<Client>`）
- **硬编码端点 URL 在接口中：** 端点 URL 应来自配置或模块级常量（如 Azure 区域替换）
- **忽略非 2xx 状态码：** 所有引擎必须检查 HTTP 状态码，非 2xx 应返回对应的 `TTSError` 变体

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP 客户端 | 自定义 TCP 连接 | `reqwest` | 已有依赖，支持 TLS/代理/超时/重定向 |
| 代理连接 | 手动 HTTP CONNECT | `reqwest::Proxy` | reqwest 原生支持代理，无需像 Edge-TTS 那样手动实现隧道 |
| JSON 解析 | 手动字符串拼接 | `serde_json` | 已有依赖，类型安全 |
| HMAC 签名 | 手写 HMAC 算法 | `hmac` + `sha2` crate | 密码学协议不应手写 |
| Base64 编解码 | 手写 Base64 | `base64` crate | 已有依赖 |
| multipart 构建 | 手动构造 multipart 边界 | `reqwest::multipart` | 边界字符串生成和流式处理复杂 |

## Common Pitfalls

### Pitfall 1: Doubao Authorization 头格式错误
**What goes wrong:** 使用标准 `Bearer {token}`（空格）而不是 `Bearer;{token}`（分号）
**Why it happens:** Bearer Token 标准的常规格式是空格，但火山引擎 API 使用分号
**How to avoid:** 在 `doubaotts.rs` 的 HTTP 请求头中明确使用 `Bearer;` + token，并在注释中说明这是非标准但必要的格式
**Warning signs:** API 返回 401 Unauthorized

### Pitfall 2: Azure REST API 无词边界
**What goes wrong:** 调用方期望 `TtsOutput::word_boundaries` 非空
**Why it happens:** Azure Speech REST API（V2）不返回词边界事件，只有 SDK 模式才支持
**How to avoid:** `AzureSpeechEngine::synthesize_once` 应返回空的 `word_boundaries: vec![]`。调用方（Phase 6 流水线）需处理空词边界的场景
**Warning signs:** 流水线中字幕生成为空

### Pitfall 3: Tencent TC3 签名时间偏差
**What goes wrong:** 签名验证失败（401），因为 `X-TC-Timestamp` 和 `CredentialScope` 的日期不匹配
**Why it happens:** TC3 签名的 Date 在 CredentialScope 中必须是 UTC 日期，与 X-TC-Timestamp 的当天日期一致
**How to avoid:** 在同一函数中同时生成 timestamp 和 date string，确保使用同一时间源。不要在代码不同处分别取当前时间
**Warning signs:** `The signature has expired` 或 `The action did not exist` 错误

### Pitfall 4: IndexTTS2 文件路径编码和重试
**What goes wrong:** 参考音频文件路径包含非 ASCII 字符（如中文），或重试时文件句柄泄漏
**Why it happens:** multipart 文件上传需处理编码；重试循环中需要重新打开文件句柄
**How to avoid:** 使用 `reqwest::multipart::Part::bytes(tokio::fs::read(path).await?)` 替代 `form.file()` 以避免路径编码问题。重试时重新读取文件到内存而非复用文件句柄
**Warning signs:** 文件上传失败或重试时「文件已关闭」错误

### Pitfall 5: reqwest Client 复用 vs 创建
**What goes wrong:** 为每个合成请求创建新 `reqwest::Client`，导致连接池浪费和 DNS 重新解析
**Why it happens:** `Client::new()` 会在内部建立连接池，频繁创建会导致性能下降
**How to avoid:** 将 `reqwest::Client` 作为引擎 struct 字段（`Arc<Client>`），在 `new()` 中创建一次后复用
**Warning signs:** 高并发下 TCP 连接数激增

## Code Examples

### Each Engine的API协议详解

#### 1. Azure Speech TTS (TTS-04)

```rust
// 端点: https://{region}.tts.speech.microsoft.com/cognitiveservices/v1
// 认证: Ocp-Apim-Subscription-Key header
// 请求体: SSML XML
// 响应: 原始音频字节 (audio/mpeg)

// 参考: D-04 (手写 REST API, 不用 SDK)
// 参考: D-01/D-02 (智能回退，路由器内联分派)

let url = format!("https://{}.tts.speech.microsoft.com/cognitiveservices/v1", region);

let ssml = format!(
    r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="{}"><voice name="{}">{}</voice></speak>"#,
    lang, voice_name, escaped_text
);

let response = client
    .post(&url)
    .header("Ocp-Apim-Subscription-Key", &speech_key)
    .header("Content-Type", "application/ssml+xml")
    .header("X-Microsoft-OutputFormat", "audio-48khz-192kbitrate-mono-mp3")
    .header("User-Agent", "NarratoAI")
    .body(ssml)
    .timeout(Duration::from_secs(60))
    .send()
    .await
    .map_err(|e| TTSError::ConnectionFailed(e.to_string()))?;

let status = response.status();
if !status.is_success() {
    return Err(TTSError::SynthesisFailed(format!("Azure TTS 返回 {}", status)));
}

let audio_bytes = response.bytes().await
    .map_err(|e| TTSError::SynthesisFailed(e.to_string()))?;

// REST API 不返回词边界，word_boundaries 为空
tokio::fs::write(output_path, &audio_bytes).await?;

Ok(TtsOutput {
    audio_file_path: output_path.to_path_buf(),
    word_boundaries: vec![],
    duration: 0.0, // REST API 不返回时长信息
})
```

#### 2. Tencent TTS (TTS-05) - TC3-HMAC-SHA256 签名

```rust
// 端点: POST https://tts.tencentcloudapi.com
// 认证: TC3-HMAC-SHA256 (需手动实现)
// 服务名: "tts"
// 请求体: JSON
// 响应: JSON {"Response": {"Audio": "<base64>", "Subtitles": [...]}, "RequestId": "..."}
//
// Python 版使用 tencentcloud-sdk-python，Rust 版需手动实现 TC3 签名

// TC3 签名步骤:

// Step 1: 构建 CanonicalRequest
let http_request_method = "POST";
let canonical_uri = "/";
let canonical_query_string = "";
let payload_hash = hex::encode(sha2::Sha256::digest(&request_body));
let canonical_headers = "content-type:application/json; charset=utf-8\nhost:tts.tencentcloudapi.com\n";
let signed_headers = "content-type;host";
let canonical_request = format!(
    "{}\n{}\n{}\n{}\n{}\n{}",
    http_request_method,
    canonical_uri,
    canonical_query_string,
    canonical_headers,
    signed_headers,
    payload_hash,
);

// Step 2: 构建 StringToSign
let algorithm = "TC3-HMAC-SHA256";
let timestamp = chrono::Utc::now().timestamp();
let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
let credential_scope = format!("{}/tts/tc3_request", date);
let hashed_canonical_request = hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()));
let string_to_sign = format!(
    "{}\n{}\n{}\n{}",
    algorithm,
    timestamp,
    credential_scope,
    hashed_canonical_request,
);

// Step 3: 派生签名密钥
use hmac::Mac;
use hmac::Hmac;
use sha2::Sha256;

let secret_key = format!("TC3{}", tencent_secret_key);
let secret_date = hmac_sha256(secret_key.as_bytes(), date.as_bytes());
let secret_service = hmac_sha256(&secret_date, b"tts");
let secret_signing = hmac_sha256(&secret_service, b"tc3_request");
let signature = hex::encode(hmac_sha256(&secret_signing, string_to_sign.as_bytes()));

// Step 4: 构建 Authorization header
let authorization = format!(
    "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
    tencent_secret_id,
    credential_scope,
    signed_headers,
    signature,
);

// 注意: 请求体格式参考 Python 版 tencent_tts() 的参数映射
let request_body = serde_json::json!({
    "Text": text,
    "SessionId": uuid::Uuid::new_v4().to_string(),
    "VoiceType": voice_type.parse::<i64>().unwrap_or(101001),
    "Speed": (rate - 1.0) * 2.0, // 映射: 1.0 -> 0, 1.5 -> 1.0, 0.5 -> -1.0
    "SampleRate": 16000,
    "Codec": "mp3",
    "ProjectId": 0,
    "ModelType": 1,
    "PrimaryLanguage": 1,
    "EnableSubtitle": true,
});

// 响应解析:
// response["Response"]["Audio"] -> base64 decode -> 音频字节
// response["Response"]["Subtitles"] -> 数组，每个元素有 BeginTime, EndTime, Text
// word_boundaries 由 Subtitles 生成: start_offset = BeginTime * 10000, end_offset = EndTime * 10000
```

#### 3. SoulVoice TTS (TTS-06)

```rust
// 端点: POST {api_url} (默认 https://tts.scsmtech.cn/tts)
// 认证: Authorization: Bearer {api_key}
// Content-Type: application/json
// 请求体: {"text": "...", "model": "...", "voice": "...", "speed": 1.0}
// 响应: 原始音频字节 (直接 body，非 JSON)

let response = client
    .post(&api_url)
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
        "text": text,
        "model": model,
        "voice": parsed_voice,  // 已去除 "soulvoice:" 前缀
        "speed": speed,
    }))
    .timeout(Duration::from_secs(60))
    .send()
    .await?;

let audio_bytes = response.bytes().await?;

tokio::fs::write(output_path, &audio_bytes).await?;

// SoulVoice 不支持词边界
Ok(TtsOutput {
    audio_file_path: output_path.to_path_buf(),
    word_boundaries: vec![],
    duration: 0.0,
})
```

#### 4. Qwen TTS (TTS-07)

```rust
// 端点: POST https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization
// 或 POST https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation
// 认证: Authorization: Bearer {api_key}
// 请求体: {"model": "qwen3-tts-flash", "input": {"text": "...", "voice": "..."}}
// 响应: {"output": {"audio": {"url": "https://...", "duration": 12.5}}}
// 需要二次 GET 下载 audio_url

let payload = serde_json::json!({
    "model": model_name,
    "input": {
        "text": text,
        "voice": parsed_voice,  // 已去除 "qwen3:" 前缀
    },
});

let response = client
    .post("https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization")
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .json(&payload)
    .timeout(Duration::from_secs(60))
    .send()
    .await?;

let result: serde_json::Value = response.json().await?;
let audio_url = result["output"]["audio"]["url"]
    .as_str()
    .ok_or_else(|| TTSError::SynthesisFailed("Qwen 响应中无 audio.url".to_string()))?;
let duration = result["output"]["audio"]["duration"].as_f64().unwrap_or(0.0);

// 下载音频
let audio_response = client
    .get(audio_url)
    .timeout(Duration::from_secs(60))
    .send()
    .await?;
let audio_bytes = audio_response.bytes().await?;

tokio::fs::write(output_path, &audio_bytes).await?;

Ok(TtsOutput {
    audio_file_path: output_path.to_path_buf(),
    word_boundaries: vec![],
    duration,
})
```

#### 5. IndexTTS2 (TTS-08)

```rust
// 端点: POST {api_url} (默认 http://127.0.0.1:8081/tts)
// 认证: 无（本地服务）
// 请求体: multipart/form-data
//   - prompt_audio: 参考音频文件（二进制）
//   - text: 合成文本
//   - infer_mode: 推理模式
//   - temperature, top_p, top_k, do_sample, num_beams, repetition_penalty: 高级参数
// 响应: 原始音频字节

use reqwest::multipart;

let reference_audio_bytes = tokio::fs::read(&ref_audio_path).await
    .map_err(|e| TTSError::SynthesisFailed(format!("读取参考音频失败: {}", e)))?;

let form = reqwest::multipart::Form::new()
    .part("prompt_audio", reqwest::multipart::Part::bytes(reference_audio_bytes)
        .file_name("reference.wav")
        .mime_str("audio/wav")
        .map_err(|e| TTSError::SynthesisFailed(e.to_string()))?)
    .text("text", text.to_string())
    .text("infer_mode", infer_mode)
    .text("temperature", temperature.to_string())
    .text("top_p", top_p.to_string())
    .text("top_k", top_k.to_string())
    .text("do_sample", do_sample.to_string())
    .text("num_beams", num_beams.to_string())
    .text("repetition_penalty", repetition_penalty.to_string());

let response = client
    .post(&api_url)
    .multipart(form)
    .timeout(Duration::from_secs(120)) // IndexTTS2 推理时间长，需 120s 超时
    .send()
    .await?;

let audio_bytes = response.bytes().await?;
tokio::fs::write(output_path, &audio_bytes).await?;

// IndexTTS2 不支持词边界
Ok(TtsOutput {
    audio_file_path: output_path.to_path_buf(),
    word_boundaries: vec![],
    duration: 0.0,
})
```

#### 6. Doubao TTS (TTS-09)

```rust
// 端点: POST https://openspeech.bytedance.com/api/v1/tts
// 认证: Authorization: Bearer;{token} (注意分号!)
// Content-Type: application/json
// 请求体:
// {
//   "app": {"appid": "...", "token": "...", "cluster": "volcano_tts"},
//   "user": {"uid": "NarratoAI"},
//   "audio": {"voice_type": "...", "encoding": "mp3", "rate": 24000,
//             "speed_ratio": 1.0, "volume_ratio": 1.0, "pitch_ratio": 1.0},
//   "request": {"reqid": "<uuid>", "text": "...", "text_type": "plain", "operation": "query"}
// }
// 响应: {"code": 3000, "message": "Success", "data": "<base64 audio>"}

let payload = serde_json::json!({
    "app": {
        "appid": appid,
        "token": token,
        "cluster": cluster,
    },
    "user": {
        "uid": "NarratoAI",
    },
    "audio": {
        "voice_type": voice_type,
        "encoding": "mp3",
        "rate": 24000,
        "speed_ratio": speed,
        "volume_ratio": volume,
        "pitch_ratio": pitch,
    },
    "request": {
        "reqid": uuid::Uuid::new_v4().to_string(),
        "text": text,
        "text_type": "plain",
        "operation": "query",
    },
});

// 如果 silence_duration > 0，添加到 audio 段
if silence_duration > 0.0 {
    payload["audio"]["silence_duration"] = serde_json::json!(silence_duration);
}

let response = client
    .post("https://openspeech.bytedance.com/api/v1/tts")
    .header("Authorization", format!("Bearer;{}", token)) // 分号! 不是空格
    .header("Content-Type", "application/json")
    .json(&payload)
    .timeout(Duration::from_secs(60))
    .send()
    .await?;

let result: serde_json::Value = response.json().await?;

if result["code"].as_i64() != Some(3000) {
    return Err(TTSError::SynthesisFailed(format!(
        "Doubao TTS 错误: {}",
        result["message"].as_str().unwrap_or("未知错误")
    )));
}

let audio_base64 = result["data"].as_str()
    .ok_or_else(|| TTSError::SynthesisFailed("Doubao 响应中无 data 字段".to_string()))?;

use base64::Engine;
let audio_bytes = base64::engine::general_purpose::STANDARD
    .decode(audio_base64)
    .map_err(|e| TTSError::SynthesisFailed(format!("Base64 解码失败: {}", e)))?;

tokio::fs::write(output_path, &audio_bytes).await?;

Ok(TtsOutput {
    audio_file_path: output_path.to_path_buf(),
    word_boundaries: vec![],
    duration: 0.0,
})
```

## State of the Art

| Old Approach (Python) | Current Approach (Rust) | When Changed | Impact |
|----------------------|------------------------|--------------|--------|
| azure-cognitiveservices-speech SDK | 手动 reqwest REST API | Phase 12 | 更轻量，无 SDK 依赖，但不支持词边界 |
| tencentcloud-sdk-python | 手动 TC3-HMAC-SHA256 + reqwest | Phase 12 | 更轻量，无 SDK 依赖 |
| dashscope SDK | 手动 reqwest REST API | Phase 12 | 更轻量，无 SDK 依赖 |
| dacherry/soulvoice SDK | 手动 reqwest REST API | Phase 12 | 简化依赖链 |

## Don't Hand-Roll (Dependency Summary)

| Problem | Don't Build | Use Instead | Crate |
|---------|-------------|-------------|-------|
| HMAC-SHA256 | 手写 HMAC | `hmac` + `sha2` | hmac 0.13, sha2 0.10 |
| SHA-256 hash | 手写 SHA | `sha2` | sha2 0.10 |
| Hex encoding | 手写 hex | `hex` | hex 0.4 |
| Date formatting | 手动计算日历 | `chrono` | chrono 0.4 |
| multipart/form-data | 手动构造边界 | `reqwest::multipart` | reqwest (multipart feature) |
| JSON 构造 | 手动序列化 | `serde_json::json!` | serde_json 1.0 (已有) |
| Base64 解码 | 手写 Base64 | `base64` | base64 0.22 (已有) |

## Assumptions Log

> Claims tagged `[ASSUMED]` that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Qwen DashScope REST API 端点使用 `https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization` | Qwen TTS | 端点不对会导致 404。Python 版使用 SDK 封装，原始 HTTP 端点需实际测试确认 |
| A2 | reqwest `json` feature 是 `"json"`（准确名称） | Standard Stack | reqwest 0.13 的 JSON 功能 feature 名需确认。如果 feature 名不同则编译失败 |
| A3 | IndexTTS2 multipart 中 `prompt_audio` 字段名正确 | Code Examples | 字段名不对会导致后端无法识别上传的音频文件。需对照实际 IndexTTS2 源码确认 |
| A4 | reqwest `multipart::Part::mime_str()` 方法名正确 | Code Examples | 方法名在 reqwest 0.13 中可能为 `mime()` 或 `mime_str()`，需检查 API |

## Open Questions (RESOLVED)

1. **Qwen TTS REST API 端点确认** — RESOLVED: 实现时使用 `https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization` 作为主端点，集成测试验证。Python 版 `dashscope.MultiModalConversation.call()` 底层对应此 REST 端点
   - What we know: Python 版使用 `dashscope.MultiModalConversation.call()`，底层可能是 `https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation` 或 `https://dashscope.aliyuncs.com/api/v1/services/audio/tts/customization`
   - What's unclear: 精确的 HTTP 端点和请求体格式 — RESOLVED: 使用 `/audio/tts/customization` 端点, `input.text` + `input.voice` 请求体格式
   - Recommendation: 实现时先使用现有公开参考端点，用集成测试验证

2. **reqwest Client 复用策略** — RESOLVED: 引擎 struct 中持有 `reqwest::Client`（`build_client()` 创建），每个引擎在 `new()` 中创建一次，实例化时复用
   - What we know: `reqwest::Client` 内部维护连接池，最优做法是全局复用
   - What's unclear: 复用方式（`Arc<Client>` 全局 static 还是传入引擎） — RESOLVED: 每个引擎在 `new()` 中通过 `build_client()` 创建，不跨引擎共享 `Arc<Client>`
   - Recommendation: 在 `synthesize()` 路由器中创建一次 `Arc<Client>`，传入每个引擎

3. **IndexTTS2 的字段名确认为 multipart** — RESOLVED: 字段名已确认——`prompt_audio`（文件），`text`（合成文本），`infer_mode`（推理模式），`temperature`/`top_p`/`top_k`/`do_sample`/`num_beams`/`repetition_penalty`（高级参数）
   - What we know: Python 版使用 `files={'prompt_audio': open(path, 'rb')}`
   - What's unclear: 其他字段名（text, infer_mode 等）是否准确 — RESOLVED: 从 Python 版 indextts2_tts() 确认所有字段名
   - Recommendation: 直接参考 Python 版 indextts2_tts() 中的字段名

4. **Azure Speech REST API 返回格式确认** — RESOLVED: Azure Speech REST API 直接返回二进制音频数据，非 JSON 封装。响应 Content-Type 为 `audio/mpeg`，可调用 `response.bytes()` 获取
   - What we know: REST API 返回 `application/ssml+xml` 响应，Response 是原始音频字节
   - What's unclear: 是直接返回音频数据还是包装在 JSON 中 — RESOLVED: 直接返回二进制音频数据（`audio/mpeg`），非 JSON 包装
   - Recommendation: 从官方文档确认：REST API 直接返回二进制音频数据

## Environment Availability

> Skip this section — Phase 12 has no external tools beyond what Phase 3 already requires. All 6 engines communicate via HTTP to external APIs. No local databases, runtimes, or CLI tools needed beyond the existing Rust toolchain and `reqwest` dependency.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none — Cargo.toml dev-dependencies |
| Quick run command | `cargo test --lib tts -- --nocapture` |
| Full suite command | `cargo test --lib` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TTS-04 | Azure engine returns TtsOutput on success | unit (mock) | `cargo test --lib tts::azure_speech::tests` | :x: Wave 0 |
| TTS-05 | Tencent engine returns TtsOutput on success | unit (mock via wiremock) | `cargo test --lib tts::tencent_tts::tests` | :x: Wave 0 |
| TTS-06 | SoulVoice engine returns TtsOutput on success | unit (mock) | `cargo test --lib tts::soulvoice::tests` | :x: Wave 0 |
| TTS-07 | Qwen engine returns TtsOutput on success | unit (mock) | `cargo test --lib tts::qwen_tts::tests` | :x: Wave 0 |
| TTS-08 | IndexTTS2 returns TtsOutput on success | unit (mock) | `cargo test --lib tts::indextts2::tests` | :x: Wave 0 |
| TTS-09 | Doubao returns TtsOutput on success | unit (mock) | `cargo test --lib tts::doubaotts::tests` | :x: Wave 0 |
| all | Unknown engine returns TTSError | integration | `cargo test --lib tts::mod::tests` | :white_check_mark: Phase 3 |
| all | Empty text returns SynthesisFailed | unit | per engine module | :x: Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib tts -- --nocapture`
- **Per wave merge:** `cargo test --lib`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/tts/soulvoice.rs` — unit tests with mocked HTTP response
- [ ] `src/tts/doubaotts.rs` — unit tests with mocked HTTP response + base64 decode
- [ ] `src/tts/qwen_tts.rs` — unit tests with mocked HTTP response + audio URL download
- [ ] `src/tts/indextts2.rs` — unit tests with mocked multipart upload
- [ ] `src/tts/azure_speech.rs` — unit tests with mocked SSML POST + smart routing
- [ ] `src/tts/tencent_tts.rs` — unit tests with mocked TC3 signature + JSON response

Note: All unit tests should use `wiremock` (already in dev-dependencies) to mock HTTP responses. Integration tests (requiring actual API keys) should be `#[ignore]`-gated.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | Yes | API keys stored in config.toml (gitignored). Engines read keys at runtime from config structs |
| V6 Cryptography | Yes | Tencent TC3 uses HMAC-SHA256 via `hmac` + `sha2` crate (never hand-roll) |
| V8 Data Protection | Yes | Audio output files written to temp/user-specified paths. No PII in audio data |

### Known Threat Patterns for reqwest-based TTS

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key in error messages | Information Disclosure | Log only truncated key (`key[..4]...`); never log full keys |
| Man-in-the-middle on API calls | Tampering | All production APIs use HTTPS. reqwest default-tls verifies certs |
| Unvalidated audio path traversal | Elevation of Privilege | `output_path` from caller — caller is the pipeline (trusted) not user input |
| TC3 signing key exposure | Information Disclosure | Signing key derived per-request, not stored. Secret key read from config at runtime |

### Key Protection Rules
- **Never log:** full API keys, secret IDs, secret keys, access tokens
- **Never include in error messages:** full API credentials. Truncate to `"{key[..4]}..."` if needed for debugging
- **Keep config.toml in .gitignore** — already enforced by Python project conventions
- **No API keys in binary** — keys read from config at runtime, not compiled in

## Sources

### Primary (HIGH confidence)
- [Codebase] `app/services/voice.py` — Python 版所有 7 个 TTS 引擎的完整实现（权威参考）
- [Codebase] `src/tts/mod.rs` — Rust TTS trait 定义、路由器、edge_tts 分支
- [Codebase] `src/tts/edge_tts.rs` — EdgeTtsEngine 完整实现（引擎模式参考）
- [Codebase] `src/error.rs` — TTSError 5 变体
- [Codebase] `src/config/types.rs` — 6 个 TTS 引擎的配置结构体（已正确定义）
- [Codebase] `Cargo.toml` — 现有依赖清单
- [Codebase] `.planning/phases/12-additional-tts-engines/12-CONTEXT.md` — 9 条锁定决策 (D-01 到 D-09)
- [WebSearch] learn.microsoft.com — Azure Speech REST API 端点 `{region}.tts.speech.microsoft.com/cognitiveservices/v1`，`Ocp-Apim-Subscription-Key` 认证，SSML 请求体，返回原始音频字节
- [WebSearch] cloud.tencent.com — TC3-HMAC-SHA256 签名算法：CanonicalRequest -> StringToSign -> HMAC-SHA256 派生链 -> Authorization header
- [WebSearch] dashscope.aliyuncs.com — Qwen TTS REST API：`/api/v1/services/audio/tts/customization`，`input.text` + `input.voice`，返回 `output.audio.url`
- [WebSearch] volcengine.com — Doubao TTS：`https://openspeech.bytedance.com/api/v1/tts`，`Bearer;{token}` 认证（分号），返回 JSON 含 base64 data
- [WebSearch] docs.rs/reqwest/0.13 — reqwest 0.13 multipart API：`reqwest::multipart::Form::new().part("field", Part::bytes(bytes))`

### Secondary (MEDIUM confidence)
- [Codebase] `config.example.toml` — 各引擎配置字段的默认值
- [Codebase] `app/config/config.py` — Python 配置加载方式，印证配置段结构

### Tertiary (LOW confidence)
- [ASSUMED] Qwen TTS 精确端点路径 — Python 使用 SDK 封装，底层 HTTP 端点需通过实际测试确认

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — 所有 crate 版本和 feature 需求已验证（已知 Cargo.toml 和 docs.rs）
- Architecture: HIGH — 6 引擎独立文件 + common.rs 模式已在 edge_tts.rs 验证
- Pitfalls: HIGH — 从 Python 版实际实现和 Rust HTTP 常见问题推导
- API 协议: HIGH — SoulVoice/Doubao/IndexTTS2 从 Python 代码直接推导；Azure REST/Tencent TC3 从官方文档验证；Qwen 端点存在 LOW 不确定性

**Research date:** 2026-04-30
**Valid until:** 2026-05-30 (30 days — crate versions stable, APIs mature)

## Wave Implementation Plan

### Wave 1: common.rs + SoulVoice + Doubao

**新增文件:**
- `src/tts/common.rs` — retry_loop, apply_proxy, write_audio_bytes, parse_engine_prefix
- `src/tts/soulvoice.rs` — SoulVoiceEngine (simplest, 验证 common.rs 模式)
- `src/tts/doubaotts.rs` — DoubaoTtsEngine (验证 base64 解码)

**修改文件:**
- `src/tts/mod.rs` — 添加 `mod common; mod soulvoice; mod doubaotts;` + 两个 match 分支

**Cargo.toml 变更:** 添加 `reqwest = { version = "0.13", features = ["json"] }`

**测试:** 每个引擎的单元测试 (wiremock)，验证 common.rs 函数

### Wave 2: Qwen + IndexTTS2

**新增文件:**
- `src/tts/qwen_tts.rs` — QwenTtsEngine (验证二次下载模式)
- `src/tts/indextts2.rs` — IndexTts2Engine (验证 multipart 文件上传)

**修改文件:**
- `src/tts/mod.rs` — 添加 `mod qwen_tts; mod indextts2;` + 两个 match 分支

**Cargo.toml 变更:** `reqwest` 添加 `"multipart"` feature

**测试:** 每个引擎的单元测试 (wiremock)，IndexTTS2 需 mock multipart 上传

### Wave 3: Azure Speech + Tencent

**新增文件:**
- `src/tts/azure_speech.rs` — AzureSpeechEngine (验证 SSML + 智能回退)
- `src/tts/tencent_tts.rs` — TencentTtsEngine (验证 TC3-HMAC-SHA256)

**修改文件:**
- `src/tts/mod.rs` — 添加 `mod azure_speech; mod tencent_tts;` + match 分支 + 智能回退条件判断

**Cargo.toml 变更:** 添加 `hmac = "0.13"`, `sha2 = "0.10"`, `hex = "0.4"`, `chrono = "0.4"`

**测试:** Azure 智能回退逻辑单元测试 + Tencent TC3 签名单元测试 (wiremock)

---

*Phase: 12-additional-tts-engines*
*Research completed: 2026-04-30*
