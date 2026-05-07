---
phase: 11-extended-features
plan: 02
type: execute
wave: 2
subsystem: audio
tags: [audio, normalization, loudnorm, volume, tdd]
requires: [01]
provides: [audio::normalizer, audio::volume]
affects: [Pipeline phase 6]
tech-stack:
  added: [symphonia, ffmpeg-sidecar]
  patterns: [thiserror + Chinese Display, subprocess + JSON parse, symphonia decode loop, pure data struct]
key-files:
  created:
    - narratoai-core/src/audio/normalizer.rs (447 lines)
    - narratoai-core/src/audio/volume.rs (319 lines)
  modified: []
decisions:
  - "AudioError 共 6 个变体，所有 Display 为中文（LoudnormAnalysisFailed / LoudnormNormalizeFailed / RmsFallbackFailed / FileNotFound / InvalidVolume / IoError）"
  - "LoudnormData 包含 target_offset 字段（Pitfall 1 防护），通过 ffmpeg_sidecar::paths::ffmpeg_path() 获取二进制路径"
  - "RMS 回退使用 symphonia 0.5 解码 PCM f32 → RMS dB"
  - "音量 profile 值硬编码在 match 语句中，不从配置加载，1:1 对齐 Python AudioConfig"
  - "ProcessingConfig / MixingConfig 定义完毕，MixingConfig 的 FFmpeg 实现在 Phase 6"
metrics:
  duration: null
  completed_date: 2026-04-30
  tasks: 2
  tests: 19
  commits: 2
---

# Phase 11-02: Audio Normalization and Volume Control Summary

## One-Liner

音频响度标准化（两遍 loudnorm + symphonia RMS 回退）和智能音量控制（4 种 profile / 4 种内容类型推荐），1:1 对齐 Python 版 `audio_normalizer.py` 和 `audio_config.py`，共 19 个单元测试全部通过。

## Tasks

### Task 1: Audio normalizer (normalizer.rs) — TDD

**RED phase:** 6 个测试（2 类型/解析测试通过 + 4 功能测试因 todo! 崩溃）
**GREEN phase:** 全部 6 个测试通过

| Item | Status |
|------|--------|
| AudioError enum | 6 变体，中文 Display |
| LoudnormData struct | serde 反序列化 + measured_I/LRA/TP/thresh + offset 辅助方法 |
| `analyze_lufs()` | FFmpeg loudnorm 第一遍分析，stderr JSON 提取 |
| `normalize_lufs()` | 两遍 loudnorm（分析 → measured 值 → 第二遍应用） |
| `get_audio_rms()` | symphonia PCM f32 解码 → RMS dB |
| `calculate_volume_adjustment()` | 纯数学增益系数，[0.1, 2.0]/[0.1, 3.0] clamp |
| `normalize_audio_for_mixing()` | loudnorm 优先 → RMS 回退编排入口 |

### Task 2: Audio volume (volume.rs) — TDD

**RED phase:** 13 个测试全部因 todo! 崩溃
**GREEN phase:** 全部 13 个测试通过

| Item | Status |
|------|--------|
| VolumeConfig struct | `{ tts_volume, original_volume, bgm_volume }` + validate() |
| ProcessingConfig | enable_smart_volume, enable_audio_normalization, target_lufs, max_peak |
| MixingConfig | crossfade_duration, bgm_fade_out, dynamic_range_compression |
| `get_optimized_volumes()` | 4 种视频类型（educational/entertainment/news/default） |
| `apply_volume_profile()` | 4 种 profile（balanced/voice_focused/original_focused/quiet_background） |
| `get_recommended_volumes_for_content()` | 4 种内容类型（mixed/voice_only/original_heavy/music_video） |
| `validate_volume()` | [0.0, 2.0] clamp + tracing::warn! |

## Deviations from Plan

None — plan executed exactly as written. All profile values 1:1 match Python AudioConfig.

### Honored Constraints

- All public function signatures use `&Path` (not `&str`)
- All FFmpeg subprocesses use `Command::args()` with arrays (no `shell(true)`, T-11-01 mitigation)
- No async wrappers on synchronous FFmpeg/symphonia calls (caller does spawn_blocking)
- Volume profiles hardcoded as literal constants in match arms

## Known Stubs

None — both modules complete with full implementations and tests.

## Threat Flags

None — all FFmpeg subprocesses use args arrays per T-11-01 mitigation. symphonia decode is pure Rust with no unsafe code (T-11-02 accepted per plan). Large file DoS risk documented and accepted (T-11-03).

## Verification

```text
cargo test --lib audio::normalizer::tests ... ok (6 passed)
cargo test --lib audio::volume::tests ... ok (13 passed)
cargo test --lib audio:: ... ok (19 passed)
cargo check ... 0 errors (1 pre-existing warning)
```

### Acceptance Criteria Verification

| Criterion | Expected | Actual |
|-----------|----------|--------|
| pub enum AudioError | 1 | 1 |
| pub fn analyze_lufs | 1 | 1 |
| pub fn normalize_lufs | 1 | 1 |
| pub fn get_audio_rms | 1 | 1 |
| pub fn calculate_volume_adjustment | 1 | 1 |
| pub fn normalize_audio_for_mixing | 1 | 1 |
| symphonia::default::get_probe | 1 | 1 |
| target_offset in code | >= 1 | 5 |
| LoudnormData | >= 1 | 5 |
| pub struct VolumeConfig | 1 | 1 |
| pub fn get_optimized_volumes | 1 | 1 |
| pub fn apply_volume_profile | 1 | 1 |
| pub fn get_recommended_volumes_for_content | 1 | 1 |
| pub fn validate_volume | 1 | 1 |
| pub struct ProcessingConfig | 1 | 1 |
| pub struct MixingConfig | 1 | 1 |
| 4 profile names present | 4 | 4 |

## Commits

```
b8cd8d7 feat(11-extended-features-02): implement AudioError, LoudnormData, and 5 normalization functions
2cb25c2 feat(11-extended-features-02): implement VolumeConfig, ProcessingConfig, MixingConfig, and 4 volume functions
```

## Self-Check: PASSED

All created files exist and are verified:
- `src/audio/normalizer.rs` — 447 lines, 6 unit tests, all data structures and 5 functions
- `src/audio/volume.rs` — 319 lines, 13 unit tests, all 4 functions and 3 config structs
- All acceptance criteria matched
- All 19 unit tests pass
- `cargo check` zero errors

## Next Steps

Plan 11-02 is complete. The audio module (normalizer + volume) is ready for:
- Phase 6 documentary pipeline to consume `normalize_audio_for_mixing()` for audio preprocessing
- Phase 6 to use `ProcessingConfig` / `MixingConfig` for audio mixing with crossfade
- Tauri command layer (Phase 10) to expose volume profiles via API
