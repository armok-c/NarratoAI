---
phase: 12-additional-tts-engines
fixed_at: 2026-05-02T01:30:00Z
review_path: .planning/phases/12-additional-tts-engines/12-REVIEW.md
iteration: 4
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report (Iteration 4)

**Fixed at:** 2026-05-02T01:30:00Z
**Source review:** .planning/phases/12-additional-tts-engines/12-REVIEW.md
**Iteration:** 4

**Summary:**
- Findings in scope: 1 (Warning)
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: DoubaoTTSSection.pitch 配置字段存在但引擎未使用

**Files modified:** `src/tts/doubaotts.rs`
**Commit:** f11e83c
**Applied fix:** 将 payload 中硬编码的 `"pitch_ratio": 1.0` 改为 `"pitch_ratio": self.config.pitch`，使配置字段生效。同步更新 `test_doubaotts_payload_structure` 测试中的 payload 构建和断言。默认值仍为 `1.0`（`defaults.rs`），现有配置行为不变。

## Verification

- `cargo check`: passes (only pre-existing `get_azure_voices` dead_code warning)
- `cargo test --lib tts`: 78 passed, 0 failed, 1 ignored

---
*Fixed: 2026-05-02T01:30:00Z*
*Fixer: Claude (gsd-code-fixer)*
*Iteration: 4*
