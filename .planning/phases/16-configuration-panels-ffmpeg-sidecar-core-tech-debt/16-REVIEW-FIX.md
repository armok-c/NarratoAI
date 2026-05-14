---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
fixed_at: 2026-05-14T16:30:00Z
review_path: .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-REVIEW.md
iteration: 1
findings_in_scope: 13
fixed: 10
skipped: 3
status: partial
---

# Phase 16: Code Review Fix Report

**Fixed at:** 2026-05-14T16:30:00Z
**Source review:** .planning/phases/16-configuration-panels-ffmpeg-sidecar-core-tech-debt/16-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 13 (3 critical + 10 warnings, per REVIEW.md frontmatter)
- Fixed: 10
- Skipped: 3

## Fixed Issues

### CR-01: dirty flags never set by any UI interaction

**Files modified:** `src/stores/bgm.ts`, `src/stores/export.ts`, `src/stores/llm.ts`, `src/stores/tts.ts`

**Applied fix:** In each store, added `dirty.value = true` to every mutator method:

- `bgm.ts`: `setFolder()` 和 `setMode()` now set `dirty = true`
- `export.ts`: `setOutputDir()` 和 `setFormat()` now set `dirty = true`
- `llm.ts`: `updateVisionConfig()` 和 `updateTextConfig()` now set `dirty = true`
- `tts.ts`: `setEngine()` 和 `updateConfig()` now set `dirty = true`

This ensures any user interaction that modifies config values triggers the Save button to become enabled and the drawer-close auto-save to fire.

---

### CR-02: NetworkProxyPanel entirely non-functional

**Files modified:** `src/stores/proxy.ts` (new), `src/components/config/NetworkProxyPanel.vue`, `src/composables/useConfig.ts`, `src/components/SettingsDrawer.vue`

**Applied fix:**

1. **New store `src/stores/proxy.ts`**: Created following the `bgm.ts`/`export.ts` pattern with `enabled`, `http`, `https` refs, `dirty` tracking, `loadConfig()`, `resetPanel()`, and `setEnabled()/setHttp()/setHttps()` mutators (each setting `dirty = true`).

2. **`NetworkProxyPanel.vue`**: Replaced local refs (`proxyEnabled`, `proxyHttp`, `proxyHttps`) with `useProxyStore()`, binding `v-model` directly to `store.enabled`, `store.http`, `store.https`.

3. **`useConfig.ts`** — integrated proxy into all persistence paths:
   - `saveDraft()`: reads proxy values from `proxyStore` (not `_lastConfig.proxy`) and stores in `networkProxy` draft field.
   - `restoreDraft()`: reads `networkProxy` from draft and applies to `proxyStore`.
   - `clearDraft()`: resets `proxy.dirty = false`.
   - `loadFromBackend()`: calls `proxyStore.loadConfig(config)` alongside other stores.
   - `collectChangedFields()`: emits `proxy_enabled`, `proxy_http`, `proxy_https` when `proxyStore.dirty` is true.

4. **`SettingsDrawer.vue`**: Added `proxyStore.dirty` to `hasUnsavedChanges` computed; added `proxyStore.markClean()` to `handleSave()` and `confirmOverwrite()`; added `proxyStore.resetPanel()` to `handleResetAll()`.

---

### CR-03: collectChangedFields sends all TTS engine configs unconditionally

**File modified:** `src/composables/useConfig.ts`

**Applied fix:** Replaced the unconditional TTS field emission with a `switch` statement on `ttsStore.engine`. Only the fields for the currently selected engine are emitted, preventing credential data loss for unused engines.

Also added `indextts2_repetition_penalty` to the indextts2 case (aligning with WR-07 fix).

---

### WR-01: saveDraft proxy capture inconsistent with restoreDraft

**File modified:** `src/composables/useConfig.ts`

**Applied fix:** Handled as part of CR-02. `saveDraft()` now reads proxy from `proxyStore` (live UI state), and `restoreDraft()` applies the saved `networkProxy` values. Both save and restore paths are now consistent.

---

### WR-04: Dead function handleEngineChange in TtsPanel.vue

**File modified:** `src/components/config/TtsPanel.vue`

**Applied fix:** Removed the unused `handleEngineChange(_val: string)` function (formerly lines 416-418). Engine selection is handled by `v-model="store.engine"` on the `<v-select>`.

---

### WR-05: Empty catch blocks in BgmPanel and ExportPanel

**Files modified:** `src/components/config/BgmPanel.vue`, `src/components/config/ExportPanel.vue`

**Applied fix:** Added `console.warn` with descriptive messages to both empty `catch {}` blocks:

- BgmPanel: `console.warn('[BgmPanel] 无法打开文件夹选择对话框:', err)`
- ExportPanel: `console.warn('[ExportPanel] 无法打开文件夹选择对话框:', err)`

---

### WR-06: TTS loadConfig uses unsafe type assertions

**File modified:** `src/stores/tts.ts`

**Applied fix:** Added explanatory comment at the top of `loadConfig()` explaining why `as unknown as Record<string, string>` casts are necessary. The generated TypeScript types (AzureSection, TencentSection, etc.) do not declare credential fields (speech_key, secret_id, api_key, etc.), but the Rust backend TOML config sections contain these fields at runtime. The casts are kept as-is with documentation of the reason.

---

### WR-07: IndexTTS2EngineConfig missing repetition_penalty field

**File modified:** `src/stores/tts.ts`

**Applied fix:**

1. Added `repetition_penalty: number` to `IndexTTS2EngineConfig` interface.
2. Added `repetition_penalty: 1.0` to `defaultEngineConfigs.indextts2`.
3. Added `repetition_penalty: config.indextts2.repetition_penalty` in `loadConfig()` indextts2 mapping.

This aligns the frontend model with the Rust-generated `IndexTTS2Section` type which already declares this field.

---

### WR-09: v-model.number on text fields can produce NaN on empty input

**Files modified:** `src/components/config/TtsPanel.vue`, `src/components/config/ModeParamsPanel.vue`

**Applied fix:** Added deep `watch` in each component that scans numeric fields and replaces `NaN` with `0`:

- **TtsPanel.vue**: Watches `store.engineConfigs` deeply, iterates all engine configs and replaces NaN numeric values with 0.
- **ModeParamsPanel.vue**: Watches `params` deeply (from `useModeStore`), replaces NaN numeric values with 0.

This prevents NaN propagation to the backend via JSON serialization (which converts NaN to null).

---

### WR-10: BgmPanel selectedFile never persisted

**Files modified:** `src/stores/bgm.ts`, `src/components/config/BgmPanel.vue`, `src/composables/useConfig.ts`

**Applied fix:**

1. **`bgm.ts`**: Added `selectedFile` ref, `setSelectedFile()` method, and included in `resetPanel()` and return block.
2. **`BgmPanel.vue`**: Changed `v-model="selectedFile"` to `v-model="store.selectedFile"`, removed local `selectedFile` ref and unused `import { ref }`.
3. **`useConfig.ts`**: Added `selectedFile` to `SettingsDraft.bgm` interface, `saveDraft()` now includes `bgm.selectedFile`, `restoreDraft()` restores it.

---

### WR-11: versionText hardcoded fallback is incorrect

**File modified:** `src/components/SettingsDrawer.vue`

**Applied fix:** Changed both hardcoded fallback values from `'v0.1.0'` to `'v0.7.8'`:
- Line 215: `const versionText = ref('v0.7.8')`
- Line 492: `versionText.value = 'v0.7.8'` (inside onMounted catch)

## Skipped Issues

### WR-02: saveToBackend argument shape inconsistency

**File:** `src/composables/useConfig.ts`

**Reason:** Requires verification against the Rust backend's `save_config` command signature. The current approach passes a flat object as named arguments to Tauri's `invoke`, which works if the Rust side accepts a single struct parameter. Cannot verify or fix this without inspecting the Rust backend code.

### WR-03: ModeParamsPanel v-model on dramaName field

**File:** `src/components/config/ModeParamsPanel.vue`

**Reason:** The review itself confirms this is NOT a bug. The `dramaName` field is a `string` and uses `v-model` (without `.number`), which is correct behavior. No fix needed.

### WR-08: modeStore.loadConfig silences frame_interval_input type conversion

**File:** `src/stores/mode.ts`

**Reason:** The review acknowledges this is "acceptable for practical frame_interval values." The `Number()` conversion of `bigint` to `number` is safe for the typical 1-30 range. Adding a runtime assertion for a non-problem in a non-bug scenario is unnecessary scope creep.

---

_Fixed: 2026-05-14T16:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
