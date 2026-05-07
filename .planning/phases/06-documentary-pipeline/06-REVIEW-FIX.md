---
phase: 06-documentary-pipeline
based_on_review: 2026-05-08T12:00:00Z
review_iteration: 8
fix_scope: critical_warning
findings_in_scope: 1
fixed: 1
skipped: 0
iteration: 2
status: all_fixed
---

# Phase 06: Code Review Fix Report (Iteration 8)

**Based on Review:** 2026-05-08T12:00:00Z (Iteration 8)
**Fix Scope:** critical_warning
**Status:** all_fixed

## Fixes Applied

### WR-01: composite 步骤 amix 滤镜缺少音量补偿 ✓

**File:** `narratoai-core/src/documentary/pipeline.rs:375-383`

**Fix:** 在 amix 后追加 `volume=N*0.5` 温和补偿，保留削波余量。

**Before:**
```rust
filter_complex_parts.push(format!(
    "{}amix=inputs={}:duration=longest[aout]",
    mix_inputs, amix_input_count
));
```

**After:**
```rust
let compensation = if amix_input_count > 1 {
    format!(",volume={:.1}", amix_input_count as f64 * 0.5)
} else {
    String::new()
};
filter_complex_parts.push(format!(
    "{}amix=inputs={}:duration=longest{}[aout]",
    mix_inputs, amix_input_count, compensation
));
```

**Rationale:** `amix` 默认将每路输入音量除以 N。此处与 audio.rs 的 `volume=N` 修复不同——composite 同时混合 2-3 路音频（orig + TTS + BGM），直接 `volume=N` 可能导致削波（0.7+1.0+0.3=2.0 > 1.0）。使用 `volume=N*0.5` 温和补偿：worst-case 峰值 `(0.7+1.0+0.3)*0.5 = 1.0`，恰好不削波。单路输入时不追加补偿（amix 仅 1 路不归一化）。

---

## Skipped (out of scope)

| ID | Severity | Description | Reason |
|----|----------|-------------|--------|
| IR-01 | INFO | `collect_keyframe_paths` 死代码 (script_gen.rs) | 信息级，不在修复范围 |
| IR-02 | INFO | `ProgressStep` 枚举生产代码未使用 (types.rs) | 信息级，不在修复范围 |
| IR-03 | INFO | `strip_and_repair_json` 尾逗号全局替换 (script_gen.rs) | 信息级，不在修复范围 |
| IR-04 | INFO | `generate_srt_from_word_boundaries` 序号不连续 (subtitle.rs) | 信息级，不在修复范围 |
| IN-01 (iter 6) | INFO | 单引号替换可损坏自然语言 (script_gen.rs) | 沿用自 iter 6，信息级 |

## Verification

- [x] WR-01: composite amix 输出经 `volume=N*0.5` 温和补偿，worst-case 峰值 ≤ 1.0
- [x] `cargo build` 编译通过（pipeline.rs 无错误）
- [x] 与 audio.rs 的 `volume=N` 修复互补（不同文件，不同函数，不同补偿系数）

---
_Generated: 2026-05-08_
_Base review iteration: 8_
