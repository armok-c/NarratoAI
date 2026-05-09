---
phase: 07-sde-pipeline
verified: 2026-05-09T12:00:00Z
status: gaps_found
score: 9/12 must-haves verified
overrides_applied: 0
re_verification: false
gaps:
  - truth: "clip.rs 和 audio.rs 封装模块存在于 src/sde/ 目录"
    status: failed
    reason: "Plan 04 SUMMARY claims clip.rs and audio.rs were created, but neither file exists in narratoai-core/src/sde/. The OST=1 timestamp correction logic is inlined in pipeline.rs, and audio logic is also inlined. The mod.rs does not reference these modules."
    artifacts:
      - path: "narratoai-core/src/sde/clip.rs"
        issue: "File does not exist"
      - path: "narratoai-core/src/sde/audio.rs"
        issue: "File does not exist"
    missing:
      - "src/sde/clip.rs with sde_step_clip function"
      - "src/sde/audio.rs module (even thin marker)"
  - truth: "src/sde/subtitle.rs 存在于 SDE 模块中"
    status: failed
    reason: "Plan 03 specified src/sde/subtitle.rs but the subtitle module was extracted to a standalone src/subtitle/ module (done in Phase 8 plan 01). The SDE module re-exports from crate::subtitle via timestamp.rs re-exports and pipeline.rs imports. Functionally equivalent but the file path differs from plan."
    artifacts:
      - path: "narratoai-core/src/sde/subtitle.rs"
        issue: "File does not exist; functionality moved to src/subtitle/"
    missing:
      - "src/sde/subtitle.rs (or plan update to reflect extraction)"
  - truth: "parse_script 中 OST=1 的 narration 以 '播放原片' 开头时不做特殊处理"
    status: partial
    reason: "parse_script does not check narration text for 'play original' prefix -- it relies purely on the OST field. The pipeline TTS step skips OST=1 via OstType::OriginalSound check, which achieves the same goal. The plan specified text-prefix detection as a safety net, but the OST-type-based approach is more robust."
    artifacts:
      - path: "narratoai-core/src/sde/script_gen.rs"
        issue: "No 'play original' prefix check in parse_script"
    missing:
      - "Narration text prefix check as supplementary guard (optional -- OST field is authoritative)"
deferred:
  - truth: "clip.rs and audio.rs missing files"
    addressed_in: "Phase 8"
    evidence: "Phase 8 plan 01 extracted subtitle to standalone module; Phase 8 has its own clip.rs in src/sdp/. The SDE clip/audio wrappers were effectively inlined into pipeline.rs, making separate files unnecessary."
---

# Phase 7: SDE Pipeline Verification Report

**Phase Goal:** Build SDE (short drama narration) pipeline module -- subtitle parsing, two-step LLM script generation (plot analysis + narration), video processing orchestration, producing final narration videos.
**Verified:** 2026-05-09
**Status:** gaps_found
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

Roadmap success criteria (contract truths):

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | From SRT/ASS subtitle files extract Chinese dialogue content, correctly handle Chinese encoding | VERIFIED | `src/subtitle/parser.rs` (746 lines): detect_encoding() handles UTF-8/UTF-8-SIG/UTF-16LE/UTF-16BE/GBK/GB18030; normalize_subtitle_text() standardizes text; parse_subtitle_file() returns (Vec<SubtitleSegment>, String, String). 55 tests pass. |
| 2 | LLM analyzes plot then generates narration script with OST type markers | VERIFIED | `src/sde/script_gen.rs` (591 lines): step_analyze_plot() renders plot_analysis v1.0 prompt + calls LLM; step_generate_script() renders script_generation v2.0 prompt + calls LLM with JSON response format. repair_json() has 6-step fallback. parse_script() deserializes to Script with OST defaulting. |
| 3 | Complete SDE pipeline (subtitle parse -> LLM analyze -> OST script -> video process) outputs final video | VERIFIED | `src/sde/pipeline.rs` (850 lines): run_sde() implements 9-step pipeline: ParseSubtitle -> PlotAnalysis -> ScriptGenerate -> LoadScript -> Tts -> Clip -> MergeAudio -> Concat -> Composite. Each step has emit_progress() calls. Full FFmpeg concat + composite implemented. |

Plan-specific truths:

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 4 | SdeError enum contains 7 variants with From impl | VERIFIED | `src/sde/error.rs` (96 lines): ParseSubtitle, PlotAnalysis, ScriptGeneration, JsonRepair, VideoProcess(#[from] PipelineError), Validation, Io. From<io::Error> and From<SubtitleError> implemented. |
| 5 | SdeRequest contains subtitle_path/video_path/drama_name/temperature + 13 documentary-aligned video params | VERIFIED | `src/sde/types.rs` (245 lines): 18 fields including all documentary-aligned params. Default impl matches DocumentaryRequest defaults. validate() checks paths, ranges. |
| 6 | SdePipelineState has all pipeline intermediate fields | VERIFIED | `src/sde/types.rs`: 16 fields (task_id through progress), new() constructor, emit_progress() method. |
| 7 | 9-step SdeProgressStep enum (ParseSubtitle through Composite) | VERIFIED | `src/sde/types.rs`: 9 variants in correct order. |
| 8 | SubtitleSegment with index/start_secs/end_secs/text | VERIFIED | Re-exported from crate::subtitle::types::SubtitleSegment in types.rs line 127. |
| 9 | plot_analysis v1.0 + script_generation v2.0 prompt templates registered | VERIFIED | `src/prompt/register.rs`: register_short_drama_narration_prompts() registers 3 prompts (plot_analysis v1.0, script_generation v1.0, script_generation v2.0). Templates exist at correct paths with all required placeholders. |
| 10 | clip.rs and audio.rs wrapper modules in src/sde/ | FAILED | Files do not exist. mod.rs only declares: error, pipeline, script_gen, timestamp, types. Functionality inlined in pipeline.rs. |
| 11 | src/sde/subtitle.rs with encoding detection | FAILED | File does not exist. Subtitle module extracted to standalone src/subtitle/ (Phase 8 plan 01). SDE uses it via re-exports and imports. |
| 12 | parse_script checks 'play original' narration prefix | PARTIAL | parse_script relies on OST field only. Pipeline TTS step uses OstType::OriginalSound to skip. Text-prefix check absent but functionally equivalent. |

**Score:** 9/12 truths verified (3 gaps: 2 failed, 1 partial)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | clip.rs/audio.rs missing from src/sde/ | Phase 8 | Phase 8 plan 01 extracted subtitle to standalone module. SDE clip/audio wrappers were inlined into pipeline.rs, making separate files unnecessary. Phase 8 has its own src/sdp/clip.rs. |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `narratoai-core/src/sde/error.rs` | SdeError 7 variants + From impl | VERIFIED | 96 lines, 7 variants, 5 unit tests |
| `narratoai-core/src/sde/types.rs` | SdeRequest/SdePipelineState/SdeProgressStep/SubtitleSegment | VERIFIED | 245 lines, 5 unit tests |
| `narratoai-core/src/sde/timestamp.rs` | Re-exports parse_srt_timestamp + find_precise_range | VERIFIED | 2 lines (re-exports from src/subtitle/timestamp.rs which is 214 lines) |
| `narratoai-core/src/sde/mod.rs` | Public API surface | VERIFIED | 9 lines, re-exports key types |
| `narratoai-core/src/sde/script_gen.rs` | Two-step LLM + JSON repair + parse_script | VERIFIED | 591 lines, 28+ tests |
| `narratoai-core/src/sde/pipeline.rs` | run_sde() 9-step orchestrator | VERIFIED | 850 lines, 8 tests |
| `narratoai-core/src/sde/clip.rs` | OST=1 timestamp correction wrapper | MISSING | File does not exist; logic inlined in pipeline.rs lines 208-219 |
| `narratoai-core/src/sde/audio.rs` | SDE audio merge wrapper | MISSING | File does not exist; audio calls inlined in pipeline.rs lines 283-298 |
| `narratoai-core/src/subtitle/parser.rs` | Subtitle parsing + encoding detection | VERIFIED | 746 lines, 30+ tests (extracted standalone module) |
| `narratoai-core/src/prompt/templates/short_drama_narration/plot_analysis_v1.0.md` | Plot analysis prompt | VERIFIED | 66 lines, contains ${subtitle_content} |
| `narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v2.0.md` | Script generation prompt v2.0 | VERIFIED | 267 lines, contains ${drama_name}/${plot_analysis}/${subtitle_content} |
| `narratoai-core/src/prompt/register.rs` | Prompt registration | VERIFIED | All 3 short_drama_narration prompts registered |
| `narratoai-core/src/documentary/pipeline.rs` | 5 pub(crate) step functions | VERIFIED | step_tts/step_clip/step_merge_audio_subtitle/step_concat/step_composite all pub(crate); step_load_script remains private |
| `narratoai-core/src/lib.rs` | pub mod sde; declaration | VERIFIED | Line 15 |
| `narratoai-core/Cargo.toml` | encoding_rs dependency | VERIFIED | encoding_rs = "0.8" at line 41 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| sde/pipeline.rs | subtitle/parser.rs | parse_subtitle_file in spawn_blocking | WIRED | Line 72: spawn_blocking parse_subtitle_file |
| sde/pipeline.rs | sde/script_gen.rs | step_analyze_plot + step_generate_script | WIRED | Lines 93-99, 107-115 |
| sde/pipeline.rs | sde/script_gen.rs | parse_script | WIRED | Line 128: parse_script(&state.narration_raw) |
| sde/pipeline.rs | documentary/clip.rs | clip_all_videos | WIRED | Line 222 |
| sde/pipeline.rs | documentary/audio.rs | merge_audio_files + merge_subtitle_files | WIRED | Lines 283-298 |
| sde/pipeline.rs | sde/timestamp.rs | find_precise_range | WIRED | Line 211 |
| sde/script_gen.rs | prompt/manager.rs | render_prompt | WIRED | Lines 32-38 (plot_analysis), 89-95 (script_generation) |
| sde/script_gen.rs | llm/provider.rs | LlmProvider::generate_text | WIRED | Lines 43-54, 101-113 |
| sde/script_gen.rs | script/mod.rs | validate | WIRED | Line 348: crate::script::validate(&clips) |
| sde/error.rs | documentary/error.rs | From<PipelineError> | WIRED | Line 19: #[from] PipelineError |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| sde/pipeline.rs run_sde() | state.subtitle_text | parse_subtitle_file() output | Yes -- reads file bytes, detects encoding, parses SRT/ASS | FLOWING |
| sde/pipeline.rs run_sde() | state.plot_analysis | step_analyze_plot() -> LLM | Yes -- LLM provider.generate_text() response | FLOWING |
| sde/pipeline.rs run_sde() | state.narration_raw | step_generate_script() -> LLM | Yes -- LLM provider.generate_text() with Json format | FLOWING |
| sde/pipeline.rs run_sde() | state.script | parse_script() -> repair_json + serde | Yes -- deserialized from LLM JSON output | FLOWING |
| sde/pipeline.rs run_sde() | state.tts_results | crate::tts::synthesize() per clip | Yes -- TTS engine generates audio files | FLOWING |
| sde/pipeline.rs run_sde() | state.output_video_path | FFmpeg composite | Yes -- real FFmpeg filter_complex pipeline | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SDE module compiles | cargo check --lib | 0 errors, 7 pre-existing warnings | PASS |
| SDE unit tests pass | cargo test --lib sde | 44/44 passed | PASS |
| Subtitle unit tests pass | cargo test --lib subtitle | 55/55 passed | PASS |
| Prompt register tests pass | cargo test --lib prompt::register | 4/4 passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| SDE-01 | 07-01, 07-04 | Complete SDE pipeline orchestration (subtitle parse -> LLM analyze -> OST script -> video process) | SATISFIED | run_sde() in pipeline.rs implements full 9-step pipeline |
| SDE-02 | 07-01, 07-03 | Subtitle text parsing -- extract dialogue from SRT/ASS files | SATISFIED | src/subtitle/parser.rs with detect_encoding + parse_subtitle_file + normalize |
| SDE-03 | 07-02, 07-03, 07-04 | LLM plot analysis -- generate narration script with OST type markers | SATISFIED | script_gen.rs (step_analyze_plot + step_generate_script + repair_json + parse_script) |

Note: User prompt mentioned SDE-01 through SDE-08, but REQUIREMENTS.md only defines SDE-01, SDE-02, SDE-03. All three are covered.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | - | - | - | No anti-patterns detected |

### Human Verification Required

### 1. End-to-end SDE Pipeline Execution

**Test:** Provide a real SRT subtitle file and video file, call run_sde() with valid SdeRequest, config, registry, and prompt_manager.
**Expected:** Pipeline completes all 9 steps, produces output video at task_dir/combined_sde.mp4 with merged narration audio and subtitles.
**Why human:** Requires running FFmpeg, active LLM provider with API key, and TTS engine -- cannot verify programmatically without external services.

### 2. OST=0/1 Branch Correctness in Output Video

**Test:** Run pipeline and verify that OST=1 segments contain original audio (no TTS) and OST=0 segments contain narration audio.
**Expected:** Output video has correct audio mixing -- original sound for OST=1 clips, narration for OST=0 clips.
**Why human:** Requires audio playback verification or spectral analysis of the output video.

### Gaps Summary

Three gaps identified, all non-blocking to the phase goal:

1. **clip.rs and audio.rs missing from src/sde/** (FAILED): Plan 04 SUMMARY claims these were created (commits b26c066, dc25de7), but they do not exist on disk. The actual code has this logic inlined in pipeline.rs (OST=1 timestamp correction at lines 208-219, audio merge at lines 283-298). This is a structural deviation from plan but functionally equivalent. The mod.rs does not declare these modules, confirming they were never present or were removed during the Phase 8 subtitle extraction refactor. **Deferred to Phase 8** which restructured the module layout.

2. **src/sde/subtitle.rs missing** (FAILED): Plan 03 created this file but Phase 8 plan 01 extracted it to a standalone src/subtitle/ module. The SDE module re-exports from crate::subtitle via timestamp.rs and pipeline.rs imports. Functionally equivalent.

3. **'Play original' prefix detection absent in parse_script** (PARTIAL): The plan specified text-prefix checking in parse_script as a secondary guard. The implementation uses OstType::OriginalSound exclusively, which is more reliable since the LLM sets the OST field per the prompt template instructions. No functional gap -- just a different implementation approach.

All three ROADMAP success criteria are met. All three REQUIREMENTS.md requirements (SDE-01, SDE-02, SDE-03) are satisfied. The gaps are structural (file organization) rather than functional (missing behavior).

---

_Verified: 2026-05-09T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
