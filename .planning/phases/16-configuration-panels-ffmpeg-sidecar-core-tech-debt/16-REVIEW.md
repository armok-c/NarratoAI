---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
reviewed: 2026-05-14T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - src/App.vue
  - src/components/SettingSection.vue
  - src/components/SettingsDrawer.vue
  - src/components/config/BgmPanel.vue
  - src/components/config/ExportPanel.vue
  - src/components/config/LlmTextPanel.vue
  - src/components/config/LlmVisionPanel.vue
  - src/components/config/ModeParamsPanel.vue
  - src/components/config/NetworkProxyPanel.vue
  - src/components/config/TtsPanel.vue
  - src/composables/useConfig.ts
  - src/stores/bgm.ts
  - src/stores/export.ts
  - src/stores/tts.ts
  - tests/components/SettingSection.test.ts
  - tests/composables/useConfig.test.ts
  - tests/stores/llm.test.ts
  - tests/stores/tts.test.ts
findings:
  critical: 3
  warning: 10
  info: 0
  total: 13
status: issues_found
---

# Phase 16: Code Review Report

**Reviewed:** 2026-05-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Reviewed 18 source files spanning Vue components, Pinia stores, composable utilities, and test files for the configuration panels system. The core issue is a systematic design flaw: **no UI interaction in any config panel sets the `dirty` flag on any store**. This renders the entire save mechanism non-functional — the Save button is permanently disabled, and draft auto-save on drawer close never fires. Additionally, the NetworkProxyPanel uses local refs with no store backing at all, and `collectChangedFields()` sends all TTS engine configs (including empty credentials for unused engines) unconditionally, risking backend data loss.

## Critical Issues

### CR-01: `dirty` flags never set by any UI interaction — save mechanism non-functional

**File:** Multiple files across all components and stores

**Affected:**
- `src/stores/bgm.ts` — `setFolder()` and `setMode()` do not set `dirty`
- `src/stores/export.ts` — `setFormat()` does not set `dirty`
- `src/stores/llm.ts` — `updateVisionConfig()` and `updateTextConfig()` do not set `dirty`
- `src/stores/tts.ts` — `setEngine()` and `updateConfig()` do not set `dirty`
- `src/components/config/LlmTextPanel.vue` — `v-model` binds directly to store ref without calling `markDirty()`
- `src/components/config/LlmVisionPanel.vue` — same pattern
- `src/components/config/TtsPanel.vue` — same pattern
- `src/components/config/BgmPanel.vue` — `:model-value` + `@update:model-value` handlers do not set `dirty`
- `src/components/config/ExportPanel.vue` — same pattern
- `src/components/SettingsDrawer.vue` — `hasUnsavedChanges` at line 231 checks `dirty` flags but nothing sets them
- `src/composables/useConfig.ts` — `saveDraft()` at line 45 is only called when `hasUnsavedChanges` is true

**Issue:** Every Pinia store that manages user-editable configuration has a `dirty` ref whose purpose is to track unsaved changes. However, no mutation path from any UI component ever sets `dirty = true`. The stores' mutator methods (`setFolder`, `setMode`, `setFormat`, `setEngine`, `updateConfig`, `updateVisionConfig`, `updateTextConfig`) all modify data without touching `dirty`. The `v-model` directives in the panels bind directly to store refs or computed refs, bypassing store methods entirely. The `hasUnsavedChanges` computed in SettingsDrawer (line 231-233) checks these `dirty` flags, which remain `false` at all times. Consequently:

1. The Save button is permanently disabled (`:disabled="!hasUnsavedChanges"` at line 101).
2. `onDrawerClose` (line 287-292) never calls `saveDraft()` because `hasUnsavedChanges` is `false`.
3. All user modifications exist only in memory and are silently lost when the drawer closes or the page reloads.

The only path that sets `dirty = true` is `restoreDraft()` in `useConfig.ts` (line 123-126), which runs once on app mount if a draft exists. After the first save successfully clears the draft (setting dirty back to `false` via `clearDraft()` at line 140), the user can never save again.

**Fix:** Each store's mutator method must set `dirty = true`, OR a `watch` must be added inside each store setup that detects mutations to the data refs and sets `dirty` accordingly. For example, in `bgm.ts`:

```typescript
function setFolder(p: string) {
  folder.value = p
  dirty.value = true
}

function setMode(m: BgmMode) {
  mode.value = m
  dirty.value = true
}
```

Same pattern for `setFormat()` in `export.ts`, `setEngine()` and `updateConfig()` in `tts.ts`, and `updateVisionConfig()`/`updateTextConfig()` in `llm.ts`. For `v-model` bindings that bypass store methods, a `watch` on the relevant refs can be used, or the panels should call an explicit store method on change.

---

### CR-02: NetworkProxyPanel entirely non-functional — uses local refs with no persistence path

**File:** `src/components/config/NetworkProxyPanel.vue`

**Issue:** Lines 47-49 declare local `ref`s for proxy configuration:

```typescript
const proxyEnabled = ref(false)
const proxyHttp = ref('')
const proxyHttps = ref('')
```

These refs have no connection to any Pinia store or composable. They are never:
- Loaded from the backend (no `loadConfig` call for proxy)
- Saved to the backend (`collectChangedFields()` in `useConfig.ts` does not emit proxy fields)
- Drafted to localStorage (`saveDraft()` at `useConfig.ts:63` reads `_lastConfig?.proxy` instead of UI state)
- Restored from draft (`restoreDraft()` does not handle `networkProxy` at all)

The template binds `v-model="proxyEnabled"`, `v-model="proxyHttp"`, and `v-model="proxyHttps"` to these local refs, so the UI appears functional. But the values are immediately discarded on any save, drawer close, or page reload. Users who configure a proxy will silently lose their configuration.

Additionally, `saveDraft()` at `useConfig.ts:63-65` reads `_lastConfig?.proxy` (the stale backend config snapshot from initial load), not the UI state, so the draft also stores the wrong values.

**Fix:** Either create a dedicated proxy store (similar to `bgm.ts`/`export.ts`) with `loadConfig`, `resetPanel`, and proper `dirty` tracking, OR add proxy fields to an existing store. Then:
1. Add proxy fields to `collectChangedFields()` / `collectAllConfig()`.
2. Add proxy restore logic to `restoreDraft()`.
3. Add proxy loading to `loadFromBackend()` if needed.

---

### CR-03: `collectChangedFields()` sends all TTS engine configs unconditionally, risking credential data loss

**File:** `src/composables/useConfig.ts`, lines 233-285

**Issue:** When `ttsStore.dirty` is `true`, `collectChangedFields()` emits every engine configuration field to the backend, including credential fields for engines the user never configured:

```typescript
if (ttsStore.dirty) {
  changes.tts_engine = ttsStore.engine
  const edge = ttsStore.engineConfigs.edge_tts
  changes.edge_voice_name = edge.voiceName        // maybe valid
  // ... all edge fields ...
  const azure = ttsStore.engineConfigs.azure_speech
  changes.azure_voice_name = azure.voiceName       // always ''
  changes.azure_speech_key = azure.speechKey       // always '' (never loaded)
  // ... 5 more engine configs ...
}
```

If the backend's `save_config` performs a simple merge (overwriting fields present in the payload), valid credential values for unused engines will be overwritten with empty strings. Since `ttsStore.engine` only reflects the currently selected engine, the typical save path is:

1. User selects `edge_tts`, configures it.
2. `restoreDraft()` (if triggered) sets `dirty = true`.
3. User clicks Save (assuming CR-01 were fixed).
4. `collectChangedFields()` sends `edge_*` fields (good) PLUS `azure_speech_key: ''`, `tencent_secret_id: ''`, `soulvoice_api_key: ''`, etc. (data loss).

The `|| ''` fallback at lines like `azure.speechKey || ''` (in `loadConfig()`) already drops these values during loading, so the UI always has empty credentials. Sending them back on save would overwrite any backend-stored values.

**Fix:** `collectChangedFields()` should only emit fields for the currently selected engine, or only emit non-empty values:

```typescript
if (ttsStore.dirty) {
  changes.tts_engine = ttsStore.engine
  const activeEngine = ttsStore.engine
  const config = ttsStore.engineConfigs[activeEngine]
  // Only emit fields for the active engine
  switch (activeEngine) {
    case 'edge_tts':
      changes.edge_voice_name = config.voiceName
      // ... only edge fields ...
      break
    case 'azure_speech':
      // ... only azure fields ...
      break
    // ... etc
  }
}
```

---

## Warnings

### WR-01: `saveDraft()` proxy capture inconsistent with `restoreDraft()`

**File:** `src/composables/useConfig.ts`

**Line:** 63-65 (saveDraft), 82-134 (restoreDraft)

**Issue:** `saveDraft()` at line 63 writes `networkProxy` from `_lastConfig.proxy` into the draft object. But `restoreDraft()` never reads `networkProxy` from the draft — the code path for restoring proxy settings is entirely absent. This means proxy settings are drafted to localStorage but never restored, making the draft storage for proxy a dead code path. Additionally, `saveDraft()` reads `_lastConfig` (the initial backend snapshot) rather than the current UI proxy values, so even if restoration logic were added, it would restore stale data.

**Fix:** Either remove `networkProxy` from `SettingsDraft` (since it cannot be saved/restored correctly) or add the missing restore logic and fix the save path to read from the actual UI state.

---

### WR-02: `saveToBackend` argument shape inconsistency

**File:** `src/composables/useConfig.ts`

**Line:** 207

**Issue:** `saveToBackend` calls `tauriInvoke('save_config', changes)` where `changes` is a flat `Record<string, unknown>`. In `src/composables/useTauri.ts`, `tauriInvoke` passes `args` as the second argument to `invoke<T>(command, normalizedArgs)`. The Tauri `invoke` API expects named arguments as a single object where each key is a parameter name. If the Rust command `save_config` expects a structured argument (e.g., `config: HashMap<String, String>`) rather than named arguments, passing `changes` directly could fail. The flat-object-as-named-arguments approach works for Tauri's `invoke` when the command takes a single struct parameter, but this should be verified against the Rust backend's signature.

**Fix:** Ensure `save_config` on the Rust side accepts a single flat JSON object parameter, not individual named fields.

---

### WR-03: ModeParamsPanel `v-model.number` on string field `dramaName`

**File:** `src/components/config/ModeParamsPanel.vue`

**Line:** 36-37

**Issue:** The `dramaName` field is typed as `string` in `ModeParams`:
```typescript
export interface ModeParams {
  dramaName: string
  // ...
}
```
But the template uses `v-model` (without `.number`):
```vue
<v-text-field v-model="params.dramaName" label="剧名" ... />
```
Wait, actually looking at the template again:
```vue
<v-text-field v-model="params.dramaName" label="剧名" .../>
```
This does NOT have `.number`. But the issue is that `dramaName` is a `string` field while the other fields like `frameInterval`, `visionBatchSize`, `clipCount`, `minDuration`, `maxDuration` all use `v-model.number`. The `.number` modifier converts input with `parseFloat`, which for number fields is correct. The `dramaName` string field should NOT use `.number`, and it currently does not. This is not actually a bug. Let me remove this finding.

Actually, revisiting the template for ModeParamsPanel.vue:

Lines 16-18 (documentary - frameInterval):
```vue
<v-text-field v-model.number="params.frameInterval" ... type="number" />
```

Lines 25-28 (documentary - visionBatchSize):
```vue
<v-text-field v-model.number="params.visionBatchSize" ... type="number" />
```

Lines 39-42 (sde - dramaName):
```vue
<v-text-field v-model="params.dramaName" label="剧名" ... />
```

Lines 58-64 (sdp - clipCount):
```vue
<v-text-field v-model.number="params.clipCount" ... type="number" />
```

Lines 67-72 (sdp - minDuration):
```vue
<v-text-field v-model.number="params.minDuration" ... type="number" />
```

Lines 76-81 (sdp - maxDuration):
```vue
<v-text-field v-model.number="params.maxDuration" ... type="number" />
```

The string field `dramaName` uses `v-model` (no modifier) which is correct. But the document mode's SDE template also has a slider for `temperature`:
```vue
<v-slider v-model="params.temperature" ... />
```

This is fine too. Let me skip this finding.

---

### WR-04: Dead function `handleEngineChange` in TtsPanel.vue

**File:** `src/components/config/TtsPanel.vue`

**Line:** 416-418

**Issue:** The function `handleEngineChange` is defined but never referenced in the template or anywhere else. The engine selection is handled entirely by `v-model="store.engine"` on the `<v-select>`. This is dead code.

**Fix:** Remove the function.

---

### WR-05: Empty catch blocks in BgmPanel and ExportPanel

**File:** 
- `src/components/config/BgmPanel.vue`, line 68-70
- `src/components/config/ExportPanel.vue`, line 56-58

**Issue:** Both `selectBgmFolder()` and `selectExportDir()` have empty `catch {}` blocks that silently swallow all errors from the Tauri dialog import. If the `@tauri-apps/plugin-dialog` import fails in a non-obvious way (beyond the expected "not installed in dev mode"), the error is swallowed without any feedback. This makes debugging difficult.

**Fix:** Add a `console.warn` or a user-visible notification (snackbar) when the dialog cannot be opened:

```typescript
} catch (err) {
  console.warn('[BgmPanel] 无法打开文件夹选择对话框:', err)
}
```

---

### WR-06: TTS `loadConfig()` uses unsafe type assertions for credential fields

**File:** `src/stores/tts.ts`

**Lines:** 119, 124-125, 129-130, 136-137, 153-155, 156-157

**Issue:** Multiple credential fields accessed via `(config.azure as unknown as Record<string, string>).speech_key` pattern. The generated types (e.g., `AzureSection`, `TencentSection`, `SoulVoiceSection`, `TtsQwenSection`, `DoubaoTTSSection`) do not declare these credential fields, yet the runtime config object presumably contains them. The `as unknown as Record<string, string>` cast bypasses TypeScript safety entirely.

Examples:
- `AzureSection` has only `{ speech_region: string }` but code accesses `speech_key`
- `TencentSection` has only `{ region: string }` but code accesses `secret_id`, `secret_key`
- `SoulVoiceSection` lacks `api_key` but code accesses it
- `DoubaoTTSSection` lacks `ak`, `sk`, `token` but code accesses them

The `|| ''` fallback means these fields silently default to empty strings. If the backend does not include these credentials in the `AppConfig` object (e.g., they're environment variables), then the credential fields in the UI are always empty and the user must re-enter them every time. This is a data persistence gap.

**Fix:** Either add the credential fields to the generated Rust types so they're included in `AppConfig`, or explicitly document that these fields are environment-variable-only and adjust the UI to reflect that (e.g., show "已通过环境变量配置" instead of an empty text field).

---

### WR-07: `IndexTTS2EngineConfig` missing `repetition_penalty` field

**File:** 
- `src/stores/tts.ts` — `IndexTTS2EngineConfig` interface, lines 41-50
- `src/types/generated/IndexTTS2Section.ts` — line 15 has `repetition_penalty: number`

**Issue:** The Rust-generated type declares `repetition_penalty` as a field of the `indextts2` config section, but the frontend model `IndexTTS2EngineConfig` does not include it, and `loadConfig()` does not map it. This means the field is silently dropped during config load and never exposed in the UI. If a user sets this value in their `config.toml`, it is lost when the UI saves the config.

**Fix:** Add `repetition_penalty: number` to the `IndexTTS2EngineConfig` interface and include it in `loadConfig()`:

```typescript
export interface IndexTTS2EngineConfig {
  // ... existing fields ...
  repetition_penalty: number
}
```

And in `loadConfig()`:
```typescript
engineConfigs.value.indextts2 = {
  // ... existing fields ...
  repetition_penalty: config.indextts2.repetition_penalty,
}
```

---

### WR-08: `modeStore.loadConfig()` silences `frame_interval_input` type conversion

**File:** `src/stores/mode.ts`

**Line:** 43

**Issue:** `Number(config.frames.frame_interval_input)` converts a `bigint` to `number`. The `FramesSection` type declares `frame_interval_input: bigint` (in `src/types/generated/FramesSection.ts`). If the value exceeds `Number.MAX_SAFE_INTEGER` (9,007,199,254,740,991), precision loss occurs silently with no error or warning. For a frame interval (typically 1-30), this is unlikely to be a real problem, but it's a silent type safety gap.

**Fix:** This is acceptable for practical frame_interval values but should be documented. A runtime assertion would be belt-and-suspenders:

```typescript
params.value.frameInterval = Number(config.frames.frame_interval_input)
```

---

### WR-09: `v-model.number` on text fields can produce NaN on empty input

**File:** Multiple files

**Files:**
- `src/components/config/TtsPanel.vue` — all `v-model.number` bindings (lines 36, 46, 56, 96, 106, 116, 247, 258, 269, 287)
- `src/components/config/ModeParamsPanel.vue` — lines 16, 25, 60, 68, 78

**Issue:** The `.number` modifier uses `parseFloat` to coerce the input string to a number. When the user clears a numeric text field (deletes all text), the empty string `''` is passed through `parseFloat('')` which returns `NaN`. This `NaN` value gets written into the store ref as the property value. Subsequent operations that use this value (e.g., arithmetic, comparison) may produce unexpected results. For example, `volume > 0` would be `false` for `NaN`, and `NaN` sent via JSON to the backend becomes `null` during serialization.

**Fix:** Use a custom handler or input validation that prevents NaN from reaching the store:

```typescript
function updateNumeric(field: string, val: string) {
  const parsed = parseFloat(val)
  if (!isNaN(parsed)) {
    edgeCfg.value[field] = parsed
  }
}
```

Or apply min/max clamping after parsing in the store methods.

---

### WR-10: BgmPanel `selectedFile` never persisted

**File:** `src/components/config/BgmPanel.vue`

**Lines:** 37-44, 58

**Issue:** The `<v-select>` for choosing a BGM file binds to a local `ref`:
```typescript
const selectedFile = ref('')
```
This value is never written to any store, never included in `saveDraft()`, and never saved to the backend. When the drawer closes and reopens, the previously selected file is lost. The `store.audioFiles` list is loaded from the store, but the selected file value is held only in the component's local state.

**Fix:** Either persist `selectedFile` to the `bgm` store (e.g., add `selectedFile: string` to the store and write to it on selection), or remove the file selection feature if it's not intended for MVP.

---

### WR-11: `versionText` hardcoded fallback is incorrect

**File:** `src/components/SettingsDrawer.vue`

**Line:** 214

**Issue:** `versionText` is initialized as `'v0.1.0'` (line 214) and overridden in `onMounted` by the Tauri `getVersion()` API (lines 483-488). The fallback value `v0.1.0` does not match the actual project version stated in `CLAUDE.md` (version 0.7.8). This is misleading during development outside Tauri (e.g., in a browser). The hardcoded fallback at line 487 also uses `v0.1.0`.

**Fix:** Update the default to match the project's actual version:

```typescript
const versionText = ref('v0.7.8')
```

Or read it from the `project_version` file or `package.json`.

---

_Reviewed: 2026-05-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
