---
phase: 09-jianying-export
reviewed: 2026-04-30T02:39:20Z
depth: standard
iteration: 8
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
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 09: Code Review Report (Iteration 8)

**Reviewed:** 2026-04-30T02:39:20Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Iteration 8 代码审查。本次验证了第 7 轮修复（WR-01 至 WR-05）全部仍然生效，未发现回归。深入审查后发现 1 个新的 WARNING 级别问题（`save` 失败时草稿目录残留）以及 3 个延续的 INFO 级别问题。所有 72 个库测试和 10 个集成测试通过（2 个需要 ffmpeg 的测试被 ignore）。

### 之前的修复验证（WR-01 至 WR-05 全部仍然生效）

- **WR-01**（Track segment type matching）：`track.rs:52-63` 和 `track.rs:66-77` 中的 `add_video_segment` / `add_audio_segment` 均在添加前检查 `track_type`。已确认。
- **WR-02**（Path validation）：`material.rs:59-63` 和 `material.rs:123-127` 中 `VideoMaterial::new` 和 `AudioMaterial::new` 均在 `canonicalize` 后验证绝对路径。已确认。
- **WR-03**（Draft directory cleanup on failure）：`builder.rs:269-271` 中 `export_draft` 在 `build_timeline` 失败时调用 `remove_dir_all` 清理草稿目录。已确认。
- **WR-04**（Zero duration rejection）：`time.rs:20` 和 `time.rs:33-34` 中 `trange` 和 `trange_from_secs` 均拒绝 `duration <= 0`。已确认。
- **WR-05**（Shared build_base_segment helper）：`segment.rs:25-56` 中提取了共享的 `build_base_segment` 函数，`VideoSegment::to_json` 和 `AudioSegment::to_json` 均调用它。已确认。

## Critical Issues

None.

## Warnings

### WR-01: save 失败时草稿目录残留不完整文件

**File:** `src/jianying/builder.rs:265-267`
**Issue:** `export_draft` 在 `build_timeline` 失败时正确清理草稿目录（前一轮 WR-03 修复），但 `script_file.save(&folder)` 失败时（例如磁盘空间不足、权限不足）不会清理。此时 `draft_meta_info.json` 已写入，但 `draft_content.json` 可能缺失或截断，留下一个不完整的剪映草稿目录。用户下次打开剪映时可能导入失败或看到损坏的草稿。
**Fix:**

```rust
// builder.rs export_draft 函数，替换当前 match 块
match result {
    Ok(()) => {
        match script_file.save(&folder) {
            Ok(draft_path) => Ok(draft_path),
            Err(e) => {
                let _ = std::fs::remove_dir_all(folder.draft_dir());
                Err(e)
            }
        }
    }
    Err(e) => {
        let _ = std::fs::remove_dir_all(folder.draft_dir());
        Err(e)
    }
}
```

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

_Reviewed: 2026-04-30T02:39:20Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 8_
