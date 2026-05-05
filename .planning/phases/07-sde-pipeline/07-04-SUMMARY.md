---
phase: 07-sde-pipeline
plan: 04
type: execute
wave: 3
tags:
  - sde
  - pipeline
  - orchestrator
  - run_sde
requires:
  - 07-03-PLAN.md (SDE script_gen, subtitle, timestamp modules)
affects:
  - src/documentary/pipeline.rs (visibility)
tech-stack:
  added:
    - FFmpeg concat demuxer (SDE replica)
    - Filter complex composite (SDE replica)
  patterns:
    - "pub(crate)" module visibility for cross-pipeline reuse
key-files:
  created:
    - src/sde/pipeline.rs (SDE 9-step orchestrator, ~914 lines)
    - src/sde/clip.rs (OST=1 timestamp correction + 7 tests)
    - src/sde/audio.rs (thin marker module)
  modified:
    - src/documentary/pipeline.rs (5 step functions to pub(crate))
decisions:
  - "SDE TTS step does NOT reuse doc_pipeline::step_tts due to PipelineState type mismatch. Instead directly calls crate::tts::synthesize()"
  - "SDE Clip step reuses documentary clip_all_videos after OST=1 timestamp correction"
  - "SDE Concat/Composite steps replicate FFmpeg logic from documentary (same filter_complex pattern)"
metrics:
  duration: "~15 min"
  completed_date: "2026-05-05"
---

# Phase 07 Plan 04: SDE Pipeline Orchestrator (run_sde) Summary

**One-liner:** Complete SDE pipeline orchestrator run_sde() with 9-step flow (ParseSubtitle -> PlotAnalysis -> ScriptGenerate -> LoadScript -> Tts -> Clip -> MergeAudio -> Concat -> Composite), 5 guardrails (G1-G5), D-23 "play original" TTS skip, and D-29 precise OST=1 timestamp correction, reusing documentary video processing primitives.

## Tasks Executed

### Task 1: Make 5 documentary pipeline step functions pub(crate)
**Commit:** 1e2bcd5
**Files:** src/documentary/pipeline.rs
**Action:** Added pub(crate) visibility to step_tts, step_clip, step_merge_audio_subtitle, step_concat, step_composite. step_load_script remains private.

### Task 2: Create clip.rs and audio.rs wrapper modules
**Commit:** b26c066
**Files:** src/sde/clip.rs (created), src/sde/audio.rs (created)
**Action:** clip.rs exports sde_step_clip() with OST=1 precise timestamp correction (D-29) via find_precise_range(), then delegates to documentary clip_all_videos(). audio.rs is a thin marker module.

### Task 3: Create pipeline.rs main orchestrator
**Commit:** dc25de7
**Files:** src/sde/pipeline.rs (created, ~914 lines)
**Action:** Implemented run_sde() with full 9-step orchestration, helper functions (detect_and_abort_overlaps, warn_ost_ratio, build_doc_request), and independent public APIs (analyze_subtitle_plot, generate_sde_script per D-07). Includes 8 tests.

## Deviations from Plan

None. Plan executed exactly as written. Minor: `>=` overlap check changed to `>` to allow adjacent (non-overlapping) clips.

## Verification Results

| Check | Result |
|-------|--------|
| cargo check --lib | Passed |
| cargo test --lib sde::pipeline | 8/8 passed |
| cargo test --lib sde::clip | 7/7 passed |
| cargo test --lib sde::audio | 1/1 passed |
| cargo test --lib | 520/521 passed (1 pre-existing ffmpeg::hwaccel failure) |
| run_sde() signature | (SdeRequest, &AppConfig, Option<&ProxySection>, &Registry, &PromptManager) -> Result<PathBuf, SdeError> |
| 5 pub(crate) step functions | Verified |
| G1 (JSON Parse Guard) | parse_script() called in LoadScript step |
| G2 (Timestamp Overlap Guard) | detect_and_abort_overlaps() called after parse_script |
| G3 (OST Ratio Warning) | warn_ost_ratio() called after parse_script |
| G4 (Encoding Fallback Chain) | Implemented in subtitle.rs (pre-existing) |
| G5 (Empty Script Guard) | parse_script() checks clips.is_empty() |
| D-23 "play original" prefix | 3 occurrences in pipeline.rs |
| 9-step progress percentages | Correct sequence: 0/15/30/40/50/70/80/90/95% |

## Commits

| Task | Hash | Message |
|------|------|---------|
| 1 | 1e2bcd5 | feat(07-04): make 5 documentary pipeline step functions pub(crate) |
| 2 | b26c066 | feat(07-04): create clip.rs and audio.rs SDE wrapper modules |
| 3 | dc25de7 | feat(07-04): implement SDE pipeline orchestrator run_sde() with 9-step flow |

## Self-Check: PASSED

- [x] src/documentary/pipeline.rs - 5 pub(crate) annotations verified
- [x] src/sde/clip.rs - exists, contains sde_step_clip, 7 tests pass
- [x] src/sde/audio.rs - exists, 1 test passes
- [x] src/sde/pipeline.rs - exists, contains run_sde, 8 tests pass
- [x] Commits 1e2bcd5, b26c066, dc25de7 exist in git log
