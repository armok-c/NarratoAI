---
status: all_fixed
phase: 06-documentary-pipeline
findings_in_scope: 16
fixed: 16
skipped: 0
iteration: 1
fixer: gsd-code-fixer
date: 2026-05-03
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline

## Summary

修复了 16 个 WARNING 级别问题，全部通过验证。56 个 documentary 模块单元测试全部通过。

## Fixed Findings

### WR-01: `secs_to_srt_time` / `secs_to_ffmpeg_time` 负数溢出
- **修复**: 在 `as u64` 转换前检查 `secs < 0.0`，负数返回零时间戳 `"00:00:00,000"` / `"00:00:00.000"`。
- **文件**: `src/documentary/timestamp.rs`

### WR-02: `parse_time_to_secs` 未验证 h/m/s 范围
- **修复**: 添加 `h ∈ [0,23]`、`m ∈ [0,59]`、`s ∈ [0,59]` 范围校验。
- **文件**: `src/documentary/timestamp.rs`

### WR-03: `parse_srt_blocks` 纯数字文本行误判
- **修复**: 改用 `split("\n\n")` 按空行分割块，块内跳过第一行（序号），解析第二行（时间戳），收集剩余行（文本）。
- **文件**: `src/documentary/subtitle.rs`

### WR-04: `merge_srt_files` 偏移后不检查负时间戳
- **修复**: `apply_offset_to_block` 中检查 `new_start < 0.0 || new_end < 0.0`，返回 `PipelineError::SrtGeneration`。
- **文件**: `src/documentary/subtitle.rs`

### WR-05: `DocumentaryRequest` 字段缺少取值验证
- **修复**: 添加 `validate(&self) -> Result<(), String>` 方法，校验 `voice_rate`、`tts_volume`、`original_volume`、`bgm_volume`、`threads` 的取值范围。
- **文件**: `src/documentary/types.rs`

### WR-06: `step_composite` 死代码
- **修复**: 删除闭包外的 `input_idx`、`filter_parts`、`audio_mix_inputs` 及相关未使用变量（`has_bgm`、`_needs_subtitle`、`_has_audio_mix`、`_subtitle_color`）。
- **文件**: `src/documentary/pipeline.rs`

### WR-07: `-map 0:v` 与 `-map [vout]` 冲突
- **修复**: 重构 map 逻辑 — 使用 `has_video_filter` 标志控制单个视频 map，不再叠加多个 `-map`。
- **文件**: `src/documentary/pipeline.rs`

### WR-08: `-vf` 与 `-filter_complex` 混用
- **修复**: 字幕视频滤镜统一通过 `filter_complex` 处理（`[0:v]subtitles=...[vout]`），移除 `-vf` 路径。
- **文件**: `src/documentary/pipeline.rs`

### WR-09: SRT 路径转义不完整
- **修复**: 添加 `.replace('[', "\\[")` 和 `.replace(']', "\\]")` 转义方括号。
- **文件**: `src/documentary/pipeline.rs`

### WR-10: 两个不同的 clip duration 计算函数
- **修复**: 统一为 `audio::calculate_clip_duration`（pub），优先级：OST 0/2 先查 TTS → 显式 duration → source_time_range → timestamp range。pipeline.rs 导入并使用该函数，删除内部 `compute_clip_duration`。
- **文件**: `src/documentary/audio.rs`、`src/documentary/pipeline.rs`

### WR-11: FFmpeg 错误处理不一致（pipeline.rs composite 步骤）
- **修复**: 在 composite 闭包中添加 `had_errors` 标志，Error 事件时设为 true，循环结束后检查并返回错误。
- **文件**: `src/documentary/pipeline.rs`

### WR-12: `strip_and_repair_json` 的 `{{`→`{` 替换破坏合法 JSON
- **修复**: 移除全局 `replace("{{", "{")`。改为仅在文本首尾同时为 `{{` / `}}` 时剥离一层。每步修复后立即尝试解析，成功即返回。
- **文件**: `src/documentary/script_gen.rs`

### WR-13: `extract_json_object` 不考虑字符串内花括号
- **修复**: 添加 `in_string` 和 `escape_next` 状态跟踪，字符串内的花括号不参与深度计数。
- **文件**: `src/documentary/script_gen.rs`

### WR-14: `amix` 滤镜 inputs=1 不合法
- **修复**: 当 `input_count == 1`（仅有静音基底）时跳过 `amix`，改用 `volume=1[aout]` 直通。
- **文件**: `src/documentary/audio.rs`

### WR-15: `audio.rs` FFmpeg Error 事件只记录不传播
- **修复**: 添加 `had_errors` 标志，与 `clip.rs` 保持一致的严格模式。
- **文件**: `src/documentary/audio.rs`

### WR-16: `clip.rs` 未使用的 `probe_video` 导入
- **修复**: 删除 `use crate::ffmpeg::probe::probe_video;`。
- **文件**: `src/documentary/clip.rs`

## Test Results

- Documentary 模块：56 passed, 0 failed
- 总体：424 passed, 1 failed（`hwaccel::test_detect_encoders_format` — 环境相关，与本次修复无关）
