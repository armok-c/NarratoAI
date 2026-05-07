---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T14:30:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - narratoai-core/src/lib.rs
  - narratoai-core/src/prompt/error.rs
  - narratoai-core/src/prompt/manager.rs
  - narratoai-core/src/prompt/mod.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/registry.rs
  - narratoai-core/src/prompt/template.rs
  - narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/prompt/types.rs
  - narratoai-core/src/prompt/validators.rs
  - narratoai-core/src/visual/analyzer.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/types.rs
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 04: Code Review Report — Prompt System / Visual Analyzer

**Reviewed:** 2026-05-07T14:30:00Z
**Depth:** standard
**Files Reviewed:** 19 (prompt system 8 + visual analyzer 4 + 6 template files + Cargo.toml + lib.rs)
**Status:** issues_found

## Summary

Reviewed the Rust `prompt` and `visual` subsystems at standard depth. The codebase has been refactored since the previous review — `strip_code_fence` was moved to `crate::text_utils` (formerly the cross-module coupling CR-01), the `analyzed_batches` count now uses an explicit counter (formerly WR-03), and error logging is present in `run_ffmpeg_with_cancel`.

Two WARNING-level findings remain: (1) `FrameObservation::validate()` is defined but never called in the analysis pipeline, so out-of-range `visual_salience` values pass through undetected; (2) the fallback frame extraction path uses system PATH `ffmpeg`/`ffprobe` directly while the fast path uses `ffmpeg_sidecar`, creating an inconsistent binary discovery strategy. Four INFO-level items note minor style/sustainability issues.

## Previously Fixed (from prior review iteration)

The following prior findings are confirmed resolved in the current code:
- **CR-01** (cross-module dependency): `strip_code_fence` is now in `crate::text_utils`, imported by both `validators.rs` and `types.rs`. Correct.
- **WR-02** (empty-observations blind spot): `analyze_video_frames` now has Step 8b check for all-empty observations. Present and correct.
- **WR-03** (missing code-fence in JSON validator): `validators.rs:35` calls `crate::text_utils::strip_code_fence`. Present.
- **WR-03** (old: success-count by subtraction): Code now uses an explicit `success_count` accumulator. Fixed.
- **WR-01** (old: silent FFmpeg error swallowing): `run_ffmpeg_with_cancel` now has `tracing::warn!` calls for spawn and wait failures. Present.

## Warnings

### WR-01: FrameObservation::validate() defined but never called in the analysis pipeline

**Files:**
- `narratoai-core/src/visual/types.rs:30-37` (definition)
- `narratoai-core/src/visual/analyzer.rs:163-178` (call site — missing)

**Issue:** `FrameObservation` defines a public `validate()` method that checks whether `visual_salience` is in the documented `[0.0, 1.0]` range. However, the analysis loop in `analyze_video_frames` parses LLM responses via `parse_and_retry()` and extends the `observations` vector directly without ever calling `.validate()` on any `FrameObservation`. Out-of-range values such as `visual_salience: 2.5` (explicitly tested as acceptable in `types.rs:224`) pass through silently.

The method exists and is public — it was clearly intended to be called post-deserialization — but no call site exists in any reviewed file.

**Fix:** Call `.validate()` on each `FrameObservation` after deserialization. Integrate into `parse_and_retry()` or into the analysis loop:

```rust
// In analyze_video_frames, Step 7:
Ok(batch) => {
    let valid: Vec<FrameObservation> = batch.observations
        .into_iter()
        .filter(|obs| {
            obs.validate().unwrap_or_else(|e| {
                warn!("Frame {} validation failed: {}", obs.frame_number, e);
                false
            })
        })
        .collect();
    success_count += 1;
    observations.extend(valid);
    // ...
}
```

### WR-02: Fallback frame extraction uses system PATH ffmpeg/ffprobe inconsistent with fast path

**Files:**
- `narratoai-core/src/visual/frame_extractor.rs:531-559` (get_video_duration, uses system `ffprobe`)
- `narratoai-core/src/visual/frame_extractor.rs:574-597` (run_ffmpeg_with_cancel, uses system `ffmpeg`)
- `narratoai-core/src/visual/frame_extractor.rs:134` (fast path uses `ffmpeg_sidecar::command::FfmpegCommand`)

**Issue:** The fast path uses `ffmpeg_sidecar::command::FfmpegCommand`, which may resolve a bundled or auto-downloaded FFmpeg binary. The fallback path (`extract_frames_fallback`) calls `std::process::Command::new("ffprobe")` and `std::process::Command::new("ffmpeg")` — always resolving from system PATH. There is no guarantee these refer to the same binary.

Concrete failure scenarios:
- User enables `download-ffmpeg` feature: bundled ffmpeg is used for the fast path (works), but if the fast path yields 0 frames, the fallback resolves a different system PATH ffmpeg (may have different capabilities or encoding defaults, or may not exist at all).
- The fallback writes `tracing::warn!` messages (line 586, 593) but the caller only receives a bare `false`. The 4-level fallback in `extract_single_frame` propagates no information about which level failed due to a missing binary versus the frame not being extractable.

The function doc comments at lines 528-531 and 571-573 acknowledge this limitation but the risk remains real.

**Fix options (pick one):**

Option A: Resolve the ffmpeg path from `ffmpeg_sidecar` and pass it to `std::process::Command`:

```rust
// At the top of extract_frames or a helper:
fn get_ffmpeg_path() -> PathBuf {
    // Prefer ffmpeg_sidecar's resolved binary
    ffmpeg_sidecar::command::ffmpeg_binary()
        .or_else(|_| PathBuf::from("ffmpeg").canonicalize())
        .unwrap_or_else(|_| PathBuf::from("ffmpeg"))
}
```

Option B: If the two code paths must remain independent, log a structured warning when falling back to alert operators:

```rust
tracing::warn!(
    "fast path produced 0 frames; falling back to system PATH ffmpeg (may differ from ffmpeg_sidecar binary)"
);
```

## Info

### IR-01: Incorrect #[allow(dead_code)] annotation on seconds_to_hhmmssmmm

**File:** `narratoai-core/src/visual/frame_extractor.rs:495`

**Issue:** The function has `#[allow(dead_code)]` but is referenced from non-test production code (`rename_fast_path_frames` at line 477). The annotation is misleading — a reader could interpret it as a signal to remove the function.

**Fix:** Remove the `#[allow(dead_code)]` attribute.

### IR-02: Regex objects compiled on every render() call

**Files:**
- `narratoai-core/src/prompt/template.rs:85-87`
- `narratoai-core/src/prompt/template.rs:121-123`

**Issue:** Both `Regex::new(r"\$\{(\w+)\}")` and `Regex::new(r"\$\{(\w+)\|(\w+)\}")` compile the same patterns on every invocation of `render()`. While regex compilation overhead is small for these patterns, this is a redundant per-call cost.

**Fix:** Use `std::sync::LazyLock` (stabilized in Rust 1.80) for one-time compilation:

```rust
use std::sync::LazyLock;

static VAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{(\w+)\}").expect("invalid var regex")
});
```

The same applies to the filter regex and possibly the `builtin_filters()` HashMap.

### IR-03: render_prompt iterates parameters list twice for default merge

**File:** `narratoai-core/src/prompt/manager.rs:77-84`

**Issue:** The parameter iteration inserts defaults (lines 77-81), then overwrites with caller vars (lines 82-84). This double pass is functionally correct but creates unnecessary allocation churn in the intermediate `HashMap<String, String>` plus the conversion to `HashMap<&str, &str>` (lines 86-89). Could be done in a single pass.

**Fix:** Optional — merge defaults and caller overrides in one pass:

```rust
let mut merged: HashMap<String, String> = HashMap::new();
for param in &prompt.metadata.parameters {
    let value = vars
        .get(param.name.as_str())
        .map(|s| s.to_string())
        .or_else(|| param.default.clone());
    if let Some(v) = value {
        merged.insert(param.name.clone(), v);
    }
}
```

### IR-04: PromptError::LockFailure discards original PoisonError type

**File:** `narratoai-core/src/prompt/error.rs:17-18`

**Issue:** The `LockFailure` variant stores a `String` message but discards the `PoisonError<T>` from the `RwLock`. Downstream code cannot inspect whether the error was a poison, timeout, or other lock failure. This is a minor loss of diagnostic information.

**Fix:** Not easily fixable without making `PromptError` generic over the lock guard type, which is likely not worth the complexity. A doc comment noting this limitation would be sufficient.

---

_Reviewed: 2026-05-07T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
