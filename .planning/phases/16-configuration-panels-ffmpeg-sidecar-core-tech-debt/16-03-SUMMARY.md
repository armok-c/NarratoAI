---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
plan: 03
type: execute
subsystem: frontend
tags:
  - pinia
  - types
  - config
  - stores
  - composables
dependency-graph:
  requires:
    - "16-01 (ffmpeg sidecar)"
    - "Phase 14 (Pinia stores skeleton)"
  provides:
    - "Wave 1 config panels (16-04+)"
    - "type imports for all panel components"
  affects:
    - src/stores/llm.ts
    - src/stores/tts.ts
    - src/stores/bgm.ts
    - src/stores/export.ts
    - src/stores/mode.ts
    - src/types/index.ts
    - src/types/generated/
    - src/composables/useConfig.ts
    - src/components/config/ProviderPresets.ts
tech-stack:
  added:
    - "ts-rs generated types (12 files)"
    - "Pinia stores with full field coverage"
  patterns:
    - "loadConfig(config: AppConfig) dispatcher pattern"
    - "dirty ref for unsaved change tracking"
    - "composable-based backend config IO"
    - "typed engine config interfaces"
key-files:
  created:
    - src/types/generated/AppConfig.ts
    - src/types/generated/AppSection.ts
    - src/types/generated/UiSection.ts
    - src/types/generated/ProxySection.ts
    - src/types/generated/FramesSection.ts
    - src/types/generated/AzureSection.ts
    - src/types/generated/TencentSection.ts
    - src/types/generated/SoulVoiceSection.ts
    - src/types/generated/TtsQwenSection.ts
    - src/types/generated/IndexTTS2Section.ts
    - src/types/generated/DoubaoTTSSection.ts
    - src/types/generated/AudioSection.ts
    - src/composables/useConfig.ts
    - src/components/config/ProviderPresets.ts
  modified:
    - src/types/index.ts
    - src/stores/llm.ts
    - src/stores/tts.ts
    - src/stores/bgm.ts
    - src/stores/export.ts
    - src/stores/mode.ts
decisions:
  - "UiSection.ts fixed from ts-rs bug (file exported AppConfig instead of UiSection type)"
  - "Engine configs typed as separate interfaces rather than generic Record"
  - "mode params merged into mode store per D-34, RESEARCH.md Q4 resolution"
  - "collectChangedFields reads dirty flags per-store for incremental save_config"
metrics:
  duration: null
  files_changed: 20
  commits: 3
---

# Phase 16 Plan 03: 状态管理层扩展 — Stores + Composables + 类型同步

**One-liner:** 将 5 个 Pinia store 骨架扩展为完整字段实现 + 创建 useConfig.ts composable + ProviderPresets，同步 ts-rs 生成 TypeScript 类型，为 Wave 1 配置面板提供完整状态管理层。

## Context

本 Plan 是 Wave 1（前端配置面板）的基础状态层。Pinia stores 从骨架状态扩展为可存储/加载/重置完整配置字段的容器，useConfig.ts 封装 get_config/save_config 后端通信，ProviderPresets 提供 11 个 LLM Provider 的推荐映射。

## Tasks

### Task 1: 同步 ts-rs 生成类型 + 创建 src/types/generated/ 目录

- **Commit:** `424b6b2`
- **Files created:** 12 个 .ts 文件（AppConfig, AppSection, UiSection, ProxySection, FramesSection, AzureSection, TencentSection, SoulVoiceSection, TtsQwenSection, IndexTTS2Section, DoubaoTTSSection, AudioSection）
- **Files modified:** `src/types/index.ts` — 添加所有 generated 类型的 re-export
- **Deviation:** UiSection.ts 内容为 AppConfig（ts-rs 生成 bug），已修复为正确的 UiSection 类型定义
- **Verification:** `npx vue-tsc --noEmit` 通过

### Task 2: 扩展 5 个配置 Pinia stores + mode store

- **Commit:** `ea1ed18`
- **Files modified:**
  - `src/stores/llm.ts` — 添加 loading/dirty refs, loadConfig, testConnection, resetPanel, markClean/markDirty
  - `src/stores/tts.ts` — 7 引擎类型接口 (EdgeEngineConfig ~ DoubaoEngineConfig), voiceList 缓存, loadConfig/loadVoices/resetPanel
  - `src/stores/bgm.ts` — 添加 audioFiles/loading/dirty, loadConfig/resetPanel
  - `src/stores/export.ts` — 添加 loading/dirty, loadConfig/resetPanel
  - `src/stores/mode.ts` — 添加 params 对象 (frameInterval, visionBatchSize, dramaName, temperature, clipCount, minDuration, maxDuration), loadConfig/resetParams
- **Verification:** `npx vue-tsc --noEmit` 通过

### Task 3: 创建 useConfig.ts composable + ProviderPresets.ts

- **Commit:** `7189631`
- **Files created:**
  - `src/composables/useConfig.ts` — loadFromBackend (get_config + dispatch to stores), saveToBackend (save_config), collectChangedFields (dirty field aggregation)
  - `src/components/config/ProviderPresets.ts` — PROVIDER_PRESETS (11 presets), PROVIDER_OPTIONS (12 items for v-select), getProviderPreset
- **Verification:** `npx vue-tsc --noEmit` 通过

## Verification

All acceptance criteria met:
- `src/types/generated/` 包含 12 个 .ts 文件
- `src/types/index.ts` 导出 AppConfig 及所有子类型
- 5 个 stores 扩展完成，TypeScript 编译通过
- `useConfig.ts` 和 `ProviderPresets.ts` 创建完成

TypeScript compilation: `npx vue-tsc --noEmit` — green (exit 0)

## Deviations from Plan

[Rule 2 - Missing Critical Functionality] **Fixed UiSection.ts type definition**
- **Found during:** Task 1
- **Issue:** ts-rs 生成的 UiSection.ts 文件内容与 AppConfig.ts 完全相同（导出 AppConfig 而非 UiSection 类型），导致 TypeScript 无法解析 UiSection 类型
- **Fix:** 根据 `narratoai-core/src/config/types.rs` 中 `UiSection` 结构体字段重新编写正确的类型定义
- **Files modified:** `src/types/generated/UiSection.ts`
- **Commit:** `424b6b2`
- **Rationale:** UiSection 类型缺失会导致前端无法导入并使用该类型，违反 CORRECTNESS 原则

## Known Stubs

None — all stores have full field coverage matching their acceptance criteria. BGM and Export stores have stub `loadConfig` implementations (pure defaults) as documented in the plan — these configs are not managed by the backend AppConfig.

## Threat Surface Scan

No new threat surface introduced. All IPC paths through `useConfig.ts` use the existing `tauriInvoke` wrapper which has sensitive field filtering and debug-mode gating.

## Self-Check: PASSED

- `src/types/generated/*.ts` — 12 files confirmed present
- `src/types/index.ts` — AppConfig + all sub-types re-exported
- `src/stores/llm.ts` — visionConfig/textConfig + loadConfig/testConnection/resetPanel/markClean/markDirty
- `src/stores/tts.ts` — engine/engineConfigs/voiceList + loadConfig/loadVoices/resetPanel + dirty
- `src/stores/bgm.ts` — folder/mode/audioFiles + loadConfig/resetPanel + dirty
- `src/stores/export.ts` — outputDir/format + loadConfig/resetPanel + dirty
- `src/stores/mode.ts` — params (7 fields) + loadConfig/resetParams
- `src/composables/useConfig.ts` — loadFromBackend/saveToBackend/collectChangedFields
- `src/components/config/ProviderPresets.ts` — PROVIDER_PRESETS (11) + PROVIDER_OPTIONS (12) + getProviderPreset
- `npx vue-tsc --noEmit` — exit 0
