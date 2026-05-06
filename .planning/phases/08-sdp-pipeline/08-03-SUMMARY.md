---
phase: 08-sdp-pipeline
plan: 03
type: execute
wave: 2
subsystem: sdp
tags: [sdp-module, script-generation, llm-orchestration]
requires:
  - Plan 08-01 (subtitle module)
  - Plan 08-02 (prompt templates)
affects:
  - Plan 08-04 (clip/pipeline — wave 3)
tech-stack:
  added: []
  patterns:
    - "两步 LLM 编排（subtitle_analysis → plot_extraction），使用 PromptManager::render_prompt"
    - "复用 crate::sde::script_gen::repair_json 做 JSON 修复"
    - "spawn_blocking 包装 CPU 密集字幕解析"
    - "uuid::Uuid::new_v4() + hex::encode 生成 narration 占位值（替代 rand）"
key-files:
  created:
    - src/sdp/mod.rs
    - src/sdp/error.rs
    - src/sdp/types.rs
    - src/sdp/script_gen.rs
  modified:
    - src/lib.rs
decisions:
  - "narration placeholder 使用 uuid::Uuid::new_v4() 替代 rand crate（减少依赖）"
  - "SdpProgressStep 精简为 3 步（Clip/Concat/Composite），区别于 SDE 的 9 步"
  - "生成函数使用 &dyn LlmProvider trait object，保持与 SDE 一致的依赖注入模式"
metrics:
  start_time: "2026-05-06T03:00:00Z"
  end_time: "2026-05-06T04:00:00Z"
  duration_minutes: 60
  tasks_total: 2
  tasks_completed: 2
---

# Phase 08 Plan 03: SDP 模块基础设施和脚本生成

创建 src/sdp/ 下的 error.rs, types.rs, script_gen.rs 模块，实现 SDP 脚本生成功能。核心函数 `generate_sdp_script()` 编排两步 LLM 调用 (subtitle_analysis → plot_extraction)，`merge_script()` 将 plot_points 合并为 OST=1 的 ScriptClip 数组。

## Completed Tasks

| # | Name | Type | Commit | Key Files |
|---|---|---|---|---|
| 1 | 创建 SDP 基础设施 (error.rs + types.rs + mod.rs) | auto | 8b38fc8 | src/sdp/error.rs, src/sdp/types.rs, src/sdp/mod.rs, src/sdp/script_gen.rs (stub), src/lib.rs |
| 2 | 创建 script_gen.rs — SDP 脚本生成核心 | auto | 2445cca | src/sdp/script_gen.rs, src/sdp/mod.rs (pub use) |

### Task 1 Details

**SdpError 枚举** (6 个变体):
- `ParseSubtitle` — 字幕解析失败，含 From<SubtitleError> 实现
- `LlmError` — LLM 调用失败
- `JsonRepair` — JSON 修复失败
- `VideoProcess` — 视频处理失败 (from PipelineError)
- `Validation` — 参数校验失败
- `Io` — IO 操作失败 (from std::io::Error)

**SdpRequest** — 10 个字段，含 validation：
- subtitle_path, video_path, script_path, temperature (0.1), custom_clips (5)
- original_volume (0.7), bgm_volume (0.3), bgm_path, output_dir, threads (4)
- validate() 校验路径非空、音量范围 [0,10]、threads > 0

**SdpPipelineState** — 精简字段设计（无 TTS/字幕字段）：
- task_id, task_dir, subtitle_text, subtitle_segments, script
- video_clips, combined_video_path, output_video_path, total_duration, progress

**SdpProgressStep** — 3 步枚举 (Clip=0, Concat=1, Composite=2)

**12 个单元测试**覆盖所有错误 Display 中文消息、请求校验、默认值、进度步骤顺序、状态初始化。

### Task 2 Details

**generate_sdp_script()** — 主编排函数（异步，11 步）：
1. `spawn_blocking` 中调用 `parse_subtitle_file()` 解析字幕
2. 提取纯对话文本 (segments.text join)
3. 校验非空
4. 保存 subtitle_source.txt
5. 渲染 subtitle_analysis prompt，调用 LLM (system: 资深编剧，temp=0.1)
6. `crate::sde::script_gen::repair_json` 修复 + `serde_json::from_str` 解析
7. 保存 subtitle_analysis.json
8. 格式化 plot_titles 为编号列表
9. 渲染 plot_extraction prompt，调用 LLM (system: 专业剪辑师)
10. 修复/解析 PlotPointsData，保存 plot_extraction_raw.json
11. merge_script 合并为 Script

**merge_script()** — PlotPoints → ScriptClip 合并：
- 每个 PlotPoint → ScriptClip (_id=sequence, OST=1, narration 占位值)
- 通过 `crate::script::validate` 校验
- 保存为 task_dir/merged_subtitle.json
- 空输入返回 Validation 错误

**Helper functions**:
- `format_plot_titles()` — 编号列表格式
- `generate_narration_placeholder()` — uuid 生成唯一占位值

**11 个单元测试**覆盖占位值格式、列表格式化、合并逻辑、OST 类型、文件保存、字段保留、空输入边界。

## Deviations from Plan

**1. [Rule 1 - Bug Fix] uuid 替代 rand 生成 narration 占位值**
- **Found during:** Task 2 implementation
- **Issue:** Cargo.toml 没有 `rand` 依赖，但 plan 使用 `rand::Rng` 生成随机字节
- **Fix:** 使用 `uuid::Uuid::new_v4()` 取前 4 字节 + `hex::encode`，行为完全对齐 Python `os.urandom(4).hex()`，且不增加新依赖（uuid 已是依赖）

## TDD Gate Compliance

Not applicable — this plan is type: execute (not type: tdd).

## Verification Results

- `cargo build -p narratoai-core` — PASS
- `cargo test -p narratoai-core -- sdp` — 22/22 PASS
- `cargo test -p narratoai-core -- subtitle` — still PASS (no regression)
- `cargo test -p narratoai-core -- sde` — still PASS (no regression)
- Pre-existing failure `ffmpeg::hwaccel::tests::test_detect_encoders_format` — unchanged

## Threat Surface Scan

No new threat surface introduced beyond what is documented in Plan 08-03 threat_model:
- All subtitle input parsing goes through existing `parse_subtitle_file()` (already threat-modeled)
- LLM JSON output is run through `repair_json` before deserialization (T-08-004 mitigation)
- Script output saved to user-specified local path (T-08-005 accepted)

## Self-Check: PASSED

| Check | Status |
|-------|--------|
| src/sdp/mod.rs exists with pub mod error, types, script_gen | PASS |
| src/sdp/error.rs has pub enum SdpError (6 variants) | PASS |
| src/sdp/error.rs has From<SubtitleError> impl | PASS |
| src/sdp/types.rs has SdpRequest, SdpPipelineState, SdpProgressStep | PASS |
| SdpPipelineState has no TTS/subtitle fields | PASS |
| src/sdp/script_gen.rs has pub async fn generate_sdp_script( | PASS |
| src/sdp/script_gen.rs has fn merge_script( | PASS |
| generate_narration_placeholder returns "播放原生_" prefix | PASS |
| merge_script output all OST=1 | PASS |
| merge_script saves to task_dir/merged_subtitle.json | PASS |
| cargo build passes | PASS |
| All SDP unit tests pass (22/22) | PASS |
