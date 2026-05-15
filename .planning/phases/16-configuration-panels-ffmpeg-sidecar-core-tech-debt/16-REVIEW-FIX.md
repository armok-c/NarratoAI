---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
fixed_at: 2026-05-15T02:05:00Z
review_path: .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-REVIEW.md
iteration: 4
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 16: Code Review Fix Report

**Fixed at:** 2026-05-15T02:05:00Z
**Source review:** .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-REVIEW.md
**Iteration:** 4

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: `indextts2_repetition_penalty` missing from `FIELD_TO_PANEL` validation error mapping

**Files modified:** `src/components/SettingsDrawer.vue`
**Commit:** b380614
**Applied fix:** Added `indextts2_repetition_penalty: 'tts'` entry to the `FIELD_TO_PANEL` object in `SettingsDrawer.vue`, between `indextts2_num_beams` and `doubaotts_ak`. This ensures validation errors referencing this field are routed to the TTS panel badge instead of falling through to 'general'.

### WR-02: `indextts2` engine missing `repetition_penalty` UI field in TtsPanel

**Files modified:** `src/components/config/TtsPanel.vue`
**Commit:** b380614
**Applied fix:** Added a `v-text-field` input for `indexCfg.repetition_penalty` to the `indextts2` template section in `TtsPanel.vue`, after the "Beam 数量" field. The field uses `type="number"`, `step="0.1"`, `min="1"`, `max="2"` to match the backend validation constraints. Bound to `indexCfg.repetition_penalty` (snake_case matching the `IndexTTS2EngineConfig` interface definition in `tts.ts`).

## Skipped Issues

None -- all findings were fixed.

---

_Fixed: 2026-05-15T02:05:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 4_
