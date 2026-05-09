---
phase: 10-tauri-command-layer
reviewed: 2026-05-09T12:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - Cargo.toml
  - narratoai-core/Cargo.toml
  - narratoai-core/src/documentary/pipeline.rs
  - narratoai-core/src/documentary/script_gen.rs
  - narratoai-core/src/documentary/types.rs
  - narratoai-core/src/jianying/builder.rs
  - narratoai-core/src/sde/pipeline.rs
  - narratoai-core/src/sde/types.rs
  - narratoai-core/src/sdp/pipeline.rs
  - narratoai-core/src/sdp/types.rs
  - src-tauri/Cargo.toml
  - src-tauri/build.rs
  - src-tauri/capabilities/default.json
  - src-tauri/src/commands/export.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/pipeline.rs
  - src-tauri/src/commands/script.rs
  - src-tauri/src/error.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/main.rs
  - src-tauri/src/models/mod.rs
  - src-tauri/src/models/progress.rs
  - src-tauri/tauri.conf.json
findings:
  critical: 2
  warning: 6
  info: 3
  total: 11
status: issues_found
---

# Phase 10: Code Review Report (Iteration 5)

**Reviewed:** 2026-05-09
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Reviewed the full Tauri 2.0 command layer and the underlying narratoai-core pipeline modules. This is iteration 5 of the review cycle (iterations 1-4 already addressed prior issues including CR-01 path traversal validation and WR-02 negative duration guard). Remaining issues are concentrated in two areas: (1) path traversal validation in the Tauri command layer is incomplete, allowing bypass via Windows drive letters or percent-encoded variants, and (2) long-lived RwLock read guards held across entire pipeline executions block all concurrent operations. Several additional quality issues were identified in cross-file logic.

## Critical Issues

### CR-02: `validate_path` can be bypassed on Windows via absolute paths and embedded dot-dot in canonicalized form

**File:** `src-tauri/src/error.rs:24-33`
**Issue:** The `validate_path` function only performs a string-level check for `..` in the lossy path representation. This can be bypassed in several ways on Windows:

1. **Canonical path bypass**: A path like `C:\Users\public\..\..\Windows\System32\config` where the attacker constructs a path whose canonical form escapes but whose string form varies. While `..` is present in this example, the more subtle case is paths with mixed separators (`/../`) or URL-encoded variants (`%2e%2e%5c`) that may be normalized downstream before reaching filesystem APIs.

2. **No canonicalization or containment check**: The function does not verify that the resolved path stays within an expected root directory. Any valid path without literal `..` passes validation, including paths pointing to arbitrary system locations (`C:\Windows\System32\*`, `/etc/shadow`). For a desktop application that processes user-supplied paths, this means the IPC boundary accepts paths to any file on the system.

3. **Missing validation in `export_jianying_draft`**: The `draft_name` parameter (line 16 of `export.rs`) is not validated for path traversal. While `validate_draft_name` in `builder.rs` does check for `..`, `/`, `\`, and `\0`, this validation occurs *inside* the core library -- the Tauri command layer in `export.rs` does not call `validate_path` on the `draft_name` string because it is not a `PathBuf`. If `validate_draft_name` were ever bypassed or removed, the command layer would have no defense.

**Fix:**
```rust
// error.rs - strengthen validate_path
pub fn validate_path(path: &std::path::PathBuf) -> Result<(), CommandError> {
    let path_str = path.to_string_lossy();
    // Block traversal sequences (including mixed separators)
    let normalized = path_str.replace('\\', "/");
    if normalized.contains("..") {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "路径不允许包含 '..' 遍历序列".into(),
        });
    }
    Ok(())
}

// export.rs - also validate draft_name at the command boundary
#[tauri::command]
pub async fn export_jianying_draft(
    script: Script,
    video_origin_path: PathBuf,
    draft_path: PathBuf,
    draft_name: String,
    width: u32,
    height: u32,
) -> Result<PathBuf, CommandError> {
    validate_path(&video_origin_path)?;
    validate_path(&draft_path)?;
    // Validate draft_name at IPC boundary too
    if draft_name.contains('/') || draft_name.contains('\\')
        || draft_name.contains("..") || draft_name.contains('\0')
    {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "草稿名称包含非法字符".into(),
        });
    }
    // ... rest unchanged
}
```

### CR-03: RwLock read guards held across entire SDE pipeline execution block all concurrent users

**File:** `src-tauri/src/commands/pipeline.rs:153-154`
**Issue:** In `run_sde`, both `registry.read().await` and `prompt_manager.read().await` are acquired at line 153-154 and held through the entire `run_sde()` call which spans 9 pipeline steps including TTS generation, video clipping, FFmpeg encoding, and LLM calls. This can take minutes. During that time, no other Tauri command can write to either the registry or prompt manager (e.g., dynamic provider reconfiguration is blocked). The TODO comment at line 151 acknowledges this but the code shipped in production this way.

The same pattern appears in `generate_sdp_script` at line 323 where `prompt_manager.read().await` is held across two sequential LLM calls plus file I/O.

Note: `generate_documentary_script` at lines 234-249 correctly scopes the registry read guard in a block, extracting `provider` before the `.await` -- this is the correct pattern and should be replicated.

**Fix:**
```rust
// Extract providers from registry in a scoped block before the pipeline
let text_provider = {
    let guard = registry.read().await;
    guard.get(&config.app.text_llm_provider).map_err(|e| CommandError {
        code: "PROVIDER_NOT_FOUND".into(),
        message: format!("获取 LLM provider 失败: {}", e),
    })?
};
// guard dropped here -- registry unlocked

// For prompt_manager, pre-render both prompts before the pipeline
// This requires core API changes to accept pre-rendered prompt strings
// instead of &PromptManager. Short-term: at minimum, document the lock
// duration in comments and add a timeout.
```

## Warnings

### WR-01: `get_script_info` uses fragile timestamp parsing that splits on all `-` characters

**File:** `src-tauri/src/commands/script.rs:62`
**Issue:** The `get_script_info` function splits timestamps on `-` with `ts.split('-').collect()`. A timestamp like `00:00:05,000-00:00:10,000` works, but a negative-value timestamp or malformed input with multiple `-` signs would produce more than 2 parts and be silently counted as `unparseable_clips`. This is the same class of bug that was identified and fixed in the core library using `splitn(2, '-')`. The command layer should use the same robust approach.

**Fix:**
```rust
let parts: Vec<&str> = ts.splitn(2, '-').collect();
```

### WR-02: `script_gen.rs` `ScriptGenRequest` has `#[serde(skip)]` on `progress` but is received from IPC

**File:** `narratoai-core/src/documentary/script_gen.rs:22-23`
**Issue:** `ScriptGenRequest` derives `Serialize, Deserialize` but marks the `progress` field with `#[serde(skip)]`. This is correct for the deserialization path (the frontend will not send a progress callback). However, in `src-tauri/src/commands/pipeline.rs:270-271`, the code does `request.progress = Some(progress_callback)` after deserialization, which means the field is set post-deserialize. This works correctly today. However, if the `ScriptGenRequest` is ever serialized back (e.g., for debugging or logging), the progress callback will be silently dropped. This is a latent design concern rather than an active bug.

**Fix:** Document the intent with a comment on the field, or consider using a separate IPC-facing struct without the progress field.

### WR-03: `strip_and_repair_json` trailing comma replacement is overly aggressive

**File:** `narratoai-core/src/documentary/script_gen.rs:354`
**Issue:** The trailing comma repair `text.replace(",}", "}").replace(",]", "]")` replaces ALL occurrences of `,}` and `,]` in the entire text, including those inside string literals. For example, a JSON string value containing `",}"` would be corrupted. This is a low-probability issue because LLM output rarely contains these exact sequences in string values, but it is a correctness defect.

**Fix:** Use a proper JSON repair approach that only operates outside of string contexts, or apply the replacement only to the outermost structural positions.

### WR-04: `blocking_write` called during Tauri `setup` could deadlock if setup is called from async context

**File:** `src-tauri/src/lib.rs:40`
**Issue:** `registry.blocking_write()` is called inside the Tauri `setup` closure. In Tauri 2.0, the `setup` closure runs synchronously during app initialization, so this is technically safe. However, if the tokio runtime is already active (which it is -- Tauri creates one), calling `blocking_write()` on a `tokio::sync::RwLock` from within an async runtime context can cause issues if the runtime's thread pool is exhausted. This is a known footgun with `tokio::sync::RwLock::blocking_write()`.

**Fix:** Prefer `std::sync::RwLock` for initialization-time-only data that is accessed synchronously during setup, or restructure to use `block_on` with `write().await`.

### WR-05: Duplicated FFmpeg command execution code across 5 pipeline modules

**Files:** `narratoai-core/src/documentary/pipeline.rs`, `narratoai-core/src/sde/pipeline.rs`, `narratoai-core/src/sdp/pipeline.rs`
**Issue:** The FFmpeg concat and composite logic (spawn child, iterate events, check errors, check exit status) is copy-pasted across at least 5 separate functions. Each copy has the same pattern but slight variations (error type wrapping). This duplication makes it easy for bug fixes to be applied to one copy but not others.

**Fix:** Extract a shared helper function for the "spawn FFmpeg child, collect errors, check status" pattern.

### WR-06: `generate_sdp_script` Tauri command cleans up task_dir but `run_sde` and `run_documentary` do not

**File:** `src-tauri/src/commands/pipeline.rs:341`
**Issue:** The `generate_sdp_script` command at line 341 explicitly cleans up its temporary directory after completion (`let _ = tokio::fs::remove_dir_all(&task_dir).await;`). However, `run_documentary`, `run_sde`, and `run_sdp` all create task directories in the temp folder (via the core library) but never clean them up. Over time, repeated pipeline runs will accumulate orphaned files in the system temp directory.

**Fix:** Add cleanup logic to all pipeline commands, or implement a shared cleanup mechanism (e.g., on app shutdown or with a periodic sweep).

## Info

### IN-01: Test-only `unwrap()` calls in non-test code reviewed -- no production impact

**Files:** `narratoai-core/src/jianying/builder.rs:750-784`, `narratoai-core/src/documentary/script_gen.rs:508-525`
**Issue:** Multiple `unwrap()` calls exist in `#[cfg(test)]` blocks. These are acceptable for test code. All production code paths use proper error propagation with `?` or explicit error handling.

### IN-02: `capabilities/default.json` lacks explicit filesystem permissions

**File:** `src-tauri/capabilities/default.json`
**Issue:** The capabilities file only grants `core:default`, `core:event:default`, `core:event:allow-emit`, `core:event:allow-listen`, and `dialog:default`. There are no explicit `fs:*` permissions. Since this application reads and writes files extensively via Tauri commands (not the fs plugin), this is actually fine -- Tauri commands run with native process permissions. However, this should be documented so future developers do not assume the frontend JavaScript has filesystem access.

### IN-03: `notify` dependency uses release candidate version

**File:** `narratoai-core/Cargo.toml:23`
**Issue:** `notify = "9.0.0-rc.3"` is an RC (release candidate) version. The comment in the file already acknowledges this risk. The SemVer range `9.0.0-rc.3` will not match the eventual `9.0.0` stable release, meaning a manual update will be required when it ships.

---

_Reviewed: 2026-05-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
