# Feature Landscape

**Domain:** Tauri + Vue 3 frontend for AI video narration desktop app (v2.0)
**Researched:** 2026-05-12
**Confidence:** HIGH (SmartEdit reference implementation + full Rust backend analysis)
**Context:** v2.0 milestone — adding Vue 3 + Vuetify 3 frontend to existing NarratoAI Rust/Tauri 2.0 backend, cleaning 6 tech debt items

## Feature Landscape

### Table Stakes (Users Expect These)

Missing these = product feels incomplete for a desktop video narration tool.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **AppHeader with title + settings + theme toggle** | Every desktop app has a title bar with system controls and a settings entry point | LOW | SmartEdit has `AppHeader.vue` pattern: logo, title, settings button (cog icon), theme toggle, window controls. NarratoAI reuses this 1:1. Replace "SmartEdit" title with "NarratoAI". |
| **HomeView with video table** | Primary workspace — user sees uploaded videos, their status, and can act on them | MEDIUM | SmartEdit's `HomeView.vue` has flex layout: video table (left 60%) + settings card (right 40%) on top, action button row + log panel on bottom. NarratoAI needs this exact layout. |
| **Video upload button** | Users must add video files to start processing | LOW | SmartEdit's "上传" button triggers Tauri file dialog. NarratoAI backend Tauri commands already handle video path. |
| **Start / Stop / Clear pipeline controls** | Core workflow: start processing, abort if something's wrong, clear state | LOW | SmartEdit has 5 button row: 上传/开始/停止/清空/文件夹. NarratoAI backend exposes `run_documentary`/`run_sde`/`run_sdp` + `stop_all_video_processing` + `clear_all_video_runtime_data`. Maps directly. |
| **Real-time log panel** | User must see what the pipeline is doing (especially for 5-20 min processing) | MEDIUM | SmartEdit `LogPanel.vue` subscribes to Tauri events, auto-scrolls. NarratoAI uses `tauri::Emitter` (`pipeline-progress` event) — frontend listens via `@tauri-apps/api/event`. |
| **Script preview dialog** | After LLM generates narration, user needs to review/edit before final render | MEDIUM | SmartEdit `ScriptPreviewDialog.vue` shows structured preview (summary, clips with visual/ narration/ timing). Backend returns `ScriptResult` JSON. NarratoAI needs same dialog, adapted per mode. |
| **Settings drawer** | Users configure LLM keys, TTS voice, BGM, export options | MEDIUM | SmartEdit `SettingsDrawer.vue` is right-side temporary drawer with ModelConfigForm (vision + text), SettingSection (theme, advanced). NarratoAI needs additional sections for mode selection. |
| **Dark/light theme toggle** | Desktop app standard, especially for video editors used in varied lighting | LOW | SmartEdit uses `useTheme()` composable + localStorage persistence. Vuetify 3 has native dark/light support. Trivial to implement. |
| **System resource monitor** | Users want to see CPU/RAM usage during processing to gauge performance | LOW | SmartEdit `SystemMonitorBar.vue` subscribes to Tauri `system-resources` events. NarratoAI needs `start_system_monitor`/`stop_system_monitor` Tauri commands exposed. |
| **Export profile selection** | Users control output quality/speed tradeoff (Speed/Balanced/Quality) | LOW | SmartEdit has 3-option `v-btn-toggle` in Export tab. NarratoAI backend has `AppConfig.export_profile` field. |
| **BGM configuration panel** | Background music is standard for narration videos | LOW | SmartEdit `BgmPanel.vue` has folder picker, random/specific selection, BGM file list. Maps directly. |
| **TTS engine selection + voice picker** | Users need to choose voice quality and style | MEDIUM | SmartEdit `TtsPanel.vue` has engine toggle (Edge TTS / Qwen3 TTS), voice selector, proxy config. NarratoAI backend has 7 TTS engines. Frontend needs engine selector + voice list. |

### Differentiators (Competitive Advantage)

Features that set NarratoAI apart from generic video editors or competing narration tools.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **3-mode switch in settings drawer** | Single app covers 3 different content creation workflows. Documentary (frame analysis narration), SDE (subtitle-based drama editing), SDP (subtitle-based highlight mixing). Competitors typically only do one. | MEDIUM | Key UX challenge: the mode switch changes which panels/fields are visible and how pipeline behaves. SmartEdit only has 1 mode (SDE). NarratoAI must handle 3. Mode selection goes in the LLM/mode tab as a dropdown or card selector. |
| **LLM dual-model config (Vision + Text)** | Documentary mode uses TWO separate LLM models: one for frame analysis (vision) and one for narration generation (text). Most tools only use one. | MEDIUM | SmartEdit `SettingsDrawer.vue` already has separate `ModelConfigForm` for vision and text. NarratoAI reuses this but: (1) Vision model config is only needed in Documentary mode, (2) Text model config is needed in all 3 modes, (3) SDE/SDP only need text model. Frontend must conditionally show/hide the vision config section based on mode. |
| **Dynamic settings panels per mode** | Each mode has different parameters. Documentary needs frame interval + vision batch size. SDE needs drama name + temperature. SDP needs custom clip count + no TTS settings. | MEDIUM | The 4 tabs (LLM/TTS/BGM/Export) must dynamically show/hide fields based on mode. SmartEdit's static panels won't work — NarratoAI needs: (1) Mode-aware field visibility, (2) TTS tab hidden in SDP mode (no narration), (3) Clip count slider shown in SDP, (4) Drama name field shown in SDE. |
| **Script preview adapted per mode** | Documentary preview shows visual analysis frames + narration pairs. SDE preview shows subtitle segments + OST types. SDP preview shows highlight clip selection. | MEDIUM | SmartEdit `ScriptPreviewDialog.vue` has a fixed layout (summary + items with visual/narration). NarratoAI needs mode-specific rendering: (1) Documentary: vision analysis snippets + narration text, (2) SDE: subtitle text + OST badges + narration, (3) SDP: clip selection overview + no narration. |
| **OST type visualization in video table** | Each video segment has an OST type (0=narration only, 1=original audio only, 2=mixed). Visualizing this helps users understand the editing strategy. | LOW | Show OST type as colored chips in the script preview or video table. Simple but powerful for power users. |
| **Progress tracking per pipeline step** | Users see exactly which of the 6 (documentary), 9 (SDE), or 3 (SDP) steps is running with percentage. | LOW | SmartEdit has basic progress. NarratoAI backend emits per-step progress via `pipeline-progress` event with `step_index` and `total_steps`. Frontend shows step progress bar with step labels. |
| **Folder open button** | Quick access to output directory from the app | LOW | SmartEdit has "文件夹" button calling `openOutputFolder`. Maps directly. |
| **FFmpeg sidecar bundled** | Users don't need to install FFmpeg separately — it's included in the installer | HIGH | Tech debt item. Requires bundling FFmpeg binary via Tauri sidecar config. SmartEdit does not have this (relies on system FFmpeg). Complexity is in build/packaging, not frontend. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems for this project.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **FFmpeg management panel** | SmartEdit has `FfmpegPanel.vue` (path config, version check, download button). Users may want to see/change FFmpeg settings. | PROHIBITED by milestone spec: "No FFmpeg management panel." FFmpeg is bundled as sidecar — user doesn't need to configure it. Managing path/version adds complexity and support burden. | Auto-detect + bundle. Show FFmpeg status only in settings (version + encoding support) without config controls. |
| **Multi-video parallel processing** | Users might want to process many videos simultaneously to save time. | Desktop app constraints: CPU/GPU encoding is resource-intensive. Parallel FFmpeg instances compete for hardware encoders. Memory pressure from multiple frame extractions. Complexity outweighs value for single-user desktop. | Sequential queue with single-threaded pipeline. SmartEdit queue pattern (videos processed one at a time). |
| **Real-time waveform/preview** | Video editors typically show audio waveform and real-time playback preview. | Massive scope increase. Requires embedding a video player (like mpv or VLC), frame-accurate seeking, WebGL rendering. Dwarfs the current frontend scope. | Static script preview + FFmpeg-generated timing information. No real-time playback. |
| **Multi-language UI** | SmartEdit is fully Chinese. English users may want localization. | Localization is a horizontal concern touching every component. Adds translation management, i18n framework, and ongoing maintenance. Not requested by target users (Chinese content creators). | Chinese-only UI for v2.0. If international demand materializes, add vue-i18n in a future milestone. |
| **Custom FFmpeg filter graph editor** | Power users may want fine control over video processing. | Extreme complexity. FFmpeg filter graphs are arcane. Building a visual filter editor is a standalone project. The app's value is automation, not manual filter tuning. | Preset-based output profiles (Speed/Balanced/Quality). Users who need custom filters can use FFmpeg directly on the output. |
| **Subtitle editor** | Users may want to edit subtitle text/timing in-app. | SmartEdit doesn't have this either. Requires text editor component, timeline visualization, sync with audio. Massive scope. | Users edit subtitles in external tools. The app reads existing subtitle files (SRT/ASS). |

## Feature Dependencies

```
Config System (Rust back-end) ──provides──> Frontend config load[#1]
                                               |
                        +------------------------+------------------------+
                        |                        |                        |
                        v                        v                        v
          Video Table[#2]              Settings Drawer[#3]        Log Panel[#4]
                        |                        |                        |
                        v                        v                        |
          Start/Stop/Clear[#5]        LLM Config (Vision+Text)           |
                        |                        |                        |
                        v                        v                        |
          Pipeline Progress[#6] <── Tauri Events ──+──────────────────────+
                        |
                        v
          Script Preview Dialog[#7]
                        |
                        v
          Export Output Folder[#8]

Three-Mode UX Split:
  Documentary:  Video Input -> Vision LLM -> Text LLM -> Script -> TTS -> FFmpeg Pipeline -> Output
  SDE:          Video + Subtitle -> Subtitle Parse -> LLM Analysis -> Script -> TTS -> FFmpeg -> Output
  SDP:          Video + Subtitle -> Subtitle Parse -> LLM Analysis -> Script -> FFmpeg (no TTS) -> Output
                 ^^^                                           ^^^                ^^^
                 No vision config                              Only text LLM      TTS panel hidden
                                                            + drama_name param   + custom_clips param
```

### Dependency Notes

- **[#1] Config System** — Frontend starts by loading `AppConfig` from Rust backend via Tauri command. Mode selection persists across sessions.
- **[#2] Video Table** — `VideoTable.vue` needs video list from backend (`fetchVideos` command). Each video row shows file name, topic (editable), subtitle association, status chip, action buttons (view script, delete).
- **[#3] Settings Drawer** — Contains 4 tabs (LLM/Mode, TTS, BGM, Export). Tab visibility and field content vary by selected mode. Each tab's save triggers Tauri command to persist config.
- **[#5] Pipeline Controls** — Action buttons (Upload/Start/Stop/Clear/Folder) are disabled based on pipeline status. Start button calls one of 3 Tauri commands depending on mode: `run_documentary`, `run_sde`, or `run_sdp`.
- **[#6] Pipeline Progress** — Progress events flow Rust -> Tauri Event -> Frontend listener. Frontend updates log panel + video table status. Step progression is mode-dependent (6/9/3 steps).
- **[#7] Script Preview** — `ScriptPreviewDialog.vue` calls `get_script_by_video_id` Tauri command. Rendering differs by mode: documentary shows vision frames + narration, SDE shows subtitles + OST, SDP shows clip selection.

## Three-Mode UX Mapping: SmartEdit Pattern to NarratoAI

SmartEdit is single-mode (SDE). NarratoAI must extend to 3 modes. Here's the exact mapping:

### AppHeader → NarratoAI

| SmartEdit Element | NarratoAI Adaptation | Rationale |
|-------------------|---------------------|-----------|
| Logo + "SmartEdit" title | Logo + "NarratoAI" title | Brand difference |
| Settings button (gear icon) | Same — opens SettingsDrawer | Identical UX |
| Theme toggle button | Same — dark/light switch | Identical UX |
| WindowControls (min/max/close) | Same | Identical — Tauri 2.0 |
| Title bar double-click → maximize | Same | Identical UX |

### HomeView → NarratoAI

| SmartEdit Element | NarratoAI Adaptation | Rationale |
|-------------------|---------------------|-----------|
| Flex layout: upper (video + settings) / lower (buttons + log) | Same layout | Proven ergonomic split |
| Single SDE mode only | Mode-aware behavior: 3 modes change panel content | Core differentiator |
| LLM tab with ScriptGenerationForm | LLM/Mode tab — adds mode selector as first control. Fields change per mode. | Mode selection must be discoverable immediately |

### SettingsDrawer → NarratoAI

| SmartEdit Element | NarratoAI Adaptation | Rationale |
|-------------------|---------------------|-----------|
| Right temporary drawer, 420px width | Same | Identical UX |
| ModelConfigForm (vision) + ModelConfigForm (text) | Vision model section HIDDEN in SDE/SDP mode. Text model shown always. | Documentary needs both, SDE/SDP only need text |
| Advanced settings (timeout) | Same, plus: frame interval, vision batch size (shown only in Documentary mode) | Mode-scoped fields |
| FFmpegPanel (collapsible) | REMOVED per spec (no FFmpeg management) | Anti-feature |
| Save button at bottom | Same | Identical UX |

### Settings Tabs → NarratoAI (Mode-Specific Content)

**LLM/Mode Tab:**

| Field | Documentary | SDE | SDP |
|-------|------------|-----|-----|
| Mode selector (dropdown/cards) | SHOWN — top of tab | SHOWN — top of tab | SHOWN — top of tab |
| Vision model config | SHOWN | HIDDEN | HIDDEN |
| Text model config | SHOWN | SHOWN | SHOWN |
| Temperature slider | SHOWN | SHOWN | SHOWN (default 0.1 lower) |
| Max tokens | SHOWN | SHOWN | SHOWN |
| Drama name input | HIDDEN | SHOWN | SHOWN |
| Custom clips count | HIDDEN | HIDDEN | SHOWN |
| Frame interval | SHOWN | HIDDEN | HIDDEN |
| Vision batch size | SHOWN | HIDDEN | HIDDEN |

**TTS Tab:**

| Field | Documentary | SDE | SDP |
|-------|------------|-----|-----|
| Engine selector | SHOWN | SHOWN | HIDDEN (entire tab) |
| Voice selector | SHOWN | SHOWN | HIDDEN |
| Voice rate/pitch | SHOWN | SHOWN | HIDDEN |
| Proxy settings | SHOWN | SHOWN | HIDDEN |
| Duration offset | SHOWN | SHOWN | HIDDEN |

**BGM Tab:**
Identical across all 3 modes (folder selector, random/specific, BGM file list).

**Export Tab:**
Nearly identical across all 3 modes. Only difference: SDP may default to different export profile (Speed as default vs Balanced).

### Button Row → NarratoAI

| Button | SmartEdit | NarratoAI |
|--------|-----------|-----------|
| 上传 (Upload) | Triggers file dialog | Same — single or multiple video files |
| 开始 (Start) | Calls `start_video_pipeline(mode, video_ids)` | Calls `run_documentary`/`run_sde`/`run_sdp` based on mode |
| 停止 (Stop) | Calls `stop_all_video_processing()` | Same — backend already supports this |
| 清空 (Clear) | Calls `clear_all_video_runtime_data()` | Same — backend already supports this |
| 文件夹 (Folder) | Opens output dir | Same — backend already has `open_output_folder` |
| SystemMonitorBar | CPU/RAM/GPU bars | Same — needs `start_system_monitor` Tauri command exposed (tech debt) |

### VideoTable → NarratoAI

| Column | SmartEdit (SDE-only) | NarratoAI (3-mode) |
|--------|---------------------|-------------------|
| Name | Video filename | Same |
| Topic | Editable text field | Same |
| Subtitle | Subtitle file name | SAME (subtitle only for SDE/SDP). Documentary shows "--" because no subtitle needed. |
| Status | Combined status chip | EXTENDED — status labels are mode-specific ("Script Generating" for doc, "Subtitle Parsing" for SDE, "Clip Analysis" for SDP) |
| Actions | View script (emoji) + Delete | Same — view script calls mode-specific preview rendering |

### LogPanel → NarratoAI

Identical to SmartEdit — mode-independent. Log entries come from Tauri events. Log level, message, timestamp.

### ScriptPreviewDialog → NarratoAI

| Section | Documentary | SDE | SDP |
|---------|------------|-----|-----|
| Summary | Vision analysis summary + topic | Plot analysis + drama overview | Clip selection summary |
| Items | Clip index + narration text + timing | Subtitle text + OST chip + narration | Clip index + timing only (no narration) |
| Item details | Visual analysis snippets | Subtitle text | Clip highlights |
| Export button | Export script JSON | Same | Same |

## Tech Debt Feature Mapping

The milestone specifies 6 tech debt items. Here's their frontend exposure:

| Tech Debt | Frontend Relevance | Frontend Work Required |
|-----------|-------------------|----------------------|
| **Edge-TTS proxy tunnel** | Zero — purely backend. Frontend already passes proxy config from TTS panel. Backend must actually use it. | None (backend only) |
| **Tauri State read lock optimization** | Zero — purely backend architecture. No frontend changes. | None (backend only) |
| **Integration test stubs -> real** | Zero — purely backend testing. | None (backend only) |
| **YouTube/Pexels Tauri commands** | LOW — if exposed, frontend could add "Import from YouTube" button and "Search Pexels" in BGM panel. | Optional: Add YouTube URL input + download button. Add Pexels search field in BGM panel. Can defer to after MVP. |
| **ConfigManager hot-reload** | LOW — frontend could show a "config changed externally" notification and offer reload. | Optional: Watch for `config-changed` Tauri event. Can defer. |
| **VisualAnalyzer facade** | ZERO — purely backend. The visual analyzer is called by documentary pipeline. No frontend change. | None (backend only) |

## MVP Definition

### Launch With (v2.0 Frontend)

These are the minimum features for the v2.0 frontend to be usable:

- [x] **AppHeader** — Logo, title, settings button, theme toggle, window controls
- [x] **HomeView layout** — Upper (video table + settings card) / Lower (button row + log panel) split
- [x] **VideoTable** — Display videos with name, topic, subtitle, status, actions
- [x] **Mode selector** — Dropdown in LLM/Mode settings tab to switch between Documentary/SDE/SDP
- [x] **Dynamic panels per mode** — Fields show/hide based on selected mode (see tables above)
- [x] **Upload + Start + Stop + Clear + Folder buttons** — Core workflow controls
- [x] **LLM dual model config** — Vision + Text model forms in settings drawer
- [x] **TTS config panel** — Engine selection, voice picker, proxy settings
- [x] **BGM config panel** — Folder picker, random/specific mode
- [x] **Export config panel** — Profile selection (Speed/Balanced/Quality), full load toggle
- [x] **Log panel** — Real-time log display with auto-scroll, export, clear
- [x] **Script preview dialog** — Mode-aware script preview with export
- [x] **SystemMonitorBar** — CPU/RAM usage display
- [x] **Dark/light theme** — Toggle with persistence
- [x] **Settings drawer** — Right-side temporary drawer with all config panels

### Add After Validation (v2.1)

Features that enhance but aren't blocking launch:

- [ ] **YouTube import** — URL input field + download button in upload area (requires YouTube Tauri command tech debt)
- [ ] **Pexels search** — Keyword search field in BGM panel (requires Pexels Tauri command tech debt)
- [ ] **Config hot-reload notification** — Toast when config.toml changes externally
- [ ] **Pipeline step progress bar** — Visual progress indicator showing which of 6/9/3 steps is active
- [ ] **OST type visualization** — Color-coded OST chips in script preview
- [ ] **Export destination path picker** — Allow choosing output directory before starting

### Future Consideration (v3+)

Features deferred until v2.0 is stable:

- [ ] **Multi-video management** — Folders, groups, batch operations
- [ ] **Pipeline queue visualization** — Visual queue showing upcoming/pending videos
- [ ] **Custom subtitle styling** — Font selector, position preview, outline/shadow
- [ ] **Audio waveform visualization** — Waveform display in script preview
- [ ] **Local STT (whisper)** — Generate subtitles from video audio
- [ ] **Clip timeline viewer** — Timeline showing clips with OST markers

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| HomeView layout (video + log split) | CRITICAL | LOW | **P1** |
| AppHeader with settings/theme | CRITICAL | LOW | **P1** |
| VideoTable with status | CRITICAL | MEDIUM | **P1** |
| Upload/Start/Stop/Clear buttons | CRITICAL | LOW | **P1** |
| Mode selector (3-mode switch) | CRITICAL | MEDIUM | **P1** |
| LLM dual model config | CRITICAL | MEDIUM | **P1** |
| TTS config panel | CRITICAL | MEDIUM | **P1** |
| BGM config panel | HIGH | LOW | **P1** |
| Export config panel | HIGH | LOW | **P1** |
| Log panel | CRITICAL | LOW | **P1** |
| Script preview dialog | HIGH | MEDIUM | **P1** |
| Dynamic panel fields per mode | HIGH | MEDIUM | **P1** |
| Dark/light theme | MEDIUM | LOW | **P1** |
| SystemMonitorBar | MEDIUM | LOW | **P1** |
| Settings drawer | CRITICAL | LOW | **P1** |
| Pipeline step progress bar | HIGH | LOW | **P2** |
| OST type visualization | MEDIUM | LOW | **P2** |
| YouTube import | LOW | MEDIUM | **P3** |
| Pexels search | LOW | MEDIUM | **P3** |
| Config hot-reload notification | LOW | LOW | **P3** |
| Export destination path picker | MEDIUM | LOW | **P2** |
| Multi-video folder management | LOW | HIGH | **P3** |

**Priority key:**
- **P1:** Must have for v2.0 launch (17 features)
- **P2:** Should have, add when possible (3 features)
- **P3:** Nice to have, future consideration (3 features)

## Key 3-Mode UX Design Decisions

### Decision 1: Where does the mode selector live?

**Recommendation:** Inside the LLM/Mode settings tab, as a card-level selector at the top — NOT in the AppHeader.

**Rationale:**
- Mode is a per-project setting that changes infrequently — it belongs in settings, not chrome
- SmartEdit already has the LLM tab as the first tab — adding mode selection there is natural
- Avoids cluttering the header with mode-specific UI
- The mode change triggers field visibility changes in the same panel — tight coupling is good here

**Implementation:**
```vue
<!-- In the LLM/Mode tab, top section -->
<v-card variant="outlined" class="mb-3">
  <v-card-text class="pa-3">
    <div class="text-caption text-high-emphasis mb-2 font-weight-medium">
      工作模式
    </div>
    <v-select
      v-model="selectedMode"
      :items="modeOptions"
      item-title="label"
      item-value="value"
      variant="outlined"
      density="compact"
      hide-details
    />
    <div class="text-caption text-medium-emphasis mt-2">
      {{ modeDescription }}
    </div>
  </v-card-text>
</v-card>
```

### Decision 2: Mode options and descriptions

| Mode | Value | Label | Description |
|------|-------|-------|-------------|
| Documentary | `documentary` | 纪录片解说 | AI 观看视频画面后自动生成解说文案，适合素材类视频 |
| SDE | `sde` | 短剧解说 | 基于字幕文件分析剧情，生成带解说和原声的视频剪辑 |
| SDP | `sdp` | 短剧混剪 | 基于字幕分析精彩片段，自动混剪高光时刻（无配音生成） |

### Decision 3: How does mode change affect the Start button?

The Start button calls a different Tauri command based on mode:
- Documentary: `run_documentary` (needs video_path + script_path)
- SDE: `run_sde` (needs video_path + subtitle_path + drama_name)
- SDP: `run_sdp` (needs video_path + subtitle_path + custom_clips)

The frontend must:
1. Validate mode-specific required fields before enabling Start
2. Show mode-specific validation errors (e.g., "SDE mode requires a subtitle file")
3. Pass the correct parameters based on mode

### Decision 4: Single video vs multiple video table

SmartEdit supports multiple videos in a queue. NarratoAI currently supports single-video pipeline calls. The frontend should:
- Still display a video table (same as SmartEdit) for consistency
- For v2.0, each row shows one video's status
- Mode switch determines how the pipeline interprets video list (single vs sequential)

### Decision 5: Mode persistence

The selected mode persists across app restarts via backend config (`AppConfig.mode` or similar). Frontend loads mode on mount, saves on change.

## SmartEdit Components Reuse Analysis

| SmartEdit Component | Reuse in NarratoAI | Changes Needed |
|--------------------|--------------------|----------------|
| `AppHeader.vue` | Direct reuse | Change title "SmartEdit" → "NarratoAI", replace logo asset |
| `HomeView.vue` | Pattern reuse | Add mode-awareness: mode switch affects Settings tab content, validation logic, pipeline dispatch |
| `SettingsDrawer.vue` | Pattern reuse | Add mode selector section. Remove FfmpegPanel. Add conditional field visibility. |
| `VideoTable.vue` | Direct reuse | Status labels may need mode prefix. Otherwise identical. |
| `LogPanel.vue` | Direct reuse | Identical — mode-independent |
| `ScriptPreviewDialog.vue` | Pattern reuse | Content rendering logic must branch on mode. Reuse shell (header, loading/error/empty states, export button). |
| `TtsPanel.vue` | Direct reuse | Identical — but entire tab must be hidden in SDP mode |
| `BgmPanel.vue` | Direct reuse | Identical — mode-independent |
| `SystemMonitorBar.vue` | Direct reuse | Identical — mode-independent |
| `WindowControls.vue` | Direct reuse | Identical — Tauri 2.0 |
| `SettingSection.vue` | Direct reuse | Identical — collapsible section component |
| `ModelConfigForm.vue` | Direct reuse | Identical — vision/text model config form |
| `FfmpegPanel.vue` | REMOVED | Per spec: no FFmpeg management panel |

## Competitor Feature Analysis

| Feature | SmartEdit | Other AI video tools | NarratoAI v2.0 |
|---------|-----------|---------------------|----------------|
| Desktop native UI | Yes (Tauri) | Web-only or Electron | Tauri 2.0 (lighter than Electron) |
| Mode selection | 1 mode (SDE) | Usually 1 mode | 3 modes (Unique differentiator) |
| LLM dual model | Vision + Text | Usually text-only | Vision + Text (Documentary mode) |
| TTS engines | Edge + Qwen3 | Proprietary/1-2 | 7 engines (Rust routing) |
| OST type control | Yes | No | Yes (color-coded in preview) |
| FFmpeg bundled | No (system) | Usually bundled | Sidecar bundled |
| Script preview | Yes (SDE) | Varies | Mode-specific preview |
| JianYing export | No | No | Yes (unique to Chinese market) |
| Dark/light theme | Yes | Varies | Yes |

## Sources

- SmartEdit codebase (`E:\GitLib\SmartEdit\`) — Full Vue 3 + Vuetify 3 reference implementation with verified UX patterns
- NarratoAI Rust backend (`E:\GitLib\NarratoAI\narratoai-core\`) — DocumentaryRequest, SdeRequest, SdpRequest types with all fields
- NarratoAI Tauri commands (`E:\GitLib\NarratoAI\src-tauri\src\commands\pipeline.rs`) — run_documentary, run_sde, run_sdp, generate_documentary_script, generate_sdp_script
- `.planning/PROJECT.md` — Milestone scope: target features, tech debt items, out-of-scope list
- `.planning/research/PITFALLS.md` — 15 domain pitfalls, Tauri error handling patterns
- `.planning/research/ARCHITECTURE.md` — Component boundaries, data flows, module organization
- `.planning/research/STACK.md` — Technology stack (Vue 3, Vite, Vuetify 3, Pinia, vue-router, Tauri 2.0)

---
*Feature research for: NarratoAI v2.0 — Tauri + Vue 3 Frontend*
*Researched: 2026-05-12*
