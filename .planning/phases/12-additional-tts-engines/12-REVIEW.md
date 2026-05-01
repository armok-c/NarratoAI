---
phase: 12-additional-tts-engines
reviewed: 2026-05-02T02:30:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - Cargo.toml
  - src/config/defaults.rs
  - src/config/types.rs
  - src/tts/azure_speech.rs
  - src/tts/common.rs
  - src/tts/doubaotts.rs
  - src/tts/edge_tts.rs
  - src/tts/indextts2.rs
  - src/tts/mod.rs
  - src/tts/qwen_tts.rs
  - src/tts/soulvoice.rs
  - src/tts/tencent_tts.rs
findings:
  critical: 0
  warning: 0
  info: 5
  total: 5
status: clean
---

# Phase 12: Code Review Report -- Additional TTS Engines (Iteration 6)

**Reviewed:** 2026-05-02T02:30:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** clean

## Summary

对 Phase 12 的 12 个源文件进行第六次审查。前五次迭代的所有修复均已确认正确应用，未发现回归。

**前次修复验证（全部正确）：**
- WR-01（pitch_ratio）：`doubaotts.rs:55` 现使用 `self.config.pitch`，不再硬编码 `1.0`
- WR-01（SSML XML 转义）：`common.rs` 中 `escape_xml_attr()` 已添加，`azure_speech.rs:279-280` 和 `edge_tts.rs:95-96` 均已使用
- WR-01（edge_tts trim）：`edge_tts.rs:601,604` 现使用 `.trim().is_empty()`，与其他 6 个引擎一致
- WR-02（SSRF）：`qwen_tts.rs:90-97` URL scheme 校验允许 `https://`、`http://127.0.0.1`、`http://localhost`
- WR-03（文件检查）：`indextts2.rs:43-51` 添加 `tokio::fs::metadata()` 前置验证

**本轮新增发现：** 0 个严重、0 个警告、5 个信息（均为历史遗留）。

**验证结果：**
- `cargo check`：编译成功（1 个 dead_code 警告，为 `get_azure_voices` 暂未引用，已知）
- `cargo test --lib tts`：78 passed, 0 failed, 1 ignored

## File-by-File Analysis

### common.rs (197 lines)
- `ProxyConfig` — `from_proxy()`/`apply_to_client()` 处理 None 和无效 URL 均正确
- `build_client()` — 简洁封装 reqwest ClientBuilder
- `retry_loop()` — 4 次尝试，`AuthenticationFailed` 短路不重试，`RetryExhausted` 带最后错误信息
- `escape_xml_attr()` — 覆盖全部 5 个 XML 特殊字符（`& < > " '`）
- `parse_engine_prefix()` — 不匹配时返回原始 voice_name，无分配
- 6 个测试，全部通过

### soulvoice.rs (225 lines)
- `synthesize_once()` — 可配置 api_url（默认 `https://tts.scsmtech.cn/tts`），Bearer auth，JSON body
- `TtsProvider::synthesize()` — `text.trim().is_empty()` 验证，retry_loop + 失败清理
- 5 个测试（含 wiremock），全部通过

### doubaotts.rs (335 lines)
- `synthesize_once()` — 嵌套 JSON payload（app/user/audio/request 四段），`Bearer;{token}` 认证（分号非空格），base64 解码 + 业务码 3000 检查
- `pitch_ratio` 使用 `self.config.pitch`（WR-01 修复已确认）
- 7 个测试，全部通过

### qwen_tts.rs (279 lines)
- `synthesize_once()` — 二次下载模式（POST → 取 audio_url → GET 音频字节），SSRF 防护已就位
- URL scheme 校验允许 `https://`、`http://127.0.0.1`、`http://localhost`（WR-02 修复已确认）
- 5 个测试（含 wiremock），全部通过

### indextts2.rs (270 lines)
- `synthesize_once()` — multipart/form-data 上传参考音频 + 文本参数，120s 超时
- 参考音频文件存在性前置验证（WR-03 修复已确认）
- 每次重试重新读取文件到内存（retry-safe），使用 `Part::bytes()` 避免路径编码问题
- 5 个测试（含 wiremock），全部通过

### azure_speech.rs (516 lines)
- `should_use_azure_services()` — OnceLock 缓存正则，-V2 后缀或 Neural 格式匹配
- `synthesize_once()` — SSML 构建：XML 转义文本 + 控制字符过滤 + `escape_xml_attr` 转义属性值
- -V2 后缀在 SSML 构建前剥离（防止 Azure API 返回 400）
- 12 个测试（含 wiremock），全部通过

### tencent_tts.rs (444 lines)
- `synthesize_at_url()` — 完整 TC3-HMAC-SHA256 手动签名（无 SDK）
- 时间戳+日期从同一 `chrono::Utc::now()` 获取（防止时间偏差）
- HMAC 派生链：SecretKey → SecretDate → SecretService → SecretSigning，每步 `.map_err()`
- WordBoundary 解析：兼容整数和浮点 JSON 值，负值钳位到 0
- 7 个测试（含 wiremock + HMAC 类型验证），全部通过

### edge_tts.rs (837 lines)
- `build_ssml()` — 使用 `escape_xml_attr()` 转义属性值，XML 转义文本内容 + 控制字符过滤
- `TtsProvider::synthesize()` — `text.trim().is_empty()` 和 `voice_name.trim().is_empty()` 验证（WR-01 iteration 5 修复已确认，lines 601, 604）
- WebSocket 连接支持 HTTP CONNECT 代理隧道：IPv6 地址、SOCKS5 拒绝、1xx 中间响应处理、TLS 升级
- 20+ 个测试（含二进制解析、代理逻辑），全部通过

### mod.rs (236 lines)
- 8 个引擎模块声明，`TtsProvider` trait 定义，`synthesize()` 字符串路由器
- `azure_speech` 分支含智能回退逻辑（should_use_azure_services 判断 → Azure REST API 或 Edge-TTS 回退）
- 3 个测试（trait mock、未知引擎），全部通过

### config/types.rs (426 lines)
- 10 个配置 section，全部 `#[serde(deny_unknown_fields)]` + `#[serde(default)]`
- `DoubaoTTSSection.ak/.sk` 字段声明但 Rust 引擎未读取（保留用于未来 OAuth 流程或 Python 兼容）
- 5 个测试（完整配置、空配置、缺失 section、默认值、validate），全部通过

### config/defaults.rs (135 lines)
- 所有 section 的 Default 实现值与 `config.example.toml` 对齐
- `SoulVoiceSection` 默认 voice_uri 对齐 Python 版默认值

## Info

### IN-01: DoubaoTTSSection.ak 和 .sk 字段声明但 Rust 引擎未读取

**File:** `src/config/types.rs:171-174`
**Issue:** `DoubaoTTSSection` 中的 `ak` 和 `sk` 字段在 Rust 引擎中从未被读取。引擎使用 `appid`/`token` 直接认证。这两个字段为 Python 版 OAuth 流程或未来功能预留。保持现状合理，仅作为信息记录。

### IN-02: Qwen 和 IndexTTS2 引擎忽略 rate/pitch 参数

**File:** `src/tts/qwen_tts.rs:138-139`, `src/tts/indextts2.rs:119-120`
**Issue:** 两个引擎的 `TtsProvider::synthesize` 实现中 `_rate` 和 `_pitch` 参数以下划线标记为未使用。这是 API 限制（DashScope 和 IndexTTS2 不支持语速/音调调整），属于设计决策，不需要修改。

### IN-03: edge_tts.rs hardcoded HeaderValue parse 使用 .unwrap()（已有 SAFETY 注释）

**File:** `src/tts/edge_tts.rs:142-152`
**Issue:** `.unwrap()` 调用在硬编码 ASCII 头字符串上。已有 SAFETY 注释说明这些是编译时常量，`parse()` 不会失败。

（历史遗留，前五次审查均记录，保持不变。）

### IN-04: Doubao TTS token 在请求中传输两次（Header + Body）

**File:** `src/tts/doubaotts.rs:44,75`
**Issue:** `Authorization` 头使用 `Bearer;{token}` 格式，同时请求体 JSON 的 `app.token` 字段也包含同一 token。token 在 HTTP 请求中传输了两次。这不是安全漏洞（HTTPS 传输），且对齐 Python 版行为以保持 API 兼容性。

（历史遗留，前五次审查均记录，保持不变。）

### IN-05: get_azure_voices() 函数声明但未引用（dead_code 警告）

**File:** `src/tts/azure_speech.rs:42`
**Issue:** `get_azure_voices()` 函数目前未在任何调用方使用，产生 `dead_code` 编译警告。该函数为 UI/调试工具预留，保留合理。

（历史遗留，前四次审查均记录，保持不变。）

---
_Reviewed: 2026-05-02T02:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
