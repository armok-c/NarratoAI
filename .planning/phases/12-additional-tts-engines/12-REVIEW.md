---
phase: 12-additional-tts-engines
reviewed: 2026-05-02T02:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - src/tts/common.rs
  - src/tts/soulvoice.rs
  - src/tts/doubaotts.rs
  - src/tts/qwen_tts.rs
  - src/tts/indextts2.rs
  - src/tts/azure_speech.rs
  - src/tts/tencent_tts.rs
  - src/tts/edge_tts.rs
  - src/tts/mod.rs
  - src/config/types.rs
  - src/config/defaults.rs
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
---

# Phase 12: Code Review Report -- Additional TTS Engines (Re-review Iteration 5)

**Reviewed:** 2026-05-02T02:00:00Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

对 Phase 12 的 11 个源文件进行第五次审查。前四次迭代的所有修复均已确认正确应用，未发现回归。

**前次修复验证（全部正确）：**
- WR-01（pitch_ratio）：`doubaotts.rs:55` 现使用 `self.config.pitch`，不再硬编码 `1.0`
- WR-02（SSRF）：`qwen_tts.rs:90-97` URL scheme 校验允许 `https://`、`http://127.0.0.1`、`http://localhost`
- WR-03（文件检查）：`indextts2.rs:43-51` 添加 `tokio::fs::metadata()` 前置验证
- WR-01（SSML XML 转义）：`common.rs` 中 `escape_xml_attr()` 已添加，`azure_speech.rs:279-280` 和 `edge_tts.rs:95-96` 均已使用

**本轮新增发现：** 0 个严重、1 个警告、4 个信息。

**验证结果：**
- `cargo check`：编译成功（1 个 dead_code 警告，为 `get_azure_voices` 暂未引用，已知）
- `cargo test --lib tts`：78 passed, 0 failed, 1 ignored

## Warnings

### WR-01: edge_tts.rs 文本和 voice_name 验证缺少 trim()，允许纯空白字符串通过

**File:** `src/tts/edge_tts.rs:601,604`
**Issue:** `EdgeTtsEngine::synthesize()` 使用 `text.is_empty()` 和 `voice_name.is_empty()` 检查空值，而其他 6 个引擎全部使用 `text.trim().is_empty()` / `voice_name.trim().is_empty()`。Phase 12 的 WR-03 修复将 `trim()` 添加到所有 6 个新引擎，但遗漏了 `edge_tts.rs`（Phase 3 遗留代码）。

```rust
// edge_tts.rs:601-604 — 缺少 trim()
if text.is_empty() {
    return Err(TTSError::SynthesisFailed("text 不能为空".to_string()));
}
if voice_name.is_empty() {
    return Err(TTSError::SynthesisFailed("voice_name 不能为空".to_string()));
}
```

对比其他引擎（全部使用 `.trim().is_empty()`）：
- `soulvoice.rs:103,106` — `text.trim().is_empty()` ✓
- `doubaotts.rs:128,131` — `text.trim().is_empty()` ✓
- `qwen_tts.rs:141,144` — `text.trim().is_empty()` ✓
- `indextts2.rs:123,126` — `text.trim().is_empty()` ✓
- `azure_speech.rs:344,347` — `text.trim().is_empty()` ✓
- `tencent_tts.rs:254,257` — `text.trim().is_empty()` ✓

纯空白文本（如 `" "` 或 `"\t\n"`）将通过验证，生成包含空白内容的 SSML 发送到 Edge TTS 服务，可能导致合成错误或产生空音频。

**Fix:** 将 `edge_tts.rs:601` 改为 `text.trim().is_empty()`，将 `edge_tts.rs:604` 改为 `voice_name.trim().is_empty()`，与其他 6 个引擎保持一致。

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

（历史遗留，前四次审查均记录，保持不变。）

### IN-04: Doubao TTS token 在请求中传输两次（Header + Body）

**File:** `src/tts/doubaotts.rs:44,75`
**Issue:** `Authorization` 头使用 `Bearer;{token}` 格式，同时请求体 JSON 的 `app.token` 字段也包含同一 token。token 在 HTTP 请求中传输了两次。这不是安全漏洞（HTTPS 传输），且对齐 Python 版行为以保持 API 兼容性。

（历史遗留，前四次审查均记录，保持不变。）

---
_Reviewed: 2026-05-02T02:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
