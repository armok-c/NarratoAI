---
phase: 03-tts-core-edge-tts
fixed_at: 2026-05-06T16:50:00Z
review_path: .planning/phases/03-tts-core-edge-tts/03-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 03: TTS Core + Edge-TTS Engine -- Code Review Fix Report

**Fixed at:** 2026-05-06T16:50:00Z
**Source review:** .planning/phases/03-tts-core-edge-tts/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1 (Critical + Warning only)
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: CONNECT response parsing deadlock when 1xx + 2xx coalesce in same TCP segment

**Files modified:** `src/tts/edge_tts.rs`
**Commit:** 2f00d43
**Applied fix:** Restructured the inner read loop so the `\r\n\r\n` complete-response check runs before `tcp.read()` instead of after. When a 1xx interim response is compacted and leftover bytes already contain a complete 2xx response, the loop now detects this immediately and breaks out without blocking on a redundant `read()` call. No behavioral change for the normal (non-coalesced) path — when `total_read < 4` the check is skipped and the code proceeds directly to read.

## Skipped Issues

None -- all 1 findings in scope were successfully fixed.

---

_Fixed: 2026-05-06T16:50:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
