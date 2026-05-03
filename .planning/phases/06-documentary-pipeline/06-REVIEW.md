---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 2
  info: 6
  total: 8
reviewer: gsd-code-reviewer
date: 2026-05-03
previous_review: 2026-05-03 (Re-Review #2 — 9 WARNING fixed)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review #3)

## Summary

第四次审查 12 个 Rust 源文件。上次 9 个 WARNING（WR-18~WR-26）全部验证已修复。本次新发现 0 个 CRITICAL、2 个 WARNING、6 个 INFO。

代码质量持续改善：错误类型体系完整、FFmpeg 错误处理一致、输入验证已到位。剩余问题集中在两个错误变体映射不准确和测试同步性。

---

## WARNING Findings

### WR-27: `request.validate()` 错误映射为 `Composite` 类型
- **文件**: `src/documentary/pipeline.rs:426`
- `request.validate().map_err(|e| PipelineError::Composite { details: e })?;` 将参数校验错误映射为「最终合成失败」。用户看到 "最终合成失败: voice_rate 超出有效范围" 会误以为合成步骤出了问题，实际是流水线启动前的参数校验。
- **修复**: 添加 `PipelineError::Validation { details: String }` 变体，或映射到更合适的现有变体（如 `Timestamp`）。

### WR-28: 零时长错误映射为 `Composite` 而非 `VideoClip`
- **文件**: `src/documentary/pipeline.rs:116-119`
- `step_clip` 中 `clip_duration <= 0.0` 时返回 `PipelineError::Composite`。该检查发生在步骤 3（视频裁剪），应使用 `PipelineError::VideoClip`。当前用户看到 "最终合成失败: 片段 X 时长计算为零"，错误归因到步骤 6。
- **修复**: 将 `PipelineError::Composite` 改为 `PipelineError::VideoClip`。

---

## INFO Findings

### IN-02: `parse_timestamp_range` 使用 `split('-')` 而非 `splitn(2, '-')`（遗留）
- **文件**: `src/documentary/timestamp.rs:7`
- `split('-')` 在含多个 `-` 的输入时产生意外分割。`splitn(2, '-')` 更安全。实际场景中时间戳不包含多余 `-`，影响极低。

### IN-05: `merge_srt_files(&[])` 返回 `Ok("")` 而非错误（遗留）
- **文件**: `src/documentary/subtitle.rs:43`
- 空输入返回空字符串，可能掩盖上游逻辑错误。当前调用方已确保传入非空 segments。

### IN-27: 关键帧缓存回退到当前工作目录（遗留）
- **文件**: `src/documentary/script_gen.rs:39`
- `video_path.parent()` 为 None 时回退到 `"."`，缓存可能写入意外位置。

### IN-28: 忽略的 FFmpeg 测试体为空（遗留）
- **文件**: `tests/documentary_integration_test.rs:166-203`
- 4 个 `#[ignore]` 测试只有空体和注释，即使 FFmpeg 可用也不测试任何功能。

### IN-29: 测试 WordBoundary 数值使用非标准分组（遗留）
- **文件**: `tests/documentary_integration_test.rs:50-72`
- `5_000_000_0` 等非标准分组降低可读性，应改为 `50_000_000`。

### IN-30: 集成测试进度值与实现不同步
- **文件**: `tests/documentary_integration_test.rs:134-135`
- 测试手动发送 `MergeAudio` 进度 60.0%，但实现（WR-21 修复后）已改为 70.0%。测试不验证实际流水线的百分比，仅验证回调机制。不影响正确性但不再反映真实行为。

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-18 | validate() 未调用 | ✅ 已修复 | `request.validate()` 在 `run_documentary` 入口调用 |
| WR-19 | \r\n 行尾不处理 | ✅ 已修复 | `content.replace("\r\n", "\n")` 已添加 |
| WR-20 | concat 静默跳过片段 | ✅ 已修复 | 缺少视频路径时返回 `PipelineError::Concat` |
| WR-21 | 进度百分比未递增 | ✅ 已修复 | `step_merge_audio_subtitle` 改为 70.0% |
| WR-22 | duration > 0 未验证 | ✅ 已修复 | `clip_ost1` 中添加零/负时长检查 |
| WR-23 | calculate_clip_duration 返回 0.0 | ✅ 已修复 | `step_clip` 中检查 `clip_duration <= 0.0` |
| WR-24 | 帧提取错误类型不准确 | ✅ 已修复 | 新增 `FrameExtraction` 变体 |
| WR-25 | 路径单引号未转义 | ✅ 已修复 | `.replace("'", "'\\''")` 已添加 |
| WR-26 | 批次解析错误静默传播 | ✅ 已修复 | 所有批次失败时返回 `PipelineError::Llm` |

---

## Priority Fix Order

1. **WR-27** — validate() 错误类型语义（1 行修复 + 可选新增变体）
2. **WR-28** — 零时长错误类型（1 行修复）
