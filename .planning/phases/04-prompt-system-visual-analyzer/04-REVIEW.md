---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-04-30T14:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
  - src/lib.rs
  - Cargo.toml
findings:
  critical: 2
  warning: 6
  info: 1
  total: 9
status: issues_found
---

# Phase 04: Prompt System + Visual Analyzer 代码审查报告

**审查时间:** 2026-04-30T14:00:00Z  
**审查深度:** standard  
**审查文件数:** 15  
**状态:** issues_found  

## 摘要

本次审查覆盖 Prompt 系统（模板引擎、注册表、管理器、校验器、注册脚本）和 Visual 分析器（帧提取、视觉分析编排）两个模块，共 15 个源文件。

发现了 2 个 Blocker 和 6 个 Warning，涵盖跨模块类型不匹配导致的运行时全链路失败、多字节 UTF-8 切片的 panic 风险、重试机制失效、死代码重复、边界条件缺失校验等缺陷。

---

## 严重问题 (Critical)

### CR-01: 提示词 schema 与 parse_and_retry 解析类型不匹配导致全链路解析失败

**文件:** `src/visual/analyzer.rs:91-98`、`src/visual/analyzer.rs:125-133`

**问题:**

`analyze_video_frames` 中的 LLM 提示词要求 LLM 返回以下 JSON schema：

```
{frame_observations: [{frame_number, timestamp, scene_description, objects, actions, ...}], overall_activity_summary: string}
```

即每批次返回一个包含多个 `FrameObservation` 的数组包裹结构。

但 `parse_and_retry` (第 176-214 行) 使用 `serde_json::from_str::<FrameObservation>` 解析每批响应，期望的是单个平铺的 `FrameObservation`：

```json
{"frame_number": 0, "timestamp": "...", "scene_description": "...", "objects": [...], "actions": [...]}
```

`FrameObservation` 带有 `#[serde(deny_unknown_fields)]`，所以当 LLM 返回包含 `frame_observations` 和 `overall_activity_summary` 字段的包裹结构时，反序列化会立即失败并返回 `unknown field` 错误。`parse_and_retry` 会在耗尽重试次数后返回 `VisualError::Analysis`。最终 `observations` 为空，`analyze_video_frames` 返回 `VisualError::BatchPartial` 错误。

**结果：** `analyze_video_frames` 在正常运行路径下**必然失败**，无法产生任何有效的 `FrameObservation` 输出。

**修复:**

方案 A（推荐）— 将 `parse_and_retry` 改为解析 `BatchAnalysisResult` 并展平其中的 observations：

```rust
// 新增辅助类型，匹配 LLM 实际返回的 schema
#[derive(Deserialize)]
struct BatchResponse {
    #[serde(alias = "frame_observations")]
    observations: Vec<FrameObservation>,
    overall_activity_summary: Option<String>,
}

// 修改 parse_and_retry 返回 Vec<FrameObservation>
pub fn parse_and_retry(
    json_text: &str,
    max_attempts: usize,
) -> Result<Vec<FrameObservation>, VisualError> {
    let mut last_error = None;
    for attempt in 1..=max_attempts {
        let cleaned = json_text.trim()
            .strip_prefix("```json")
            .or_else(|| json_text.trim().strip_prefix("```"))
            .map(|s| s.trim().trim_end_matches("```").trim())
            .unwrap_or(json_text.trim());

        match serde_json::from_str::<BatchResponse>(cleaned) {
            Ok(resp) => return Ok(resp.observations),
            Err(e) => { last_error = Some(e); if attempt >= max_attempts { break; } }
        }
    }
    Err(VisualError::Analysis(format!(
        "JSON 解析失败 ({} 次尝试): {}",
        max_attempts,
        last_error.as_ref().unwrap()
    )))
}
```

方案 B — 修改提示词 schema 为单帧格式，并保证 `analyze_images` 的 `batch_size=1`。

### CR-02: truncate 过滤器使用字节切片可能引发 UTF-8 panic

**文件:** `src/prompt/template.rs:42-48`

**问题:**

`truncate` 过滤器使用字节索引切片 `&s[..97]` 而非字符边界切片。对于多字节 UTF-8 字符（如中文，每字 3 字节），当字符串字节长度 > 100 且第 97 字节落在一个多字节字符中间时，Rust 会 panic：

```
thread '...' panicked at 'byte index 97 is not a char boundary; it is inside '...' (bytes 96..99)'
```

该项目重度使用中文内容（Prompt 模板和用户内容均以中文为主），此问题在正常使用中极易触发。

现有测试 `test_truncate_filter_long` 仅使用 ASCII `'A'`，未覆盖多字节字符场景。

**修复:**

改用 `char_indices()` 按字符边界截断：

```rust
m.insert("truncate", |s: &str| {
    let max_chars = 97; // 保留 97 个字符 + "..."
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
});
```

同时新增测试用例覆盖中文截断。

---

## 警告 (Warnings)

### WR-01: `\$` 转义语法未实现，文档与实际行为不一致

**文件:** `src/prompt/template.rs:75`（文档注释）

**问题:**

文档注释声称支持 `\$` 转义为字面 `$`，但 `render` 函数的正则 `r"\$\{(\w+)\}|\$(\w+)"` 中没有处理前置反斜杠的逻辑。模板中的 `\$variable` 在第二遍替换时，正则 `\$(\w+)` 在第二个字符位置 (`$variable`) 会匹配成功，而 `\` 作为未匹配字符留在原地。结果是 `\$variable` → `\value`，而非期望的 `$variable`。

**修复:**

在变量替换前插入一次反斜杠转义处理——将 `\$` 替换为占位符，替换完成后再恢复：

```rust
// 第 0 遍：处理转义
let escaped = template.replace("\\$", "\x00DOLLAR\x00");

// 第 1-2 遍在 escaped 上进行...
// 最后将 \x00DOLLAR\x00 恢复为 $
```

### WR-02: `strip_code_fence` 为死代码，逻辑在 `analyzer.rs` 中重复

**文件:** `src/visual/types.rs:46-52`、`src/visual/analyzer.rs:184-189`

**问题:**

`types.rs` 中定义了 `pub(crate) fn strip_code_fence`（标记 `#[allow(dead_code)]`），但 `analyzer.rs` 的 `parse_and_retry` 中内联了完全相同的代码块剥离逻辑。本应复用此函数却未使用，导致：
1. 死代码
2. 重复逻辑，维护时需要同步修改两处

**修复:** 在 `parse_and_retry` 中调用 `strip_code_fence` 替代内联实现，移除 `#[allow(dead_code)]` 属性。

### WR-03: `interval_seconds=0` 无校验，可能触发无限循环

**文件:** `src/visual/frame_extractor.rs:171-193`

**问题:**

`extract_frames_fallback` 中 `total_frames = (duration / interval_seconds).ceil() as usize`。当 `interval_seconds = 0.0` 时，`duration / 0.0` = `inf`，`inf.ceil()` = `inf`，`inf as usize`  在 Rust 中饱和转换为 `usize::MAX`。后续 `for i in 0..usize::MAX` 会导致近似无限循环（或帧文件写满磁盘）。

当前调用者 (`analyze_video_frames`) 传递固定值 `3.0`，但 `extract_frames` 是公共函数，缺乏防御性校验。

**修复:** 在 `extract_frames` 入口添加校验：

```rust
if interval_seconds <= 0.0 {
    return Err(VisualError::FrameExtraction(
        "帧提取间隔必须 > 0".into()
    ));
}
```

### WR-04: `convert_image_to_jpeg` 使用硬编码 quality 85，与 `extract_single_frame` 的 quality 参数不一致

**文件:** `src/visual/frame_extractor.rs:515-528`

**问题:**

`extract_single_frame` 的 Level 1/2 使用调用方传入的 `quality` 参数（通过 `-q:v` 传递给 FFmpeg），但 Level 3 (PNG→JPEG) 和 Level 4 (BMP→JPEG) 在 `convert_image_to_jpeg` 中硬编码 quality 为 85。当用户指定不同的 `quality` 时，各级回退路径的输出质量不一致。

此外，FFmpeg 的 `-q:v`（越低越好，范围 2-31）和 `image` crate 的 `new_with_quality`（越高越好，范围 1-100）语义相反，目前没有任何转换映射。

**修复:** 将 `quality` 参数传递到 `convert_image_to_jpeg`，并建立语义映射：

```rust
fn convert_image_to_jpeg(input: &Path, output: &Path, ffmpeg_quality: u32) -> Result<(), VisualError> {
    // 将 FFmpeg 的 q:v (2-31, 低=好) 映射到 image crate 的 quality (1-100, 高=好)
    let img_quality = ((31 - ffmpeg_quality.min(31)) as f32 / 29.0 * 100.0) as u8;
    // ...
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, img_quality.max(1));
    // ...
}
```

### WR-05: `analyze_video_frames` 未传递 `CancellationToken`，不支持外部取消

**文件:** `src/visual/analyzer.rs:65-66`

**问题:**

`extract_frames` 支持 `CancellationToken` 实现取消，但 `analyze_video_frames` 在调用 `extract_frames` 时传入 `None`（第 66 行），且自身也未提供接收 `CancellationToken` 的参数。对于一个可能处理长时间视频（数小时）的流水线，无法通过外部机制取消帧提取过程。

**修复:** 在 `analyze_video_frames` 函数签名中添加 `cancel: Option<CancellationToken>` 参数并传递给 `extract_frames`。

### WR-06: `parse_and_retry` 重试不改变状态，实际无效

**文件:** `src/visual/analyzer.rs:176-214`

**问题:**

`parse_and_retry` 的 `for attempt in 1..=max_attempts` 循环在每次迭代中使用完全相同的 `json_text` 输入。`cleaned` 值在各次迭代中完全相同，因此每次 `serde_json::from_str` 的结果也完全相同。重试次数 `max_attempts` 的设置对结果毫无影响——第一次失败后所有后续尝试也必然失败。

**修复:**

如果重试机制是有意保留的（为未来不同的重试策略做准备），应在循环内对原始文本进行不同策略的清理尝试。或者直接去掉循环，只尝试一次：

```rust
pub fn parse_and_retry(json_text: &str, _max_attempts: usize) -> Result<FrameObservation, VisualError> {
    // 单次尝试，移除无用循环
    let cleaned = json_text.trim()
        .strip_prefix("```json")
        .or_else(|| json_text.trim().strip_prefix("```"))
        .map(|s| s.trim().trim_end_matches("```").trim())
        .unwrap_or(json_text.trim());

    serde_json::from_str::<FrameObservation>(cleaned)
        .map_err(|e| VisualError::Analysis(format!("JSON 解析失败: {}", e)))
}
```

### WR-07: `has_chinese` 中文字符检测使用 `>` 而非 `>=`，漏检 `U+4E00`

**文件:** `src/prompt/validators.rs:94`

**问题:**

```rust
let has_chinese = trimmed.chars().any(|c| c > '\u{4E00}');
```

`U+4E00`（`一`，最常用的汉字之一）被此条件排除在外，因为 `c > '\u{4E00}'` 是严格大于，`一` 恰好等于 `\u{4E00}` 所以不会被检测到。虽然在实际文本中 `一` 单独出现且无其他汉字的概率很低，但这使得校验语义不准确。

**修复:** 改为 `c >= '\u{4E00}'` 并增加完整的 CJK 范围判断：

```rust
let has_chinese = trimmed.chars().any(|c| {
    (c >= '\u{4E00}' && c <= '\u{9FFF')   // CJK Unified Ideographs
        || (c >= '\u{3400}' && c <= '\u{4DBF}') // Extension A
        || (c >= '\u{F900}' && c <= '\u{FAFF}') // Compatibility Ideographs
});
```

---

## 信息 (Info)

### IN-01: `FilterFn` 使用函数指针限制可扩展性

**文件:** `src/prompt/template.rs:7`

**问题:**

`type FilterFn = fn(&str) -> String;` 使用函数指针类型，无法捕获闭包环境。这意味着所有过滤器必须是无状态的纯函数，无法支持带参数的过滤器（如 `truncate:N` 指定截断长度），也无法插入依赖注入或上下文感知的过滤器。长期看，如果需要扩展过滤器系统，函数指针边界会成为阻碍。

**建议:** 若短期无扩展计划可保留现状；未来需要可改为 `Box<dyn Fn(&str) -> String + Send + Sync>` 或 trait object。

---

## 统计汇总

| 严重级别 | 数量 | 关键风险 |
|---------|------|---------|
| Critical | 2 | 全链路解析失败 + UTF-8 panic |
| Warning  | 6 | 转义未实现、死代码、循环风险、quality 不一致、缺少取消、重试无效、字符检测不准 |
| Info     | 1 | 函数指针限制可扩展性 |

**重点关注:** CR-01 会导致 `analyze_video_frames` 在任何正常输入下都会返回错误，属于必须在合并前修复的阻塞问题。CR-02 在使用中文 Prompt 或内容时极大概率触发 panic。

---

_Reviewed: 2026-04-30T14:00:00Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: standard_
