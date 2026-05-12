---
phase: 13-gap-closure
plan: 04
status: complete
completed: 2026-05-12
---

# SUMMARY: Plan 13-04 — ProcessingConfig 更新 + 纪录片流水线集成

## Changes

### Task 1: volume.rs 更新
- ProcessingConfig::default() target_lufs: -20.0 → -14.0
- get_optimized_volumes() 新增 "documentary"/"sde"/"sdp" 三个分支
- 3 个新测试 + 断言更新
- Commit: `318e952`

### Task 2: pipeline.rs 集成
- step_merge_audio_subtitle: config 参数 + normalize_merged_audio 调用
- step_composite: config 参数 + resolve_volumes 调用
- run_documentary: 传递 config
- Commit: `75d353a`

## Self-Check: PASSED

- `cargo test -p narratoai-core`: 573 passed, 0 failed
- `cargo check` (workspace): clean
- `audio::volume::tests`: 5 new assertions all pass
- `audio::pipeline::tests`: 4 tests pass (documentary assert updated to 1.0)
- `documentary::pipeline::tests`: 3 existing tests still pass

## Key Files

- `narratoai-core/src/audio/volume.rs` — ProcessingConfig default + new branches
- `narratoai-core/src/documentary/pipeline.rs` — Normalization in step 4 + smart volume in step 6
