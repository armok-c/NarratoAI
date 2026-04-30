---
phase: 09-jianying-export
plan: 02
subsystem: jianying
tags: [builder-pattern, material, segment, track, tdd]
dependency_graph:
  requires: [jianying-module, JianYingError, Timerange, trange, SegmentJson, VideoSegmentJson, AudioSegmentJson, VideoMaterialJson, AudioMaterialJson, SpeedJson, TrackJson]
  provides: [VideoMaterial, AudioMaterial, Speed, VideoSegment, AudioSegment, SegmentOutput, TrackType, Track]
  affects: [src/jianying/mod.rs]
tech_stack:
  added: []
  patterns: [builder pattern, UUID hex ID generation, serde JSON serialization, canonicalize path fallback]
key_files:
  created:
    - src/jianying/material.rs
    - src/jianying/segment.rs
    - src/jianying/track.rs
  modified:
    - src/jianying/mod.rs
decisions:
  - VideoSegment 内部持有 VideoMaterial + Speed 所有权，构造时生成 UUID
  - SegmentOutput enum 让 Track 统一存储不同类型的 segment JSON
  - canonicalize 失败时 fallback 到原始路径（文件可能尚未就绪）
  - VideoSegment 不存储 width/height 字段（已传给 VideoMaterial，不需要重复存储）
metrics:
  duration: 475s
  completed: 2026-04-29
  tasks: 2
  files: 4
  tests: 18
---

# Phase 09 Plan 02: Core Building Blocks Summary

剪映 JSON 核心构建块：VideoMaterial/AudioMaterial/Speed 素材构建器、VideoSegment/AudioSegment 片段构建器、TrackType 枚举 + Track 轨道构建器。所有组件使用 builder 模式构造，to_json() 方法生成对应的 serde JSON 结构体。

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Material 构建器（VideoMaterial, AudioMaterial, Speed） | e373b3d | src/jianying/material.rs, src/jianying/mod.rs |
| 2 | Segment + Track 构建器 | 06d3317 | src/jianying/segment.rs, src/jianying/track.rs, src/jianying/mod.rs |
| refactor | 修复 segment.rs 未使用导入警告 | 56ecc8a | src/jianying/segment.rs |

## Key Decisions

1. **VideoSegment 内部持有 VideoMaterial + Speed 所有权** -- 构造时一次性生成 UUID 和素材对象，material_id/speed_id 通过 getter 暴露。后续 builder 层调用 collect_materials() 收集所有素材。
2. **SegmentOutput enum 统一 segment 存储** -- Track 需要存储不同类型的 segment（Video/Audio），enum 提供类型安全的统一容器，to_json() 时序列化为 serde_json::Value。
3. **canonicalize fallback 到原始路径** -- 文件可能在导出时尚未完全就绪（如 TTS 生成的音频），剪映打开时才需要文件存在，因此不强制文件存在。
4. **VideoSegment 不重复存储 width/height** -- width/height 仅传给 VideoMaterial::new()，VideoSegment 本身不需要这些字段。

## Deviations from Plan

None -- 计划按原文执行。

## Verification Results

```
cargo build: zero warnings, zero errors
cargo test --lib jianying: 41 passed, 0 failed (18 new in plan 02)
grep checks: all 6 passed (pub struct VideoMaterial, pub struct VideoSegment, pub enum TrackType, pub mod material/segment/track)
```

## Test Coverage

| Module | Tests | Coverage |
|--------|-------|----------|
| material.rs | 7 | VideoMaterial JSON/id/crop/fixed_fields, AudioMaterial JSON/id/fixed_fields, Speed JSON, canonicalize fallback |
| segment.rs | 7 | VideoSegment target_timerange/clip/hdr/source_timerange, AudioSegment null_fields/volume_speed, extra_material_refs |
| track.rs | 4 | TrackType as_str, Track new/empty, Track add_segments, Track type_matches_json |

## Files Created/Modified

### Created
- `src/jianying/material.rs` -- VideoMaterial, AudioMaterial, Speed 构建器（7 个测试）
- `src/jianying/segment.rs` -- VideoSegment, AudioSegment, SegmentOutput（7 个测试）
- `src/jianying/track.rs` -- TrackType 枚举, Track 构建器（4 个测试）

### Modified
- `src/jianying/mod.rs` -- 新增 pub mod material, pub mod segment, pub mod track

## Self-Check: PASSED

- [x] src/jianying/material.rs -- FOUND
- [x] src/jianying/segment.rs -- FOUND
- [x] src/jianying/track.rs -- FOUND
- [x] .planning/phases/09-jianying-export/09-02-SUMMARY.md -- FOUND
- [x] Commit e373b3d -- FOUND
- [x] Commit 06d3317 -- FOUND
- [x] Commit 56ecc8a -- FOUND
