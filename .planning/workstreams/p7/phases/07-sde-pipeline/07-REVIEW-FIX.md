---
phase: 07-sde-pipeline
fixed_at: 2026-05-06T17:00:00Z
review_path: .planning/workstreams/p7/phases/07-sde-pipeline/07-REVIEW.md
iteration: 2
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 07: Code Review Fix Report (Second Pass)

**Fixed at:** 2026-05-06T17:00:00Z
**Source review:** .planning/workstreams/p7/phases/07-sde-pipeline/07-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 7 (2 Critical, 5 Warning)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: `strip_code_fence` corrupts `json5`-tagged LLM output

**Files modified:** `src/sde/script_gen.rs`
**Commit:** 7c69585
**Applied fix:** Reordered the `starts_with` checks in `strip_code_fence` so `json5` is checked before `json`. This prevents the `"json5"` tag from being partially stripped to `"5"`, which would corrupt the extracted JSON content.

### CR-02: FFmpeg `amix` filter produces silent output when `tts_volume=0.0`

**Files modified:** `src/sde/types.rs`
**Commit:** 7e48409
**Applied fix:** Changed `tts_volume` validation from `!(0.0..=10.0).contains()` to `tts_volume <= 0.0 || tts_volume > 10.0`, rejecting zero volume. This prevents the user from inadvertently producing a completely silent video.

### WR-01: Redundant range check in `voice_rate` validation

**Files modified:** `src/sde/types.rs`
**Commit:** 7e48409
**Applied fix:** Simplified the `voice_rate` validation from `!(0.0..=5.0).contains(&self.voice_rate) || self.voice_rate <= 0.0` to the equivalent single expression `self.voice_rate <= 0.0 || self.voice_rate > 5.0`.

### WR-02: Duplicated repair steps in `repair_json` code paths

**Files modified:** `src/sde/script_gen.rs`
**Commit:** 692e834
**Applied fix:** Extracted the duplicated Steps 3-6 (extract JSON object, fix double braces, fix trailing commas, fix single quotes) into a new `apply_repair_steps()` helper function. Both the "code fence found" and "no code fence" branches now call this helper, eliminating ~70 lines of duplicated logic. All 28 existing unit tests pass.

### WR-03: Unguarded `unwrap()` in `find_precise_range`

**Files modified:** `src/sde/timestamp.rs`
**Commit:** 579e8b8
**Applied fix:** Replaced bare `.unwrap()` calls on `matched.first()` and `matched.last()` with `.expect("checked non-empty above")` to provide a clear message if the invariant is ever violated during refactoring.

### WR-04: Regex compiled on every call in `has_srt_timecodes` and `normalize_subtitle_text`

**Files modified:** `src/sde/subtitle.rs`
**Commit:** 26fd4bd
**Applied fix:** Introduced two `std::sync::LazyLock<Regex>` statics (`SRT_TIMECODE_RE` and `MILLIS_SEP_RE`) to compile regexes once at first use instead of on every function call.

### WR-05: Magic number `20` in `has_meaningful_content` threshold

**Files modified:** `src/sde/subtitle.rs`
**Commit:** 26fd4bd
**Applied fix:** Extracted the magic number `20` into a named constant `MIN_MEANINGFUL_CONTENT_CHARS` with documentation explaining its purpose and rationale.

## Verification

- `cargo check`: Compiles with 0 errors (4 pre-existing warnings unrelated to fixes)
- `cargo test --lib sde::script_gen`: 28 passed
- `cargo test --lib sde::timestamp`: 12 passed
- `cargo test --lib sde::subtitle`: 30 passed
- All modified files verified by re-reading and syntax check

---

_Fixed: 2026-05-06T17:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
