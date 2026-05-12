# Pitfalls Research

**Domain:** Tauri 2.0 + Vue 3 + Vuetify 3 frontend integration + Rust tech debt cleanup
**Researched:** 2026-05-12
**Confidence:** MEDIUM (web research on Tauri 2.0 patterns partially relies on community sources; code-level analysis of existing codebase is HIGH)

## Critical Pitfalls

### Pitfall 1: Tauri 2.0 Sidecar FFmpeg Dynamic Library Crash on macOS

**What goes wrong:**
The FFmpeg sidecar binary crashes at runtime with `dyld: Library not loaded` errors on macOS, or fails silently on all platforms because the binary cannot be found.

**Why it happens:**
Three compounding issues: (1) Pre-built FFmpeg binaries are dynamically linked and reference `.dylib` files at relative paths that break inside the Tauri app bundle. (2) Tauri 2.0 requires sidecar binaries to have a specific filename pattern (`bin/ffmpeg-{target-triple}{.exe}`) in a specific directory (`src-tauri/bin/`), and developers often get the naming or location wrong. (3) The `externalBin` config in `tauri.conf.json` (`bundle.externalBin`) is forgotten entirely. (4) Tauri v2's new capability-based permission system (not the old v1 allowlist) must grant `shell:allow-execute` and scope the sidecar -- if these are missing, the command silently fails with no IPC error.

**How to avoid:**
- Use a statically-linked FFmpeg build (from ffmpeg.org or johnvansickle.com for Linux; for macOS use `osxfuse/ffmpeg` static build).
- Place binaries at `src-tauri/bin/ffmpeg-x86_64-pc-windows-msvc.exe`, `src-tauri/bin/ffmpeg-x86_64-apple-darwin`, `src-tauri/bin/ffmpeg-aarch64-apple-darwin`, `src-tauri/bin/ffmpeg-x86_64-unknown-linux-gnu`.
- Add `"bundle": { "externalBin": ["bin/ffmpeg"] }` to `tauri.conf.json`.
- Add capabilities in `src-tauri/capabilities/default.json`: `"shell:allow-execute"`, `"shell:allow-spawn"`, and scope `"shell:allow-execute"` with `"name": "bin/ffmpeg", "sidecar": true, "args": true`.
- Use `@tauri-apps/plugin-shell` on the frontend -- import `Command` and call `Command.sidecar('bin/ffmpeg', args)`, never `Command.new('ffmpeg')`.
- Always check `output.code` and `output.stderr` instead of assuming empty stdout = success.
- On macOS, create a universal binary via `lipo -create` for Intel + Apple Silicon, or provide two separate binaries for each target triple.

**Warning signs:**
- In development, FFmpeg works; in the production bundle, it fails silently.
- `tauri::api::process::Command::new_sidecar("ffmpeg")` returns `Err` during development.
- macOS users see `dyld: Library not loaded` in the console.
- Frontend receives empty `{ stdout: '', stderr: '' }` (sandbox blocked the process).

**Phase to address:**
Phase 2 (Frontend Scaffold) should verify the sidecar works end-to-end before building any UI that depends on it. Add a smoke-test Tauri command that runs `ffmpeg -version` via sidecar and returns the version string to the frontend.

---

### Pitfall 2: Tauri State Read Lock Held Across Long Async Pipelines

**What goes wrong:**
The `prompt_manager.read().await` guard in `pipeline.rs` is held for the entire duration of LLM calls and FFmpeg processing (potentially minutes). While the read lock is held, any write access to `PromptManager` or `Registry` is blocked. This means hot-reload of prompts, registry mutations, or any other write operation will deadlock or block until the pipeline finishes.

**Why it happens:**
The `run_sde()` and `generate_documentary_script()` and `generate_sdp_script()` commands acquire `tokio::sync::RwLock` read guards on `PromptManager` and `Registry`, then pass `&registry` and `&pm` references into the core pipeline APIs that require them. Because the references borrow from the guard, the guard cannot be dropped until the entire async pipeline completes.

**How to avoid:**
- Refactor the core pipeline APIs to accept pre-extracted values (e.g., `Arc<dyn LlmProvider>` and pre-rendered prompt strings) instead of `&Registry` / `&PromptManager`. This lets the Tauri command scope the guard to a block that completes before any `.await`.
- For `PromptManager`, pre-render all prompts into a `Vec<String>` before releasing the lock, then pass the pre-rendered strings into the pipeline.
- For `Registry`, extract provider arcs (`Arc<dyn LlmProvider>`) in a scoped block, drop the guard, then pass the arcs into the pipeline.
- Measure the current lock contention. The longest hold time is `generate_sdp_script` (two sequential LLM calls with interleaved prompt rendering). Even if the lock is held for "only seconds," it blocks prompt registry writes.

**Warning signs:**
- Any operation that tries to update prompts (e.g., a "reload prompts" admin command) while a pipeline is running will block until the pipeline completes.
- If someone later adds a write operation to `PromptManager` (e.g., user-customizable prompt editing), it will deadlock with a running pipeline.
- The existing `// TODO(WR-02)` comments in `pipeline.rs` (lines 149, 275, 340) are the warning signs.

**Phase to address:**
Phase 4 (Pipeline Integration) or preferably sooner in Phase 1 (Tech Debt: State Lock). This is a blocking architecture issue that affects all three pipeline modes.

---

### Pitfall 3: 3-Mode Dynamic Panel Switching Causes State Fragmentation

**What goes wrong:**
The three modes (Documentary, SDE, SDP) share some UI panels (VideoTable, LogPanel) but have different settings panels, different pipeline commands, and different configuration parameters. Naively implementing mode switching with `v-if`/`v-show` creates stale state, lost form data, and broken event listeners.

**Why it happens:**
SmartEdit has a single mode (short drama narration). NarratoAI has three modes with overlapping but distinct configurations. When a user switches modes:
- The previous mode's Pinia store state may be stale when they switch back.
- Tauri event listeners (`listen("pipeline-progress", ...)`) registered for one mode persist across modes and may fire for the wrong pipeline type.
- Form fields for mode-specific settings (e.g., SDE subtitle parsing parameters vs Documentary frame interval) get destroyed and rebuilt, losing user input.

**How to avoid:**
- Use a single `modeStore` Pinia store that tracks `currentMode: 'documentary' | 'sde' | 'sdp'` and dispatches all pipeline-related actions through a centralized `runPipeline()` action that reads the mode and calls the correct Tauri command.
- For event listeners: use `watch(() => modeStore.currentMode, ...)` to unregister/re-register `pipeline-progress` listeners when mode changes. Maintain separate progress state per mode.
- For form state: persist mode-specific settings in the store keyed by mode (e.g., `configs.documentary`, `configs.sde`, `configs.sdp`). Use `<component :is="dynamicPanel" :key="currentMode">` to force fresh component instances per mode switch.
- Do NOT let individual mode panels call `invoke()` directly. Route all Tauri IPC through store actions that include the mode context. This prevents the wrong command being called for the current mode.

**Warning signs:**
- Switching from SDE to Documentary and back loses the SDE subtitle path the user selected.
- Progress events from a completed SDP pipeline appear in the Documentary UI.
- User fills out Documentary form, switches to SDE to check something, switches back, form is empty.

**Phase to address:**
Phase 2 (HomeView + 3-Mode Layout) must define the store architecture and event lifecycle before any mode-specific components are built.

---

### Pitfall 4: Tauri invoke() Serialization Breaking BigInt / Large u64 Values

**What goes wrong:**
Rust `u64` values larger than `Number.MAX_SAFE_INTEGER` (2^53 - 1) lose precision when serialized through Tauri IPC to JavaScript. The frontend receives a corrupted number. Similarly, Rust's `i64` for timestamps (100ns units in Edge-TTS word boundaries) can exceed safe integer range.

**Why it happens:**
Tauri IPC serializes Rust types to JSON. JavaScript numbers are IEEE 754 doubles with 53-bit integer precision. Tauri 2.0 does not automatically serialize big integers as strings. The SmartEdit reference codebase already has `BigInt` handling in `useTauri.ts` (`normalizeTauriArgs`) but this handles JS->Rust direction only, not Rust->JS direction.

**How to avoid:**
- Audit all Tauri command return types for `u64` fields that can exceed 2^53. Key offenders: `EdgeTtsEngine` word boundary offsets (100ns units, up to `u64::MAX`), TTS duration (100ns units), file sizes.
- Use `#[serde(serialize_with = "as_string")]` on these fields, or use `String` instead of `u64` for values that JS receivers must handle precisely.
- On the frontend: create a `reviver` function for `JSON.parse` on Tauri responses that converts known string-encoded numbers back to `BigInt` for display, or simply display them as strings.
- The SmartEdit `tauriInvoke` wrapper handles `bigint` args for JS->Rust but does NOT handle Rust->JS big integers. This needs to be addressed independently.

**Warning signs:**
- Word boundary timestamps in SRT generation are off by ~1ms at large offsets.
- TTS duration displays as a rounded number for long audio files.
- The value `9007199254740993` (2^53+1) serialized as JSON becomes `9007199254740992`.

**Phase to address:**
Phase 2 (Tauri event/command layer) -- verify all u64/i64 fields in Tauri return types and apply string serialization where needed.

---

### Pitfall 5: ConfigManager Hot-Reload Race Condition During Pipeline

**What goes wrong:**
The `ConfigWatcher` uses `notify` to detect file changes. When a user saves `config.toml` while a pipeline is running, the watcher acquires `config.write()` and replaces the config. Meanwhile, the pipeline holds a cloned `AppConfig` snapshot from `config.get()` at startup. The replaced config may be half-written (atomic write tools use temp file + rename, which triggers Create + Modify events). If the watcher reads during the rename, it may parse an empty or partial file.

**Why it happens:**
The existing `ConfigWatcher` uses a two-step approach: parse in the notify callback thread, then acquire write lock and replace. This is partially correct (parsing outside the lock prevents holding the lock during I/O). But the watcher subscribes to `EventKind::Modify(_)` and `EventKind::Create(_)` -- atomic editors (vim, VS Code) use write-to-temp + rename strategies that generate multiple events. The watcher may fire before the rename completes.

**How to avoid:**
- Add a debounce mechanism: coalesce multiple file change events within 200ms into a single reload. The `notify` crate's `Config::default()` already does some debouncing, but verify this.
- For the read lock: the current `ConfigManager.get()` clones the entire config under the read lock. For a pipeline that runs for 10+ minutes, this means a 10+ minute read lock hold time. Prefer snapshot-at-start: let the pipeline capture an `Arc<AppConfig>` at init time, and only release when the pipeline completes.
- Consider an explicit "reload on next pipeline start" approach instead of live hot-reload during a running pipeline. If the user modifies config mid-pipeline, queue the reload for when the pipeline finishes.
- The watcher should use `notify::EventKind::Modify(notify::event::ModifyKind::Data(_))` specifically, not catch-all `Modify(_)` which includes metadata changes.

**Warning signs:**
- User edits config.toml during a pipeline run; the next pipeline step starts failing with config validation errors.
- Config values "reset" to default temporarily during save.
- The test `test_hot_reload_invalid_toml_preserves_config` exists but doesn't test mid-save race.

**Phase to address:**
Phase 6 (Tech Debt: ConfigManager hot-reload). Must be paired with Pitfall 2 (state lock) fixing because both touch the same `Arc<RwLock<AppConfig>>`.

---

### Pitfall 6: Integration Test Stubs Mask Broken Pipelines

**What goes wrong:**
Integration tests that use `assert!(true)` pass unconditionally, giving a false sense of coverage. When a real integration or e2e test later fails, the team must debug issues that a proper unit/integration test would have caught immediately.

**Why it happens:**
The v1.0 milestone audit identified `assert!(true)` stubs in integration tests (`.planning/todos/pending/2026-05-12-replace-integration-test-stubs-assert-true.md`). These were deliberately placed to mark locations where real tests were needed but deferred. After several months, these stubs blend in with real tests and nobody knows which are real and which are stubs.

**How to avoid:**
- Find all `assert!(true)` stubs across `tests/` using the existing grep results (11 files contain references to this pattern, including planning documents). Replace each with real assertions.
- For tests requiring external dependencies (FFmpeg, network, actual TTS), use `#[ignore]` with a descriptive reason instead of `assert!(true)`.
- Add a CI lint rule: `grep -rn "assert!(true)" tests/` should return zero matches.
- For the `edge_tts_real_test.rs` integration test: it already has real assertions. But verify it runs correctly on CI (currently likely `#[ignore]`).
- For `pipeline_real_test.rs`: it is a real end-to-end test but depends on a specific local video file (`E:/GitLib/视频/华星大酒店.mp4`). Parameterize or mock the video source so it can run on CI.

**Warning signs:**
- `grep -rn "assert!(true)" tests/` returns any results.
- Code review shows "test coverage is green" when key paths are untested.
- A regression in the pipeline is caught by manual testing only.

**Phase to address:**
Phase 1 (Tech Debt: Integration tests). Make this the very first tech debt item fixed, because it blocks confidence in all subsequent changes.

---

### Pitfall 7: YouTube/Pexels Tauri Commands Exposed Without Progress Events

**What goes wrong:**
The `youtube::downloader::download_video()` and `material::downloader::download_videos()` functions are synchronous blocking or use `tokio::process::Command` for subprocesses. If exposed directly as Tauri commands without progress events, the frontend will freeze (for blocking operations) or show no feedback (for subprocess operations that take 30+ seconds).

**Why it happens:**
The existing YouTube downloader already uses `tokio::process::Command` (async), so it won't block the event loop. But it has no progress callback -- the frontend receives `Result<DownloadResult, YoutubeError>` only after the entire download completes. For Pexels/Pixabay downloads, `save_video()` uses blocking `reqwest::blocking` calls and `std::io::copy` inside a `pub fn` (not async). If wrapped in `tokio::task::spawn_blocking`, it still has no progress reporting.

**How to avoid:**
- For YouTube downloads: wrap `yt-dlp` with `--progress` flag and parse stderr output for progress percentage. Emit Tauri events from a background task that reads the stderr stream line by line.
- For Pexels downloads: convert `save_video()` to async using `reqwest::Client` (not blocking), and periodically emit progress based on bytes read vs `Content-Length` header.
- On the frontend: create a `useDownloadProgress` composable that listens to download progress events and shows a progress bar.
- Add cancellation support: pass a `CancellationToken` (already used elsewhere in the codebase, see `analyzer.rs`) through the download chain so the user can cancel long downloads.

**Warning signs:**
- User clicks "Download from YouTube" and the UI is unresponsive for 30+ seconds.
- User clicks "Search Pexels" and the UI freezes while downloading multiple videos.
- No visual feedback during download -- user may think the app crashed.

**Phase to address:**
Phase 7 (Tech Debt: YouTube/Pexels command exposure). The Tauri commands should be thin wrappers that spawn background tasks and emit progress events.

---

### Pitfall 8: Edge-TTS Proxy Tunnel Network Edge Cases Unhandled

**What goes wrong:**
The existing `EdgeTtsEngine` proxy implementation (in `edge_tts.rs`, lines 244-440) is sophisticated with HTTP CONNECT tunneling, TLS upgrade, IPv6 support, and credential injection. However, it has several unhandled edge cases: proxy DNS resolution failures, proxy authentication challenges beyond Basic (Digest, NTLM), proxy redirects (305 responses), and proxy timeouts during the CONNECT handshake.

**Why it happens:**
The implementation does a raw TCP connect to the proxy, sends a CONNECT request manually, and parses the response with a fixed-size buffer (4096 bytes). If the proxy response header exceeds 4096 bytes (unlikely but possible with many headers), the parsing fails. The loop for handling 1xx interim responses is present but untested. SOCKS5 is explicitly unsupported, which may frustrate users who use SOCKS proxies.

**How to avoid:**
- Increase the response buffer to at least 8192 bytes and add a `response_buf.resize()` approach if the buffer is exhausted.
- Replace the manual TCP CONNECT with a dedicated proxy library (e.g., `tokio-tungstenite` with proxy support, or the `hyper` crate's `ProxyConnector`). The manual approach, while correct for the common case, misses edge cases that battle-tested libraries handle.
- Add a test mode that simulates various proxy responses: immediate CONNECT success, CONNECT requiring auth, CONNECT with 1xx interim, CONNECT timeout, proxy DNS failure.
- For SOCKS5: document the limitation clearly in the UI (show an error message when SOCKS5 is detected in the proxy URL) rather than the current `return Err(...)` which produces a raw error message.
- Add a configurable connect timeout for the proxy CONNECT stage (currently hardcoded to `CONNECT_TIMEOUT = 30s`, which is appropriate but should be configurable).

**Warning signs:**
- User reports "Edge-TTS works without proxy but fails with proxy."
- Proxy error messages contain raw connection error text rather than user-friendly guidance.
- Large proxy response headers cause "代理响应头过大" error.

**Phase to address:**
Phase 6 (Tech Debt: Edge-TTS proxy tunnel). The proxy code exists but needs hardening. Allocate time for edge case testing.

---

### Pitfall 9: Specta/ts-rs Type Generation Creates Brittle Type Mismatches

**What goes wrong:**
TypeScript types generated from Rust structs via ts-rs or specta become out of sync with the actual Rust types. When a Rust struct field is renamed or removed, the frontend TypeScript still expects the old field. The mismatch surfaces as runtime crashes (undefined properties) rather than compile-time errors if type generation is not integrated into the build pipeline.

**Why it happens:**
ts-rs exports `.ts` files on `cargo test`. If developers forget to run `cargo test` after changing a struct, or if CI does not verify that the checked-in `.ts` files are up-to-date, the types drift. The SmartEdit reference project uses `@/types/generated/LlmConfig` etc. -- these are likely generated and committed. If the generation step is manual, drift is inevitable.

**How to avoid:**
- Integrate type generation into the build pipeline: `cargo build` should trigger `cargo test --test export_types` (or whatever generates `.ts` files) and fail the build if the files have changed but aren't committed.
- Add a CI step: `git diff --exit-code` on the generated `.ts` files after running the type export. If there are changes, the CI fails with a message like "TypeScript types are out of sync with Rust structs. Run `cargo test` and commit the generated files."
- Consider `specta` + `tauri-specta` which generates the types as part of the Tauri command registration and can emit a single `.ts` file automatically. `specta` is the more modern choice for Tauri 2.0 and has better community support than standalone `ts-rs`.
- Store generated types in `src/types/generated/` with a clear header comment: `// AUTO-GENERATED BY ts-rs. DO NOT EDIT.`
- For the specific NarratoAI case: the Tauri commands already accept and return complex types (`DocumentaryRequest`, `SdeRequest`, `SdpRequest`, `Script`). These must all have TypeScript counterparts.

**Warning signs:**
- Frontend builds succeed but runtime crash with `Cannot read properties of undefined (reading 'fieldName')`.
- A PR changes a Rust struct but does not update any `.ts` files.
- `cargo test` produces different output than what's committed.

**Phase to address:**
Phase 1 (Frontend Scaffold) should set up the type generation pipeline. Verify it works with a single struct before generating all types.

---

### Pitfall 10: System Monitor Tauri Command Leaks Background Threads

**What goes wrong:**
The system monitor (CPU/RAM) requires a background thread that periodically polls system resources and emits Tauri events to the frontend. If the `start_system_monitor` command starts a background task but `stop_system_monitor` does not reliably shut it down (e.g., due to app crashes, rapid component mount/unmount), background threads accumulate, wasting resources and causing duplicate events.

**Why it happens:**
The SmartEdit reference pattern (`SystemMonitorBar.vue`) calls `start_system_monitor` in `onMounted` and `stop_system_monitor` in `onUnmounted`. In Tauri, if the frontend crashes or the component unmounts unexpectedly (e.g., route change), `onUnmounted` may not fire reliably. The Rust side has no way to detect that the frontend has disconnected.

**How to avoid:**
- Use a single background task with a `CancellationToken` that the frontend holds and passes to `start_system_monitor`. When the frontend calls `stop_system_monitor(monitor_id)`, the Rust side cancels the specific token.
- Alternatively, use a `Drop`-based RAII approach in Rust: the Tauri command returns a `MonitorHandle` struct that, when dropped, cancels the background task. If the frontend never calls `stop`, the handle is dropped when the IPC connection drops (Tauri 2.0 supports this pattern with `tauri::ipc::Response`).
- On the frontend: hook `window.addEventListener('beforeunload', ...)` to call `stop_system_monitor` as a safety net.
- Use `tauri::async_runtime::spawn` and store the `JoinHandle` in a `HashMap<String, JoinHandle<()>>` managed state, keyed by a unique client ID.

**Warning signs:**
- Task Manager shows multiple NarratoAI processes or threads that don't go away after closing settings.
- `system-resources` events arrive at double frequency (two monitors running).
- The app's CPU usage stays at 2-5% even when idle (the monitor itself consumes resources).

**Phase to address:**
Phase 3 (System Monitor + Theme). The system monitor command must be designed for clean lifecycle management from day one.

---

### Pitfall 11: Vuetify 3 Theme Customization Breaks After Vuetify Upgrade

**What goes wrong:**
The custom `app-bar` color defined in the SmartEdit Vuetify theme (`'app-bar': '#f8fafc'` for light, `'app-bar': '#1e293b'` for dark) may stop working after a Vuetify 3.x minor upgrade because Vuetify 3.5+ changed CSS variable naming conventions. The theme toggle between light/dark may appear to work (switch moves) but the actual colors don't change because the CSS variables are not being applied.

**Why it happens:**
Vuetify 3 evolved its theme system across versions. The `theme: { themes: { light: { colors: { ... } } } }` API remains stable, but custom color names not in Vuetify's built-in palette (like `app-bar`) depend on CSS variable injection that changed between Vuetify 3.4 and 3.6. Additionally, the SmartEdit `useTheme` composable sets `vuetifyTheme.global.name.value = appStore.theme` which is the correct approach, but custom CSS classes that use `rgb(var(--v-theme-app-bar))` will not update if the component uses scoped styles without the CSS variable cascade.

**How to avoid:**
- Pin the Vuetify version in `package.json` to the exact minor version used during development (e.g., `"vuetify": "3.8.1"` as seen in the McSTools production example).
- Use only Vuetify built-in color names (`primary`, `secondary`, `surface`, `background`, `on-surface`, `on-background`) for theme colors. Avoid custom color names like `app-bar`. Instead, use `surface-variant` or a new `app-bar` color that is also defined in Vuetify's `$utilities`.
- If custom colors are necessary, define them using the `variables` API or add them to Vuetify's `$color-pack`.
- Test theme switching explicitly: create a test that toggles dark/light mode and verifies that specific DOM elements change computed style.
- Use the `useTheme()` composable from the SmartEdit reference as-is -- it's well-tested. But do NOT reimplement theme persistence with localStorage directly; use the existing `appStore` pattern.

**Warning signs:**
- Theme toggle works but `app-bar` stays the same color in both modes.
- CSS custom properties (`--v-theme-*`) are undefined for custom color names.
- After `npm update`, the app looks different.

**Phase to address:**
Phase 3 (System Monitor + Theme). Lock the Vuetify version and verify theme switching in a test.

---

### Pitfall 12: Router Guard Race with Tauri Init on App Startup

**What goes wrong:**
The Vue 3 app mounted inside Tauri's webview calls `useUserStore()` or `invoke('get_initial_data')` in a router guard (`beforeEach`). The guard fires before Tauri's IPC bridge is fully initialized. The `invoke()` call silently fails (returns a generic "not allowed" error or times out), the router guard rejects, and the app shows a blank white screen with no error feedback.

**Why it happens:**
Tauri 2.0 initializes the IPC bridge asynchronously when the webview loads. If the Vue app's `createRouter()` and initial navigation happen before `window.__TAURI__` is available, `invoke` calls will fail. The SmartEdit `useTauri.ts` wraps `invoke` but does not guard against this race.

**How to avoid:**
- Use `app.onMounted()` to initialize the app state (call the initial data commands), not in top-level router guards.
- Add an app-level loading screen that renders while the IPC bridge initializes. Only show the real app when `invoke('platform_check')` succeeds.
- Create a `useTauriReady` composable:
  ```ts
  export function useTauriReady() {
    const isReady = ref(false);
    const error = ref<string | null>(null);
    onMounted(async () => {
      try {
        await invoke('get_version');
        isReady.value = true;
      } catch (e) {
        error.value = `Tauri init failed: ${e}`;
      }
    });
    return { isReady, error };
  }
  ```
- Use `v-if="isReady"` on the root `<router-view>` to prevent any navigation until Tauri is ready.

**Warning signs:**
- App shows white screen on startup; works after refreshing.
- Console shows `Uncaught (in promise) Error: "invoke" is not allowed` or `__TAURI__ is not defined`.
- The DevTools `window.__TAURI__` is `undefined` for the first few seconds.

**Phase to address:**
Phase 2 (HomeView + 3-Mode Layout). Add the Tauri readiness guard before the HomeView layout is implemented.

---

### Pitfall 13: Frontend "Script Preview Dialog" Sharing Breaks for 3 Modes

**What goes wrong:**
The SmartEdit `ScriptPreviewDialog` component displays generated scripts in a unified format. In NarratoAI's 3-mode context, Documentary scripts, SDE scripts, and SDP scripts have different structures (different fields, different OST semantics, different timestamps). A single preview dialog that works for all three modes either shows too much information for one, or too little for another.

**Why it happens:**
Documentary scripts have frame-level observations with `scene_description`, `objects`, `actions`. SDE scripts have plot analysis, narrative structure, character analysis. SDP scripts have clip lists with subtitle references. The `ScriptPreviewDialog` in SmartEdit only handles the SDE-like format. Reusing it directly will lose information or show empty sections for the other two modes.

**How to avoid:**
- Create a `ScriptPreviewDialog` that accepts a `mode` prop and renders mode-specific sections using `<component :is>`.
- Define three TypeScript interfaces: `DocumentaryScript`, `SdeScript`, `SdpScript`. The dialog can use a union type `ScriptData = DocumentaryScript | SdeScript | SdpScript` and display accordingly.
- Alternatively, use separate dialog components per mode (`DocumentaryScriptPreviewDialog`, `SdeScriptPreviewDialog`, `SdpScriptPreviewDialog`) and switch via dynamic component. This is cleaner when the three dialogues share very little UI.
- Verify the Rust `Script` type (`Vec<ScriptClip>`) is what all three modes return, but each mode fills different fields. The frontend preview must handle `ost` field, `narration`, `timestamp`, and mode-specific metadata.

**Warning signs:**
- Script preview shows empty "objects" sections for SDP scripts.
- Narration text is displayed but OST type selection is missing.
- Timeline stamps are formatted differently per mode and the dialog doesn't handle all three formats.

**Phase to address:**
Phase 5 (Script Preview + Export). Build the script preview dialog with mode-awareness from the start. Don't build it for one mode and retrofit.

---

### Pitfall 14: Vuetify 3 v-data-table VideoTable Performance with Large Video Lists

**What goes wrong:**
The VideoTable component uses `v-data-table` to list videos. If the user uploads 500+ videos, the table renders all rows in the DOM (no virtualization in Vuetify 3's default data table), causing significant lag, slow scrolling, and high memory usage.

**Why it happens:**
Vuetify 3's `v-data-table` does NOT virtualize rows by default. Unlike `v-data-table-virtual`, the standard data table renders all items into the DOM. The SmartEdit reference likely handles smaller lists, but NarratoAI users may batch-process many videos.

**How to avoid:**
- Use `v-data-table-virtual` (Vuetify 3.3+) which provides row virtualization.
- Implement server-side pagination: load only the current page of videos via Tauri command. The existing `narratoai_core` likely has list/query functions.
- Set a reasonable default page size (e.g., 25 or 50 rows per page).
- Use `shallowRef` for the video list to avoid deep reactivity on large arrays.
- Consider debouncing the "upload file" flow to prevent rapid successive uploads from triggering multiple data refreshes.

**Warning signs:**
- App feels sluggish when 100+ videos are loaded.
- DevTools performance tab shows high DOM node count.
- Scrolling the video table triggers jank.

**Phase to address:**
Phase 3 (VideoTable + operation buttons). Use `v-data-table-virtual` from day one; do not start with `v-data-table` and plan to migrate later.

---

### Pitfall 15: `_max_retries` Parameter Accepted But Unwired in LLM Pipeline

**What goes wrong:**
The Tauri command `generate_sdp_script` and other LLM commands accept parameters that include `temperature`, `custom_clips`, but `llm_max_retries` from `AppConfig` is parsed but not actually used in some pipeline paths. The `openai_compatible.rs` does use it for `ExponentialBackoff`, but the `generate_documentary_script` and script generation functions may not propagate it correctly.

**Why it happens:**
The v1.0 milestone audit identified `_max_retries` parameter accepted but unwired. The config field `llm_max_retries` defaults to 3, but some code paths create `ProviderConfig` without reading this value. The retry mechanism relies on `async-openai`'s `RetryConfig` which has its own default.

**How to avoid:**
- Audit every place `ProviderConfig` is constructed. Current known locations: `register.rs` lines 72 and 98. These correctly pass `config.app.llm_max_retries`. Verify there are no other `ProviderConfig` constructions that use a hardcoded default.
- Add a test that constructs a `ProviderConfig` with `max_retries: 0` and verifies that no retry occurs on a simulated 429 response.
- Verify the `AppConfig.validate()` method (currently a no-op) actually validates that `llm_max_retries` is within a reasonable range (0-10).

**Warning signs:**
- User sets `llm_max_retries = 0` in config but sees retry behavior on API errors.
- User sets `llm_max_retries = 10` and experiences excessive wait times.

**Phase to address:**
Phase 1 (Tech Debt: _max_retries audit). Small fix, high impact on reliability.

---

### Pitfall 16: VisualAnalyzer Facade API Surface Missing Async Cancellation

**What goes wrong:**
The `analyze_video_frames()` function in `visual/analyzer.rs` already accepts a `CancellationToken` parameter. However, if the facade exposes additional convenience methods, they may wrap `analyze_video_frames` without forwarding the cancellation token. The frontend cannot cancel a running visual analysis, forcing the user to wait for completion or kill the app.

**Why it happens:**
The `CancellationToken` is already plumbed through `analyzer.rs` (line 97, `cancel: Option<CancellationToken>`), and the extract/frame/LLM phases check it. But if Tauri commands that invoke visual analysis do not expose a `cancel` parameter, or if the frontend does not have a "Cancel" button that creates a `CancellationToken` and passes it, the cancellation feature exists but is unreachable.

**How to avoid:**
- Audit all Tauri commands that call `analyze_video_frames()` and verify they accept a `cancel` signal from the frontend.
- For the frontend: add a "Cancel Analysis" button that is visible during the script generation step (Documentary mode uses visual analysis). This button should call a Tauri command that cancels the associated token.
- Store the `CancellationToken` in a Tauri state map keyed by `task_id` so it can be cancelled from a separate command.
- Do NOT store the token in a global variable -- use `Arc<tokio_util::sync::CancellationToken>` in a `HashMap<String, CancellationToken>` managed state.

**Warning signs:**
- The "Start" button turns into a "Stop" button during pipeline execution, but clicking "Stop" does not actually stop visual analysis (only stops FFmpeg steps).
- Visual analysis continues running even after the user cancels the pipeline.

**Phase to address:**
Phase 4 (Pipeline Integration). The cancellation plumbing must be part of the pipeline start/stop architecture, not added as an afterthought.

---

### Pitfall 17: Over-fetching In Settings Form (LLM/TTS/BGM Config Loaded on Every Mount)

**What goes wrong:**
Every time the `SettingsDrawer` opens, it fetches LLM config, TTS config, and BGM config from the Rust backend via multiple `invoke()` calls. The drawer may open and close frequently (user toggling to check a setting), causing unnecessary IPC traffic and visible loading spinners.

**Why it happens:**
The SmartEdit reference `SettingsDrawer.vue` loads config in `onMounted` and watches `llmStore.config` for changes. If the store is not populated (e.g., initial state is null), it fetches on every open. There's no caching or stale-while-revalidate pattern.

**How to avoid:**
- Implement a "load once, cache in store" pattern. The `LlmStore` should have a `loaded` flag that prevents re-fetching if already loaded.
- Add a TTL-based refresh: re-fetch only if the cached config is older than N seconds (e.g., 30s).
- Use Pinia's `$state` hydration: persist the fetched config to `localStorage` (with Tauri's `Store` plugin, not raw localStorage which persists across app restarts).
- For NarratoAI specifically: since there are THREE modes with THREE sets of settings, fetch mode-specific settings only when the user switches to that mode (lazy), not all three at once when the drawer opens.

**Warning signs:**
- Opening SettingsDrawer shows 3 loading spinners for 1-2 seconds each time.
- Network tab (or Tauri invoke logs) shows repeated calls to `get_llm_config`, `get_tts_config`, etc.
- Closing and reopening the drawer re-fetches the same data.

**Phase to address:**
Phase 3 (Settings panels). Design the store architecture with caching before building the settings UI.

---

### Pitfall 18: Tauri 2.0 `shell:allow-execute` Capability Blocks Sidecar If Scoped Incorrectly

**What goes wrong:**
The FFmpeg sidecar fails at runtime with a permission error despite being configured in `externalBin`. The error is silently swallowed by the Tauri shell plugin, resulting in an empty `CommandEvent::Terminated` response with no error details.

**Why it happens:**
Tauri 2.0 changed from v1's allowlist system to a capability-based permission model. The `shell` plugin requires:
1. The plugin registered: `.plugin(tauri_plugin_shell::init())`
2. The capability entry in `default.json`: `"shell:allow-execute"` or `"shell:allow-spawn"`
3. A scoped permission entry that specifies the exact sidecar name, with `"sidecar": true` and `"args": true`

If step 2 is done but step 3 is missing, or if the sidecar name in step 3 doesn't match the binary filename, the error is silent.

**How to avoid:**
- Add the following to `src-tauri/capabilities/default.json`:
```json
{
  "windows": ["main"],
  "permissions": [
    "shell:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "bin/ffmpeg",
          "sidecar": true,
          "args": true
        }
      ]
    }
  ]
}
```
- Register the shell plugin in `setup()` (in `lib.rs`): `.plugin(tauri_plugin_shell::init())`
- Note: The sidecar name in the capability scope should be `"bin/ffmpeg"` (relative to `externalBin` path), NOT including the target triple suffix.
- Verify the plugin import is `tauri_plugin_shell` (not the old `tauri::api::process`).

**Warning signs:**
- `Command.sidecar('bin/ffmpeg')` returns `undefined` or fails on the frontend.
- The Tauri command that uses `app.shell().sidecar("ffmpeg")` returns an error at runtime but not at compile time.
- `cargo build` succeeds, but the sidecar doesn't execute in production.

**Phase to address:**
Phase 3 (Sidecar: FFmpeg bundling). This is a setup-time issue -- verify it works before building any processing logic that depends on FFmpeg.

---

### Pitfall 19: Frontend Hash Routing Breaks Tauri Navigation After Build

**What goes wrong:**
When the frontend uses vue-router's `createWebHashHistory()` (as specified in PROJECT.md for Tauri compatibility), route transitions work in development but fail in the production build. The user sees a blank page or the app fails to load entirely.

**Why it happens:**
Tauri 2.0 serves the frontend from `tauri://localhost` or `https://tauri.localhost` in production. Hash-based routing (`window.location.hash`) works with these protocols, but if the `dist` output is configured incorrectly (wrong `frontendDist` path in `tauri.conf.json`), the `index.html` won't load. Additionally, if Vite's build output includes relative paths for assets (e.g., `./assets/...` instead of `/assets/...`), the asset loading fails because the base URL in a Tauri webview is not `./`.

**How to avoid:**
- Set `build.frontendDist` to `"../dist"` (relative to `src-tauri/`) and verify it matches the Vite output directory.
- In `vite.config.ts`, set `base: './'` or use `process.env.TAURI_DEBUG ? '/' : './'` to ensure production assets are loaded relative to the HTML file.
- Use `createWebHashHistory()` as specified in PROJECT.md. Do NOT use `createWebHistory()` or `createMemoryHistory()`.
- Test the production build locally with `pnpm tauri build` before doing any significant frontend work. A broken build that only fails after a week of development is a major setback.

**Warning signs:**
- `pnpm tauri dev` works perfectly; `pnpm tauri build` shows a blank page.
- DevTools console shows `Failed to load resource: net::ERR_FILE_NOT_FOUND` for JS/CSS assets.
- The `index.html` loads but Vue app never mounts (hash router not initialized).

**Phase to address:**
Phase 1 (Frontend Scaffold). Verify the production build works before writing any application code. Just the scaffold + "Hello World" running as a Tauri production build.

---

### Pitfall 20: LLM Vision/Text Dual Config Synchronization During Pipeline

**What goes wrong:**
The backend distinguishes between `vision_llm_provider` and `text_llm_provider`. The frontend SettingsDrawer has separate "Vision Model" and "Text Model" sections. But when a user changes the vision model and saves, the text model config might accidentally be overwritten with defaults (or vice versa), because the save handler builds a single `LlmConfig` and sends it to the backend, potentially losing mode-specific fields.

**Why it happens:**
The SmartEdit `SettingsDrawer.vue` reference has `visionConfig` and `textConfig` as separate refs, then merges them into a single `LlmConfig` for saving (see `buildConfig()` function). The merge logic has fallback chains: `visionBaseUrl || textBaseUrl || null`. If the user configured only a text model but the save handler reads `visionConfig.model` as empty and sets the primary `model` to the text model, everything works. But if the user configures both models, the save handler must preserve both sets of values without one overwriting the other.

**How to avoid:**
- The Rust `AppConfig` (in `types.rs`) already has separate fields for `vision_openai_api_key`, `text_openai_api_key`, etc. The Tauri command that saves config should accept all fields explicitly and persist them independently.
- On the frontend: the save handler must send all fields, not merge and drop. The SmartEdit `buildConfig()` does this correctly for its single-mode case. For NarratoAI's 3-mode case, do NOT add mode-specific field merging in the save handler -- send raw values to the backend and let the backend validate and normalize.
- Add a Tauri command specifically for testing each model independently: `test_vision_connection` and `test_text_connection` (SmartEdit already has `testConnectionForTask`).
- Never auto-fill one model's values from the other model. If the user leaves vision fields empty, that's intentional (they may not need visual analysis for SDP mode).

**Warning signs:**
- User configures Vision model with Gemini, Text model with DeepSeek. After saving and reopening, both show the same provider.
- After saving, the vision API key is empty even though it was filled before saving.
- Test Vision connection succeeds, Test Text connection fails with "wrong provider credentials" using Vision's API key.

**Phase to address:**
Phase 3 (Settings panels). The dual-model config is a core complexity -- get it right before building pipeline integration.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `assert!(true)` in integration tests | Ship v1.0 milestone on time | False sense of coverage; regressions undetected for months | NEVER -- replace immediately in Phase 1 |
| Tauri State read lock held across pipelines | Simple implementation, no refactoring needed | Blocks prompt registry writes during long pipelines; deadlock risk if write operations are added later | Acceptable for MVP only if lock hold time < 5s. Current hold time can be minutes -- NOT acceptable |
| ConfigManager hot-reload without mid-pipeline debounce | Works for the common case (config edit between pipelines) | Race condition during atomic saves; partial config reads | Acceptable as MVP if documented limitation. Must fix in tech debt phase |
| Manual TCP CONNECT for Edge-TTS proxy | Avoids adding a dependency | Proxy edge cases unhandled: Digest auth, large response headers, IPv6 edge cases | Acceptable for MVP if documented. Close with proxy library in tech debt phase |
| Hardcoded local path in integration tests (`E:/GitLib/视频/...`) | Easy to write on developer's machine | Tests only run on one machine; CI cannot verify | NEVER for checked-in tests. Must use env vars or generated test fixtures |
| Sidecar FFmpeg with dynamic linking | Quick to download pre-built binary | macOS dyld crashes; larger bundle size; signing issues | Acceptable only on Windows/Linux with static builds. macOS requires static or bundled dylibs |
| Single ScriptPreviewDialog for all 3 modes | Less code initially | Cramped UI that shows wrong data for 2/3 modes | NEVER -- build mode-aware from the start |
| Over-fetching config on every drawer open | Simple implementation, no caching logic | Noticeable loading delays; extra IPC traffic | Acceptable only if config fetch is <100ms. Otherwise, implement caching |
| Router guard firing before Tauri IPC is ready | Simple auth/init flow | Blank white screen on app startup | NEVER -- always add a Tauri readiness guard |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| FFmpeg sidecar | Using `Command::new("ffmpeg")` (system PATH) | Use `Command.sidecar("bin/ffmpeg")` with `@tauri-apps/plugin-shell` |
| Tauri event listeners | Registering in component `onMounted`, forgetting to unregister | Register in Pinia store or composable; unregister in `onUnmounted` and `watch(currentMode)` |
| Type generation | Running `cargo test` manually, forgetting after struct changes | Integrate into CI build step: `git diff --exit-code` on generated `.ts` files |
| Proxy config | Only testing proxy-less paths, assuming proxy "just works" | Test with HTTP CONNECT, auth-required proxy, SOCKS5 fallback, and no-proxy |
| Config hot-reload | User edits config during pipeline; watcher reads partial file | Debounce file events (200ms); queue reload for after pipeline completion |
| LLM dual-model | One model's config overwrites the other on save | Send all fields independently; never auto-merge in frontend save handler |
| Tauri commands + progress | Returning `Result` only at the end of long operations | Emit progress events via `app.emit()` during execution; return final result |
| vue-router hash routing | Using `createWebHistory()` for clean URLs | Use `createWebHashHistory()` for Tauri compatibility |
| CancellationToken | Not exposing cancel signal to frontend | Store token in managed state map; expose `cancel_task(task_id)` command |
| BigInt serialization | Assuming JS can handle u64 precisely | Use string serialization for u64 fields that exceed 2^53 |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| v-data-table without virtualization | DOM node count high, scrolling jank | Use `v-data-table-virtual` with pagination | 100+ video rows |
| Deep reactive video list | Slow store mutations, watchers fire for unrelated changes | Use `shallowRef` for video array | 200+ videos in store |
| Multiple concurrent IPC calls on startup | Slow initial load, race conditions | Batch initial data fetch into one Tauri command returning a combined struct | Every startup |
| Over-fetching config on each drawer open | Visible loading spinners, extra IPC traffic | Cache config in store with TTL; reload only on stale | Every drawer toggle |
| Heavy logging via Tauri events | UI lag during pipeline execution; log store grows unbounded | Buffer logs in Rust backend; emit batched events; cap log store at N entries | Long pipeline runs (1000+ log entries) |
| Deep watch on Pinia store | Watcher fires for nested changes unnecessarily | Use `storeToRefs` + targeted `watch` on specific fields | Complex settings forms with many fields |
| Sidecar process not killed on pipeline cancel | zombie ffmpeg processes consume CPU | Track child process ID; kill on cancel/drop | User cancels pipeline mid-FFmpeg operation |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Path traversal in YouTube download `rename` parameter | Write files to arbitrary locations | Validate `rename` with path traversal check (already done in `youtube/downloader.rs` line 254 -- verify all paths are covered) |
| API keys in Tauri event payloads | API keys leaked in Tauri event stream | The existing `ProgressPayload` skips sensitive fields, but verify all custom event types follow the same pattern |
| Config file world-readable | API keys stored in plaintext, readable by other processes | On Unix, set `config.toml` permissions to 600. Document this in the user guide. |
| Sidecar binary replaced at runtime | Malicious FFmpeg binary if `src-tauri/bin/` is writable | Tauri's sidecar is bundled into the app -- not user-replaceable. Verify no runtime code writes to `bin/`. |
| Unvalidated IPC input leading to command injection | Arbitrary FFmpeg arguments passed through command args | The FFmpeg wrapper already uses args arrays (not shell strings). Audit all `Command::new("yt-dlp")` calls for args injection. |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No progress events for YouTube downloads | User thinks app is frozen for 30+ seconds | Emit download progress events; show progress bar |
| Mode switch loses unsaved settings | User fills form, switches mode, switches back, form is empty | Persist mode-specific settings per mode; show unsaved changes warning on mode switch |
| LLM connection test shows "success" but pipeline fails | User has to debug why the pipeline fails despite config being valid | Test with the actual pipeline parameters (model, temperature, max_tokens) not just a generic ping |
| No visual feedback during config save | User may close app before config is persisted | Show snackbar on save success/failure; disable save button while saving |
| FFmpeg sidecar failure shows generic "process failed" | User can't tell if FFmpeg is missing, broken, or incompatible | Run `ffmpeg -version` on startup; show detailed error if missing |
| CLI-style parameters in Tauri config (e.g., `threads=4`) | User doesn't know optimal values | Use presets (Speed/Balanced/Quality) instead of raw parameters where possible. SmartEdit already does this for export profiles -- extend to other settings. |
| 3 TTS engine selectors visible at once when user only needs 1 | Overwhelming settings panel | Show only the selected engine's settings; hide irrelevant engine parameters |

## "Looks Done But Isn't" Checklist

- [ ] **FFmpeg sidecar:** Binary bundled but NOT verified in CI (OS-specific, architecture-specific). Verify with a `ffmpeg -version` smoke test in a Tauri command.
- [ ] **Tauri commands:** 15 commands registered but some may lack progress events (YouTube download, visual analysis, Pexels download). Add progress events to any command that takes >3 seconds.
- [ ] **Integration tests:** Test files exist but some use `assert!(true)` which passes unconditionally. Run `grep -rn "assert!(true)" tests/` and replace every match with real assertions.
- [ ] **Config hot-reload:** Watcher implemented but only tested for simple edits. Test the mid-atomic-save race condition. Test the mid-pipeline reload scenario.
- [ ] **Edge-TTS proxy:** Complex implementation with manual CONNECT, IPv6, auth. Test with authenticating proxy, IPv6 proxy, SOCKS5 proxy, no-proxy, and proxy timeout.
- [ ] **Tauri State read lock:** Scoped to short blocks in `generate_documentary_script` but NOT in `run_sde` and `generate_sdp_script`. Verify all three pipeline paths.
- [ ] **VisualAnalyzer facade:** `analyze_video_frames()` exists with CancellationToken support but the Tauri wrapper may not expose cancel. Verify the frontend can cancel visual analysis.
- [ ] **YouTube/Pexels commands:** Download functions exist in core but may not be exposed as Tauri commands. Verify command registration in `register_all_commands!()`.
- [ ] **TypeScript types:** Generated from Rust but CI does not verify they are up-to-date. Add `git diff --exit-code` check on generated `.ts` files in CI.
- [ ] **`_max_retries` parameter:** Accepted but potentially unwired in some LLM code paths. Audit all `ProviderConfig` construction sites.
- [ ] **vue-router hash routing:** Works in dev but may break in production Tauri webview. Verify `pnpm tauri build` produces a working app.
- [ ] **System monitor thread:** `start` implemented but `stop` may not handle rapid mount/unmount. Test by rapidly toggling the settings drawer.
- [ ] **3-mode settings persistence:** Settings saved per-mode but may not survive app restart. Verify with `localStorage` or Tauri Store plugin.
- [ ] **Tauri IPC BigInt:** `normalizeTauriArgs` handles JS->Rust bigint but NOT Rust->JS big integers. Check Tauri command return types for u64 fields > 2^53.
- [ ] **LLM dual-model save:** Save handler merges vision and text config; verify neither model's values overwrite the other on save.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| FFmpeg sidecar not working | MEDIUM (1-2 days) | 1. Verify binary naming and `externalBin` config. 2. Check capabilities scope. 3. Test with static build. 4. Verify on macOS with universal binary. |
| Tauri State read lock deadlock | HIGH (3-5 days) | 1. Refactor core API to accept pre-extracted values. 2. Scope locks to blocks before `.await`. 3. Test with concurrent pipeline + prompt edit. |
| Mode switch state fragmentation | MEDIUM (2-3 days) | 1. Create `modeStore` with mode dispatch. 2. Move all invoke() calls to store actions. 3. Re-register event listeners on mode change. |
| Integration test stubs | LOW (1 day) | 1. Grep all `assert!(true)`. 2. Replace with real assertions or `#[ignore]`. 3. Add CI lint rule. |
| Config hot-reload race | MEDIUM (2-3 days) | 1. Add debounce. 2. Queue reloads during pipeline. 3. Test with atomic save editors. |
| Edge-TTS proxy edge cases | MEDIUM (1-2 days) | 1. Increase buffer size. 2. Add Digest auth support (or document limitation). 3. Add comprehensive proxy tests. |
| BigInt precision loss | MEDIUM (1 day) | 1. Audit return types for u64 > 2^53. 2. Add string serialization. 3. Update frontend parsing. |
| Build broken after hash routing | LOW (few hours) | 1. Verify `base: './'` in vite.config. 2. Verify `frontendDist` path. 3. Rebuild and test. |
| System monitor thread leak | MEDIUM (1-2 days) | 1. Implement CancellationToken pattern. 2. Add `beforeunload` safety net. 3. Test rapid mount/unmount. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| FFmpeg sidecar crash (P1) | Phase 3 | `ffmpeg -version` smoke test in Tauri command; verify in CI |
| State read lock (P2) | Phase 4 AND Phase 1 (tech debt) | Concurrent pipeline + prompt write test; measure lock hold time |
| 3-mode state fragmentation (P3) | Phase 2 | Mode switch test: fill form, switch, switch back, form preserved |
| BigInt serialization (P4) | Phase 2 | Test returns a known large u64; verify frontend receives exact value |
| Config hot-reload race (P5) | Phase 6 | Write config with atomic save during pipeline; verify config stable |
| Integration test stubs (P6) | Phase 1 | CI grep for `assert!(true)` returns zero matches |
| YouTube/Pexels progress (P7) | Phase 7 | Download progress test: verify frontend receives >1 progress event |
| Edge-TTS proxy edge cases (P8) | Phase 6 | Test with authenticating proxy; test with SOCKS5 fallback error message |
| Type type drift (P9) | Phase 1 | CI `git diff --exit-code` on generated `.ts` files |
| System monitor leak (P10) | Phase 3 | Rapid mount/unmount test: verify only one background task active |
| Vuetify theme breakage (P11) | Phase 3 | Theme toggle test: verify specific DOM element styles change |
| Router guard race (P12) | Phase 2 | App startup test: verify Tauri IPC ready before router navigation |
| Script preview per mode (P13) | Phase 5 | Test preview with Documentary, SDE, SDP script data |
| v-data-table performance (P14) | Phase 3 | Load 500 test videos; verify scroll performance >30fps |
| _max_retries unwired (P15) | Phase 1 | Audit all ProviderConfig constructions; test with max_retries=0 |
| VisualAnalyzer cancel (P16) | Phase 4 | Start analysis, cancel, verify analysis stops within 5s |
| Over-fetching settings (P17) | Phase 3 | Open SettingsDrawer; verify only one IPC call per config section |
| Sidecar capability scope (P18) | Phase 3 | Production build test: verify sidecar executes in bundled app |
| Hash routing build break (P19) | Phase 1 | `pnpm tauri build` produces working production binary |
| LLM dual-model save (P20) | Phase 3 | Save different vision/text models; verify both persist independently |

## Sources

- **Tauri 2.0 sidecar documentation:** https://tauri.app/develop/sidecar/
- **Tauri 2.0 shell plugin (v2 docs):** https://v2.tauri.org.cn/reference/javascript/shell/
- **Tauri GitHub issue #11949 (state panics):** https://github.com/tauri-apps/tauri/issues/11949
- **Tauri sidecar dynamic library issue (GitHub #9029):** https://github.com/orgs/tauri-apps/discussions/9029
- **Tauri v2 sandbox permissions article:** https://dev.to/hiyoyok/the-tauri-sandbox-permissions-that-blocked-me-for-two-days-1n0a
- **Tauri sidecar universal binary guide:** https://dev.to/hiyoyok/building-a-universal-binary-intel-apple-silicon-with-tauri-its-easier-than-you-think-22ca
- **Tauri sidecar filename dots bug (GitHub #2310):** https://github.com/tauri-apps/plugins-workspace/issues/2310
- **Specta + tauri-specta v2 docs:** https://specta.dev/docs/tauri-specta/v2
- **ts-rs crate documentation:** https://docs.rs/ts-rs/latest/ts_rs/
- **tauri-ts-generator crate:** https://crates.io/crates/tauri-ts-generator
- **Vuetify 3 CSS issue (GitHub #14738):** https://github.com/vuetifyjs/vuetify/issues/14738
- **Tauri + Vuetify 3 getting started guide:** https://www.bcow.me/posts/tauri-and-vuetify-helloworld/
- **SmartEdit reference codebase:** E:/GitLib/SmartEdit (local reference, not public)
- **NarratoAI v1.0 codebase:** E:/GitLib/NarratoAI (all 23,532 LOC Rust analyzed)
- **NarratoAI v1.0 milestone audit:** .planning/milestones/v1.0-MILESTONE-AUDIT.md
- **NarratoAI integration test stubs TODO:** .planning/todos/pending/2026-05-12-replace-integration-test-stubs-assert-true.md

---
*Pitfalls research for: NarratoAI v2.0 - Tauri + Vue 3 frontend + tech debt cleanup*
*Researched: 2026-05-12*
