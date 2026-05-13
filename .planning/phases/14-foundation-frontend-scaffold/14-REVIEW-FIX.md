---
phase: 14
fixed_at: "2026-05-13T06:00:00Z"
review_path: .planning/phases/14-foundation-frontend-scaffold/14-REVIEW.md
iteration: 3
findings_in_scope: 12
fixed: 3
skipped: 9
status: partial
---

# Phase 14: Code Review Fix Report (Iteration 3)

**Fixed at:** 2026-05-13T06:00:00Z
**Source review:** .planning/phases/14-foundation-frontend-scaffold/14-REVIEW.md
**Iteration:** 3 (--fix --all pass)

**Summary:**
- Findings in scope: 12 (10 deferred from iteration 1 + 2 actionable from iteration 2)
- Fixed: 3 (WR-05, IN-09 committed from iteration 2 + IN-12 new fix)
- Skipped: 9 (legitimate deferrals for Phase 16, architecture decisions, or acceptable patterns)

## Fixed Issues

### WR-05: ScriptClip._id ts-rs generates bigint instead of number

**Files modified:** `narratoai-core/src/script/types.rs`, `narratoai-core/src/types/generated/ScriptClip.ts`
**Commit:** `f950d21`
**Applied fix:** Added `#[ts(type = "number")]` attribute to `_id` field in Rust struct, and manually updated generated TypeScript file `_id: bigint` to `_id: number`. ts-rs v11's `#[ts(export)]` does not auto-regenerate `.ts` files during build in this project configuration, so both the Rust annotation (for future regeneration) and the manual `.ts` edit (for immediate correctness) were needed.

### IN-09: repetition_penalty default vs test value documentation gap

**Files modified:** `narratoai-core/src/config/defaults.rs`
**Commit:** `b339ac4`
**Applied fix:** Added clarifying comment above `repetition_penalty: 1.5` explaining that `test_load_full_config` uses `10.0` to verify TOML override of defaults.

### IN-12: manualChunks returns undefined for unmatched packages

**Files modified:** `vite.config.ts`
**Commit:** `5c2cb39`
**Applied fix:** Added explicit `return 'vendor'` fallback at the end of `manualChunks` so unmatched node_modules packages go to a predictable chunk instead of the implicit default chunk.

### Generated TS file update (from iteration 1 CR-01/CR-02)

**Files modified:** 17 files in `narratoai-core/src/types/generated/`
**Commit:** `072bd3c`
**Applied fix:** Committed pending generated TypeScript file updates that reflect iteration 1 Rust annotation changes: `#[ts(skip)]` on sensitive fields (CR-01), OstType integer type override (CR-02), and line ending normalization.

## Skipped Issues (9)

| ID | Reason |
|----|--------|
| WR-07 | Acceptable as smoke test -- find_ytdlp validates installation, test re-validates version output. Changing find_ytdlp to return version would complicate the helper for marginal benefit. |
| WR-09 | Semantically correct (`field?: string \| null`) -- no fix needed, frontend should use optional chaining |
| IN-01 | Phase 16 ts-rs migration will unify hand-written and generated types |
| IN-02 | Phase 16 type replacement |
| IN-03 | Phase 16 type replacement |
| IN-05 | Known stub -- backend command planned for later phase |
| IN-06 | Architecture decision -- separate crate output dirs |
| IN-10 | Future work -- internal pipeline types |
| IN-11 | Acceptable Vue/Pinia pattern -- .push() is idiomatic in Vue reactivity; spread would create unnecessary array copies |

---

_Fixed: 2026-05-13T06:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 3_
