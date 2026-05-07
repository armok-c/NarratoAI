---
phase: 04-prompt-system-visual-analyzer
fixed_at: '2026-05-07T22:30:00Z'
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 3
skipped: 1
status: partial
---

# Phase 04: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-05-07T22:30:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 2 (基于第二轮审查：0 critical, 4 warning, 5 info)

**Summary:**
- Findings in scope: 4 (warning only)
- Fixed: 3
- Skipped: 1

## Fixed Issues

### WR-01: TEMPLATE_VAR_REGEX 重复定义导致双重编译

**Files modified:** `narratoai-core/src/prompt/mod.rs`, `registry.rs`, `template.rs`
**Commit:** `b3881ff fix(04): WR-01 deduplicate TEMPLATE_VAR_REGEX into shared prompt::template_var_regex`
**Applied fix:** 在 `prompt/mod.rs` 新增共享 `pub(crate) fn template_var_regex()` 函数（单例 `OnceLock<Regex>`），`registry.rs` 和 `template.rs` 删除各自的本地定义，改用 `use super::template_var_regex;` 引用。

### WR-02: collect_frame_paths 生产代码未使用

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** `0fd520a fix(04): WR-02 move dead collect_frame_paths into #[cfg(test)] module`
**Applied fix:** 将 `collect_frame_paths()` 和 `extract_frame_number_from_keyframe()` 从生产代码移入 `#[cfg(test)] mod tests` 块。同步调整 `use std::path::PathBuf` 到测试模块内。

### WR-03: strip_code_fence 不处理嵌套代码块

**Files modified:** `narratoai-core/src/text_utils.rs`
**Commit:** `336d443 fix(04): WR-03 use line-based strip_code_fence to handle nested code blocks`
**Applied fix:** 重写 `strip_code_fence()` 为逐行解析：仅在首行以 `` ``` `` 开头且末行为独立 `` ``` `` 时剥离，避免误切 JSON 内容中包含的反引号序列。函数签名不变（`&str -> &str`），下游 re-export 和所有测试兼容。

## Skipped Issues

### WR-04: notify 依赖使用 RC 版本

**Reason:** `notify = "9.0.0-rc.3"` 为外部依赖预发布版本，无法在本项目源码内修复。需跟踪上游 notify 9.0.0 正式版发布后升级。当前不影响功能正确性。

## Verification

- `cargo check` 通过，无新增编译错误或警告
- 3 个预先存在的编译错误（`frame_extractor.rs` 非穷尽匹配、`documentary/script_gen.rs` 缺少参数、`sde/script_gen.rs` 未使用导入）均非本次修改引入
- WR-02 修复消除了原来的 `unused import: PathBuf` 警告

## Iteration History

| Iteration | Scope | Fixed | Skipped | Status |
|-----------|-------|-------|---------|--------|
| 1 | 16 (critical+warning) | 12 | 4 | all_fixed |
| 2 | 4 (warning) | 3 | 1 | partial |

---

_Fixed: 2026-05-07T22:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
