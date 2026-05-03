---
status: all_fixed
phase: 06-documentary-pipeline
findings_in_scope: 9
fixed: 9
skipped: 0
iteration: 3
fixer: gsd-code-fixer
date: 2026-05-03
previous_fix: 2026-05-03 (WR-17 fixed in iteration 2)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #3)

## Summary

第三次修复：修复了 9 个 WARNING 级别问题，全部通过验证。57 个 documentary 模块单元测试全部通过。

## Fixed Findings

### WR-18: `run_documentary()` 不调用 `request.validate()`
- **修复**: 在 `run_documentary` 入口添加 `request.validate().map_err(|e| PipelineError::Composite { details: e })?;`，无效参数在流水线启动时即被拦截。
- **文件**: `src/documentary/pipeline.rs:417`

### WR-19: `parse_srt_blocks` 不处理 `\r\n` 行尾
- **修复**: 在分割前添加 `let content = content.replace("\r\n", "\n");`，兼容 Windows 行尾。
- **文件**: `src/documentary/subtitle.rs:70`

### WR-20: `step_concat` 静默跳过无视频路径的片段
- **修复**: 将 `if let Some` 改为 `if let ... { ... } else { return Err(...) }`，缺少视频路径时立即报错。
- **文件**: `src/documentary/pipeline.rs:185-190`

### WR-21: 合并步骤进度百分比未递增
- **修复**: `step_merge_audio_subtitle` 进度从 60.0 改为 70.0。
- **文件**: `src/documentary/pipeline.rs:172`

### WR-22: `clip_ost1_original_sound` 不验证 duration > 0
- **修复**: 在调用 `run_clip_ffmpeg` 前添加 `duration <= 0.0` 检查，返回明确的错误信息。
- **文件**: `src/documentary/clip.rs:51-58`

### WR-23: `calculate_clip_duration` 可返回 0.0
- **修复**: 在 `step_clip` 调用 `calculate_clip_duration` 后检查 `clip_duration <= 0.0`，零时长立即报错。
- **文件**: `src/documentary/pipeline.rs:117-121`

### WR-24: 帧提取错误包装为 VideoClip 错误
- **修复**: 添加 `PipelineError::FrameExtraction { details: String }` 变体，`script_gen.rs` 中的帧提取错误改用该变体。新增对应单元测试。
- **文件**: `src/documentary/error.rs:42`, `src/documentary/script_gen.rs:56-64`

### WR-25: Concat 列表中路径单引号未转义
- **修复**: 在路径字符串上添加 `.replace("'", "'\\''")` 转义单引号。
- **文件**: `src/documentary/pipeline.rs:186`

### WR-26: `parse_batch_response` 静默吞没解析错误
- **修复**: 在 `analyze_video` 的批次解析循环后添加检查：所有批次均有错误时返回 `PipelineError::Llm`，避免无效帧分析静默传播。
- **文件**: `src/documentary/script_gen.rs:103-111`

## Test Results

- Documentary 模块：57 passed, 0 failed
- 全库：425 passed, 1 failed (无关：`test_detect_encoders_format` 为环境依赖的硬件编码器检测测试)

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-17 | step_concat FFmpeg 错误处理 | ✅ 已修复（iteration 2） | 保持不变 |
