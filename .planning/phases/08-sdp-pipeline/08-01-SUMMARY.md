---
phase: 08-sdp-pipeline
plan: 01
subsystem: subtitle
tags: [refactor, extraction, shared-module]
requires: []
provides: [src/subtitle/]
affects: [src/sde/]
tech-stack:
  added:
    - src/subtitle/ as new public module
  patterns:
    - SubtitleError reuses SdeError::ParseSubtitle pattern with thiserror
    - Re-export pattern: sde/types.rs uses `pub use crate::subtitle::SubtitleSegment`
    - sde/timestamp.rs re-exports from subtitle/timestamp.rs
key-files:
  created:
    - src/subtitle/mod.rs (module entry, pub use re-exports)
    - src/subtitle/error.rs (SubtitleError enum, thiserror, Chinese error messages)
    - src/subtitle/types.rs (SubtitleSegment struct)
    - src/subtitle/parser.rs (detect_encoding, normalize_subtitle_text, parse_subtitle_file, all helpers)
    - src/subtitle/timestamp.rs (parse_srt_timestamp, find_precise_range)
  modified:
    - src/sde/types.rs (SubtitleSegment replaced with re-export)
    - src/sde/timestamp.rs (impls replaced with re-export)
    - src/sde/error.rs (added From<SubtitleError> for SdeError)
    - src/sde/pipeline.rs (import path crate::sde::subtitle → crate::subtitle::parser)
    - src/sde/mod.rs (removed pub mod subtitle)
    - src/lib.rs (added pub mod subtitle)
  deleted:
    - src/sde/subtitle.rs (all code migrated to src/subtitle/parser.rs)
decisions:
  - SubtitleError is independent from SdeError; SdeError gains From<SubtitleError> for automatic ? conversion
  - sde/timestamp.rs and sde/types.rs retain re-exports so existing SDE callers need zero import changes
  - Private helper functions (extract_text_from_srt, extract_text_from_ass) remain in parser.rs
    for future SDP pipeline use despite current dead_code warnings
metrics:
  duration: ~8 minutes
  completed: "2026-05-06"
  subtitle_tests: 55 passed
  sde_tests: 54 passed
---

# Phase 08 Plan 01: Subtitle Module Extraction Summary

## Objective

Extract subtitle parsing from SDE module into a standalone `src/subtitle/` public module, shared by SDE (Phase 7) and SDP (Phase 8) pipelines.

## One-liner

Subtitle parsing extracted from `src/sde/subtitle.rs` into `src/subtitle/` as an independent public module with SubtitleError, SubtitleSegment, parser, and timestamp sub-modules, maintaining full backward compatibility via SDE re-exports.

## Tasks Executed

**Task 1: Create public subtitle module `src/subtitle/`** (c94f505)
- Created `error.rs`: SubtitleError enum with `ParseSubtitle` and `Io` variants, `thiserror` derives, Chinese error messages, `From<std::io::Error>` impl, and 4 tests
- Created `types.rs`: SubtitleSegment struct moved from `sde/types.rs`, with 2 tests
- Created `timestamp.rs`: `parse_srt_timestamp` and `find_precise_range` migrated from `sde/timestamp.rs`, error type changed from `SdeError` to `SubtitleError`, with 9 tests
- Created `parser.rs`: All parsing code migrated from `sde/subtitle.rs` — `detect_encoding`, `normalize_subtitle_text`, `parse_subtitle_file`, plus 12 private helper functions and 27 tests
- Created `mod.rs`: Module entry with public re-exports of all public types and functions

**Task 2: Update SDE module to use new paths + delete old file** (c13a4d9)
- `sde/types.rs`: Replaced `SubtitleSegment` struct definition with `pub use crate::subtitle::types::SubtitleSegment`
- `sde/timestamp.rs`: Replaced `parse_srt_timestamp`/`find_precise_range` implementations with re-exports
- `sde/error.rs`: Added `From<SubtitleError> for SdeError` conversion impl
- `sde/pipeline.rs`: Updated import from `crate::sde::subtitle::parse_subtitle_file` to `crate::subtitle::parser::parse_subtitle_file`
- `sde/mod.rs`: Removed `pub mod subtitle`
- `lib.rs`: Added `pub mod subtitle`
- `sde/subtitle.rs`: Deleted (731 lines, all migrated)

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo build -p narratoai-core`: Zero errors, only pre-existing warnings
- `cargo test -p narratoai-core -- subtitle`: 55/55 passed
- `cargo test -p narratoai-core -- sde`: 54/54 passed
- All acceptance criteria met

## Known Stubs

None.

## Threat Flags

No new security surface introduced — all migrated code existed prior in `sde/subtitle.rs`.
