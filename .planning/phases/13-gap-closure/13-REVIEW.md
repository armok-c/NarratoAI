---
phase: 13-gap-closure
reviewed: 2026-05-12T10:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - config.example.toml
  - narratoai-core/src/audio/mod.rs
  - narratoai-core/src/audio/pipeline.rs
  - narratoai-core/src/audio/volume.rs
  - narratoai-core/src/config/defaults.rs
  - narratoai-core/src/config/types.rs
  - narratoai-core/src/documentary/error.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sdp/pipeline.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/tests/audio_normalize_test.rs
  - narratoai-core/tests/sdp_real_test.rs
  - src-tauri/src/commands/pipeline.rs
findings:
  critical: 2
  warning: 6
  info: 4
  total: 12
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-05-12T10:00:00Z
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

Reviewed 16 files across the audio normalization pipeline, documentary/SDE/SDP processing pipelines, Tauri command layer, config types, and integration tests. Found 2 critical bugs (audio loss for OST=1 clips in SDE, and amix=inputs=1 edge case that fails on some FFmpeg versions), 6 warnings (ranging from serde default mismatches to concurrency lock contention and validation inconsistencies), and 4 informational items. The SDE and documentary pipelines share a common root cause: `concat -c copy` strips audio when clips have heterogeneous OST types, which both pipelines handle incorrectly.

## Critical Issues

### CR-01: SDE composite uses `anullsrc` silence, discarding OST=1 original audio

**File:** `narratoai-core/src/sde/pipeline.rs:479-484`
**Issue:** The SDE pipeline's final composite step generates a silence track via `anullsrc` instead of extracting and using the original audio from OST=1 (OriginalSound) clips. When the concat step (`-c copy`) assembles clips with mixed OST types (OST=0 has no audio, OST=1 has audio), the resulting concatenated file may lose its audio stream. The developer was aware of this and added the `anullsrc` workaround, but the workaround replaces all original audio with silence. This means any OST=1 clip's original sound is completely lost in the final output.

The comment on lines 478-480 acknowledges the root cause: "concat -c copy 在混合 OST=0 (无音频) 和 OST=1 (有音频) 的片段时会丢失音频流，此处统一用 anullsrc 生成静音轨替代". However, generating silence is not an acceptable substitute — it breaks the core functionality of OST=1 (OriginalSound).

**Fix:** Either (a) pre-extract the original audio from each OST=1 clip before concat and mix it back in during composite, (b) use a concat method that preserves audio streams (e.g., re-encode audio instead of `-c copy`), or (c) split the pipeline so OST=0 and OST=1 clips are concat'd separately and combined in composite. The current approach silently drops audio.

```rust
// Instead of anullsrc silence:
// Option A: Extract original audio from source video for OST=1 segments
// and mix them as [orig] instead of generating silence.
// Option B: Use FFmpeg concat with audio re-encoding to ensure consistent
// audio stream across mixed-OST clips.
```

### CR-02: FFmpeg `amix=inputs=1` is invalid — pipeline fails when single audio source

**File:** `narratoai-core/src/sdp/pipeline.rs:336-339`
**Affected files:** `narratoai-core/src/documentary/pipeline.rs:384-393`, `narratoai-core/src/sde/pipeline.rs:524-532`
**Issue:** When the composite step has only one audio source (no BGM, no normalized audio, no additional TTS), `amix_count` equals 1. The generated filter becomes `[orig]amix=inputs=1:duration=longest[aout]`. FFmpeg's `amix` filter requires at least 2 inputs (its docs specify inputs >= 2). With `inputs=1`, the filter will fail on many (possibly all) FFmpeg versions, crashing the pipeline.

This affects all three pipelines:
- **SDP:** Default config (normalization disabled, no BGM) hits `amix_count=1` on line 337.
- **Documentary:** Same condition hits `amix_count=1` on line 391.
- **SDE:** Same condition hits `amix_count=1` on line 529.

The SDP test (`sdp_real_test.rs`) uses `AppConfig::default()` (normalization disabled) and no BGM, which exercises this code path directly. The test would fail on any FFmpeg version that rejects `amix=inputs=1`.

**Fix:** When `amix_count == 1`, skip the `amix` filter entirely and use the single audio source directly:

```rust
if amix_count == 1 {
    // Single audio source — no mixing needed, just use volume-adjusted orig
    filter_complex_parts.push(format!(
        "[0:a]volume={:.2}[aout]", original_volume
    ));
} else if amix_count > 1 {
    // Multiple audio sources — use amix
    filter_complex_parts.push(format!(
        "{}amix=inputs={}:duration=longest{}[aout]",
        mix_inputs, amix_count, compensation
    ));
}
```

## Warnings

### WR-01: `AudioSection::volume_profile` serde default inconsistent with Rust Default

**File:** `narratoai-core/src/config/types.rs:244`, `narratoai-core/src/config/defaults.rs:132`
**Issue:** The `volume_profile` field in `AudioSection` has `#[serde(default)]`, which uses `String::default()` (= `""`) when the field is missing from the TOML file. However, `AudioSection::default()` (used when the entire section is missing) sets `volume_profile = "balanced"`. This means loading `config.toml` with `volume_profile` absent produces `""`, while `AppConfig::default()` produces `"balanced"`. The `apply_volume_profile("")` function falls through to the `_` default branch, which uses `original_volume=1.3` versus `"balanced"` branch which uses `1.2`. This is a silent behavior difference.

The test `test_load_full_config` in `types.rs:370` asserts `assert_eq!(config.audio.volume_profile, "")`, confirming the serde behavior — but this is inconsistent with the Rust Default.

**Fix:** Use `#[serde(default = "default_audio_profile")]` instead:

```rust
fn default_volume_profile() -> String {
    "balanced".to_string()
}

#[serde(default = "default_volume_profile")]
pub volume_profile: String,
```

Or change the test assertion to expect `"balanced"` and add the field to the test TOML string.

### WR-02: `RwLockReadGuard` held across long async pipelines

**File:** 
- `src-tauri/src/commands/pipeline.rs:151-152` (SDE)
- `src-tauri/src/commands/pipeline.rs:272` (documentary script gen)
- `src-tauri/src/commands/pipeline.rs:337` (SDP script gen)

**Issue:** The `RwLockReadGuard` for `registry` and `prompt_manager` is acquired and held across the entire async pipeline execution, which includes LLM API calls (seconds to minutes) and FFmpeg processing. During this time, no other task can acquire a write lock on these resources. While reads are not prevented, the long-held guard prevents any registry mutations (e.g., adding/reloading providers) and effectively blocks the Tauri event loop from processing other commands that need the write lock.

The TODO on line 149 (`// TODO: Lock guards held across long async pipelines; requires core API refactoring`) acknowledges the issue but provides no mitigation.

The `generate_documentary_script` command (line 233-248) correctly scopes the registry guard to a block and drops it before `.await`, demonstrating the correct pattern. The SDE and SDP script gen commands should follow the same pattern.

**Fix:** Extract necessary data (provider references, prompts) from the registry/prompt_manager before the async section, and drop the guards:

```rust
// Extract text provider before entering long async pipeline
let text_provider = {
    let guard = registry.read().await;
    guard.get(&config.app.text_llm_provider).map_err(...)?
};
// guard dropped here

// Extract prompt early if possible
let prompt = {
    let pm = prompt_manager.read().await;
    pm.render_prompt(...).map_err(...)?
};
// pm guard dropped here

// Now run pipeline without holding locks
run_sde(..., text_provider, prompt, ...).await?;
```

### WR-03: `SdpRequest::validate()` volume range [0,10] inconsistent with `VolumeConfig::validate()` [0,2]

**File:** `narratoai-core/src/sdp/types.rs:75-86`, `narratoai-core/src/audio/volume.rs:27-40`
**Issue:** `SdpRequest::validate()` accepts `original_volume` and `bgm_volume` in the range [0.0, 10.0], but `VolumeConfig::validate()` only accepts [0.0, 2.0]. A request with `original_volume=5.0` passes the SDP request validation but is outside the range that `VolumeConfig` considers valid. The validation chains are inconsistent.

**Fix:** Align the SDP validation range with VolumeConfig's contract:

```rust
// In sdp/types.rs validate():
if !(0.0..=2.0).contains(&self.original_volume) {
    return Err(format!("original_volume 超出有效范围 [0, 2]: {}", self.original_volume));
}
if !(0.0..=2.0).contains(&self.bgm_volume) {
    return Err(format!("bgm_volume 超出有效范围 [0, 2]: {}", self.bgm_volume));
}
```

### WR-04: Documentary pipeline `[0:a]` reference may fail after mixed-OST concat

**File:** `narratoai-core/src/documentary/pipeline.rs:348`
**Issue:** Same root cause as CR-01 but in the documentary pipeline. The composite step references `[0:a]` (the concatenated video's audio stream) directly without checking if it exists. When clips with mixed OST types are concatenated with `-c copy`, the resulting video may not have an audio stream (if the first clip happens to be OST=0/NarrationOnly which has no audio). The `has_original_audio` flag (line 347) checks if any clip is NOT OST=0, but this is a logical check on the script, not a runtime check for audio stream existence. FFmpeg will fail with "Stream not found" if `[0:a]` doesn't exist.

**Fix:** Either (a) extract original audio from clipped segments separately from concat (similar to SDP's approach), (b) use `anullsrc` as a fallback like SDE does (but then add audio extraction for OST=1 clips), or (c) use a forced audio generation approach:

```rust
// At minimum, add a guard to detect the condition:
let actual_has_audio = check_concat_output_has_audio(&merged_video);
if has_original_audio && !actual_has_audio {
    // Concat dropped audio — extract from source clips and remix
    // Fall through to anullsrc + separate audio extraction path
}
```

### WR-05: Hardcoded local paths in `sdp_real_test.rs`

**File:** `narratoai-core/tests/sdp_real_test.rs:23-25`
**Issue:** The integration test uses absolute file paths specific to one developer's machine:
- `VIDEO_PATH = "E:/GitLib/视频/华星大酒店.mp4"`
- `SUBTITLE_PATH = "E:/GitLib/视频/华星大酒店.srt"`
- `TEST_OUTPUT_DIR = "E:/GitLib/NarratoAI/test_output/sdp"`

These paths will not exist on any other machine or CI environment. The test has no `#[ignore]` attribute and no feature gate to skip it in non-development environments.

**Fix:** Either add `#[ignore]` to the test (with a note on how to run it locally), or provide test fixture files in the repository:

```rust
/// This test requires real media files that are not checked into the repository.
/// Set environment variables SDP_TEST_VIDEO and SDP_TEST_SUBTITLE to run.
#[ignore]
#[tokio::test]
async fn test_sdp_pipeline() {
```

### WR-06: `ProcessingConfig::default()` has features enabled, inconsistent with `AudioSection::default()`

**File:** `narratoai-core/src/audio/volume.rs:56-57`
**Issue:** `ProcessingConfig::default()` sets `enable_smart_volume: true` and `enable_audio_normalization: true`, but `AudioSection::default()` (which models the TOML config) sets both to `false`. While `ProcessingConfig::from_section()` correctly copies from `AudioSection`, the `Default` impl is misleading — any code path that uses `ProcessingConfig::default()` will get features enabled when the application defaults have them disabled. Currently only used in tests, but as a public API, this is a correctness trap.

**Fix:** Align the defaults or remove `Default` in favor of only allowing construction via `from_section()`:

```rust
impl ProcessingConfig {
    /// ProcessingConfig should only be created from an AudioSection.
    /// Use from_section() instead of Default.
    #[deprecated(since = "0.8.0", note = "use ProcessingConfig::from_section() instead")]
    fn default() -> Self { /* ... */ }
}
```

## Info

### IN-01: `let _ = ...?` pattern is redundant

**File:** `narratoai-core/src/documentary/pipeline.rs:172`, `narratoai-core/src/sde/pipeline.rs:294`

The pattern `let _ = expr.map_err(...)?;` works correctly but the `let _ =` binding is redundant — the `?` operator already unwraps the Ok value on success. On error, `?` returns immediately. This may confuse readers into thinking the discarded value matters. Replace with just `expr.map_err(...)?;`.

### IN-02: SDP pipeline doc string says "4 步" but only 3 steps exist

**File:** `narratoai-core/src/sdp/pipeline.rs:10`

The doc comment says "4 步顺序编排" but lists only 3 steps: Clip, Concat, Composite. Either update the count or add the missing step. The comment block on lines 97-167 confirms only 3 concrete steps exist.

### IN-03: `normalize_merged_audio` edge case with bare filename

**File:** `narratoai-core/src/audio/pipeline.rs:26`

`input.parent().unwrap_or(Path::new("."))` returns `Some(Path::new(""))` (not `None`) for bare filenames like `"file.mp3"` without a directory component. The `unwrap_or` does not trigger, and `Path::new("")` is used as the output directory. In practice, all audio paths are absolute paths within task_dir, so this edge case is not currently triggered, but it's a latent bug.

### IN-04: `generate_sdp_script` cleanup on success discards task_dir

**File:** `src-tauri/src/commands/pipeline.rs:355`

`tokio::fs::remove_dir_all(&task_dir)` runs after the script generation succeeds. This is fine since the script is returned as function output, but any intermediate files in task_dir are silently discarded. This is intentional cleanup behavior — noting it for awareness.

---

_Reviewed: 2026-05-12T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
