---
status: issues_found
phase: 04-prompt-system-visual-analyzer
reviewed: 2026-05-07T12:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - narratoai-core/src/prompt/mod.rs
  - narratoai-core/src/prompt/types.rs
  - narratoai-core/src/prompt/error.rs
  - narratoai-core/src/prompt/registry.rs
  - narratoai-core/src/prompt/template.rs
  - narratoai-core/src/prompt/manager.rs
  - narratoai-core/src/prompt/validators.rs
  - narratoai-core/src/prompt/register.rs
  - narratoai-core/src/prompt/templates/documentary/frame_analysis_v1.0.md
  - narratoai-core/src/prompt/templates/documentary/narration_generation_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_editing/plot_extraction_v2.0.md
  - narratoai-core/src/prompt/templates/short_drama_narration/script_generation_v1.0.md
  - narratoai-core/src/visual/mod.rs
  - narratoai-core/src/visual/error.rs
  - narratoai-core/src/visual/types.rs
  - narratoai-core/src/visual/frame_extractor.rs
  - narratoai-core/src/visual/analyzer.rs
  - narratoai-core/src/lib.rs
  - narratoai-core/src/text_utils.rs
  - narratoai-core/Cargo.toml
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

本轮审查基于前次审查（40 个发现，16 WARNING / 24 INFO）的修复后代码进行。前次 16 个 WARNING 中 12 个已确认修复，4 个属于有意设计或低优先级保留。当前代码质量显著提升，未发现 CRITICAL 级别问题。剩余 4 个 WARNING 和 5 个 INFO 均为低优先级改进项。

**已修复确认（12/16）：**
- WR-01/WR-02: 正则 OnceLock 缓存 (registry.rs + template.rs)
- WR-03: BUILTIN_FILTERS OnceLock 缓存 (template.rs)
- WR-04: unreachable! 替换为 expect 含描述性消息 (template.rs)
- WR-05: 版本排序 fallback 改为 u64::MAX (registry.rs)
- WR-06/WR-07: chars().count() 缓存到局部变量 (validators.rs)
- WR-09: seconds_to_hhmmssmmm 添加 .max(0.0) clamp (frame_extractor.rs)
- WR-10: MAX_TOTAL_FRAMES = 100_000 上限检查 (frame_extractor.rs)
- WR-13: 排序 unwrap_or_else + warn 日志 (analyzer.rs)
- WR-14: 模板措辞统一为"输出语言" (frame_analysis_v1.0.md)
- WR-15: 测试注释更新为准确描述 (register.rs)

**有意保留（4/16）：**
- WR-08: strip_code_fence 嵌套代码块 — 当前实现足够
- WR-11: 无主 CancellationToken — 已添加文档说明
- WR-12: LLM 结构体不使用 deny_unknown_fields — 容错设计
- WR-16: notify RC 版本 — 已标注风险

## Warnings

### WR-01: TEMPLATE_VAR_REGEX 重复定义导致双重编译 (registry.rs + template.rs)

**File:** `narratoai-core/src/prompt/registry.rs:10`, `narratoai-core/src/prompt/template.rs:11`
**Issue:** `TEMPLATE_VAR_REGEX` 在两个文件中各定义了一个独立的 `OnceLock<Regex>` 实例，使用相同的正则模式 `r"\$\{(\w+)(?:\|(\w+))?\}"`。虽然每个 `OnceLock` 只编译一次，但整个程序生命周期内会编译两次相同的正则。更重要的是，如果未来需要修改正则模式，必须同时更新两处，容易遗漏导致不一致。

**Fix:**
```rust
// 在 prompt/mod.rs 或一个共享位置定义一次
pub(crate) mod template_var_regex {
    use regex::Regex;
    use std::sync::OnceLock;

    static RE: OnceLock<Regex> = OnceLock::new();

    pub fn get() -> &'static Regex {
        RE.get_or_init(|| {
            Regex::new(r"\$\{(\w+)(?:\|(\w+))?\}")
                .expect("TEMPLATE_VAR_REGEX 编译失败")
        })
    }
}
```
然后在 `registry.rs` 和 `template.rs` 中统一引用 `crate::prompt::template_var_regex::get()`。

---

### WR-02: collect_frame_paths 生产代码未使用 (analyzer.rs)

**File:** `narratoai-core/src/visual/analyzer.rs:321`
**Issue:** `collect_frame_paths()` 函数及其辅助函数 `extract_frame_number_from_keyframe()` 在生产代码中未被调用。`analyze_video_frames()` 使用 `extract_frames()` 返回的路径列表，而非自己收集。这两个函数仅在测试模块中使用，属于死代码。

虽然函数签名是 `fn`（私有），不会增加 API 表面积，但它们增加了编译产物大小和维护负担。如果未来有人修改了这些函数但未发现它们未被使用，可能引入回归。

**Fix:** 将 `collect_frame_paths` 和 `extract_frame_number_from_keyframe` 移入 `#[cfg(test)] mod tests` 块内，或添加 `#[cfg(test)]` 属性。

---

### WR-03: strip_code_fence 不处理嵌套代码块 (text_utils.rs)

**File:** `narratoai-core/src/text_utils.rs:7-18`
**Issue:** `strip_code_fence` 使用简单的 `strip_prefix`/`strip_suffix` 剥离 markdown 代码块。如果 LLM 返回的 JSON 内容中恰好包含 `` ``` `` 字符串（例如 JSON 字符串值包含反引号），`strip_suffix("```")` 会错误剥离内容尾部。

实际场景中，LLM 返回的 JSON 内部包含裸反引号序列的概率较低，但并非不可能（如 JSON 中嵌入的代码片段）。

**Fix:** 当前实现对于绝大多数 LLM 输出足够健壮。如需更严格处理，可使用逐行状态机：
```rust
pub fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.len() >= 2 && lines[0].starts_with("```") && lines.last().unwrap_or(&"").trim() == "```" {
        let start = lines[0].find('\n').map(|i| i + 1).unwrap_or(lines[0].len());
        let end = text.len() - lines.last().unwrap().len();
        text[start..end].trim()
    } else {
        trimmed
    }
}
```

---

### WR-04: notify 依赖使用 RC 版本 (Cargo.toml)

**File:** `narratoai-core/Cargo.toml:23`
**Issue:** `notify = "9.0.0-rc.3"` 是预发布版本，其 API 在 9.0.0 正式版中可能发生破坏性变更。代码中已通过注释标注此风险，但使用 RC 版本在生产环境中仍有供应链稳定性风险。

**Fix:** 跟踪 notify 9.0.0 正式版发布，优先升级。当前不影响功能正确性。

## Info

### IN-01: collect_keyframe_paths_from_dir 使用字典序排序 (frame_extractor.rs)

**File:** `narratoai-core/src/visual/frame_extractor.rs:625-633`
**Issue:** `collect_keyframe_paths_from_dir` 使用 `paths.sort()`（字典序）。由于帧号格式化为 `{:06}` 零填充 6 位，且 `MAX_TOTAL_FRAMES = 100_000`，字典序在当前约束下等价于数字序。但如果未来放宽帧数限制（帧号超过 999999），字典序将出错。

这不是当前 bug，仅作为防御性记录。

**Fix:** 如果未来放宽帧数限制，改用 `analyzer.rs` 中 `collect_frame_paths` 的数字排序逻辑（或提取为共享函数）。

---

### IN-02: truncate 过滤器魔法数字 (template.rs)

**File:** `narratoai-core/src/prompt/template.rs:56-63`
**Issue:** `truncate` 过滤器中 `100` 和 `97` 硬编码。语义上 "100 字符上限、97 字符内容 + 3 字符省略号" 不够直观。

**Fix:**
```rust
const TRUNCATE_MAX_CHARS: usize = 100;
const TRUNCATE_ELLIPSIS_LEN: usize = 3;
let keep = TRUNCATE_MAX_CHARS - TRUNCATE_ELLIPSIS_LEN; // 97
```

---

### IN-03: RwLock 中毒处理策略 (manager.rs)

**File:** `narratoai-core/src/prompt/manager.rs:42-44`, `111-113`, `119-121`, `138-140`, `147-149`
**Issue:** 所有 `RwLock` 获取操作使用 `.map_err(|e| PromptError::LockFailure(...))` 将 `PoisonError` 转换为业务错误。这意味着如果某个线程 panic 导致锁中毒，后续所有操作都会返回 `LockFailure` 错误而非恢复或终止。这是合理的防御性选择，但调用方需要意识到锁中毒是不可恢复的。

当前处理方式合理，仅作为文档记录。

---

### IN-04: tokio features = ["full"] 可精简 (Cargo.toml)

**File:** `narratoai-core/Cargo.toml:15`
**Issue:** `tokio = { version = "1.52.1", features = ["full"] }` 启用了所有 tokio 功能，包括 `full` 隐含的 `net`、`io-util`、`io-std`、`fs`、`signal`、`process` 等。实际使用的功能仅为 `rt-multi-thread`、`macros`、`sync` 和 `time`。`features = ["full"]` 增加编译时间和二进制大小。

**Fix:** 按需启用：
```toml
tokio = { version = "1.52.1", features = ["rt-multi-thread", "macros", "sync", "time", "process"] }
```

---

### IN-05: progress callback 在 extract_frames 回退路径中已消费 (frame_extractor.rs)

**File:** `narratoai-core/src/visual/frame_extractor.rs:94-108`, `109-124`
**Issue:** 当快路径失败并回退到 `extract_frames_fallback` 时，`progress` callback 被传递给 fallback 函数。但如果快路径成功（`Ok(count) if count > 0`），progress callback 仅在 `cb(Some(1.0), "帧提取完成")` 时被调用一次，跳过了中间进度。快路径中 FFmpeg 进度事件未被转发给 callback。

这是已知的局限性，不影响功能正确性。

---

_Reviewed: 2026-05-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
