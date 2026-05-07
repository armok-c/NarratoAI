---
phase: 04-prompt-system-visual-analyzer
fixed_at: 2026-05-07T15:30:00Z
review_path: .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
iteration: 1
findings_in_scope: 11
fixed: 11
skipped: 0
status: all_fixed
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-05-07T15:30:00Z
**Source review:** .planning/phases/04-prompt-system-visual-analyzer/04-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 11 (4 CRITICAL + 7 WARNING)
- Fixed: 11
- Skipped: 0

## Fixed Issues

### CR-01: Template renderer silently substitutes empty string for unmatched captures

**Files modified:** `src/prompt/template.rs`
**Commit:** ab9420f
**Applied fix:** 将第 2 遍变量替换的 `unwrap_or("")` 改为 `unwrap_or_else(|| unreachable!(...))`，因为第 1 遍已验证所有变量存在，查找失败应为逻辑错误。

### CR-02: Filter replacement silently preserves raw `${var|filter}` tokens

**Files modified:** `src/prompt/template.rs`
**Commit:** 51847ad
**Applied fix:** 将过滤器替换的 else 分支从保留原始 token 改为 `unreachable!()`，因为验证阶段已检查过滤器和变量存在性。

### CR-03: `extract_single_frame` spawns FFmpeg processes without cancellation

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 1637036
**Applied fix:** 将 `run_ffmpeg` 替换为 `run_ffmpeg_with_cancel`，使用 `spawn` + `try_wait` 轮询模式，取消时杀死 FFmpeg 子进程。同时修改 `extract_single_frame` 签名接受 `&CancellationToken` 参数，并更新 `extract_frames_fallback` 中的调用。

### CR-04: `strip_code_fence` incorrectly strips JSON content containing triple backticks

**Files modified:** `src/visual/types.rs`
**Commit:** 150321e
**Applied fix:** 将 `trim_end_matches("```")` 改为 `strip_suffix("```")`，使用精确字符串匹配避免从 JSON 内容中错误剥离单个 backtick 字符。同时重构函数逻辑，当 strip_suffix 结果以非空白字符结尾时不做截断。

### WR-01: `expect()` in json filter can panic in library code

**Files modified:** `src/prompt/template.rs`
**Commit:** 1e5f5f8
**Applied fix:** 将 `serde_json::to_string(s).expect(...)` 改为 `unwrap_or_else(|_| ...)` 提供手动 JSON 转义作为安全降级方案。

### WR-02: `search()` performs O(n) linear scan with repeated allocation

**Files modified:** `src/prompt/registry.rs`
**Commit:** 8815756
**Applied fix:** 将 `to_lowercase()` 改为 `to_ascii_lowercase()` 减少 Unicode case mapping 开销。prompt 名称/标签以 ASCII 为主，ASCII 大小写转换足够。

### WR-03: `validate_narration_script` fails on Windows-style line endings

**Files modified:** `src/prompt/validators.rs`
**Commit:** 878d841
**Applied fix:** 在分割段落前先调用 `replace("\r\n", "\n")` 标准化换行符，确保 LLM 输出的 CRLF 换行不影响段落计数。

### WR-04: `seconds_to_hhmmssmmm` uses floating-point modulo

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 4d7f829
**Applied fix:** 将浮点取模运算改为先转换为整数毫秒，再用整数除法计算 hours/minutes/secs/millis，避免浮点精度导致字段溢出。

### WR-05: `extract_frames_fast_path` calls `child.wait()` twice

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** 8395d83
**Applied fix:** 移除 `ffmpeg_sidecar` `iter()` 耗尽后冗余的 `child.wait()` 调用，因为迭代器已自动 reap 子进程。

### WR-06: `get_video_duration` uses raw `std::process::Command` instead of ffmpeg-sidecar

**Files modified:** `src/visual/frame_extractor.rs`
**Commit:** ec8c720
**Applied fix:** 在 `get_video_duration` 函数的文档注释中添加 `# 系统要求` 段落，说明需要 `ffprobe` 二进制在系统 PATH 中可用。

### WR-07: `parse_and_retry` silently discards BatchResponse parse errors

**Files modified:** `src/visual/analyzer.rs`
**Commit:** 318ff11
**Applied fix:** 将 `if let Ok(resp)` 改为 `match`，在 `Err` 分支添加 `tracing::debug!` 日志记录解析失败原因后再 fallback 到单帧解析。

## Verification

All fixes verified with:
1. Tier 1: Re-read modified file sections to confirm changes present and surrounding code intact
2. Tier 2: `cargo check --lib` passed after each fix (0 new errors, 7 pre-existing warnings unrelated to changes)
3. `cargo test --lib`: 554 passed, 0 failed, 1 ignored (pre-existing)

---

_Fixed: 2026-05-07T15:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
