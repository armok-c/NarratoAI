---
phase: 07-sde-pipeline
reviewed: 2026-05-06T12:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - src/sde/error.rs
  - src/sde/mod.rs
  - src/sde/timestamp.rs
  - src/sde/subtitle.rs
  - src/sde/types.rs
  - src/sde/script_gen.rs
  - src/sde/pipeline.rs
  - src/documentary/pipeline.rs
findings:
  critical: 0
  warning: 5
  info: 1
  total: 6
status: issues_found
---

# Phase 7: Code Review Report (Sixth Pass)

**Reviewed:** 2026-05-06T12:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Sixth-pass review of the SDE pipeline module. All 15 previously-fixed findings (across 5 prior passes) remain resolved. The SDE module itself (`src/sde/`) is clean -- no new issues found in the 7 SDE-specific files.

New findings are concentrated in `src/documentary/pipeline.rs`, which was added to the review scope this pass. This file shares the same FFmpeg concat and composite patterns as the SDE pipeline but lacks several security hardening fixes that were applied only to the SDE side. Additionally, one minor issue remains in the SDE pipeline.

## Warnings

### WR-01: Unsantized subtitle_font in documentary composite step

**File:** `src/documentary/pipeline.rs:303-304`
**Issue:** The `subtitle_font` value is inserted directly into the FFmpeg `force_style` string without sanitization. If the font name contains special characters (quotes, semicolons, backslashes), this could break the FFmpeg filter graph or enable injection of arbitrary filter parameters. The SDE pipeline (`src/sde/pipeline.rs:440-444`) sanitizes the font to alphanumeric + space + dash + underscore, but this protection was not backported to the documentary pipeline.
**Fix:**
```rust
if let Some(ref font) = request.subtitle_font {
    let sanitized: String = font
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    if !sanitized.is_empty() {
        style.push_str(&format!(",FontName={}", sanitized));
    }
}
```

### WR-02: No newline sanitization in documentary concat file paths

**File:** `src/documentary/pipeline.rs:187`
**Issue:** Video file paths are written to the FFmpeg concat list file without checking for embedded newline characters. A path containing `\n` or `\r` would allow injection of arbitrary file directives into the concat list, potentially causing FFmpeg to process unintended files. The SDE pipeline (`src/sde/pipeline.rs:304-308`) validates against newlines, but the documentary pipeline does not.
**Fix:**
```rust
let path_str = video_path.to_string_lossy().replace('\\', "/");
if path_str.contains('\n') || path_str.contains('\r') {
    return Err(PipelineError::Concat {
        details: format!("video path contains illegal characters for clip {}", clip._id),
    });
}
let escaped = path_str.replace("'", "'\\''");
concat_content.push_str(&format!("file '{}'\n", escaped));
```

### WR-03: Sync std::fs::write in async documentary concat step

**File:** `src/documentary/pipeline.rs:195`
**Issue:** `std::fs::write` is called in the async function `step_concat`, blocking the Tokio runtime thread during disk I/O. The equivalent call in the SDE pipeline was fixed to `tokio::fs::write` in Pass 5 (WR-02), but this fix was not applied to the documentary pipeline.
**Fix:**
```rust
tokio::fs::write(&concat_list_path, &concat_content)
    .await
    .map_err(|e| PipelineError::Io(e))?;
```

### WR-04: Sync write_srt_file called from async SDE TTS loop

**File:** `src/sde/pipeline.rs:184`
**Issue:** `write_srt_file` (which uses `std::fs::write` internally) is called from the async TTS generation loop. Each call blocks the Tokio runtime thread. While individual SRT files are small and the blocking duration is brief, this is inconsistent with the project's pattern of using `tokio::fs::write` for async I/O in the pipeline.
**Fix:** Either wrap the call in `tokio::task::spawn_blocking`, or refactor `write_srt_file` to provide an async variant:
```rust
tokio::fs::write(&srt_path, &srt_content)
    .await
    .map_err(|e| SdeError::Io { source: e })?;
```

### WR-05: Documentary tts_volume=0 allowed (silences TTS output)

**File:** `src/documentary/types.rs:65`
**Issue:** The documentary `DocumentaryRequest::validate()` uses `self.tts_volume < 0.0`, which allows `tts_volume = 0.0`. A zero TTS volume produces silent narration output with no user-visible error, which is almost certainly unintentional. The SDE `SdeRequest::validate()` correctly rejects this with `self.tts_volume <= 0.0` (fixed in Pass 2).
**Fix:**
```rust
if self.tts_volume <= 0.0 || self.tts_volume > 10.0 {
    return Err(format!("tts_volume 超出有效范围 (0, 10]: {}", self.tts_volume));
}
```

## Info

### IN-01: Redundant "播放原片" check in SDE TTS loop

**File:** `src/sde/pipeline.rs:156-158`
**Issue:** The check `clip.narration.starts_with("播放原片") && clip.ost == OstType::OriginalSound` on line 156 is a strict subset of the check `clip.ost == OstType::OriginalSound` on line 160. The first condition never triggers independently -- any clip matching line 156 also matches line 160. The first check is dead code that adds a redundant branch.
**Fix:** Remove lines 155-158 (the first `if` block). The second check on line 160-162 already handles all OST=1 clips correctly.

---

_Reviewed: 2026-05-06T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
