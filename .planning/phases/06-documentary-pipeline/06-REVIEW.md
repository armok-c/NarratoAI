---
phase: 06-documentary-pipeline
reviewed: 2026-05-08T15:20:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - narratoai-core/src/documentary/mod.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/documentary/subtitle.rs
  - narratoai-core/src/documentary/types.rs
  - tests/documentary_integration_test.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
previous_review:
  date: 2026-05-08
  iteration: 10
  findings: "0 critical, 0 warning, 5 info"
---

# Phase 6: Code Review Report (Iteration 11)

**Reviewed:** 2026-05-08T15:20:00Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** clean

## Summary

本次为第 11 轮迭代审查，重点验证 iter 10 修复的 3 个 INFO 级别发现（IN-01、IN-02、IN-04），并重新评估 2 个跳过的发现（IN-03、IN-05）。

**IN-01 验证结果：已修复。** `collect_keyframe_paths()` 函数已从 `script_gen.rs` 中完整移除。全 crate 搜索确认无残留引用。`visual/frame_extractor.rs` 中的同名函数 `collect_keyframe_paths_from_dir()` 不受影响。

**IN-02 验证结果：已修复。** (1) `ProgressStep` 枚举已从 `types.rs` 移除，`mod.rs` 中对应的 re-export 已删除。(2) 全 crate 搜索确认 `documentary::ProgressStep` 无任何残留引用（`sde/types.rs` 和 `sdp/types.rs` 中的 `SdeProgressStep` / `SdpProgressStep` 是独立类型，不受影响）。(3) 集成测试 `test_progress_callback_receives_all_steps` 已重写，使用实际的 `ProgressCallback` 类型签名 `Box<dyn Fn(&str, f32, &str) + Send + Sync>`，测试逻辑清晰，断言完整。

**IN-04 验证结果：已修复。** `generate_srt_from_word_boundaries()` 中的 SRT 序号生成已从 `enumerate()` 的 `i + 1` 改为独立计数器 `seq`（第 22 行声明 `let mut seq = 1u32;`，第 35 行使用 `seq`，第 42 行 `seq += 1`）。计数器仅在成功 `push` 块后递增，跳过负时间戳的 word boundary 时序号仍连续。`enumerate()` 变量 `i` 仅用于日志中的边界编号（第 27 行），保留合理。

**IN-03 / IN-05 重新评估：维持跳过决定。** 两个发现的原始分析仍然成立——风险极低（中文 LLM 输出场景），修复复杂度高（需要 JSON tokenizer 或引号对匹配），可能引入新 bug。维持 iter 10 的跳过决定。

**整体评估：** 全部 5 个文件审查完毕，3 个修复均正确实现，未引入新的 BLOCKER 或 WARNING 级别问题。2 个跳过的发现经重新评估确认合理。代码质量良好，可以合并。

**注意：** 项目整体存在一个审查范围外的编译错误（`visual/frame_extractor.rs:78` 非 exhaustive match），不在本次审查文件范围内。

## Fix Verification

| ID (iter) | Description | Verification |
|-----------|-------------|-------------|
| IN-01 (10) | 移除 `collect_keyframe_paths()` 死代码 | Pass - 函数已移除，全 crate 无残留引用 |
| IN-02 (10) | 移除 `ProgressStep` 枚举，修正测试回调签名 | Pass - 枚举和 re-export 已移除，测试使用正确的 `ProgressCallback` 类型 |
| IN-04 (10) | SRT 序号使用独立计数器 | Pass - `seq` 仅在 push 后递增，跳过项不影响连续性 |

## Skipped Issues (Re-evaluated)

### IN-03 (maintained skip): strip_and_repair_json trailing comma replacement risk

**File:** `narratoai-core/src/documentary/script_gen.rs:349`
**Status:** 维持跳过。全局 `text.replace(",}", "}")` 存在理论上误替换 JSON 字符串值的风险，但仅在直接解析失败时触发，中文 LLM 输出极少包含 `,}` 文字模式。基于 JSON tokenizer 的精确替换实现复杂度高，引入新 bug 风险大于收益。

### IN-05 (maintained skip): single-quote replacement risk

**File:** `narratoai-core/src/documentary/script_gen.rs:363-367`
**Status:** 维持跳过。`!text.contains('"')` 前置条件已大幅缩小误替换范围，中文场景下英文缩写（"it's"）出现概率极低。

## Info

### IN-06: `enumerate()` 变量 `i` 在无跳过场景下未使用

**File:** `narratoai-core/src/documentary/subtitle.rs:23`
**Issue:** 修复 IN-04 后，`for (i, wb) in word_boundaries.iter().enumerate()` 中的 `i` 仅在第 27 行的 `tracing::warn!` 中使用（日志输出 word boundary 编号）。在正常运行路径（无负时间戳跳过）时 `i` 不参与任何计算。这是合理的设计——保留 `i` 用于诊断日志——但值得注意，若未来确认不再需要诊断日志，可简化为 `for wb in word_boundaries.iter()`。
**Fix:** 保持现状。`i` 在 warn 日志中提供有价值的诊断信息。

### IN-07: 测试中 `ProgressCallback` import 路径通过公共 re-export

**File:** `tests/documentary_integration_test.rs:117`
**Issue:** `use narratoai_core::documentary::ProgressCallback;` 通过 `mod.rs` 的 `pub use types::ProgressCallback` re-export 引用。这是一个类型别名（`Box<dyn Fn(&str, f32, &str) + Send + Sync>`），在测试中用于显式标注回调变量类型。这确保了测试与生产代码的类型一致性，是正确的做法。此条目仅为记录，非问题。
**Fix:** 无需修改。

## Iteration History

| Iter | Date       | Critical | Warning | Info | Fixed | Status   | Key Changes                            |
|------|------------|----------|---------|------|-------|----------|----------------------------------------|
| 1-5  | 2026-05-07 | (see iter 6) |     |      |       |          | 初始审查与修复迭代                      |
| 6    | 2026-05-07 | 0        | 2       | 2    | 2     | partial  |                                        |
| 7    | 2026-05-07 | 1        | 1       | 4    | 2     | found    | subtitle 路径转义 + amix 补偿          |
| 8    | 2026-05-08 | 0        | 1       | 5    | 0     | found    | composite amix 补偿验证                |
| 9    | 2026-05-08 | 1        | 1       | 5    | 2     | found    | 编译错误 + 补偿系数不一致              |
| 10   | 2026-05-08 | 0        | 0       | 5    | 2     | clean    | 全部修复验证通过，无新问题             |
| 11   | 2026-05-08 | 0        | 0       | 2    | 3     | clean    | IN-01/02/04 修复验证通过，IN-03/05 维持跳过 |

---
_Reviewed: 2026-05-08T15:20:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 11_
