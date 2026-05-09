---
phase: 10-tauri-command-layer
reviewed: 2026-05-09T18:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - Cargo.toml
  - narratoai-core/Cargo.toml
  - src-tauri/Cargo.toml
  - src-tauri/build.rs
  - src-tauri/tauri.conf.json
  - src-tauri/capabilities/default.json
  - src-tauri/src/main.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/error.rs
  - src-tauri/src/models/mod.rs
  - src-tauri/src/models/progress.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/pipeline.rs
  - src-tauri/src/commands/script.rs
  - src-tauri/src/commands/export.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sdp/types.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/jianying/builder.rs
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sdp/pipeline.rs
findings:
  critical: 1
  warning: 2
  info: 3
  total: 6
status: issues_found
---

# Phase 10: Tauri Command Layer - Code Review Report (Re-Review Iteration 4)

**Reviewed:** 2026-05-09T18:00:00Z
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Fourth re-review iteration. Previous iterations found and fixed CR-01 (path traversal in `generate_sdp_script`), WR-03 (silent zero duration for unparseable clips), and documented WR-01/WR-02 (lock holding across async).

Verification of previously reported issues:

| Previous Finding | Status | Assessment |
|-----------------|--------|------------|
| CR-01 path traversal in `generate_sdp_script` | FIXED | `validate_path()` call added at line 259 |
| WR-03 unparseable clips silent zero | FIXED | `unparseable_clips` count added to `ScriptInfo` |
| WR-01 lock holding (`run_sde`) | ACKNOWLEDGED | TODO comment at line 117 |
| WR-02 lock holding (`generate_sdp_script`) | ACKNOWLEDGED | TODO comment at line 284 |
| IN-01 scattered use statements | NOT FIXED | Style only |
| IN-02 `#[macro_export]` on crate-local macro | NOT FIXED | Style only |
| IN-03 unused SdeProgressStep/SdpProgressStep | NOT FIXED | Minor |

This iteration identified one new CRITICAL issue: the path traversal protection (`validate_path()`) is incomplete. While it was correctly added to commands accepting bare `PathBuf` parameters (`load_script`, `save_script`, `export_jianying_draft`, `generate_sdp_script`), four pipeline commands that accept request structs containing `PathBuf` fields (`run_documentary`, `run_sde`, `run_sdp`, `generate_documentary_script`) do NOT call `validate_path()` on any of the paths embedded in those structs. These structs are deserialized directly from IPC input, making all their path fields attacker-controlled.

## Critical Issues

### CR-01: Path traversal protection missing for request struct path fields

**File:** `src-tauri/src/commands/pipeline.rs:18-68, 75-136, 142-184, 190-244`
**Issue:** Four Tauri IPC commands accept request structs that are fully deserialized from frontend IPC input, but none of them validate path traversal on the `PathBuf` fields within those structs:

1. `run_documentary` (line 19) accepts `DocumentaryRequest` with fields: `video_path`, `script_path`, `bgm_path`, `output_dir` -- none validated
2. `run_sde` (line 76) accepts `SdeRequest` with fields: `subtitle_path`, `video_path`, `bgm_path`, `output_dir` -- none validated
3. `run_sdp` (line 143) accepts `SdpRequest` with fields: `subtitle_path`, `video_path`, `script_path`, `bgm_path`, `output_dir` -- none validated
4. `generate_documentary_script` (line 191) accepts `ScriptGenRequest` with field: `video_path` -- not validated

These paths are used to read files (video, subtitles, scripts), write output (output_dir), and read BGM files. A compromised or malicious frontend could send paths containing `..` sequences (e.g., `../../etc/passwd`) through these request structs to read or write arbitrary files on the host system.

The previous CR-01 fix correctly added `validate_path()` to commands with bare `PathBuf` parameters but missed the struct-based commands entirely.

**Fix:** Add `validate_path()` calls for all `PathBuf` fields in each request struct, at the top of each command function, before any core function calls. Create a helper to reduce repetition:

```rust
fn validate_request_paths(request: &DocumentaryRequest) -> Result<(), CommandError> {
    validate_path(&request.video_path)?;
    validate_path(&request.script_path)?;
    if let Some(ref bgm_path) = request.bgm_path {
        validate_path(bgm_path)?;
    }
    if let Some(ref output_dir) = request.output_dir {
        validate_path(output_dir)?;
    }
    Ok(())
}

// Then in each command:
pub async fn run_documentary(/* ... */) -> Result<CommandResponse, CommandError> {
    validate_request_paths(&request)?;
    // ... rest of command
}
```

Apply the same pattern for `SdeRequest`, `SdpRequest`, and `ScriptGenRequest`.

## Warnings

### WR-01: run_sde and generate_sdp_script hold tokio RwLock read guards across long async pipelines

**File:** `src-tauri/src/commands/pipeline.rs:117-131, 284-296`
**Issue:** Carried over from previous reviews with TODO comments added. Re-confirming still present:

- `run_sde` (lines 119-120): acquires `registry.read().await` and `prompt_manager.read().await`, then passes `&registry` and `&pm` to a 9-step pipeline that includes TTS and FFmpeg operations lasting potentially minutes.
- `generate_sdp_script` (line 287): acquires `prompt_manager.read().await`, then passes `&pm` to a 2-step SDP script generation with two sequential LLM API calls.

Both block write operations to their respective registries for the entire pipeline duration. The `generate_documentary_script` command correctly scopes the registry guard (lines 198-213), extracting `Arc<dyn LlmProvider>` before any `.await`. The same pattern should be applied where possible.

**Fix:** Acknowledged as requiring core API refactoring. For `run_sde`, the core API needs to accept pre-extracted providers. For `generate_sdp_script`, consider pre-rendering prompts in a scoped block if the core API can be adjusted.

### WR-02: get_script_info can produce negative total_duration_secs for malformed timestamps

**File:** `src-tauri/src/commands/script.rs:82-83`
**Issue:** The duration calculation at line 83 computes `end - start` without checking that `end >= start`. If a clip's timestamp has a reversed range (e.g., `"00:00:10,000-00:00:05,000"` where end < start), the result would be negative, causing `total_duration_secs` to decrease. This would produce an incorrect (possibly negative) total duration without any error or warning to the caller.

While the clip is not counted as `unparseable_clips` (both timestamps parse successfully), the resulting duration is invalid.

**Fix:** Add a guard for negative duration:
```rust
match (parse_time(start_ts), parse_time(end_ts)) {
    (Some(start), Some(end)) => {
        let dur = end - start;
        if dur < 0.0 {
            unparseable_clips += 1;
        } else {
            total_duration_secs += dur;
        }
    }
    _ => unparseable_clips += 1,
}
```

## Info

### IN-01: error.rs use statements scattered throughout file

**File:** `src-tauri/src/error.rs:38,48,58,68,78,88,98`
**Issue:** Each `impl From<T>` block has its own `use` statement immediately above it, rather than consolidating at the file top. This deviates from Rust convention. Retained from previous reviews as unfixed.

**Fix:** Consolidate all `use` statements at file top.

### IN-02: register_all_commands macro uses #[macro_export] but is crate-local only

**File:** `src-tauri/src/commands/mod.rs:14`
**Issue:** `#[macro_export]` exports the macro as public crate API, but the macro references `crate::commands::*` paths making it unusable from external crates. Retained from previous reviews as unfixed.

**Fix:** Remove `#[macro_export]` and use crate-local `macro_rules!` visibility.

### IN-03: SdeProgressStep and SdpProgressStep enums defined but unused in production code

**File:** `narratoai-core/src/sde/types.rs:107-118`, `narratoai-core/src/sdp/types.rs:103-108`
**Issue:** Both enums are only referenced in tests. Production code uses string literals for step names. Retained from previous reviews as unfixed.

**Fix:** Either implement `Display` and use in production code, or remove to reduce cognitive load.

---

_Reviewed: 2026-05-09T18:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
