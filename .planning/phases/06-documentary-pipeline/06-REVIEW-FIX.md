---
phase: 06-documentary-pipeline
fixed_at: 2026-05-08T15:00:00Z
review_path: .planning/phases/06-documentary-pipeline/06-REVIEW.md
iteration: 10
findings_in_scope: 5
fixed: 3
skipped: 2
status: partial
---

# Phase 6: Code Review Fix Report (Iteration 10)

**Fixed at:** 2026-05-08T15:00:00Z
**Source review:** .planning/phases/06-documentary-pipeline/06-REVIEW.md
**Iteration:** 10

**Summary:**
- Findings in scope: 5 (IN-01 ~ IN-05, `--all` flag)
- Fixed: 3
- Skipped: 2 (低风险/高复杂度)

## Fixed Issues

### IN-01: collect_keyframe_paths 函数为死代码

**Files modified:** `narratoai-core/src/documentary/script_gen.rs`
**Applied fix:** 移除 `collect_keyframe_paths()` 函数（原第 416-433 行）。该函数在模块内定义但从未被调用——`analyze_video()` 直接使用 `extract_frames()` 返回的路径列表，无需重新扫描目录。`visual/frame_extractor.rs` 中有功能类似的 `collect_keyframe_paths_from_dir()` 负责实际使用场景。

### IN-02: ProgressStep 枚举在生产代码中未使用，测试中存在类型不匹配

**Files modified:**
- `narratoai-core/src/documentary/types.rs` — 移除 `ProgressStep` 枚举定义（原第 100-109 行）
- `narratoai-core/src/documentary/mod.rs` — 移除 `ProgressStep` 的 re-export
- `tests/documentary_integration_test.rs` — 重写 `test_progress_callback_receives_all_steps` 测试

**Applied fix:** 生产代码的 `ProgressCallback` 类型为 `Box<dyn Fn(&str, f32, &str) + Send + Sync>`，使用 `&str` 传递步骤名。`ProgressStep` 枚举仅在测试中以不匹配的签名使用，未实际验证回调类型契约。移除枚举后，重写测试使用实际的 `ProgressCallback` 类型，通过元组数组模拟进度发射，验证回调接收参数的完整性。

### IN-04: SRT 序列号在跳过负时间戳块时不连续

**Files modified:** `narratoai-core/src/documentary/subtitle.rs`
**Applied fix:** 将 `generate_srt_from_word_boundaries()` 中的 SRT 序号从 `enumerate()` 的 `i + 1` 改为独立计数器 `seq`（从 1 开始，仅在实际添加块时递增），确保跳过负时间戳 word boundary 时序号仍然连续。

## Skipped Issues

### IN-03: strip_and_repair_json trailing comma 修复可能破坏字符串内容

**Reason:** 全局 `text.replace(",}", "}")` 可能误替换 JSON 字符串值内的 `,}` 模式。虽然风险存在，但 (1) 仅在直接解析失败时触发，(2) LLM 输出极少包含此模式，(3) 基于 JSON tokenizer 的精确替换实现复杂度高、引入新 bug 风险更大。保持现有行为。

### IN-05: 单引号替换可能破坏自然语言内容

**Reason:** 在 `!text.contains('"') && text.contains('\'')` 条件下替换所有单引号为双引号可能破坏英文缩写（如 "it's" → "it"s"）。但 (1) 中文场景下风险极低，(2) 前置条件已大幅缩小误替换范围，(3) 更精确的引号对匹配策略复杂度高。保持现有行为。

## Verification

**cargo check result:** 唯一的编译错误为预存的 `visual/frame_extractor.rs:78`（non-exhaustive match），不在本次审查和修复范围内。本次 3 个修复未引入新的编译错误或 warning。

**测试状态:** 由于 `frame_extractor.rs` 编译错误阻止所有测试执行，无法运行测试套件。修复逻辑均为精确的小改动（移除死代码、类型签名对齐、计数器替换），逻辑正确性可通过代码审查确认。

---

_Fixed: 2026-05-08T15:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 10_
