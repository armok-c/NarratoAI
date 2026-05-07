---
phase: 06-documentary-pipeline
based_on_review: 2026-05-08T00:45:00Z
review_iteration: 7
fix_scope: critical_warning
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 1
status: all_fixed
commits:
  - 0e0169a fix(06): CR-01 remove unnecessary escape chars in subtitle path
  - 6e63f8b fix(06): WR-01 compensate amix 1/N normalization with volume filter
---

# Phase 06: Code Review Fix Report (Iteration 7)

**Based on Review:** 2026-05-08T00:45:00Z (Iteration 7)
**Fix Scope:** critical_warning
**Status:** all_fixed

## Fixes Applied

### CR-01: FFmpeg 字幕滤镜路径在 Windows 上双重转义 ✓

**File:** `narratoai-core/src/documentary/pipeline.rs:385-389`
**Commit:** `0e0169a`

**Fix:** 移除单引号上下文中不必要的 `:`, `'`, `[`, `]`, `;` 转义。仅保留：
1. `\` → `/`（FFmpeg 路径兼容）
2. `'` → `'\''`（单引号内转义单引号本身）
3. `\n`, `\r` 移除

**Before:**
```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace(':', "\\:")
    .replace("'", "\\'")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace(';', "\\;")
    .replace('\n', "")
    .replace('\r', "");
```

**After:**
```rust
let escaped_srt = srt_str
    .replace('\\', "/")
    .replace('\'', "'\\''")
    .replace('\n', "")
    .replace('\r', "");
```

**Rationale:** FFmpeg filter 语法中单引号保护内部特殊字符（`:`, `[`, `]`, `;`），无需额外转义。旧代码在单引号内添加 `\:` 等，导致 FFmpeg 查找路径 `C\:/Users/...` 而非 `C:/Users/...`，Windows 上字幕合成必定失败。

---

### WR-01: amix 滤镜按 1/N 归一化导致多片段音频衰减 ✓

**File:** `narratoai-core/src/documentary/audio.rs:70-73`
**Commit:** `6e63f8b`

**Fix:** 在 amix 后追加 `volume=N` 增益补偿。

**Before:**
```rust
filter_parts.push(format!(
    "{}amix=inputs={}:duration=longest[aout]",
    amix_inputs, input_count
));
```

**After:**
```rust
filter_parts.push(format!(
    "{}amix=inputs={}:duration=longest,volume={}[aout]",
    amix_inputs, input_count, input_count
));
```

**Rationale:** FFmpeg `amix` 默认将每个输入音量除以 N（总输入数）。20 个旁白片段 + 1 静音基底 = 21 个输入时，每个片段有效音量 = 原始 / 21。由于同一时刻仅一个片段播放，`volume=N` 精确补偿归一化衰减。

---

## Skipped (out of scope)

| ID | Severity | Description | Reason |
|----|----------|-------------|--------|
| IR-01 | INFO | `collect_keyframe_paths` 死代码 | 信息级，不在修复范围 |
| IR-02 | INFO | `ProgressStep` 枚举生产代码未使用 | 信息级，不在修复范围 |
| IR-03 | INFO | `strip_and_repair_json` 尾逗号全局替换 | 信息级，不在修复范围 |
| IR-04 | INFO | `generate_srt_from_word_boundaries` 序号不连续 | 信息级，不在修复范围 |
| IN-01 (iter 6) | INFO | 单引号替换可损坏自然语言 | 沿用自 iter 6，信息级 |

## Verification

- [x] CR-01: Windows 路径 `C:\Users\...\Temp\merged_subtitle.srt` → `C:/Users/.../Temp/merged_subtitle.srt`，单引号内无多余转义
- [x] WR-01: amix 输出经 `volume=N` 补偿，还原至原始音量水平
- [x] 两个修复互不干扰（不同文件，不同逻辑路径）

---
_Generated: 2026-05-08_
_Base review iteration: 7_
