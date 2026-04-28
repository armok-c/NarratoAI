---
phase: 01-foundation
fixed_at: 2026-04-28T08:35:00Z
review_path: .planning/phases/01-foundation/01-REVIEW.md
iteration: 2
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 01: Code Review Fix Report (Iteration 2)

**Fixed at:** 2026-04-28T08:35:00Z
**Source review:** .planning/phases/01-foundation/01-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-06: probe_video does not return BinaryNotFound when ffprobe binary is missing

**Files modified:** `src/ffmpeg/probe.rs`
**Commit:** dd97a89
**Applied fix:** Replaced the `.output().map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?` chain in `probe_video` (lines 25-44) with a match expression that checks `e.kind() == std::io::ErrorKind::NotFound`. When the ffprobe binary is not installed, the function now returns `Err(FFmpegError::BinaryNotFound)` instead of `Err(FFmpegError::SpawnFailed(...))`, making it consistent with the `detect_hw_encoders` pattern in `hwaccel.rs:131-143`.

**Verification:**
- Tier 1: Re-read modified lines 25-44, confirmed fix text present and surrounding code intact.
- Tier 2: `cargo check` passed with no errors in the modified file.

---

_Fixed: 2026-04-28T08:35:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
