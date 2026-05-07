---
phase: 04-prompt-system-visual-analyzer
plan: 02
subsystem: visual
tags: [frame-extraction, ffmpeg, serde, thiserror, visual-analyzer]
type: execute
requires: []
provides: [VisualError, FrameObservation, BatchAnalysisResult, extract_frames]
affects: [src/visual/]
tech-stack:
  added: []
  patterns: [spawn_blocking + ffmpeg-sidecar, thiserror error enum, serde deny_unknown_fields, 4-level output format fallback, progress callback reuse]
key-files:
  created:
    - narratoai-core/src/visual/mod.rs
    - narratoai-core/src/visual/error.rs
    - narratoai-core/src/visual/types.rs
    - narratoai-core/src/visual/frame_extractor.rs
  modified:
    - narratoai-core/src/lib.rs
decisions:
  - "VisualError 使用模块级错误枚举而非统一 error.rs，与 D-21/D-22 一致"
  - "strip_code_fence 放在 types.rs 中作为 pub(crate) 工具函数，在 Plan 04-04 analyzer.rs 中消费"
  - "extract_single_frame 使用 std::process::Command 而非 ffmpeg-sidecar，因为 per-frame 调用不需要事件迭代"
  - "seconds_to_hhmmssmmm 格式为 9 位字符串 (HHMMSSmmm)，使用 format! 宏拼接"
metrics:
  duration: "~25 min"
  completed_date: "2026-04-30"
  task_count: 3
  file_count: 5 (4 created + 1 modified)
  test_count: 22
  test_pass: 22
---

# Phase 4 Plan 2: Visual Analyzer Foundation — Frame Extraction + Data Types

## One-Liner

构建视觉分析器的核心基础设施：VisualError/FrameObservation/BatchAnalysisResult 数据模型，以及带快路径 + 4 级回退的 FFmpeg 帧提取模块。

## Tasks Completed

### Task 1: VisualError 和视觉分析数据类型

**创建了 `src/visual/error.rs` 和 `src/visual/types.rs`：**

- `VisualError` 枚举：`FrameExtraction(String)`、`Analysis(String)`、`BatchPartial { analyzed_count, total_count, errors }`，全部使用中文 `#[error("...")]` 消息
- `FrameObservation` 结构体：7 个字段（frame_number/timestamp/scene_description/objects/actions/on_screen_text/visual_salience），使用 `#[serde(deny_unknown_fields)]`
- `BatchAnalysisResult` 结构体：5 个字段（observations/overall_activity_summary/total_frames/analyzed_batches/errors）
- `strip_code_fence` 工具函数：剥离 ````json`/```` 代码块包裹，供后续 Plan 04-04 analyzer.rs 消费
- 13 个测试（3 error + 10 types）：覆盖完整 JSON/必需字段/代码块剥离/缺失字段/camelCase 拒绝/空列表/null 字段/round-trip/超范围值
- 更新 `src/lib.rs` 添加 `pub mod visual;`

**Commit:** `6995b9c`

### Task 2: 帧提取快路径

**实现了 `src/visual/frame_extractor.rs` 中的快路径：**

- `extract_frames_fast_path()`: 异步函数，使用 `spawn_blocking` 包装 ffmpeg-sidecar
- 命令构建：`-vf fps=1/{interval}` + `-codec:v mjpeg` + `-q:v {quality}` + `-start_number 0`
- `CancellationToken` 支持取消（循环中检查 `cancel.is_cancelled()`）
- `rename_fast_path_frames()`: 将 `fastframe_XXXXXX.jpg` 重命名为 `keyframe_XXXXXX_HHMMSSmmm.jpg`，帧序号 + 时间戳计算
- `seconds_to_hhmmssmmm()`: 秒数到 9 位 HHMMSSmmm 格式转换
- 4 个单元测试：重命名操作/命名格式/空目录/时间戳格式

**Commit:** `66e74e2`

### Task 3: 帧提取回退路径 + 4 级回退

**在 `frame_extractor.rs` 中追加回退路径：**

- `extract_frames()`: 主入口，创建输出目录 → 尝试快路径 → 0 帧时清理并回退
- `extract_frames_fallback()`: 通过 ffprobe 获取视频时长，逐帧提取，上报进度，收集错误继续执行（D-14）
- `extract_single_frame()`: 4 级回退：
  - L1: 硬件加速 JPEG（`-hwaccel auto`）
  - L2: 软件解码 JPEG（`-avoid_negative_ts make_zero`）
  - L3: PNG 输出 → image crate 转 JPEG（quality 85）
  - L4: BMP 输出 → image crate 转 JPEG
- `cleanup_fast_path_files()`: 清理快路径残留文件
- `get_video_duration()`: ffprobe 包装器
- `convert_image_to_jpeg()`: 使用 `image::codecs::jpeg::JpegEncoder::new_with_quality`
- 7 个测试：帧序号解析/输出目录创建/快照清理/文件验证/空目录/持续时间获取/命名格式

**Commit:** `7687fe0`

## Deviations from Plan

### Rule 1 — Auto-fixed Bugs

1. **ffmpeg-sidecar API 不匹配**
   - **Found during:** Task 2
   - **Issue:** `loglevel()`, `video_filter()`, `start_number()` 等方法在 ffmpeg-sidecar 2.5.1 上不存在
   - **Fix:** 替换为 `.arg("-loglevel").arg("error")`、`.arg("-vf").arg(...)`、`.arg("-start_number").arg("0")` 等等效调用
   - **Files modified:** `src/visual/frame_extractor.rs`
   - **Commit:** `66e74e2` (iteration during Task 2)

2. **Rust format 宏不支持 `d` 后缀**
   - **Found during:** Task 2
   - **Issue:** `{:06d}` 和 `{:02d}`/`{:03d}` 在 Rust 中无效（仅 Python/C 支持 `d` 后缀）
   - **Fix:** 替换为 `{:06}`、`{:02}`、`{:03}`
   - **Files modified:** `src/visual/frame_extractor.rs`
   - **Commit:** `66e74e2` (iteration during Task 2)

3. **`read_dir` 迭代模式错误**
   - **Found during:** Task 3
   - **Issue:** `Ok(Some(entry))` 匹配顺序错误，应为 `Some(Ok(entry))`
   - **Fix:** 修复 `cleanup_fast_path_files` 中的模式匹配
   - **Files modified:** `src/visual/frame_extractor.rs`
   - **Commit:** `7687fe0` (iteration during Task 3)

### Rule 3 — Auto-fixed Blocking Issues

无 — 所有依赖（ffmpeg-sidecar、tokio-util、image、serde）已在 Cargo.toml 中。

### Rule 4 — Architectural Changes

无 — 全部按计划执行。

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| src/visual/ 包含 mod.rs, error.rs, types.rs, frame_extractor.rs 四个文件 | PASS |
| VisualError 枚举有 FrameExtraction/Analysis/BatchPartial 三个变体，中文消息 | PASS |
| FrameObservation 和 BatchAnalysisResult 可通过 serde_json 反序列化 | PASS |
| extract_frames() 实现快路径 + 回退路径 + 4 级回退 | PASS |
| 帧命名遵守 D-24: `keyframe_{frame_number:06}_{HHMMSSmmm}.jpg` | PASS |
| 所有 FFmpeg 操作使用 spawn_blocking | PASS |
| `cargo build` 编译成功 | PASS |

## File Manifest

### Created

| File | Lines | Purpose |
|------|-------|---------|
| `src/visual/mod.rs` | 3 | 模块声明（pub mod error/types/frame_extractor） |
| `src/visual/error.rs` | 55 | VisualError 枚举 + 3 个中文消息测试 |
| `src/visual/types.rs` | 222 | FrameObservation + BatchAnalysisResult + strip_code_fence + 10 测试 |
| `src/visual/frame_extractor.rs` | 536 | 帧提取全功能：快路径 + 回退路径 + 4 级回退 + 9 测试 |

### Modified

| File | Lines | Change |
|------|-------|--------|
| `src/lib.rs` | +1 | 添加 `pub mod visual;` |

## Test Results

```
22 passed; 0 failed; 0 ignored
- visual::error::tests: 3
- visual::types::tests: 10
- visual::frame_extractor::tests: 9
```

## Known Stubs

无。`#[allow(dead_code)]` 标记的 `strip_code_fence` 和 `parse_frame_number_from_name` 是供后续 Plan 04-04 使用的跨模块工具函数，属于前向声明而非桩代码。

## Self-Check: PASSED

- [x] All 4 files exist: `src/visual/mod.rs`, `src/visual/error.rs`, `src/visual/types.rs`, `src/visual/frame_extractor.rs`
- [x] All 3 commits exist: `6995b9c`, `66e74e2`, `7687fe0`
- [x] `cargo build -p narratoai-core` succeeds
- [x] `cargo test -p narratoai-core visual::` 22/22 pass
- [x] No unexpected file deletions
