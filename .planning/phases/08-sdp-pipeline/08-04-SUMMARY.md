---
phase: 08-sdp-pipeline
plan: 04
type: execute
tags: [sdp, pipeline, clip, concat, composite, ffmpeg]
key-files:
  created:
    - narratoai-core/src/sdp/clip.rs
    - narratoai-core/src/sdp/pipeline.rs
  modified:
    - narratoai-core/src/sdp/mod.rs
    - narratoai-core/src/sdp/error.rs
    - narratoai-core/src/lib.rs
metrics:
  tasks: 3
  tasks-completed: 3
  total-duration-minutes: ~45
  new-tests: 26
  total-tests-passed: 549
  commits: 4
  pre-existing-failures: 1 (ffmpeg::hwaccel::tests::test_detect_encoders_format)
---

# Phase 08 Plan 04: SDP 视频处理流水线 (clip + pipeline) 执行总结

## 一句话总结

实现 `run_sdp()` 4 步流水线编排：`step_clip` 复用 SDE 裁剪逻辑（全部 OST=1）→ `step_concat` 使用 FFmpeg concat demuxer → `step_composite` 使用 filter_complex + amix（原始音频 + 可选 BGM，无字幕烧录）。

## 完成情况

| Task | Name | Status | Commit |
|------|------|--------|--------|
| 0 | 创建 Plan 03 基础设施依赖（error.rs, types.rs, mod.rs） | 完成 | eb4995e |
| 1 | 创建 sdp_step_clip() 包装 | 完成 | 47acba7 |
| 2 | 创建 pipeline.rs — run_sdp() 主编排 + step_concat + step_composite | 完成 | 6dbfd6b |
| 3 | 更新 lib.rs + sdp/mod.rs — 注册 SDP 模块 | 完成 | f6e3580 |

## 实现详情

### Task 1: src/sdp/clip.rs
- `sdp_step_clip()` 遍历 OST=1 片段，使用 `crate::sde::timestamp::find_precise_range` 进行精确时间戳校正
- 调用 `crate::documentary::clip::clip_all_videos` 执行实际 FFmpeg 裁剪（传空 HashMap<i64, TtsResult>）
- 计算每个片段的 `duration`、`source_time_range`、`edited_time_range` 和累计总时长
- 包含 `secs_to_timestamp_str()` 辅助函数和 5 个单元测试

### Task 2: src/sdp/pipeline.rs
- `run_sdp()` 主编排器：参数校验 → 创建任务目录 → 初始化状态 → 加载脚本 → G4 OST 校验 → 3 步流水线
- `sdp_step_concat()`: FFmpeg concat demuxer（-f concat -safe 0 -c copy），路径转义防止注入
- `sdp_step_composite()`: FFmpeg filter_complex 实现原始音频音量控制 + 可选 BGM 混音（aloop + atrim + volume + afade + amix），无字幕烧录
- 进度回调：Clip(0%→30%) → Concat(30%→60%) → Composite(60%→100%)
- 包含 6 个单元测试

### Task 3: src/lib.rs + src/sdp/mod.rs
- lib.rs 添加 `pub mod sdp;`（按字母顺序放在 sde 之后）
- mod.rs 包含 clip、error、pipeline、types 四个子模块声明和 pub use 导出

## 关键设计决策

1. **Import 修正**: 计划中引用 `crate::subtitle::timestamp::find_precise_range` 但该模块不存在。实际函数在 `crate::sde::timestamp`，已修正导入路径。
2. **FFmpeg closure 变量提取**: 遵循 SDE 流水线模式，所有被 move closure 捕获的值在 closure 前预先提取为本地变量（f64 copy / String 所有权转移），避免捕获 &mut state 引用。
3. **From\<ScriptError>**: 为 `load_script()` 的 `?` 操作添加 `From<ScriptError>` 到 `SdpError` 的转换，按变体语义映射到正确的 SdpError 变体。

## 覆盖的安全威胁

| Threat ID | Category | Mitigation | Status |
|-----------|----------|------------|--------|
| T-08-006 | Tampering - concat | 路径转义（replace `\\` to `/`, escape `'`） | 已实现 |
| T-08-007 | DoS - composite | `total_duration.max(0.0)` 边界检查，aloop `size=2e+09` 限制 | 已实现 |
| T-08-008 | Tampering - OST | G4 guard 在校验后加载脚本时拒绝非 OST=1 片段 | 已实现 |

## 偏差记录

### Rule 3 - 修复依赖缺失（Plan 03）
- **问题**: 工作目录基于 `b81c753` 创建，不包含 Plan 03（SDP 基础设施）的输出文件
- **影响**: src/sdp/error.rs、src/sdp/types.rs、src/sdp/mod.rs 不存在，Plan 04 无法编译
- **处理**: 自动创建缺失的 Plan 03 基础设施文件（error.rs 含 6 个错误变体 + From 实现，types.rs 含 SdpRequest/SdpPipelineState/SdpProgressStep，mod.rs 含模块声明）
- **注意**: script_gen.rs 因依赖 subtitle 模块（通过 `pub mod subtitle;` 注册在 Plan 01 中，本工作目录未包含），无法编译。当前 mod.rs 不包含 script_gen 声明，未来执行 Plan 03 时需手动合并。

### Import 修正（计划与实际不匹配）
- 计划代码引用 `crate::subtitle::timestamp::find_precise_range`、`crate::subtitle::types::SubtitleSegment`、`crate::subtitle::timestamp::parse_srt_timestamp`，但这些类型/函数实际在 `crate::sde::*` 中定义，已在实现中修正。

### scope boundary 遵守
- 不修改 pre-existing 代码（sde/*, documentary/*, script/*, ffmpeg/* 等无改动）
- pre-existing 失败测试 `ffmpeg::hwaccel::tests::test_detect_encoders_format` 未触碰

## 验证结果

```
cargo build -p narratoai-core  →  OK（仅 pre-existing warnings）
cargo test -p narratoai-core   →  549 passed, 1 failed（pre-existing ffmpeg hwaccel 测试）
```

- SDP 相关模块测试全部通过（sdp::error: 7, sdp::types: 8, sdp::clip: 5, sdp::pipeline: 6 = 26 个测试）

## 已知限制

1. **script_gen.rs 缺失**: Plan 03 的脚本生成模块未包含（因 subtitle 模块依赖不存在），需在后续执行 Plan 03 时补充。
2. **FFmpeg hardware acceleration 检测失败**: pre-existing 问题（`test_detect_encoders_format`），与 Plan 04 无关。
