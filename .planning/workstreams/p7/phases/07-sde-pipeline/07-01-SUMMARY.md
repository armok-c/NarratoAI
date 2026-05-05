---
phase: 07-sde-pipeline
plan: 01
subsystem: core
tags: [sde, error-handling, types, timestamp, rust, encoding-detection]

# Dependency graph
requires:
  - phase: 06-documentary-pipeline
    provides: DocumentaryRequest field alignment, PipelineError conversion, ProgressStep pattern
provides:
  - SdeError enum with 7 variants and Chinese Display messages
  - SdeRequest with 19 fields aligned to DocumentaryRequest plus drama_name/temperature
  - SdePipelineState with 16 fields tracking 9-step pipeline progress
  - SdeProgressStep enum covering 9 stages from ParseSubtitle to Composite
  - SubtitleSegment struct for parsed SRT entry representation
  - parse_srt_timestamp() for HH:MM:SS,mmm/.mmm parsing with strict validation
  - find_precise_range() for LLM time range to subtitle segment alignment (+/-1s tolerance)
  - encoding_rs 0.8 dependency for subtitle file encoding detection
affects: [07-02, 07-03, 07-04]

# Tech tracking
tech-stack:
  added: [encoding_rs 0.8]
  patterns: [thiserror enum derivation for error types, DocumentaryRequest field alignment for request types, documentary PipelineState pattern for state management]

key-files:
  created:
    - src/sde/error.rs
    - src/sde/types.rs
    - src/sde/timestamp.rs
    - src/sde/mod.rs
    - src/sde/subtitle.rs (stub)
    - src/sde/script_gen.rs (stub)
    - src/sde/pipeline.rs (stub with run_sde async fn)
    - src/sde/clip.rs (stub)
    - src/sde/audio.rs (stub)
  modified:
    - src/lib.rs (added pub mod sde)
    - Cargo.toml (added encoding_rs dependency)

key-decisions:
  - "SdeProgressCallback defined locally (not reusing documentary ProgressCallback) because callback type is bound to SdeProgressStep, which has 9 variants vs documentary's 6"
  - "Subtitle segment spacing set to 2s gap for reliable find_precise_range testing with +/-1s tolerance"

patterns-established:
  - "SdeError follows documentary PipelineError pattern — thiserror enum with Chinese Display messages"
  - "SdeRequest.validate() follows DocumentaryRequest.validate() pattern for parameter validation"

requirements-completed: [SDE-01, SDE-02]

# Metrics
duration: 12min
completed: 2026-05-05
---

# Phase 07 Plan 01: SDE Module Core Infrastructure

**SDE module skeleton with error types, request/state/segment data structures, SRT timestamp parser, and LLM-to-subtitle alignment function, all aligned to the existing documentary pipeline patterns**

## Performance

- **Duration:** 12 minutes
- **Started:** 2026-05-05T16:30:00Z
- **Completed:** 2026-05-05T16:42:00Z
- **Tasks:** 1 (single task with all 6 files + stubs)
- **Files modified:** 11 (9 new + 2 modified)

## Accomplishments

- SdeError enum with 7 variants (ParseSubtitle, PlotAnalysis, ScriptGeneration, JsonRepair, VideoProcess with From<PipelineError>, Validation, Io) — all with Chinese Display messages and Send auto-derivation
- SdeRequest with 19 fields including all documentary video processing parameters plus subtitle_path, drama_name, and temperature; Default and validate() implementations with full range checks
- SdeProgressStep enum with 9 variants (ParseSubtitle through Composite) matching AI-SPEC definition
- SubtitleSegment struct with index, start_secs, end_secs, and text fields
- SdePipelineState with 16 fields, new() constructor, and emit_progress() following documentary PipelineState pattern
- parse_srt_timestamp() supporting HH:MM:SS,mmm and HH:MM:SS.mmm formats with strict range validation (h 0-23, m 0-59, s 0-59, ms 0-999)
- find_precise_range() with +/-1 second tolerance for matching LLM time ranges to subtitle segments
- 9-file module skeleton with stub modules for future plans (subtitle, script_gen, pipeline, clip, audio)
- 20 unit tests across error.rs (5), types.rs (5), and timestamp.rs (10) — all passing

## Task Commits

1. **Task 1: Create SDE error types + data structures + timestamp module + module entry + lib declaration + dependency** - `b4f318f` (feat)

## Files Created/Modified
- `src/sde/error.rs` - SdeError enum with 7 variants, From<PipelineError>, From<std::io::Error>, Chinese Display messages, 5 unit tests
- `src/sde/types.rs` - SdeRequest, SdePipelineState, SdeProgressStep, SubtitleSegment, SdeProgressCallback, 5 unit tests
- `src/sde/timestamp.rs` - parse_srt_timestamp(), find_precise_range(), 10 unit tests covering valid/invalid/tolerance/no-match cases
- `src/sde/mod.rs` - Module declarations for all 8 sub-modules + re-exports of SdeError, SdeRequest, SdePipelineState, SdeProgressStep, SubtitleSegment, run_sde
- `src/sde/subtitle.rs` - Stub for future plan 07-03
- `src/sde/script_gen.rs` - Stub for future plan 07-03
- `src/sde/pipeline.rs` - Stub with async run_sde() function for future plan 07-04
- `src/sde/clip.rs` - Stub for future plan 07-04
- `src/sde/audio.rs` - Stub for future plan 07-04
- `src/lib.rs` - Added `pub mod sde;` after documentary module
- `Cargo.toml` - Added `encoding_rs = "0.8"` dependency

## Decisions Made

- Used local `SdeProgressCallback` type instead of reusing documentary's `ProgressCallback`. The documentary callback is bound to 6-variant `ProgressStep`, while SDE needs 9-variant `SdeProgressStep`. A dedicated type avoids type mismatch and is cleaner.
- Subtitle segment test data spaced 2 seconds apart to reliably test `find_precise_range` tolerance behavior (the original 0.5s spacing caused adjacent segments to be picked up within +/-1s tolerance).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Local SdeProgressCallback type**
- **Found during:** Task 1 (types.rs creation)
- **Issue:** Plan specified `progress: Option<crate::documentary::types::ProgressCallback>` but the documentary ProgressCallback takes `ProgressStep` (6 variants), while emit_progress needs `SdeProgressStep` (9 variants). Would cause type mismatch.
- **Fix:** Defined local `pub type SdeProgressCallback = Box<dyn Fn(SdeProgressStep, f32, &str) + Send + Sync>` in types.rs and used it in SdePipelineState.
- **Files modified:** src/sde/types.rs
- **Verification:** cargo check --lib passes
- **Committed in:** b4f318f (Task 1 commit)

**2. [Rule 3 - Blocking] Precedence error in timestamp.rs**
- **Found during:** First cargo check run
- **Issue:** `10u32.pow(millis.len() as u32.min(3))` — Rust precedence causes `as u32` to apply before `.min(3)`, leading to `cannot cast followed by method call` error.
- **Fix:** Added parentheses: `10u32.pow((millis.len() as u32).min(3))`
- **Files modified:** src/sde/timestamp.rs
- **Verification:** cargo check --lib passes
- **Committed in:** b4f318f (Task 1 commit)

**3. [Rule 1 - Bug] Test segment spacing too tight for tolerance**
- **Found during:** cargo test --lib sde::timestamp
- **Issue:** Original segments spaced 0.5s apart (0-5, 5.5-10, 10.5-15), causing the "exact match" test to pick up all 3 segments within +/-1s tolerance instead of just the target segment.
- **Fix:** Respaced segments to 2s gaps (0-5, 7-12, 14-19) and updated test expectations.
- **Files modified:** src/sde/timestamp.rs
- **Verification:** All 10 timestamp tests pass
- **Committed in:** b4f318f (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 missing critical, 1 blocking, 1 bug)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered

- Required initial resetting the worktree branch from commit 66a85db to 0f61f65 (the correct base) as the worktree was created from an older base. Standard worktree setup procedure.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- SDE module skeleton is complete and compilable. Types, errors, and timestamp utilities are ready for downstream consumption.
- Plan 07-02 will build on this skeleton with subtitle file loading and encoding detection.
- Plan 07-03 will implement subtitle parsing, LLM-based plot analysis, and script generation.
- Plan 07-04 will implement the full 9-step pipeline orchestration.
- 5 stub modules (subtitle, script_gen, pipeline, clip, audio) await implementation in future plans.

## Self-Check: PASSED

| Check | Status |
|-------|--------|
| cargo check --lib | PASSED (no errors) |
| cargo test --lib sde::error (5 tests) | PASSED |
| cargo test --lib sde::types (5 tests) | PASSED |
| cargo test --lib sde::timestamp (10 tests) | PASSED |
| grep -c "encoding_rs" Cargo.toml (=1) | PASSED |
| grep -c "pub mod sde" src/lib.rs (=1) | PASSED |
| grep -c "SdeError" src/sde/mod.rs (>=1) | PASSED |
| grep -c "SdeRequest" src/sde/mod.rs (>=1) | PASSED |
| All 9 SDE module files exist | PASSED |
| No unexpected deletions in commit | PASSED |

---
*Phase: 07-sde-pipeline*
*Completed: 2026-05-05*
