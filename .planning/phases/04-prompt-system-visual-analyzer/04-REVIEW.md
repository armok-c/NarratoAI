---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-06T22:30:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 0
  warning: 1
  info: 5
  total: 6
status: issues_found
---

# Phase 04: Code Review Report (Iteration 17)

**Reviewed:** 2026-05-06T22:30:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

审查了 Phase 04 的 19 个源文件，涵盖 Prompt 系统（模块声明、类型、错误、注册表、模板渲染、管理器、校验器、注册函数）和 Visual 模块（模块声明、错误、类型、帧提取器、分析编排器），以及 4 个 Prompt 模板文件、`src/lib.rs` 和 `Cargo.toml`。

编译通过（7 warnings 均不在此 Phase 范围内），全部 88 个单元测试通过，clippy 对 prompt/visual 模块无 warning。

整体代码质量良好。本轮新发现 1 个 WARNING（`analyze_video_frames` 中 `format!` 对外部 `prompt_template` 参数的潜在 panic 风险），5 个 INFO 级别发现均延续自 Iteration 16 且未修复。无 CRITICAL 级别问题。

## Warnings

### WR-17-01: analyze_video_frames 中 format! 对外部 prompt_template 的潜在 panic

**File:** `src/visual/analyzer.rs:109-116`
**Severity:** WARNING
**Category:** 正确性/健壮性

`analyze_video_frames()` 使用 `format!("{}", prompt_template)` 将外部传入的 prompt 模板字符串嵌入到最终 prompt 中。如果 `prompt_template` 包含裸露的 Rust `format!` 格式说明符（如 `{}`、`{:?}`、`{name}` 等），会在运行时 panic。

当前内置模板使用 `${variable}` 语法（不会被 `format!` 解释），所以内置场景安全。但如果未来有外部调用者传入包含 `{}` 的模板字符串，会导致 panic 且无法捕获。

```rust
// 当前代码（analyzer.rs:109-116）
let rendered_prompt = format!(
    "{}\n\nIMPORTANT: Respond with valid JSON matching this schema: \
     {{frame_observations: [{{frame_number: u64, ...}}], ...}}",
    prompt_template  // <-- 如果包含 {} 或 {:?} 会 panic
);
```

**Fix:** 使用字符串拼接替代 `format!`，避免将外部输入作为格式化参数：

```rust
let schema_suffix = "\n\nIMPORTANT: Respond with valid JSON matching this schema: \
     {frame_observations: [{frame_number: u64, timestamp: string, \
     scene_description: string, objects: [string], actions: [string], \
     on_screen_text: string|null, visual_salience: f64|null}], \
     overall_activity_summary: string}";
let rendered_prompt = format!("{}{}", prompt_template, schema_suffix);
```

更彻底的修复是使用 `.to_string() + schema_suffix` 或 `concat!` 宏，完全避免 `format!` 的格式化行为作用于 `prompt_template`。

## Info

### IN-17-01: template.rs filter_re 三次独立迭代（未修复，延续自 IN-16-01）

**File:** `src/prompt/template.rs`
**Lines:** 118, 132, 141
**Severity:** INFO
**Category:** 性能/可维护性

`filter_re` 正则在 `render()` 函数中被遍历三次：第一次检查缺失变量（行 118），第二次校验过滤器名称（行 132），第三次执行替换（行 141）。每次遍历都会重新执行正则匹配，对于包含多个 `${var|filter}` 的模板产生不必要的重复计算。

**Fix:** 合并为单次遍历，在一次 `captures_iter` 中同时校验变量存在性、过滤器名称有效性，并收集替换结果：

```rust
let mut replacements: Vec<(std::ops::Range<usize>, String)> = Vec::new();
for caps in filter_re.captures_iter(&result) {
    let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    if !vars.contains_key(var_name) { /* error */ }
    if !filters.contains_key(filter_name) { /* error */ }
    if let (Some(filter_fn), Some(value)) = (filters.get(filter_name), vars.get(var_name)) {
        replacements.push((caps.get(0).unwrap().range(), filter_fn(value)));
    }
}
```

### IN-17-02: manager.rs validate_output 方法未使用 self（未修复，延续自 IN-16-02）

**File:** `src/prompt/manager.rs`
**Line:** 112
**Severity:** INFO
**Category:** API 设计

`PromptManager::validate_output()` 接受 `&self` 但方法体内完全不访问 `self`，仅委托给 `validators::validate_output()`。这是一个纯函数包装，`&self` 参数是多余的。

**Fix:** 可以改为关联函数（去掉 `&self`），或保留当前设计但在文档中说明这是为 API 一致性而有意保留的门面方法。

### IN-17-03: frame_extractor.rs seconds_to_hhmmssmmm 上多余的 #[allow(dead_code)]（未修复，延续自 IN-16-03）

**File:** `src/visual/frame_extractor.rs`
**Line:** 470
**Severity:** INFO
**Category:** 代码整洁

`seconds_to_hhmmssmmm()` 在非测试代码中被调用（行 217 的 `extract_frames_fallback` 和行 452 的 `rename_fast_path_frames`），因此该函数并非 dead code，`#[allow(dead_code)]` 注解是多余的。

**Fix:** 删除行 470 的 `#[allow(dead_code)]`。

### IN-17-04: frame_extractor.rs parse_frame_number_from_name 上 #[allow(dead_code)] 合理但存在重复逻辑（未修复，延续自 IN-16-04）

**File:** `src/visual/frame_extractor.rs`
**Line:** 562
**Severity:** INFO
**Category:** 代码整洁

`parse_frame_number_from_name()` 仅在测试中使用（行 675-682），非测试代码中未调用。`#[allow(dead_code)]` 在此处是合理的，但该函数的功能与 `rename_fast_path_frames()` 中行 443-449 的内联解析逻辑完全重复。

**Fix:** 考虑在 `rename_fast_path_frames()` 中调用 `parse_frame_number_from_name()` 替代内联解析，消除代码重复。如果决定保留两份独立实现，建议添加注释说明原因。

### IN-17-05: types.rs strip_code_fence 冗余 trim() 调用（未修复，延续自 IN-16-05）

**File:** `src/visual/types.rs`
**Lines:** 44-49
**Severity:** INFO
**Category:** 代码整洁

`strip_code_fence()` 函数在链式调用中多次调用 `.trim()`：行 45 对 `text` 调用一次，行 47-48 对匹配后的结果又调用 `.trim()` 两次。第一次 `trim()` 的结果在 `strip_prefix` 失败时被 `unwrap_or` 重新 `trim()` 了一次。

**Fix:** 先 `trim()` 一次并绑定到局部变量，后续复用：

```rust
pub(crate) fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim().trim_end_matches("```").trim())
        .unwrap_or(trimmed)
}
```

## Previous Fix Verification (Iteration 16 Findings)

| ID | Finding | Status | Notes |
|---|---|---|---|
| IN-16-01 | template.rs filter_re 三次独立迭代 | **Unchanged** | 三次遍历仍存在于行 118/132/141 |
| IN-16-02 | manager.rs validate_output 未使用 self | **Unchanged** | `&self` 仍未使用 |
| IN-16-03 | frame_extractor.rs seconds_to_hhmmssmmm 多余 allow(dead_code) | **Unchanged** | 注解仍存在；函数已被非测试代码使用（行 217, 452），注解确认多余 |
| IN-16-04 | frame_extractor.rs parse_frame_number_from_name allow(dead_code) | **Unchanged** | 注解仍存在；仅测试使用，allow 合理但存在代码重复 |
| IN-16-05 | types.rs strip_code_fence 冗余 trim() | **Unchanged** | 五次 trim 调用仍存在 |

所有 5 个 INFO 级别发现自 Iteration 16 以来均未修改。

## Build Verification

| Check | Result | Notes |
|-------|--------|-------|
| `cargo check` | PASS | 0 errors, 7 warnings (均在其他模块：sdp/script_gen.rs, subtitle/parser.rs) |
| `cargo clippy --lib -- -W clippy::all` (prompt + visual) | PASS | 0 warnings in Phase 04 modules |
| `cargo test --lib -- prompt:: visual::` | PASS | 88 passed, 0 failed, 0 ignored |

## Security Assessment

**命令注入：** `frame_extractor.rs` 中所有 FFmpeg/ffprobe 调用均通过 `std::process::Command` 或 `ffmpeg-sidecar` 的 `FfmpegCommand` 执行。两者均直接传递参数给操作系统，不经过 shell 解释，因此文件路径中的特殊字符不会被解释为 shell 命令。**未发现命令注入风险。**

**路径遍历：** `output_dir` 和 `video_path` 由调用方传入，本模块内未做路径规范化或边界检查。但帧文件名由代码内部生成（`keyframe_{:06}_{timestamp}.jpg`），不接受外部输入，因此不存在通过文件名进行的路径遍历。路径安全性依赖调用方。

**ReDoS：** `template.rs` 中的两个正则 `r"\$\{(\w+)\}|\$(\w+)"` 和 `r"\$\{(\w+)\|(\w+)\}"` 均为简单模式，不包含回溯风险。**无 ReDoS 风险。**

**输入校验：** FFmpeg quality 参数在 `extract_frames()` 入口处校验范围 2-31（行 42-46），`interval_seconds` 校验非 NaN 且 > 0（行 48-50）。视频时长通过 ffprobe 获取后校验 > 0（行 195-197）。

---

_Reviewed: 2026-05-06T22:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 17_
