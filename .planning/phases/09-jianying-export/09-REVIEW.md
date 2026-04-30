---
phase: 09-jianying-export
reviewed: 2026-04-30T03:15:00Z
depth: standard
iteration: 9
files_reviewed: 13
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/jianying/mod.rs
  - src/jianying/error.rs
  - src/jianying/time.rs
  - src/jianying/types.rs
  - src/jianying/template.rs
  - src/jianying/material.rs
  - src/jianying/segment.rs
  - src/jianying/track.rs
  - src/jianying/builder.rs
  - src/ffmpeg/probe.rs
  - tests/jianying_export.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 09: Code Review Report (Iteration 9)

**Reviewed:** 2026-04-30T03:15:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Iteration 9 代码审查。本次验证了第 8 轮修复（WR-01 save 失败清理）已作为未提交更改存在于 `builder.rs` 中，且所有前序修复（WR-01 至 WR-05）均无回归。未发现新的 BLOCKER 或 WARNING 级别问题。3 个延续的 INFO 级别问题仍然有效。所有 72 个库测试和 10 个集成测试通过（2 个需要 ffmpeg 的测试被 ignore）。

### 之前的修复验证（全部仍然生效）

- **WR-01**（Track segment type matching）：`track.rs:52-63` 和 `track.rs:66-77` 中的 `add_video_segment` / `add_audio_segment` 均在添加前检查 `track_type`。已确认。
- **WR-02**（Path validation）：`material.rs:59-63` 和 `material.rs:123-127` 中 `VideoMaterial::new` 和 `AudioMaterial::new` 均在 `canonicalize` 后验证绝对路径。已确认。
- **WR-03**（Draft directory cleanup on build_timeline failure）：`builder.rs:272-275` 中 `export_draft` 在 `build_timeline` 失败时调用 `remove_dir_all` 清理草稿目录。已确认。
- **WR-04**（Zero duration rejection）：`time.rs:20` 和 `time.rs:33-34` 中 `trange` 和 `trange_from_secs` 均拒绝 `duration <= 0`。已确认。
- **WR-05**（Shared build_base_segment helper）：`segment.rs:25-56` 中提取了共享的 `build_base_segment` 函数，`VideoSegment::to_json` 和 `AudioSegment::to_json` 均调用它。已确认。
- **WR-06**（save 失败时清理草稿目录）：`builder.rs:265-270` 中 `export_draft` 在 `script_file.save` 失败时调用 `remove_dir_all` 清理草稿目录。已确认为未提交更改，与第 8 轮 WR-01 修复一致。

## Critical Issues

None.

## Warnings

None.

## Info

### IN-01: probe.rs 错误消息语言不一致（延续自 iteration 4）

**File:** `src/ffmpeg/probe.rs:63,67,97,151,154`
**Issue:** `probe_video` 和 `probe_audio` 中的错误消息混用英文和中文。例如第 63 行 `"Missing or invalid duration in ffprobe output"`（英文）vs 第 151 行 `"音频时长解析失败"`（中文）。项目其余 jianying 模块统一使用中文错误消息。
**Fix:** 将 `probe.rs` 中所有用户可见的错误消息统一为中文：
```rust
// 第 63 行:
"ffprobe 输出中缺少 duration 字段或格式无效".into()
// 第 67 行:
format!("无效的视频时长: {}", duration_secs)
// 第 97 行:
"ffprobe 输出中未找到视频流".into()
// 第 154 行:
format!("ffprobe 返回无效的音频时长: {}", duration)
```

### IN-02: 集成测试 make_clip 时间戳格式溢出风险（延续自 iteration 4）

**File:** `tests/jianying_export.rs:46-49`
**Issue:** `make_clip` 中的时间戳格式 `"00:00:{:02},000-00:00:{:02},000"` 使用 `id * 5` 和 `id * 5 + 5` 填充秒位。当 `id >= 12` 时秒值超过 59，产生无效时间戳如 `"00:00:65,000"`。当前测试中 `id` 范围为 1-3，不会触发此问题，但如果未来添加新测试用例并复用 `make_clip` 且使用较大的 `id`，可能产生难以调试的测试失败。
**Fix:** 修改时间戳格式以支持大于 59 秒的情况（使用分钟位或小时位），或在 `make_clip` 中添加 `assert!(id < 12)` 上限断言。

### IN-03: draft_meta_info.json 使用紧凑格式而 draft_content.json 使用美化格式（延续自 iteration 4）

**File:** `src/jianying/builder.rs:101,237`
**Issue:** `DraftFolder::create_draft` 使用 `serde_json::to_string`（紧凑格式）写入 `draft_meta_info.json`，而 `ScriptFile::save` 使用 `serde_json::to_string_pretty`（美化格式）写入 `draft_content.json`。两者都是给剪映软件读取的文件，格式不一致影响可读性和版本控制一致性。
**Fix:** 将第 101 行 `serde_json::to_string(&meta)` 改为 `serde_json::to_string_pretty(&meta)`。

---

_Reviewed: 2026-04-30T03:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 9_
