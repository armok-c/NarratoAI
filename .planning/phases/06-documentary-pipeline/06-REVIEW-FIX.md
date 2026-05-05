---
status: partial
phase: 06-documentary-pipeline
review_date: 2026-05-05
fix_date: 2026-05-05
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 3
fixer: gsd-code-fixer
previous_fix: 2026-05-05 (Re-Fix #9 — CR-01/CR-02/WR-01/WR-03/WR-04/IN-04/IN-05 fixed)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #10, Re-Review #2)

## Summary

第十次修复：基于 Re-Review #2 的 2 项 Warning 发现，全部修复。通过 `cargo check` 和全部测试验证。

## Fixes Applied

### WR-01: `original_volume` validated but never applied in composite step — **已修复**

- **提交**: `8177521`
- **变更**: `src/documentary/pipeline.rs:286-287,313-356,385` — 在 `step_composite` 中将视频原始音频 `[0:a]volume={original_volume}[orig]` 加入 `amix` 混音链
- **逻辑**: 通过 `state.script` 检测是否存在非 `NarrationOnly` 片段，仅在有原始音频时加入混音，避免无音频流时的 FFmpeg 错误。新增 `amix_input_count` 统一管理混音输入数（含原始音频），`external_audio_count` 仅用于计算外部输入的 FFmpeg index

### WR-02: `video_aspect` field defined but never used — **已修复**

- **提交**: `8177521`
- **变更**: `src/documentary/types.rs:23,46` — 移除 `video_aspect` 字段及其 Default 值（`"9:16"`）
- **原因**: 该字段从未在 pipeline.rs 或 clip.rs 中使用，`-aspect` 标志仅设置元数据会导致拉伸问题，正确的宽高比缩放需要设计决策（目标分辨率、pad 策略），因此移除死代码

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error（1 warning：`get_azure_voices` 不在审查范围） |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| WR-01 修复: 原始音频加入 amix | ✅ pipeline.rs:315-317 `[0:a]volume={orig_vol}[orig]` |
| WR-02 修复: video_aspect 字段移除 | ✅ types.rs 已移除 |

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| CR-01 | BGM aloop + amix 超长输出 | ✅ 已修复 (Re-Fix #9) | 保持不变 |
| CR-02 | SRT 路径分号注入 | ✅ 已修复 (Re-Fix #9) | 保持不变 |
| WR-01→WR-01 | original_volume 未应用 | ✅ 已修复 (本次) | pipeline.rs:315-317 |
| WR-02→WR-03→WR-02 | video_aspect 未使用 | ✅ 已修复 (本次) | types.rs 已移除 |
| WR-04 | collect_keyframe_paths 静默错误 | ✅ 已修复 (Re-Fix #9) | 保持不变 |
| IN-04 | 极端 f64 时间值 | ✅ 已修复 (Re-Fix #9) | 保持不变 |
| IN-05 | FFmpeg Progress 事件忽略 | ✅ 已修复 (Re-Fix #9) | 保持不变 |

## Outstanding (Info-level, not in fix scope)

| ID | 问题 | 说明 |
|----|------|------|
| IN-01 | `SubtitleSegment.offset_secs` 始终为 0.0 | 设计决策：双偏移模式需统一 |
| IN-02 | `PipelineState.progress` 硬编码 None | 功能补全：需添加 progress 字段并传播 |
| IN-03 | 4 个集成测试被 ignore 且体为空 | 功能补全：需创建合成视频和完整测试体 |
| IN-04 | `threads` 字段未传递给 FFmpeg | 设计决策：需决定是否传递 `-threads` |

---

_Fixed: 2026-05-05_
_Fixer: Claude (gsd-code-fixer)_
