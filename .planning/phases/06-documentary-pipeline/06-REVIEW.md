---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 9
  info: 5
  total: 14
reviewer: gsd-code-reviewer
date: 2026-05-03
previous_review: 2026-05-03 (WR-17 fixed, re-review)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review #2)

## Summary

第三次审查 12 个 Rust 源文件。上次 WR-17 已验证修复。本次新发现 0 个 CRITICAL、9 个 WARNING、5 个 INFO。

代码整体质量良好：错误类型体系完整、FFmpeg 错误处理一致、单元测试覆盖充分。主要问题集中在输入验证缺失和边界条件处理。

---

## WARNING Findings

### WR-18: `run_documentary()` 不调用 `request.validate()`
- **文件**: `src/documentary/pipeline.rs:412-462`
- `DocumentaryRequest` 提供了 `validate()` 方法（校验 voice_rate、volumes、threads），但入口函数 `run_documentary` 从未调用它。无效参数（如 voice_rate=0、threads=0）会静默传播到 TTS/FFmpeg 阶段才报错，错误信息不明确。
- **修复**: 在 `run_documentary` 开头添加 `request.validate().map_err(|e| PipelineError::Composite { details: e })?;`

### WR-19: `parse_srt_blocks` 不处理 `\r\n` 行尾
- **文件**: `src/documentary/subtitle.rs:73`
- `content.split("\n\n")` 无法匹配 `\r\n\r\n`（Windows 行尾），导致整段 SRT 被当作单个块解析失败。当前仅处理内部生成的 SRT（使用 `\n`），暂无影响。但 `merge_srt_files` 接收外部内容时会出问题。
- **修复**: 在分割前 `let content = content.replace("\r\n", "\n");`

### WR-20: `step_concat` 静默跳过无视频路径的片段
- **文件**: `src/documentary/pipeline.rs:179-185`
- `if let Some(ref video_path) = clip.video` 跳过没有视频路径的片段而不报错。如果上游裁剪步骤意外失败但未抛出错误，拼接阶段会悄悄跳过这些片段，产生不完整的输出视频。
- **修复**: 在 `else` 分支返回 `PipelineError::Concat { details: format!("片段 {} 缺少视频文件", clip._id) }`

### WR-21: 合并步骤进度百分比未递增
- **文件**: `src/documentary/pipeline.rs:169`
- `step_merge_audio_subtitle` 在完成时发送 60.0%，与 `step_clip` 结束时的 60.0% 相同。用户看不到合并阶段的进度变化。
- **修复**: 改为 `state.emit_progress(ProgressStep::MergeAudio, 70.0, "音频字幕合并完成");`

### WR-22: `clip_ost1_original_sound` 不验证 duration > 0
- **文件**: `src/documentary/clip.rs:49`
- 如果 timestamp 范围中 start == end（如 "00:00:05-00:00:05"），duration 为 0，FFmpeg 会产生不明确的错误。
- **修复**: 在调用 `run_clip_ffmpeg` 前添加 `if duration <= 0.0 { return Err(...); }`

### WR-23: `calculate_clip_duration` 可返回 0.0
- **文件**: `src/documentary/audio.rs:211`
- 如果 TTS 结果无时长、无显式 duration、无 source_time_range、timestamp 也不是有效范围，函数返回 0.0。零长度片段会导致累积时间计算错误和 FFmpeg 失败。
- **修复**: 在所有回退失败后返回 `Err` 而非 `0.0`，或在调用方检查 `dur > 0.0`。

### WR-24: 帧提取错误包装为 VideoClip 错误
- **文件**: `src/documentary/script_gen.rs:56-58`
- `analyze_video` 中帧提取失败时错误类型为 `PipelineError::VideoClip`，但帧提取属于脚本生成（LLM 分析）阶段，不是视频裁剪阶段。这会导致错误报告误导用户。
- **修复**: 使用更精确的错误变体，或在 `PipelineError` 中添加 `FrameExtraction` 变体。

### WR-25: Concat 列表中路径单引号未转义（IN-07 升级）
- **文件**: `src/documentary/pipeline.rs:183`
- `file '{}'\n` 格式中，如果路径包含单引号（如 `User's Videos/clip.mp4`），FFmpeg concat demuxer 解析会失败。
- **修复**: 转义路径中的单引号：`path_str.replace("'", "'\\''")`

### WR-26: `parse_batch_response` 静默吞没解析错误
- **文件**: `src/documentary/script_gen.rs:200-238`
- 当 LLM 返回的 JSON 无法解析时，函数返回一个空的 `BatchAnalysisResult` 并将错误信息放入 `errors` 字段。调用方 `analyze_video` 不检查 `errors`，直接将结果传递给后续步骤。无效的帧分析会静默传播。
- **修复**: 在 `analyze_video` 中检查 `batch.errors`，如果所有批次都有错误则返回失败。

---

## INFO Findings

### IN-02: `parse_timestamp_range` 应使用 `splitn(2, '-')`（遗留）
- **文件**: `src/documentary/timestamp.rs:7`
- `split('-')` 在含多个 `-` 的输入时可能意外分割。`splitn(2, '-')` 更安全。

### IN-05: 空 segments 输入返回 `Ok("")` 而非错误（遗留）
- **文件**: `src/documentary/subtitle.rs:43`
- `merge_srt_files(&[])` 返回 `Ok("")`，可能掩盖上游逻辑错误。

### IN-27: 关键帧缓存回退到当前工作目录（原 IN-09）
- **文件**: `src/documentary/script_gen.rs:39`
- `video_path.parent()` 为 None 时回退到 `"."`，缓存可能写入意外位置。且缓存目录永不清理。

### IN-28: 忽略的 FFmpeg 测试体为空（原 IN-12）
- **文件**: `tests/documentary_integration_test.rs:166-203`
- 4 个 `#[ignore]` 测试只有空体和注释，即使 FFmpeg 可用也不测试任何功能。

### IN-29: 测试 WordBoundary 数值使用非标准分组（原 IN-13）
- **文件**: `tests/documentary_integration_test.rs:50-72`
- `5_000_000_0`、`500_000_0` 等非标准分组降低可读性，应改为 `50_000_000`、`5_000_000`。

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-17 | step_concat FFmpeg 错误处理 | ✅ 已修复 | `had_errors` 标志已添加，与其他步骤一致 |

---

## Priority Fix Order

1. **WR-18** — validate() 未调用（高影响，一行修复）
2. **WR-20** — concat 静默跳过片段（可能导致不完整输出）
3. **WR-23** — 零时长片段（可能导致 FFmpeg 错误）
4. **WR-21** — 进度百分比（一行修复）
5. **WR-19** — \r\n 行尾兼容（跨平台正确性）
6. **WR-22** — duration > 0 验证（防御性编程）
7. **WR-25** — 路径单引号转义（边缘情况）
8. **WR-24** — 错误类型语义（代码清晰度）
9. **WR-26** — 批次解析错误传播（数据质量）
