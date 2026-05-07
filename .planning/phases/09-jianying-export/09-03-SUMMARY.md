---
phase: 09-jianying-export
plan: 03
subsystem: jianying
tags: [builder-api, draft-folder, script-file, export-draft, ffprobe-audio]
dependency_graph:
  requires: [VideoSegment, AudioSegment, TrackType, Track, DRAFT_CONTENT_TEMPLATE, DRAFT_META_INFO_TEMPLATE, trange_from_secs, JianYingError, Script, ScriptClip, OstType]
  provides: [ExportRequest, DraftFolder, ScriptFile, export_draft, probe_audio, parse_timestamp_start]
  affects: [src/ffmpeg/probe.rs, src/jianying/mod.rs]
tech_stack:
  added: []
  patterns: [builder pattern for draft folder hierarchy, template-based JSON generation with dynamic replacement, material collection during segment addition]
key_files:
  created:
    - narratoai-core/src/jianying/builder.rs
  modified:
    - narratoai-core/src/ffmpeg/probe.rs
    - narratoai-core/src/jianying/mod.rs
decisions:
  - ScriptFile 自行维护素材列表（video_materials, audio_materials, speed_materials），而非通过 Track.collect_materials()——因为 Track 只存储已序列化的 SegmentOutput JSON
  - add_video_segment/add_audio_segment 分离为独立方法（替代统一的 add_segment_to_track），提供更好的类型安全
  - parse_timestamp_start 内联实现时间戳解析（HH:MM:SS,mmm 格式），不复用 script/mod.rs 的私有函数
  - draft_content.json 通过模板替换方式生成（从 DRAFT_CONTENT_TEMPLATE 加载后修改字段），保持与模板的一致性
  - calculate_total_duration 扫描所有 segment 的 target_timerange 计算实际总时长
metrics:
  duration: 420s
  completed: 2026-04-29
  tasks: 1
  files: 3
  tests: 16
---

# Phase 09 Plan 03: Builder API + Export Summary

剪映导出核心逻辑层：probe_audio 音频时长探测、DraftFolder 草稿文件夹创建、ScriptFile 轨道/素材管理 + draft_content.json 生成、export_draft 导出主函数（OST 分支 + 视频来源回退 + 音频安全时长）。

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | FFmpeg probe_audio + DraftFolder/ScriptFile builder + export_draft | cd9da3e | src/ffmpeg/probe.rs, src/jianying/builder.rs, src/jianying/mod.rs |

## Key Decisions

1. **ScriptFile 自行维护素材列表** -- Track 只存储已序列化的 SegmentOutput JSON（VideoSegmentJson/AudioSegmentJson），无法反向提取素材。因此 ScriptFile 在 add_video_segment/add_audio_segment 时同时收集素材 JSON，保存时直接填充到 materials 字段。
2. **分离 add_video_segment / add_audio_segment** -- 替代计划中的统一 `add_segment_to_track(seg: impl Into<SegmentOutput>)`，独立方法签名更清晰，避免运行时类型匹配。
3. **内联时间戳解析** -- `parse_timestamp_start` 直接实现 HH:MM:SS,mmm-HH:MM:SS,mmm 格式解析，不复用 `script/mod.rs` 中的私有函数 `timestamp_to_millis`（后者返回毫秒整数，builder 需要浮点秒）。
4. **模板替换方式生成 draft_content.json** -- 从 DRAFT_CONTENT_TEMPLATE 加载 serde_json::Value，替换 canvas_config/id/duration/materials/tracks 字段。保持模板中所有固定字段（color_space, fps, platform 等）完整输出。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ScriptFile 素材收集策略变更**
- **Found during:** Task 1 实现
- **Issue:** Track.collect_materials() 方法不存在（Wave 2 的 Track 只存储 SegmentOutput JSON，没有反向提取素材的能力）
- **Fix:** ScriptFile 自行维护 video_materials/audio_materials/speed_materials 列表，在 add_video_segment/add_audio_segment 时同步收集
- **Files modified:** src/jianying/builder.rs
- **Commit:** cd9da3e

## Verification Results

```
cargo build: zero warnings, zero errors
cargo test --lib: 129 passed, 0 failed (16 new in plan 03)
grep checks: all 4 passed (pub fn export_draft, pub struct ExportRequest, pub fn probe_audio, pub mod builder)
```

## Test Coverage

| Module | Tests | Coverage |
|--------|-------|----------|
| builder.rs (ExportRequest) | 6 | default_resolution, validate_empty_script, validate_empty_path, validate_zero_resolution, validate_valid_request, draft_name_auto_generate, draft_name_preserved |
| builder.rs (DraftFolder) | 4 | creates_files, save_content (canvas_config/fps/version/materials), meta_info_has_draft_id |
| builder.rs (ScriptFile) | 3 | new_empty, add_tracks, save_parseable |
| builder.rs (parse_timestamp_start) | 3 | with_millis, minutes, invalid |
| probe.rs | 1 | probe_audio_nonexistent_file |

## Files Created/Modified

### Created
- `src/jianying/builder.rs` -- ExportRequest, DraftFolder, ScriptFile, export_draft, validate_export_request, parse_source_start_time, parse_timestamp_start（16 个测试）

### Modified
- `src/ffmpeg/probe.rs` -- 新增 probe_audio 函数（ffprobe 获取音频时长）+ 1 个测试
- `src/jianying/mod.rs` -- 新增 `pub mod builder;`

## Self-Check: PASSED

- [x] src/jianying/builder.rs -- FOUND
- [x] src/ffmpeg/probe.rs (probe_audio) -- FOUND
- [x] src/jianying/mod.rs (pub mod builder) -- FOUND
- [x] Commit cd9da3e -- FOUND
