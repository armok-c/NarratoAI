---
status: partial
phase: 06-documentary-pipeline
review_date: 2026-05-05
fix_date: 2026-05-05
findings_in_scope: 11
fixed: 7
skipped: 4
iteration: 1
fixer: gsd-code-fixer
previous_fix: 2026-05-05 (Re-Fix #8 — IN-02/IN-05/IN-29/IN-32 fixed)
---

# Code Review Fix Report: Phase 06 — Documentary Pipeline (Re-Fix #9, --all)

## Summary

第九次修复：基于最新 REVIEW.md 的 11 项发现（2 CR + 4 WR + 5 IN），修复 7 项，跳过 4 项设计决策/功能补全类问题。通过 `cargo check` 和全部测试验证。

## Fixes Applied

### CR-01: BGM aloop + amix=duration=longest 导致超长输出 — **已修复**
- **提交**: `abf5098`
- **变更**: `src/documentary/pipeline.rs:308-311` — 在 `aloop` 后添加 `atrim=0:{total_dur},asetpts=PTS-STARTPTS` 限制 BGM 时长不超过视频总时长

### CR-02: SRT 路径未转义分号导致 filter_complex 注入 — **已修复**
- **提交**: `abf5098`
- **变更**: `src/documentary/pipeline.rs:335-343` — 添加 `.replace(';', "\\;")` `.replace('\n', "")` `.replace('\r', "")`

### WR-01: validate() 未检查 video_path 和 script_path — **已修复**
- **提交**: `4454411`
- **变更**: `src/documentary/types.rs:54-58` — 添加 `video_path.as_os_str().is_empty()` 和 `script_path.as_os_str().is_empty()` 非空校验

### WR-03: subtitle_font/color/position 字段未使用 — **已修复**
- **提交**: `abf5098`
- **变更**: `src/documentary/pipeline.rs:287-301,354-356` — 将 `subtitle_color`（转换为 ASS &H00BBGGRR 格式）、`subtitle_position`（映射为 ASS Alignment）、`subtitle_font`（可选 FontName）纳入 `force_style` 参数

### WR-04: collect_keyframe_paths 静默丢弃 read_dir 错误 — **已修复**
- **提交**: `e2862a6`
- **变更**: `src/documentary/script_gen.rs:385-401` — 改用显式 `for` 循环 + `map_err` 将 `io::Error` 转为 `PipelineError::FrameExtraction`

### IN-04: 极端 f64 值产生无效时间字符串 — **已修复**
- **提交**: `28e1061`
- **变更**: `src/documentary/timestamp.rs:85,99` — 在 `secs_to_srt_time` 和 `secs_to_ffmpeg_time` 中添加 `secs.min(86399.999)` 钳位

### IN-05: FFmpeg Progress 事件被静默忽略 — **已修复**
- **提交**: `04772ad`
- **变更**: `src/documentary/clip.rs:182-184` — 添加 `FfmpegEvent::Progress` 匹配分支，记录 `tracing::trace!` 日志

## Skipped (设计决策/功能补全)

| ID | 问题 | 跳过原因 |
|----|------|----------|
| WR-02 | `original_volume` 定义但未在流水线使用 | 设计决策：当前合成步骤仅混入 TTS+BGM，原声音量控制需重新设计音频混入逻辑 |
| IN-01 | `SubtitleSegment.offset_secs` 始终为 0.0 | 设计决策：双偏移模式需统一为单一偏移策略，涉及 audio.rs 和 subtitle.rs 联动 |
| IN-02 | `PipelineState.progress` 硬编码 None | 功能补全：需在 `DocumentaryRequest` 添加 `progress` 字段并传播 |
| IN-03 | 4 个集成测试被 ignore 且体为空 | 功能补全：需创建合成视频和完整测试体 |

## Verification

| 验收条件 | 结果 |
|----------|------|
| `cargo check` 编译通过 | ✅ 零 error（1 warning：`get_azure_voices` 不在审查范围） |
| `cargo test --lib documentary` | ✅ 58 passed, 0 failed |
| `cargo test --test documentary_integration_test` | ✅ 9 passed, 4 ignored, 0 failed |
| CR-01 修复: atrim 限制 BGM 时长 | ✅ pipeline.rs:309 |
| CR-02 修复: 分号转义 | ✅ pipeline.rs:341-343 |
| WR-01 修复: 路径非空校验 | ✅ types.rs:54-58 |
| WR-03 修复: 字幕样式字段使用 | ✅ pipeline.rs:287-301,354-356 |
| WR-04 修复: read_dir 错误处理 | ✅ script_gen.rs:385-401 |
| IN-04 修复: 时间值钳位 | ✅ timestamp.rs:85,99 |
| IN-05 修复: Progress trace 日志 | ✅ clip.rs:182-184 |

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-32 | Re-Fix #7 — `extract_json_object` 字节/字符索引混用 | ✅ 已修复 | 保持不变 |
| WR-31 | Re-Fix #6 — `parse_batch_response` 字符串切片 panic | ✅ 已修复 | 保持不变 |
| WR-29/WR-30 | Re-Fix #5 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-27/WR-28 | Re-Fix #4 全部 2 项 | ✅ 已修复 | 保持不变 |
| WR-18~WR-26 | Re-Fix #3 全部 9 项 | ✅ 已修复 | 保持不变 |
| IN-02/IN-05/IN-29/IN-32 | Re-Fix #8 全部 4 项 | ✅ 已修复 | 保持不变 |

---

_Fixed: 2026-05-05_
_Fixer: Claude (gsd-code-fixer)_
