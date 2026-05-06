---
phase: 07-sde-pipeline
iteration: 8
fix_scope: critical_warning
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 07: Code Review Fix Report

**Fixed at:** 2026-05-06T16:30:00+08:00
**Source review:** .planning/phases/07-sde-pipeline/07-REVIEW.md
**Iteration:** 8

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-08: Sync `write_srt_file` in async `merge_subtitle_files`

**Files modified:** `src/documentary/audio.rs`
**Commit:** `31e93c7`
**Applied fix:** Replaced synchronous `crate::documentary::subtitle::write_srt_file(&merged, &output_path)?;` (line 170) with async `tokio::fs::write(&output_path, &merged).await.map_err(PipelineError::from)?;`. The `write_srt_file` function internally uses `std::fs::write`, which blocks the Tokio runtime thread during disk I/O. The direct `tokio::fs::write` call eliminates this blocking. `PipelineError` already has `From<std::io::Error>` for correct error mapping. This mirrors the seventh-pass WR-01 fix in `documentary/pipeline.rs` step_tts.

### WR-09: Sync `std::fs::create_dir_all` in async functions

**Files modified:** `src/sde/pipeline.rs`, `src/documentary/pipeline.rs`
**Commit:** `b36c82f`
**Applied fix:** Replaced `std::fs::create_dir_all(&task_dir)?;` with `tokio::fs::create_dir_all(&task_dir).await?;` at four locations:
- `src/sde/pipeline.rs:52` (inside `run_sde` async function)
- `src/sde/pipeline.rs:701` (inside `analyze_subtitle_plot` async function)
- `src/sde/pipeline.rs:734` (inside `generate_sde_script` async function)
- `src/documentary/pipeline.rs:484` (inside `run_documentary` async function)

Both `SdeError` and `PipelineError` implement `From<std::io::Error>`, so the `?` operator correctly maps errors without additional conversion.

---

_Fixed: 2026-05-06T16:30:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 8_
