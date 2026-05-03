---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 2
  info: 7
  total: 9
reviewer: gsd-code-reviewer
date: 2026-05-03
previous_review: 2026-05-03 (Re-Review #3 — WR-27/WR-28 fixed)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review #4)

## Summary

第五次审查 12 个 Rust 源文件。上次 2 个 WARNING（WR-27、WR-28）全部验证已修复。本次新发现 0 个 CRITICAL、2 个 WARNING（含 IN-30 升级为 WR-30）、新增 1 个 INFO（IN-31）。

代码质量持续改善：错误类型语义准确、FFmpeg 错误传播一致、Validation 变体已正确使用。剩余问题集中在 `voice_pitch` 缺少范围校验和测试进度值与实现不同步。

---

## WARNING Findings

### WR-29: `validate()` 未校验 `voice_pitch` 范围
- **文件**: `src/documentary/types.rs:54-71`
- **描述**: `validate()` 校验了 `voice_rate` (0, 5]、`tts_volume` [0, 10]、`original_volume` [0, 10]、`bgm_volume` [0, 10]，但 `voice_pitch`（默认 0.0，第 36 行）从未被校验。该字段直接传递给 `crate::tts::synthesize()`（pipeline.rs:68）。如果 `voice_pitch` 收到极端值（NaN、无穷大或超出 TTS 引擎范围的值），行为完全取决于下游实现，可能导致 panic 或静默生成错误音频。这与所有其他数值参数的校验策略不一致。
- **修复**: 在 `validate()` 中添加 `voice_pitch` 范围检查：
  ```rust
  if self.voice_pitch < -10.0 || self.voice_pitch > 10.0 {
      return Err(format!("voice_pitch 超出有效范围 [-10, 10]: {}", self.voice_pitch));
  }
  ```

### WR-30: 集成测试进度值与实现不同步（IN-30 升级）
- **文件**: `tests/documentary_integration_test.rs:134-135`
- **描述**: 测试手动发送 `ProgressStep::MergeAudio` 进度 `60.0`，但实现（pipeline.rs:174）已改为 `70.0`。测试断言非递减进度，数据本身不反映实际值。如果开发者复制此示例编写真实流水线测试，将传播错误预期。
- **修复**: 将第 134 行改为 `70.0`：
  ```rust
  callback(ProgressStep::MergeAudio, 70.0, "音频字幕合并完成");
  ```

---

## INFO Findings

### IN-02: `parse_timestamp_range` 使用 `split('-')` 而非 `splitn(2, '-')`（遗留）
- **文件**: `src/documentary/timestamp.rs:7`
- **状态**: 无变化，影响极低。

### IN-05: `merge_srt_files(&[])` 返回 `Ok("")` 而非错误（遗留）
- **文件**: `src/documentary/subtitle.rs:43`
- **状态**: 无变化，调用方已确保传入非空。

### IN-27: 关键帧缓存回退到当前工作目录（遗留）
- **文件**: `src/documentary/script_gen.rs:39`
- **状态**: 无变化。

### IN-28: 忽略的 FFmpeg 测试体为空（遗留）
- **文件**: `tests/documentary_integration_test.rs:166-203`
- **状态**: 无变化。

### IN-29: 测试 WordBoundary 数值使用非标准分组（遗留）
- **文件**: `tests/documentary_integration_test.rs:50-72`
- **状态**: 无变化。

### IN-31: `subtitle_font`、`subtitle_color`、`subtitle_position`、`video_aspect` 字段声明但未使用
- **文件**: `src/documentary/types.rs:19,21-23`、`src/documentary/pipeline.rs`（step_composite）
- **描述**: `DocumentaryRequest` 声明了 `subtitle_font`、`subtitle_color`、`subtitle_position`、`video_aspect` 四个字段。`step_composite` 的 FFmpeg `force_style` 只使用 `FontSize`，未使用 `FontName`、`PrimaryColour`、`MarginV`。`video_aspect` 在流水线中从未被引用。这些字段代表未实现的配置面，可能误导用户以为设置后会生效。
- **修复**: 在 `step_composite` 的 FFmpeg filter 中实现这些字段（`force_style='FontName=...,PrimaryColour=...,MarginV=...'`），或标注为未来版本的占位符。

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-27 | validate() 错误映射为 Composite | ✅ 已修复 | `PipelineError::Validation` 变体存在于 error.rs:43-44，中文消息 "参数校验失败"，测试在 error.rs:172-178 |
| WR-28 | 零时长错误映射为 Composite | ✅ 已修复 | pipeline.rs:116 使用 `PipelineError::VideoClip`，正确的步骤 3 错误变体 |
| WR-18~WR-26 | Re-Fix #3 全部 9 项 | ✅ 已修复 | 保持不变 |

---

## Priority Fix Order

1. **WR-29** — 添加 `voice_pitch` 范围校验（1-3 行）
2. **WR-30** — 修复集成测试进度值 60.0 → 70.0（1 行）
3. **IN-31** — 文档化或实现未使用的字幕/宽高字段（低优先级，设计决策）
