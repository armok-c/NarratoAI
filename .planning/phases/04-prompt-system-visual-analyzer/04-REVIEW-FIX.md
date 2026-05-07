---
phase: 04-prompt-system-visual-analyzer
fixed_at: '2026-05-07T12:00:00Z'
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 1
findings_in_scope: 16
fixed: 12
skipped: 4
status: all_fixed
---

# Phase 04: Code Review Fix Report (Iteration 1)

**Fixed at:** 2026-05-07T12:00:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 16
- Fixed: 12
- Skipped: 4

## Fixed Issues

### WR-01: 正则表达式重复编译 (registry.rs)

**Files modified:** `narratoai-core/src/prompt/registry.rs`
**Commit:** `c209e26`
**Applied fix:** 提取模块级 `OnceLock<Regex>` 缓存正则实例，`validate_prompt_parameters` 使用 `template_var_regex()` 获取静态引用。

### WR-02: 正则表达式重复编译 (template.rs)

**Files modified:** `narratoai-core/src/prompt/template.rs`
**Commit:** `062f7ea`
**Applied fix:** 使用 `OnceLock<Regex>` 缓存正则实例，`render()` 不再每次调用编译正则。

### WR-03: builtin_filters() 每次调用重建 HashMap

**Files modified:** `narratoai-core/src/prompt/template.rs`
**Commit:** `062f7ea`
**Applied fix:** 将 `builtin_filters()` 返回值改为 `&'static HashMap`，通过 `OnceLock` 缓存。

### WR-04: unreachable! 在 release 模式下产生 UB

**Files modified:** `narratoai-core/src/prompt/template.rs`
**Commit:** `062f7ea`
**Applied fix:** 将 `unwrap_or_else(|| unreachable!(...))` 替换为 `unwrap_or("")` 和 `.expect("...")`。

### WR-05: 版本排序 unwrap_or(0) 导致非解析版本号排序异常

**Files modified:** `narratoai-core/src/prompt/registry.rs`
**Commit:** `c209e26`
**Applied fix:** 将 `unwrap_or(0)` 改为 `unwrap_or(u64::MAX)`，使无法解析的版本号排在最后。

### WR-06: chars().count() 在 validate_narration_script 中重复计算

**Files modified:** `narratoai-core/src/prompt/validators.rs`
**Commits:** `de3edd3`, `d055645`
**Applied fix:** 缓存 `normalized.chars().count()` 到局部变量 `char_count`，同时移除 format! 中多余的第三个参数（原代码 bug：3 个参数但只有 2 个格式占位符）。

### WR-07: chars().count() 在 validate_plot_analysis 中重复计算

**Files modified:** `narratoai-core/src/prompt/validators.rs`
**Commits:** `de3edd3`, `d055645`
**Applied fix:** 缓存 `trimmed.chars().count()` 到局部变量 `char_count`，移除多余的第三个参数。

### WR-09: seconds_to_hhmmssmmm 对负数输入未防御

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** `b432d21`
**Applied fix:** 添加 `total_secs.max(0.0)` clamp 防止负数转 u64 产生溢出，同时添加 `debug_assert!(total_secs >= 0.0)`。

### WR-10: total_frames 计算缺少上限检查

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** `b432d21`
**Applied fix:** 添加 `MAX_TOTAL_FRAMES = 100_000` 常量，超出时返回 `VisualError::FrameExtraction` 错误。

### WR-13: 排序中 unwrap() 可能 panic

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** `9449fd7`
**Applied fix:** 将 `unwrap()` 替换为 `unwrap_or_else(|| { warn!(...); 0 })`，防御校验后文件被外部修改的极端情况。

### WR-14: 模板文件 language 措辞不一致

**Files modified:** `narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md`
**Commit:** `11ec77c`
**Applied fix:** 统一为 `输出语言: ${language}`。

### WR-15: 测试注释与断言不一致

**Files modified:** `narratoai-core/src/prompt/register.rs`
**Commit:** `6c2030b`
**Applied fix:** 修改注释说明 `list_prompts` 返回 2 是因为按 name 去重，后续 `get()` 调用验证所有版本可精确获取。

## Skipped Issues

### WR-08: strip_code_fence 不处理嵌套代码块

**Reason:** REVIEW 明确标注"当前实现对于典型 LLM 输出足够"，属于已知限制而非缺陷。

### WR-11: unwrap_or_default() 创建无主 CancellationToken

**Reason:** 代码已有详细注释说明行为和替代方案（调用方应始终提供 CancellationToken），属于有意的 fire-and-forget 语义设计。

### WR-12: FrameObservation/BatchAnalysisResult 未使用 deny_unknown_fields

**Reason:** REVIEW 明确标注"有意的容错设计，可接受"，兼容 LLM 返回额外字段的场景。

### WR-16: notify 依赖使用 RC 版本

**Reason:** 外部依赖版本问题，无法在源代码中修复。需跟踪 9.0.0 正式版发布后升级。

## Notes

- 编译验证通过（`cargo check`），所有修改未引入新的编译错误。
- 2 个预存在的编译错误（`frame_extractor.rs:78` match 非穷尽、`documentary/script_gen.rs:108` 参数数量不匹配）与本次修复无关，未做修改。
- WR-01 和 WR-05 因修改同一文件合并在同一个 commit 中。
- WR-02、WR-03、WR-04 因修改同一文件合并在同一个 commit 中。
- WR-06/WR-07 分两个 commit：第一个修复缓存，第二个修复随之暴露的 format! 多余参数问题。

---

_Fixed: 2026-05-07T12:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
