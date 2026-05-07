---
phase: 04-prompt-system-visual-analyzer
fixed_at: '2026-05-07T20:10:00Z'
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 2
findings_in_scope: 14
fixed: 14
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-05-07T20:10:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 14
- Fixed: 14
- Skipped: 0

## Fixed Issues

### CR-01: Add CancellationToken to analyze_images trait

**Files modified:** `narratoai-core/src/llm/provider.rs`, `narratoai-core/src/llm/openai_compatible.rs`, `narratoai-core/src/visual/analyzer.rs`
**Commit:** 57f5240
**Applied fix:** Added `cancel: Option<CancellationToken>` parameter to the `analyze_images` trait method in `provider.rs`. Updated the `OpenAICompatibleProvider` implementation with a pre-flight cancellation check per batch. Forwarded `cancel_after_extract` token from `analyzer.rs` to the `analyze_images` call.

### CR-02: Add bare JSON array fallback in parse_and_retry

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** 2c947c5
**Applied fix:** Added third fallback path in `parse_and_retry` attempting `serde_json::from_str::<Vec<FrameObservation>>(cleaned)` after the single `FrameObservation` fallback fails, with `tracing::warn!` when all paths fail.

### WR-01: Capture FFmpeg stderr

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** 4e4b0e0
**Applied fix:** Changed `run_ffmpeg_with_cancel` to use `Stdio::piped()` for stderr. On non-zero exit, reads and logs stderr with `tracing::warn!`, including the exit code.

### WR-02: Return extracted file paths from extract_frames

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`, `narratoai-core/src/visual/analyzer.rs`, `narratoai-core/src/documentary/script_gen.rs`
**Commit:** 91b44f8
**Applied fix:** Changed `extract_frames` return type from `Result<usize, _>` to `Result<(usize, Vec<PathBuf>), _>`. Added `collect_keyframe_paths_from_dir` helper to collect sorted paths after extraction. Updated `analyzer.rs` to use returned paths directly instead of re-scanning the directory via `collect_frame_paths`. Updated `script_gen.rs` to destructure the returned tuple directly into `keyframe_files`.

### WR-03: Separate match arms for fast path

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** 1a370fc
**Applied fix:** Replaced the `_ =>` catch-all with separate `Err(e)` and `Ok(0)` arms. Logs fast-path errors with `tracing::warn!` and zero-frame result with `tracing::info!` before falling back.

### WR-04: Fix quality range docstring

**Files modified:** `narratoai-core/src/visual/analyzer.rs`
**Commit:** 0de9654
**Applied fix:** Changed docstring from `"JPEG 压缩质量（1-31）"` to `"JPEG 压缩质量（2-31，值越小质量越高，默认 5）"`.

### WR-05: Remove dead_code attribute on seconds_to_hhmmssmmm

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** 61591dd
**Applied fix:** Removed `#[allow(dead_code)]` annotation from the actively used `seconds_to_hhmmssmmm` function.

### WR-06: Fix BatchPartial Display format

**Files modified:** `narratoai-core/src/visual/error.rs`, `narratoai-core/src/visual/analyzer.rs`
**Commit:** f13e1e6
**Applied fix:** Changed `BatchPartial` error field from `Vec<String>` to `String` and Display format from `{errors:?}` (Debug) to `{errors}` (Display). Updated `analyzer.rs` to pass `errors.join("; ")` instead of raw `Vec<String>`. Updated test to construct `String` field.

### WR-07: Fix version sorting in list_prompts fallback

**Files modified:** `narratoai-core/src/prompt/registry.rs`
**Commit:** f8e3cb7
**Applied fix:** Replaced lexicographic `.sort()` on version strings with `sort_by` using numeric comparison of the major version number extracted from `v{N}.{M}` format strings.

### IN-01: Extract shared directory iteration helper

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** 13dbd88
**Applied fix:** Created `iterate_dir_files()` helper iterating a directory with prefix/suffix pattern matching and configurable silent error handling. Refactored three existing iteration patterns (stale keyframe cleanup, `rename_fast_path_frames` fastframe collection, `cleanup_fast_path_files` fastframe cleanup) to use the shared helper.

### IN-02: Extract magic values as constants

**Files modified:** `narratoai-core/src/visual/analyzer.rs`, `narratoai-core/src/prompt/validators.rs`
**Commit:** b9fe7a9
**Applied fix:** Added module-level constants `DEFAULT_INTERVAL_SECONDS`, `ANALYSIS_TEMPERATURE`, `ANALYSIS_MAX_TOKENS` in `analyzer.rs` and `MIN_PLOT_ANALYSIS_CHARS`, `MIN_NARRATION_CHARS`, `MIN_NARRATION_PARAGRAPHS` in `validators.rs`. Replaced hardcoded values with named constant references.

### IN-03: Document two-tier validation in render_prompt

**Files modified:** `narratoai-core/src/prompt/manager.rs`
**Commit:** f44d624
**Applied fix:** Added detailed comment in `render_prompt` explaining the two-tier validation design: Tier 1 checks `ParameterDef.required` parameters, Tier 2 checks all `${variable}` references in template rendering.

### IN-04: Add debug_assert for NaN in seconds_to_hhmmssmmm

**Files modified:** `narratoai-core/src/visual/frame_extractor.rs`
**Commit:** 77441f2
**Applied fix:** Added `debug_assert!(!total_secs.is_nan())` before the millisecond calculation in `seconds_to_hhmmssmmm`.

### IN-05: Document strip_code_fence re-export

**Files modified:** `narratoai-core/src/visual/types.rs`
**Commit:** a3f141d
**Applied fix:** Updated comment on the re-export to clarify it is only consumed by the test module in the same file.

---

_Fixed: 2026-05-07T20:10:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
