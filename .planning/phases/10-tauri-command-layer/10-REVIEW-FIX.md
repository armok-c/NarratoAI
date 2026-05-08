---
phase: 10-tauri-command-layer
fix_date: 2026-05-08T14:30:00Z
review_path: .planning/phases/10-tauri-command-layer/10-REVIEW.md
fix_scope: critical_warning
iteration: 1
findings_in_scope: 9
fixed: 8
skipped: 1
status: partial
---

# Phase 10: Code Review Fix Report

**Fixed at:** 2026-05-08T14:30:00Z
**Source review:** .planning/phases/10-tauri-command-layer/10-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 9
- Fixed: 8
- Skipped: 1

## Fixed Issues

### CR-01: Content Security Policy is completely disabled

**Files modified:** `src-tauri/tauri.conf.json`
**Commit:** `a559cc0`
**Applied fix:** Replaced `"csp": null` with a restrictive CSP policy: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https: data:; media-src 'self' asset: https:; connect-src 'self' https:; font-src 'self'`. The `unsafe-inline` for `style-src` is retained for frontend framework compatibility.

### CR-02: generate_documentary_script uses text LLM provider for vision analysis

**Files modified:** `narratoai-core/src/documentary/script_gen.rs`, `src-tauri/src/commands/pipeline.rs`
**Commit:** `56f1a57`
**Applied fix:** Changed `generate_documentary_script()` signature to accept `vision_provider: &dyn LlmProvider` and `text_provider: &dyn LlmProvider` as separate parameters. The Tauri command now resolves both providers from the registry, using `request.vision_model`/`request.text_model` with fallback to `config.app.vision_llm_provider`/`config.app.text_llm_provider`. Vision analysis uses `vision_provider`, narration generation uses `text_provider`.

### WR-01: save_script and load_script accept arbitrary file paths without validation

**Files modified:** `src-tauri/src/commands/script.rs`
**Commit:** `cae4676`
**Applied fix:** Added `.json` extension validation to both `load_script` and `save_script`. Paths without a `.json` extension now return a `CommandError` with code `INVALID_PATH`.

### WR-02: ScriptGenRequest.progress is always None via serde(skip)

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** `6fd5f50`
**Applied fix:** Constructed a progress callback in the `generate_documentary_script` Tauri command that emits `pipeline-progress` events via `app.emit()`. The callback is set on the request before passing to the core function, matching the pattern used by `run_documentary`, `run_sde`, and `run_sdp`.

### WR-03: generate_sdp_script creates a temp directory that is never cleaned up

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** `4fd8f31`
**Applied fix:** Changed the `generate_sdp_script` call to store the result (without `?`), then unconditionally clean up the temp directory with `tokio::fs::remove_dir_all`, then apply `?` to the stored result. This ensures cleanup happens on both success and error paths.

### WR-04: From<String> for CommandError is overly broad

**Files modified:** `src-tauri/src/error.rs`, `src-tauri/src/lib.rs`
**Commit:** `0d68fbb`
**Applied fix:** Removed the blanket `From<String> for CommandError` impl. Updated `find_config()` in `lib.rs` to return `Result<PathBuf, Box<dyn std::error::Error>>` instead of `Result<PathBuf, String>`, which is compatible with the Tauri `setup` closure return type. All existing call sites were verified to use domain-specific `From<T>` conversions rather than the removed blanket impl.

### WR-06: Proxy support is hardcoded to None in all Tauri pipeline commands

**Files modified:** `src-tauri/src/commands/pipeline.rs`
**Commit:** `e40da2a`
**Applied fix:** Replaced hardcoded `None` proxy parameter in `run_documentary` and `run_sde` with `config.proxy.enabled.then_some(&config.proxy)`, which extracts the proxy section from the injected `AppConfig` only when proxy is enabled.

### WR-07: CommandResponse step_durations and intermediate_paths are always empty

**Files modified:** `src-tauri/src/models/progress.rs`, `src-tauri/src/commands/pipeline.rs`
**Commit:** `93b0f28`
**Applied fix:** Removed the unused `step_durations` and `intermediate_paths` fields from `CommandResponse`, and removed the `StepDuration` struct. All three `CommandResponse` construction sites in `pipeline.rs` were updated to only set `output_video_path`.

## Skipped Issues

### WR-05: run_sde holds both Registry and PromptManager read locks across entire long-running pipeline

**File:** `src-tauri/src/commands/pipeline.rs:117-127`
**Reason:** Requires modifying the core library's `run_sde()` function signature (in `narratoai-core/src/sde/pipeline.rs`) to accept pre-resolved `Arc<dyn LlmProvider>` and `Arc<PromptManager>` instead of `&Registry` and `&PromptManager`. This would require updating all internal step functions (`step_analyze_plot`, `step_generate_script`) that accept `prompt_manager` references, changing the API contract across module boundaries. The scope of change is too large for an automated fix and risks breaking other consumers of the core library. This should be addressed in a dedicated refactoring pass.
**Original issue:** The `run_sde` command acquires read guards on both `registry` and `prompt_manager`, then holds them across the entire long-running pipeline call. No other command can acquire a write lock on either during this time.

---

_Fixed: 2026-05-08T14:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
