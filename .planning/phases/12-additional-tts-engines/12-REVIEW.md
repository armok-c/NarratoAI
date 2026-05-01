---
phase: 12-additional-tts-engines
reviewed: 2026-05-01T13:00:00Z
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
  - src/tts/mod.rs
  - src/config/types.rs
  - src/config/defaults.rs
  - Cargo.toml
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 12: Code Review Report -- Additional TTS Engines (Re-review Iteration 3)

**Reviewed:** 2026-05-01T13:00:00Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

对 Phase 12 的 11 个源文件进行第三次审查（前两次迭代发现的所有问题均已确认修复）。本轮审查确认所有历史修复（CR-01、WR-01~WR-07）均正确应用，同时发现了 3 个新的警告级别问题和 4 个信息级问题。

**历史修复验证：**
- CR-01（Doubao pitch_ratio）：已修复，生产代码硬编码 `1.0`，测试断言同步更新
- WR-01（build_client Result 传播）：已修复，全部 6 个引擎 `new()` 返回 `Result<Self, TTSError>`，mod.rs 路由器正确处理
- WR-02（Tencent 时间戳负值）：已修复，添加 `if start_ms >= 0` / `if end_ms >= 0` 守卫子句
- WR-03（Doubao 测试断言不同步）：已修复，测试 `pitch_ratio` 断言改为 `1.0`

**本轮新增发现：** 0 个严重、3 个警告、4 个信息。

## Warnings

### WR-01: Azure/Edge-TTS SSML 中 voice_name 和 lang 未转义直接插入 XML 属性

**File:** `src/tts/azure_speech.rs:278-279` 和 `src/tts/edge_tts.rs:94-95`
**Issue:** `voice_name` 和 `lang` 参数未经 XML 转义直接插入 SSML 模板的 XML 属性值中。虽然 `text` 参数经过 `&amp;` / `&lt;` / `&gt;` 转义，但 `voice_name` 和 `lang` 完全未转义。

Azure Speech 引擎（azure_speech.rs:278-279）：
```rust
let ssml = format!(
    r#"<speak ... xml:lang="{}"><voice name="{}">..."#,
    lang, processed_voice_name, rate_str, pitch_str, escaped_text
);
```

Edge-TTS 引擎（edge_tts.rs:94-95）：
```rust
format!(
    r#"<speak ... xml:lang="{}"><voice name="{}">..."#,
    voice_lang, voice_name, rate_str, pitch_str, escaped_text
);
```

如果 `voice_name` 包含双引号 `"` 或其他 XML 特殊字符，SSML 结构将被破坏。例如 `voice_name = "zh-CN-\"Neural"` 将生成 `<voice name="zh-CN-"Neural">`，导致 Azure API 返回 400 错误。虽然本项目的 `voice_name` 通常来自配置文件（受控输入），但作为 defense-in-depth 措施，应对所有插入 XML 的外部值进行转义。

**Fix:** 对 `voice_name` 和 `lang` 进行 XML 属性值转义：

```rust
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// Azure Speech 使用:
let ssml = format!(
    r#"<speak ... xml:lang="{}"><voice name="{}">..."#,
    escape_xml_attr(&lang),
    escape_xml_attr(processed_voice_name),
    rate_str, pitch_str, escaped_text
);

// Edge-TTS 使用:
let ssml = format!(
    r#"<speak ... xml:lang="{}"><voice name="{}">..."#,
    escape_xml_attr(&voice_lang),
    escape_xml_attr(voice_name),
    rate_str, pitch_str, escaped_text
);
```

### WR-02: Qwen TTS audio_url 未验证，存在 SSRF 风险

**File:** `src/tts/qwen_tts.rs:87-98`
**Issue:** 从 DashScope API 响应中提取的 `audio_url` 未经验证直接用于 `self.client.get(audio_url)` 下载。如果 DashScope API 返回非预期 URL（如指向内网地址），客户端将跟随该请求。

```rust
let audio_url = result["output"]["audio"]["url"].as_str()
    .ok_or_else(|| ...)?;
// 直接使用，无 URL 验证
let audio_response = self.client.get(audio_url).timeout(...).send().await...;
```

DashScope 是阿里云官方 API，信任度较高，但 defense-in-depth 原则要求验证外部返回的 URL scheme。

**Fix:** 添加 URL scheme 校验，确保仅允许 `https://` 协议：

```rust
let audio_url = result["output"]["audio"]["url"].as_str()
    .ok_or_else(|| TTSError::SynthesisFailed("Qwen 响应中无 output.audio.url".to_string()))?;

if !audio_url.starts_with("https://") {
    return Err(TTSError::SynthesisFailed(format!(
        "Qwen audio_url 协议不合法（仅允许 https）: {}", audio_url
    )));
}
```

### WR-03: IndexTTS2 参考音频路径来自用户输入，缺少基本的文件存在性检查

**File:** `src/tts/indextts2.rs:33-46`
**Issue:** `voice_name` 中 `indextts2:` 前缀后的内容直接作为文件路径读取。未做任何路径验证或文件存在性检查。如果路径不存在，错误将在 `tokio::fs::read()` 时以一个不太友好的 IO 错误信息暴露。

```rust
let ref_audio_path_str = common::parse_engine_prefix(voice_name, &["indextts2:"]);
let ref_audio_path = Path::new(ref_audio_path_str);
// 直接读取，无前置验证
let reference_audio_bytes = tokio::fs::read(ref_audio_path).await
    .map_err(|e| TTSError::SynthesisFailed(format!("读取参考音频失败: {}", e)))?;
```

虽然 `tokio::fs::read` 的错误最终会被捕获并包装为 `TTSError::SynthesisFailed`，但提前验证文件存在性可以提供更清晰的错误信息，并避免在 `read` 阶段出现意外行为。

**Fix:** 在读取前添加文件存在性和类型检查：

```rust
let ref_audio_path = Path::new(ref_audio_path_str);
match tokio::fs::metadata(ref_audio_path).await {
    Ok(meta) if meta.is_file() => {},
    Ok(_) => return Err(TTSError::SynthesisFailed(format!(
        "参考音频路径不是文件: {}", ref_audio_path.display()
    ))),
    Err(e) => return Err(TTSError::SynthesisFailed(format!(
        "无法访问参考音频文件 '{}': {}", ref_audio_path.display(), e
    ))),
}
```

## Info

### IN-01: edge_tts.rs hardcoded HeaderValue parse 使用 .unwrap()（已有 SAFETY 注释）

**File:** `src/tts/edge_tts.rs:142-152`
**Issue:** `.unwrap()` 调用在硬编码 ASCII 头字符串上。已有 SAFETY 注释说明这些是编译时常量，`parse()` 不会失败。这是合理的工程选择，不需要修改。

（历史遗留，前两次审查均记录，保持不变。）

### IN-02: parse_edge_tts_binary 为每个二进制消息分配 Vec<u8>

**File:** `src/tts/edge_tts.rs:577`
**Issue:** `data[sep_pos + 4..].to_vec()` 对每个二进制消息（包括大量音频块）都创建新的 `Vec<u8>` 分配，音频数据随后又被 `extend_from_slice` 复制一次。

（历史遗留，前两次审查均记录，保持不变。）

### IN-03: extract_azure_lang 与 voice_name_to_lang 逻辑重复

**File:** `src/tts/azure_speech.rs:321-330` 和 `src/tts/edge_tts.rs:62-72`
**Issue:** 两个模块各自实现了从 voice name 提取语言标签的逻辑，方法略有不同（`splitn(3, '-')` vs `char_indices().filter().nth(1)`）。对格式良好的 voice name 结果一致，但存在维护风险。

（历史遗留，前两次审查均记录，保持不变。）

### IN-04: Doubao TTS token 在请求中传输两次（Header + Body）

**File:** `src/tts/doubaotts.rs:44,75`
**Issue:** `Authorization` 头使用 `Bearer;{token}` 格式（带分号，对齐 Python 版），同时请求体 JSON 的 `app.token` 字段也包含同一 token。token 在 HTTP 请求中传输了两次（一次在 header，一次在 body）。

这不是安全漏洞（token 已通过 HTTPS 传输），且这是对齐 Python 版行为以保持 API 兼容性，不建议修改。仅作为信息记录。

---

_Reviewed: 2026-05-01T13:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
