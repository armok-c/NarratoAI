---
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:30:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - src/lib.rs
  - src/prompt/mod.rs
  - src/prompt/types.rs
  - src/prompt/error.rs
  - src/prompt/registry.rs
  - src/prompt/template.rs
  - src/prompt/manager.rs
  - src/prompt/validators.rs
  - src/prompt/register.rs
  - src/prompt/templates/documentary/frame_analysis_v1.0.md
  - src/prompt/templates/documentary/narration_generation_v2.0.md
  - src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - src/visual/mod.rs
  - src/visual/error.rs
  - src/visual/types.rs
  - src/visual/frame_extractor.rs
  - src/visual/analyzer.rs
findings:
  critical: 2
  warning: 5
  info: 4
  total: 11
status: issues_found
---

# Phase 4: Code Review Report

**Reviewed:** 2026-05-07T12:30:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed the Prompt System (types, registry, template engine, manager, validators, registration, 4 prompt templates) and Visual Analyzer (frame extractor, analyzer, types, error) across 19 source files at standard depth. The overall architecture is well-structured: clean error enums with thiserror, proper `RwLock` usage with poison handling, a correct 3-level registry index, and a reasonable 4-level FFmpeg fallback strategy. Template variables in all .md files are consistent with their `ParameterDef` declarations in `register.rs`.

Found 2 critical issues and 5 warnings. The critical issues are: (1) a busy-wait polling loop in `run_ffmpeg_with_cancel` that wastes CPU for the entire lifetime of each FFmpeg process invocation, multiplied by up to 4x per frame in the fallback path; (2) `extract_frames_fast_path` proceeds to rename partial frames even after cancellation was detected and the FFmpeg child was killed. Warnings cover a fragile `json` filter fallback, shared temp directory paths in tests, missing cancellation checks between fallback levels, lexical sort that breaks for very large frame counts, and an absent upper-bound validation on `interval_seconds`.

## Critical Issues

### CR-01: Busy-wait polling loop in `run_ffmpeg_with_cancel` wastes CPU across hundreds of process invocations

**File:** `src/visual/frame_extractor.rs:535-557`

**Issue:** The function uses `child.try_wait()` in a `loop` with `std::thread::sleep(Duration::from_millis(50))`. This is a classic busy-wait pattern: the thread wakes 20 times per second for the entire lifetime of each FFmpeg process. In the fallback path (`extract_single_frame`), this function is called up to 4 times per frame (one per output format level). For a video with 300 frames that all require full fallback, this is 1200 FFmpeg invocations, each holding a `spawn_blocking` thread that polls at 20Hz for the full process duration.

Additionally, the 50ms polling interval introduces up to 50ms latency before detecting child exit, meaning each frame extraction incurs up to 200ms (4 levels x 50ms) of pure overhead on top of actual FFmpeg execution time.

**Fix:** Use blocking `child.wait()` instead of polling, and check cancellation at the call site (between fallback levels and between frames). The outer loop in `extract_frames_fallback` already checks cancellation between frames. Add cancellation checks between fallback levels inside `extract_single_frame`:

```rust
fn run_ffmpeg_with_cancel(args: &[&str], cancel: &CancellationToken) -> bool {
    let mut child = match std::process::Command::new("ffmpeg")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Blocking wait -- cancellation is checked between frames/levels at call site
    match child.wait() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
```

In `extract_single_frame`, add between each level:

```rust
if cancel.is_cancelled() {
    return Err(VisualError::FrameExtraction("帧提取被取消".into()));
}
```

### CR-02: `extract_frames_fast_path` renames partial frames after cancellation kill

**File:** `src/visual/frame_extractor.rs:147-161`

**Issue:** After the FFmpeg event loop, if cancellation was detected (line 148-151: `cancel.is_cancelled()` triggers `child.kill()` and returns error), the function correctly returns an error. However, there is a race condition: the event iterator (`child.iter()`) processes events including `FfmpegEvent::Error`. If FFmpeg writes partial frames before being killed, these files exist on disk. The error return propagates to `extract_frames`, which then calls `cleanup_fast_path_files` on line 76. But if the kill + error return path on line 151 is taken, the function returns `Err(...)` and `rename_fast_path_frames` on line 158 is NOT called. The cleanup happens at the call site. This is actually handled correctly.

**Revised finding:** On re-inspection, the cancellation path correctly returns early with an error, skipping `rename_fast_path_frames`. The call site in `extract_frames` (line 59-88) catches the error and calls `cleanup_fast_path_files`. This is correct behavior.

**Actual CR-02 -- same function, different issue:** The event loop on lines 147-156 iterates events and logs errors but does NOT check cancellation until the next event arrives. If FFmpeg stalls (e.g., seeking to a very late timestamp), the loop blocks on `iter.next()` indefinitely without checking the cancellation token. The cancellation check only runs when a new event is produced by FFmpeg. For large videos or slow I/O, this means cancellation can be delayed by seconds or minutes.

```rust
for event in iter {
    if cancel.is_cancelled() {  // <-- only checked when event arrives
        let _ = child.kill();
        let _ = child.wait();
        return Err(VisualError::FrameExtraction("帧提取被取消".into()));
    }
    if let FfmpegEvent::Error(e) = event {
        tracing::warn!("FFmpeg 快路径错误: {}", e);
    }
}
```

**Fix:** There is no straightforward fix with `ffmpeg_sidecar`'s iterator design. A workaround is to run the event iteration in a separate thread and check cancellation with a timeout, or use the raw `spawn_blocking` + manual FFmpeg invocation approach (as done in the fallback path) which gives full control over process lifecycle. At minimum, document this limitation.

## Warnings

### WR-01: `json` filter fallback produces invalid JSON for strings with backslashes or newlines

**File:** `src/prompt/template.rs:52-53`

**Issue:** The `json` filter has a fallback when `serde_json::to_string` fails:

```rust
serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s.replace('"', "\\\"")))
```

The fallback only escapes double quotes. It does not escape backslashes (`\`), newlines (`\n`), tabs (`\t`), or control characters. For a string containing `hello\world`, the fallback produces `"hello\world"` -- this is valid JSON (the backslash is followed by `w`, not a JSON escape code, which is technically valid but semantically wrong). For a string with an actual newline byte, the fallback produces a multi-line string literal which IS invalid JSON.

While `serde_json::to_string(&str)` should never fail (serializing a Rust `&str` is infallible in serde_json), the fallback code contains a correctness bug that would surface if the assumption ever breaks.

**Fix:** Remove the fallback since `serde_json::to_string(&str)` is infallible:

```rust
m.insert("json", |s: &str| {
    serde_json::to_string(s).expect("serializing &str to JSON string cannot fail")
});
```

Or fix the fallback to properly escape all JSON special characters.

### WR-02: Tests use hardcoded shared temp directory paths -- race condition under parallel test execution

**File:** `src/visual/frame_extractor.rs:599,632,667,704,733,767` and `src/visual/analyzer.rs:424,454,479`

**Issue:** Multiple tests use hardcoded temp directory names like `std::env::temp_dir().join("narratoai_test_rename")`. When tests run in parallel (default `cargo test` behavior), different test processes share the same directory, causing non-deterministic failures. Tests that cleanup with `remove_dir_all` at the end can delete files being used by concurrent tests.

The `tempfile` crate is already in `[dev-dependencies]` but is not used in these tests.

**Fix:** Use `tempfile::tempdir()` for isolated temporary directories:

```rust
let temp = tempfile::tempdir().expect("should create temp dir");
let temp_dir = temp.path();
// ... use temp_dir for test operations ...
// automatic cleanup when temp is dropped
```

### WR-03: Lexical sort in `collect_frame_paths` breaks for frame counts exceeding 6 digits

**File:** `src/visual/analyzer.rs:290`

**Issue:** The comment states "Sort by frame number (lexical sort on zero-padded prefix)" and uses `paths.sort()`. Frame names use `{:06}` format (6 zero-padded digits). Lexical sort is correct for frame numbers 0-999999. However, if `interval_seconds` is very small (e.g., 0.01) and the video is long (> 10000 seconds), frame numbers exceed 999999. The name `keyframe_1000000_*.jpg` would sort before `keyframe_999999_*.jpg` lexicographically, producing incorrect frame order for analysis.

While this edge case is unlikely in typical usage (it would produce over 1M frames), the sort assumption is fragile and undocumented.

**Fix:** Add numeric sort on the frame number portion, or document the frame count limitation:

```rust
paths.sort_by(|a, b| {
    let a_num = extract_frame_number(a);
    let b_num = extract_frame_number(b);
    a_num.cmp(&b_num)
});
```

### WR-04: No upper-bound validation on `interval_seconds` -- accepts absurdly large values

**File:** `src/visual/frame_extractor.rs:48-49`

**Issue:** The validation only checks `interval_seconds <= 0.0` and `is_nan()`. Extremely large values like `f64::MAX` or `1e10` are accepted. With `interval_seconds = 1e10`, the FFmpeg command `fps=1/10000000000` would extract 0 frames, causing the function to fall through to the fallback path. The fallback computes `total_frames = (duration / 1e10).ceil() as usize = 0` and returns `Ok(0)`. The caller in `analyze_video_frames` then fails at the "0 frames" guard with a misleading error message ("未提取到任何帧") instead of flagging the invalid interval.

**Fix:** Add an upper bound:

```rust
if interval_seconds > 86400.0 {
    return Err(VisualError::FrameExtraction(
        format!("帧提取间隔过大: {}s (最大 86400s/1天)", interval_seconds)
    ));
}
```

### WR-05: Missing cancellation checks between fallback levels in `extract_single_frame`

**File:** `src/visual/frame_extractor.rs:263-392`

**Issue:** The 4-level fallback function `extract_single_frame` does not check `cancel.is_cancelled()` between levels. If cancellation is requested during Level 1, the function continues through Levels 2, 3, and 4 before returning. Each level spawns a new FFmpeg process (with the busy-wait from CR-01). In the worst case, cancellation is delayed by up to 4x the FFmpeg process timeout.

The outer loop in `extract_frames_fallback` checks cancellation between frames, but within a single frame's 4-level fallback, there is no escape.

**Fix:** Add cancellation checks between each fallback level:

```rust
if cancel.is_cancelled() {
    return Err(VisualError::FrameExtraction("帧提取被取消".into()));
}
```

## Info

### IN-01: Regex patterns recompiled on every `render()` call

**File:** `src/prompt/template.rs:85-87,119-121`

**Issue:** Two `Regex::new()` calls compile the same patterns on every `render()` invocation. Since these are compile-time-constant patterns, use `std::sync::OnceLock<Regex>` or `lazy_static!` to compile once.

### IN-02: `builtin_filters()` allocates new `HashMap` on every `render()` call

**File:** `src/prompt/template.rs:10-57`

**Issue:** The 6-entry filter HashMap is recreated on every template render. Use `OnceLock` for lazy one-time initialization.

### IN-03: `script_generation` v2.0 silently overrides v1.0 default status

**File:** `src/prompt/register.rs:131-144`

**Issue:** Both `script_generation` v1.0 (line 115) and v2.0 (line 131) are registered with `is_default: true`. The second registration silently overrides the first's default. This is intentional (v2.0 should be the default), but the behavior is implicit. A comment would make the intent clear.

### IN-04: `validate_narration_script` paragraph validation may reject valid single-paragraph scripts

**File:** `src/prompt/validators.rs:74-83`

**Issue:** The validator requires at least 3 paragraphs (split by `\n\n`). Some LLM outputs may produce valid narration as a single continuous paragraph without double-newline separators. This strict requirement could cause false rejection of valid outputs. The validation is intentionally strict per the spec, but the 3-paragraph minimum is a heuristic that may need tuning based on actual LLM output patterns.

---

_Reviewed: 2026-05-07T12:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
