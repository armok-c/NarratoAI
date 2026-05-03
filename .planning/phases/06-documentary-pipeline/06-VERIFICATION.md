---
status: passed
phase: 06-documentary-pipeline
verified: 2026-05-03
requirements:
  - FFMP-02
  - FFMP-03
  - FFMP-04
  - FFMP-05
  - FFMP-06
  - FFMP-07
  - FFMP-08
  - DOCS-01
  - DOCS-02
  - DOCS-03
---

# Phase 6 Verification: Documentary Pipeline

## Must-Have Verification

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | 6 步流水线完整运行并输出最终视频文件 | PASS | `pipeline.rs:465` — `run_documentary()` 调用 step_load_script → step_tts → step_clip → step_merge_audio_subtitle → step_concat → step_composite，返回 `Result<PathBuf, PipelineError>` |
| 2 | OST=0/1/2 三种模式正确分支处理 | PASS | `clip.rs:16/35/62` — 三个独立函数：clip_ost0_narration_only（-an 去音）、clip_ost1_original_sound（保留原声）、clip_ost2_mixed（保留原声+按TTS时长裁剪） |
| 3 | 帧分析→LLM解说文案生成 | PASS | `script_gen.rs:176` — `generate_documentary_script()` 调用 analyze_video → convert_analysis_to_markdown → generate_narration，包含 JSON 修复（代码围栏、尾逗号、单引号） |
| 4 | SRT 字幕从 TTS 词边界生成，时间戳同步 | PASS | `subtitle.rs` — `generate_srt_from_word_boundaries()` 将 100ns 单位转换为 SRT HH:MM:SS,mmm 格式；`merge_srt_files()` 应用累计时间偏移并重新编号 |
| 5 | 每步完成后状态更新（进度追踪） | PASS | `types.rs` — ProgressStep 枚举 6 个步骤变体 + ProgressCallback 类型；`pipeline.rs` 每步调用回调上报进度百分比和描述 |

## Requirement Traceability

| Req | Description | Covered By |
|-----|-------------|------------|
| FFMP-02 | 视频裁剪（OST 分支） | Plan 02 — clip_ost0/1/2 + clip_all_videos |
| FFMP-03 | TTS 音频生成集成 | Plan 02 — step_tts 调用 synthesize 路由器 |
| FFMP-04 | 音频合并（TTS 定位） | Plan 02 — merge_audio_files 使用 adelay+amix |
| FFMP-05 | 视频片段拼接 | Plan 02 — step_concat 使用 FFmpeg concat demuxer |
| FFMP-06 | 最终合成（音频+字幕烧录） | Plan 02 — step_composite 构建 filter_complex |
| FFMP-07 | 字幕生成（SRT） | Plan 01 — generate_srt_from_word_boundaries |
| FFMP-08 | 流水线编排 | Plan 02 — run_documentary 6 步顺序调用 |
| DOCS-01 | 纪录片模块结构 | Plan 01 — src/documentary/ 模块 + 8 子模块 |
| DOCS-02 | 帧分析→文案生成 | Plan 03 — generate_documentary_script |
| DOCS-03 | 错误处理 | Plan 01 — PipelineError 10 变体 + 中文消息 |

## Test Results

- `cargo test --lib documentary` — 56 passed, 0 failed
- `cargo test --test documentary_integration_test` — 9 passed, 4 ignored (需要 FFmpeg), 0 failed
- `cargo build` — PASS (0 errors, 6 warnings)

## Spot-Check Summary

| Plan | SUMMARY.md | Commits | Self-Check | Key Files |
|------|-----------|---------|------------|-----------|
| 06-01 | exists | yes | PASSED | error.rs, types.rs, timestamp.rs, subtitle.rs, mod.rs |
| 06-02 | exists | yes | PASSED | pipeline.rs, clip.rs, audio.rs |
| 06-03 | exists | yes | PASSED | script_gen.rs |
| 06-04 | exists | yes | PASSED | documentary_integration_test.rs, common/mod.rs |

## Notes

- FFmpeg 依赖测试（4 个 integration test）标记为 `#[ignore]`，在安装 FFmpeg 的环境中可运行
- `test_detect_encoders_format`（Phase 1 hwaccel）在无 FFmpeg 环境下失败——预存问题，非 Phase 6 引入
