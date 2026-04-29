---
phase: 09-jianying-export
reviewed: 2026-04-29T12:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - Cargo.toml
  - src/ffmpeg/probe.rs
  - src/jianying/builder.rs
  - src/jianying/error.rs
  - src/jianying/material.rs
  - src/jianying/mod.rs
  - src/jianying/segment.rs
  - src/jianying/template.rs
  - src/jianying/time.rs
  - src/jianying/track.rs
  - src/jianying/types.rs
  - src/lib.rs
  - tests/jianying_export.rs
findings:
  critical: 2
  warning: 2
  info: 1
  total: 5
status: issues_found
---

# Phase 09: Code Review Report (Iteration 4)

**Reviewed:** 2026-04-29T12:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Reviewed 13 source files for the JianYing (CapCut) draft export module at standard depth. Previous iterations (1-3) addressed validation gaps in `trange_from_secs`, empty timelines, ffprobe duration parsing, and path validation. All 8 findings from iteration 3 have been correctly fixed. This iteration found 2 new critical issues and 2 warnings that remain in the codebase.

Both critical issues stem from the same root cause: `draft_name` is user-controlled input with no validation or sanitization. (1) JSON injection via naive `.replace()` in the meta info template corrupts the output when the name contains special characters. (2) Path traversal via `draft_name` allows creating directories outside the intended `draft_path`.

## Critical Issues

### CR-01: JSON injection via unescaped draft_name in DRAFT_META_INFO_TEMPLATE

**File:** `src/jianying/builder.rs:79-81`
**Severity:** BLOCKER

**Issue:** The `draft_name` string is inserted into the JSON template via naive `String::replace()` without any JSON escaping. If `draft_name` contains double quotes (`"`), backslashes (`\`), newlines, or other JSON-special characters, the resulting `draft_meta_info.json` will be malformed JSON that JianYing cannot parse.

Example: `draft_name = "Test\"Draft"` produces:
```json
"draft_name": "Test"Draft"
```

This breaks the entire draft -- JianYing will reject it as invalid JSON. More concerning, a crafted name like `foo","draft_type":0}//` could inject arbitrary JSON fields.

**Fix:** Parse the template as `serde_json::Value` (same safe pattern already used in `ScriptFile::save` for `draft_content.json`), then set the fields programmatically:

```rust
let mut meta: serde_json::Value = serde_json::from_str(DRAFT_META_INFO_TEMPLATE)?;
meta["draft_id"] = serde_json::json!(draft_id);
meta["draft_name"] = serde_json::json!(draft_name);
let meta_content = serde_json::to_string(&meta)?;
std::fs::write(draft_dir.join("draft_meta_info.json"), meta_content)?;
```

### CR-02: Path traversal via unsanitized draft_name

**File:** `src/jianying/builder.rs:74`
**Severity:** BLOCKER

**Issue:** The `draft_name` is passed directly to `draft_path.join(draft_name)` without any validation. If `draft_name` contains path traversal sequences like `../../etc/cron.d`, the resulting directory will be created outside the intended `draft_path` hierarchy. Combined with the file writes at lines 82 (`draft_meta_info.json`) and 218 (`draft_content.json`), this allows arbitrary file creation on the filesystem.

The `validate_export_request` function at line 313 validates `draft_path` is non-empty but does not check `draft_name` at all.

**Fix:** Add validation to reject `draft_name` containing path separators or traversal sequences. Call this in `validate_export_request` or at the top of `DraftFolder::create_draft`:

```rust
fn validate_draft_name(name: &str) -> Result<(), JianYingError> {
    if name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains('\0')
    {
        return Err(JianYingError::Validation {
            details: format!("草稿名称包含非法字符: {}", name),
        });
    }
    Ok(())
}
```

## Warnings

### WR-01: OST=NarrationOnly silently skips audio when clip.audio is None

**File:** `src/jianying/builder.rs:286-297`
**Severity:** WARNING

**Issue:** When `clip.ost == OstType::NarrationOnly` (OST=0) but `clip.audio` is `None`, the inner `if let Some(ref audio_path) = clip.audio` does not match, and no audio segment is created. The export succeeds without error, but the resulting draft is semantically incorrect -- a "narration only" segment has no audio track.

This is a silent data loss scenario. The caller expects narration audio for OST=0 clips, but the exported draft silently omits it. The OST enum variant `NarrationOnly` explicitly declares that audio is required; silently proceeding without it violates the contract.

**Fix:** Return an error when OST requires audio but none is available:

```rust
if clip.ost == OstType::NarrationOnly || clip.ost == OstType::Mixed {
    match clip.audio {
        Some(ref audio_path) => {
            let audio_duration = probe_audio(audio_path)
                .map_err(|e| JianYingError::ProbeError(e.to_string()))?;
            let safe_duration = duration.min(audio_duration);
            let audio_target = trange_from_secs(current_time_secs, safe_duration)
                .ok_or_else(|| JianYingError::Validation {
                    details: format!("第 {} 段构造音频时间范围失败", i + 1),
                })?;
            let audio_seg = AudioSegment::new(audio_path, audio_target)?;
            script_file.add_audio_segment(audio_seg, "音频轨道")?;
        }
        None => {
            return Err(JianYingError::MissingField {
                field: "audio".to_string(),
                clip_index: i,
            });
        }
    }
}
```

### WR-02: trange() accepts negative start values

**File:** `src/jianying/time.rs:17-27`
**Severity:** WARNING

**Issue:** The `trange()` function validates `duration_us < 0` (line 20) but does not validate `start_us < 0`. A negative start value will pass through silently. In contrast, `trange_from_secs()` correctly validates both parameters. While `trange()` is currently only called from test code, it is a `pub` API and this inconsistency could cause bugs if used in production code later.

**Fix:** Add start validation to `trange()`:

```rust
pub fn trange(start: &str, duration: &str) -> Option<Timerange> {
    let start_us = parse_seconds(start)?;
    let duration_us = parse_seconds(duration)?;
    if start_us < 0 || duration_us < 0 {
        return None;
    }
    Some(Timerange {
        start: start_us,
        duration: duration_us,
    })
}
```

## Info

### IN-01: Inconsistent error message language in probe.rs

**File:** `src/ffmpeg/probe.rs:63, 97, 154`
**Severity:** INFO

**Issue:** Error messages are inconsistently mixed between English and Chinese. `probe_audio` uses Chinese at line 151 (`"音频时长解析失败"`) but English at lines 153-155 (`"Invalid audio duration from ffprobe"`). `probe_video` uses English throughout (`"Missing or invalid duration"`, `"No video stream found"`). All error messages in this project should use Chinese for consistency (per project conventions in CLAUDE.md and the existing Chinese error messages in `error.rs` and `jianying/error.rs`).

**Fix:** Standardize all error messages in `probe.rs` to Chinese, matching the style used elsewhere in the codebase.

---

_Reviewed: 2026-04-29T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 4_
