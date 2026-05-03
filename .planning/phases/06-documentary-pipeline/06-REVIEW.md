---
status: issues_found
phase: 06-documentary-pipeline
depth: standard
files_reviewed: 12
findings:
  critical: 0
  warning: 1
  info: 6
  total: 7
reviewer: gsd-code-reviewer
date: 2026-05-03
previous_review: 2026-05-03 (16 WARNING fixed)
---

# Code Review: Phase 06 — Documentary Pipeline (Re-Review)

## Summary

二次审查 12 个 Rust 源文件（9 个核心模块 + 1 个 lib.rs + 2 个测试文件）。
上次审查发现的 16 个 WARNING 已全部修复并验证。
本次发现 0 个 CRITICAL、1 个 WARNING（上次 WR-11 修复遗漏）、6 个 INFO（上次遗留）。

修复质量评估：16/16 WARNING 修复正确，代码质量显著提升。

---

## WARNING Findings

### WR-17: `step_concat` FFmpeg Error 事件仅记录未标记（WR-11 修复遗漏）
- **文件**: `src/documentary/pipeline.rs:218-222`
- 上次 WR-11 修复覆盖了 `clip.rs`、`audio.rs` 和 `composite` 步骤，但 `step_concat` 仍然只 `tracing::error!` 而不设置 `had_errors` 标志。FFmpeg 可能在 stderr 报错但退出码为 0，导致拼接错误被静默吞掉。
- **修复**: 在 concat 闭包中添加 `had_errors` 标志，与 clip/audio/composite 保持一致的严格模式。

---

## INFO Findings (遗留)

### IN-02: `parse_timestamp_range` 应使用 `splitn(2, '-')`
- **文件**: `src/documentary/timestamp.rs:7`
- `split('-')` 在极端输入（含多个 `-`）时可能意外分割。`splitn(2, '-')` 更安全。

### IN-05: 空 segments 输入返回 `Ok("")` 而非错误
- **文件**: `src/documentary/subtitle.rs:43`
- `merge_srt_files(&[])` 返回 `Ok("")`，可能掩盖上游逻辑错误。

### IN-07: Concat 列表路径单引号未转义
- **文件**: `src/documentary/pipeline.rs:183`
- `file '{}'\n` 中路径包含单引号会导致 concat demuxer 解析失败。

### IN-09: 关键帧缓存回退到当前工作目录
- **文件**: `src/documentary/script_gen.rs:39`
- `video_path.parent()` 为 None 时回退到 `"."`，缓存可能写入意外位置。

### IN-12: 忽略的 FFmpeg 测试体为空
- **文件**: `tests/documentary_integration_test.rs:167-203`
- 4 个 `#[ignore]` 测试只有空体和冗余的 `ffmpeg_available()` 检查。

### IN-13: 测试 WordBoundary 数值使用非标准分组
- **文件**: `tests/documentary_integration_test.rs:50-72` + `src/documentary/subtitle.rs:160`
- `5_000_000_0`、`500_000_0` 等非标准分组降低可读性，建议改为 `50_000_000`、`5_000_000`。

---

## Previous Fix Verification

| ID | 问题 | 修复状态 | 验证结果 |
|----|------|----------|----------|
| WR-01 | secs_to_srt_time 负数溢出 | ✅ 已修复 | `secs < 0.0` 返回零时间戳 |
| WR-02 | parse_time_to_secs 范围验证 | ✅ 已修复 | h/m/s 范围校验已添加 |
| WR-03 | parse_srt_blocks 数字行误判 | ✅ 已修复 | 改用空行分割块 |
| WR-04 | merge_srt 负时间戳 | ✅ 已修复 | 检查 `new_start < 0.0` |
| WR-05 | DocumentaryRequest 字段验证 | ✅ 已修复 | `validate()` 方法已添加 |
| WR-06 | step_composite 死代码 | ✅ 已修复 | 已删除 |
| WR-07 | -map 冲突 | ✅ 已修复 | `has_video_filter` 标志控制 |
| WR-08 | -vf 与 -filter_complex 冲突 | ✅ 已修复 | 统一使用 filter_complex |
| WR-09 | SRT 路径转义 | ✅ 已修复 | `[]` 转义已添加 |
| WR-10 | 双 clip duration 函数 | ✅ 已修复 | 统一为 audio::calculate_clip_duration |
| WR-11 | FFmpeg 错误处理 | ⚠️ 部分修复 | composite/clip/audio 已修复，concat 遗漏 |
| WR-12 | JSON {{ }} 替换 | ✅ 已修复 | 仅剥首尾双层花括号 |
| WR-13 | extract_json_object 字符串内花括号 | ✅ 已修复 | in_string/escape_next 跟踪 |
| WR-14 | amix inputs=1 | ✅ 已修复 | 单输入使用 volume 直通 |
| WR-15 | audio.rs Error 事件 | ✅ 已修复 | had_errors 标志已添加 |
| WR-16 | clip.rs 未使用导入 | ✅ 已修复 | probe_video 已删除 |

---

## Priority Fix Order

1. **WR-17** — concat 步骤 FFmpeg 错误处理遗漏（一致性问题，低风险）
