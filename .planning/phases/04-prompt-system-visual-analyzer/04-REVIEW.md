---
status: clean
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T23:59:00Z
depth: standard
iteration: 3
files_reviewed: 20
files_reviewed_list:
  - narratoai-core/src/prompt/mod.rs
  - narratoai-core/src/prompt/types.rs
  - narratoai-core/src/prompt/error.rs
  - narratoai-core/src/prompt/registry.rs
  - narratoai-core/src/prompt/template.rs
  - narratoai-core/src/prompt/manager.rs
  - narratoai-core/src/prompt/validators.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/subtitle_analysis_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/types.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/analyzer.rs
  - narratoai-core/src/lib.rs
  - narratoai-core/src/text_utils.rs
  - narratoai-core/Cargo.toml
findings:
  critical: 0
  warning: 1
  info: 5
  total: 6
---

# Phase 04: Code Review Report (3rd Pass)

**Reviewed:** 2026-05-07T23:59:00Z
**Depth:** standard
**Iteration:** 3
**Files Reviewed:** 20 (+ 3 auxiliary: lib.rs, text_utils.rs, Cargo.toml)
**Status:** clean (no new actionable issues)

## Summary

本轮为第 3 次审查，基于第 2 次审查后的修复代码进行。第 2 次审查的 3 个修复（WR-01 正则去重、WR-02 死代码移入测试、WR-03 逐行代码块剥离）均已验证正确且无回归。

**修复验证结果（3/3 通过）：**
- WR-01: `template_var_regex()` 共享单例在 `prompt/mod.rs` 中定义，`registry.rs` 和 `template.rs` 均通过 `use super::template_var_regex` 引用，编译期保证一致性。
- WR-02: `collect_frame_paths()` 和 `extract_frame_number_from_keyframe()` 已移入 `analyzer.rs` 的 `#[cfg(test)] mod tests` 块，生产代码中不再存在。
- WR-03: `strip_code_fence()` 重写为逐行解析，仅在首行 `starts_with("```")` 且末行 `trim() == "```"` 时剥离，正确处理嵌套反引号内容。Windows \r\n 行尾和混合行尾均已验证安全。

**新增发现：无。** 所有 WARNING 和 INFO 均为已知遗留项。

## Warnings

### WR-04: notify 依赖使用 RC 版本 (Cargo.toml)

**File:** `narratoai-core/Cargo.toml:23`
**Status:** 已知遗留，无法在项目内修复
**Issue:** `notify = "9.0.0-rc.3"` 是预发布版本。9.0.0 正式版 API 可能发生破坏性变更。

**Fix:** 跟踪 notify 9.0.0 正式版发布后升级。已标注注释。

## Info

### IN-01: collect_keyframe_paths_from_dir 使用字典序排序 (frame_extractor.rs)

**File:** `narratoai-core/src/visual/frame_extractor.rs:625-633`
**Issue:** 使用 `paths.sort()`（字典序）。当前帧号格式为 `{:06}` 零填充 6 位且 `MAX_TOTAL_FRAMES = 100_000`，字典序等价于数字序。若未来放宽帧数限制需改用数字排序。

### IN-02: truncate 过滤器魔法数字 (template.rs)

**File:** `narratoai-core/src/prompt/template.rs:47-55`
**Issue:** `100` 和 `97` 硬编码。建议提取为 `const TRUNCATE_MAX_CHARS` 和 `const TRUNCATE_ELLIPSIS_LEN`。

### IN-03: RwLock 中毒处理策略 (manager.rs)

**File:** `narratoai-core/src/prompt/manager.rs:42-44,111-113,119-121,138-140,147-149`
**Issue:** 所有 `RwLock` 获取将 `PoisonError` 转换为 `PromptError::LockFailure`。锁中毒后所有后续操作均失败，属于不可恢复错误。当前处理方式合理，仅作文档记录。

### IN-04: tokio features = ["full"] 可精简 (Cargo.toml)

**File:** `narratoai-core/Cargo.toml:15`
**Issue:** 实际使用功能为 `rt-multi-thread`、`macros`、`sync`、`time`、`process`。`full` 增加编译时间和二进制大小。低优先级优化项。

### IN-05: 快路径不报告中间进度 (frame_extractor.rs)

**File:** `narratoai-core/src/visual/frame_extractor.rs:87-89`
**Issue:** 快路径成功时仅在完成时报告 100%，不转发 FFmpeg 进度事件。回退路径有逐帧进度。已知限制，不影响功能正确性。

## Cross-File Analysis

### 模板变量一致性

所有 7 个模板的 `${variable}` 引用均与 `register.rs` 中对应的 `ParameterDef` 声明一致。`registry.rs` 的 `validate_prompt_parameters()` 在注册时进行编译期校验。

| Template | Variables | Declared | Match |
|----------|-----------|----------|-------|
| frame_analysis_v1.0 | video_description, language | 2 | ✓ |
| narration_generation_v2.0 | video_title, frame_analysis_json, style, language | 4 | ✓ |
| subtitle_analysis_v2.0 | subtitle_content, custom_clips | 2 | ✓ |
| plot_extraction_v2.0 | subtitle_content, plot_summary, plot_titles | 3 | ✓ |
| plot_analysis_v1.0 | subtitle_content | 1 | ✓ |
| script_generation_v1.0 | plot_analysis, language, style | 3 | ✓ |
| script_generation_v2.0 | drama_name, plot_analysis, subtitle_content | 3 | ✓ |

### 错误处理链完整性

- `PromptError` 5 变体完整覆盖：TemplateRender / NotFound / LockFailure / Registration / Validation
- `VisualError` 3 变体完整覆盖：FrameExtraction / Analysis / BatchPartial
- 两套错误枚举通过 `thiserror` 宏提供中文消息，`Display` trait 一致

### 安全性

- 模板注入防护：单遍 `replace_all` 防止变量值中的 `${...}` 被重新解释
- FFmpeg 命令注入防护：使用 `std::process::Command` 参数数组，无 shell 拼接
- 路径安全：所有路径参数由调用方控制，本层不处理用户输入
- DoS 防护：`MAX_TOTAL_FRAMES = 100_000` 限制帧数，正则 `\w+` 保证线性复杂度

## Iteration History

| Iteration | Critical | Warning | Info | Fixed | Status |
|-----------|----------|---------|------|-------|--------|
| 1 | 0 | 16 | 24 | 16 | all_fixed |
| 2 | 0 | 4 | 5 | 3 | partial (1 skipped) |
| 3 (this) | 0 | 1 | 5 | 0 | clean |

---
_Reviewed: 2026-05-07T23:59:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 3_
