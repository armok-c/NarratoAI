---
phase: 09-jianying-export
reviewed: 2026-05-08T09:15:00Z
depth: standard
iteration: 10
files_reviewed: 11
files_reviewed_list:
  - narratoai-core/src/jianying/mod.rs
  - narratoai-core/src/jianying/error.rs
  - narratoai-core/src/jianying/time.rs
  - narratoai-core/src/jianying/types.rs
  - narratoai-core/src/jianying/template.rs
  - narratoai-core/src/jianying/material.rs
  - narratoai-core/src/jianying/segment.rs
  - narratoai-core/src/jianying/track.rs
  - narratoai-core/src/jianying/builder.rs
  - narratoai-core/src/ffmpeg/probe.rs
  - tests/jianying_export.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 09: Code Review Report (Iteration 10)

**Reviewed:** 2026-05-08T09:15:00Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Iteration 10 代码审查。逐行审阅了全部 11 个文件（共约 1600 行 Rust 代码）。所有前序修复（WR-01 至 WR-06）均无回归。未发现新的 BLOCKER 或 WARNING 级别问题。3 个延续的 INFO 级别问题自 iteration 4 以来未修复，仍然有效。

代码整体质量稳定：错误处理链路完整（JianYingError 覆盖 Io/JsonSerialize/Validation/MissingField/ProbeError 五种变体），路径验证（canonicalize + is_absolute）、草稿名称注入防护（validate_draft_name 拒绝 /\\.. 路径遍历）、轨道类型匹配（add_video_segment/add_audio_segment 前置类型检查）、失败清理（build_timeline/save 失败时 remove_dir_all）均已到位。类型系统通过 SegmentOutput 枚举统一 VideoSegment/AudioSegment 输出，TrackJson.segments 使用 serde_json::Value 避免泛型复杂度。时间处理（trange/trange_from_secs/parse_timestamp_start）均有 NaN/Inf/负数/零值边界检查。

### 之前的修复验证（全部仍然生效）

- **WR-01**（Track segment type matching）：`track.rs:53` 和 `track.rs:67` 中 `add_video_segment` / `add_audio_segment` 均在添加前检查 `track_type`。已确认。
- **WR-02**（Path validation）：`material.rs:59-63` 和 `material.rs:123-127` 中 `VideoMaterial::new` 和 `AudioMaterial::new` 均在 `canonicalize` 后验证绝对路径。已确认。
- **WR-03**（Draft directory cleanup on build_timeline failure）：`builder.rs:272-275` 中 `export_draft` 在 `build_timeline` 失败时调用 `remove_dir_all` 清理草稿目录。已确认。
- **WR-04**（Zero duration rejection）：`time.rs:20` 和 `time.rs:33-38` 中 `trange` 和 `trange_from_secs` 均拒绝 `duration <= 0`。已确认。
- **WR-05**（Shared build_base_segment helper）：`segment.rs:25-56` 中提取了共享的 `build_base_segment` 函数，`VideoSegment::to_json` 和 `AudioSegment::to_json` 均调用它。已确认。
- **WR-06**（save 失败时清理草稿目录）：`builder.rs:267-270` 中 `export_draft` 在 `script_file.save` 失败时调用 `remove_dir_all` 清理草稿目录。已确认。

## Critical Issues

None.

## Warnings

None.

## Info

### IN-01: probe.rs 错误消息语言不一致（延续自 iteration 4，未修复）

**File:** `narratoai-core/src/ffmpeg/probe.rs:63,67,97,141,154`
**Issue:** `probe_video` 和 `probe_audio` 中的错误消息混用英文和中文。例如第 63 行 `"Missing or invalid duration in ffprobe output"`（英文）vs 第 151 行 `"音频时长解析失败"`（中文）。`probe_video` 的所有错误消息均为英文（第 47、63、67、97 行），而 `probe_audio` 部分使用中文（第 151 行）部分使用英文（第 141、154 行）。项目 jianying 模块统一使用中文错误消息。
**Fix:** 将 `probe.rs` 中所有用户可见的错误消息统一为中文：
```rust
// 第 47 行:
format!("ffprobe 退出码 {:?}: {}", output.status.code(), stderr)
// 第 63 行:
"ffprobe 输出中缺少 duration 字段或格式无效".into()
// 第 67 行:
format!("无效的视频时长: {}", duration_secs)
// 第 97 行:
"ffprobe 输出中未找到视频流".into()
// 第 141 行:
format!("ffprobe 退出码 {:?}: {}", output.status.code(), stderr)
// 第 154 行:
format!("ffprobe 返回无效的音频时长: {}", duration)
```

### IN-02: 集成测试 make_clip 时间戳格式溢出风险（延续自 iteration 4，未修复）

**File:** `tests/jianying_export.rs:46-49`
**Issue:** `make_clip` 中的时间戳格式 `"00:00:{:02},000-00:00:{:02},000"` 使用 `id * 5` 和 `id * 5 + 5` 填充秒位。当 `id >= 12` 时秒值超过 59，产生无效时间戳如 `"00:00:65,000"`。当前测试中 `id` 范围为 1-3，不会触发此问题，但如果未来添加新测试用例并复用 `make_clip` 且使用较大的 `id`，可能产生难以调试的测试失败。
**Fix:** 修改时间戳格式以支持大于 59 秒的情况（使用分钟位或小时位），或在 `make_clip` 中添加 `assert!(id < 12)` 上限断言：
```rust
fn make_clip(...) -> ScriptClip {
    assert!(id >= 0 && id < 12, "id 应在 0..12 范围内以避免秒位溢出");
    // ...
}
```

### IN-03: draft_meta_info.json 使用紧凑格式而 draft_content.json 使用美化格式（延续自 iteration 4，未修复）

**File:** `narratoai-core/src/jianying/builder.rs:103,239`
**Issue:** `DraftFolder::create_draft` 使用 `serde_json::to_string`（紧凑格式）写入 `draft_meta_info.json`，而 `ScriptFile::save` 使用 `serde_json::to_string_pretty`（美化格式）写入 `draft_content.json`。两者都是给剪映软件读取的文件，格式不一致影响可读性和版本控制一致性。
**Fix:** 将第 103 行 `serde_json::to_string(&meta)` 改为 `serde_json::to_string_pretty(&meta)`。

---
_Reviewed: 2026-05-08T09:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 10_
