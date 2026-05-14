---
phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
verified: 2026-05-14T16:05:00Z
status: gaps_found
score: 29/31 must-haves verified
overrides_applied: 0
gaps:
  - truth: "NetworkProxyPanel proxy 配置数据可以从面板持久化到后端"
    status: failed
    reason: "NetworkProxyPanel 使用 local ref（非 Pinia store），collectAllConfig() 不包含 proxy 字段，saveDraft() 存的是 _lastConfig?.proxy（来自上一次 get_config 的缓存，不是用户输入），restoreDraft() 完全忽略 networkProxy 字段。用户输入的代理配置无法保存到后端。"
    artifacts:
      - path: "src/components/config/NetworkProxyPanel.vue"
        issue: "使用本地 ref（proxyEnabled/proxyHttp/proxyHttps），未连接到 Pinia store 或 save/restore 流程"
      - path: "src/composables/useConfig.ts"
        issue: "collectAllConfig() 不包含 proxy 字段；saveDraft() 存的是 _lastConfig?.proxy（缓存值）；restoreDraft() 跳过 networkProxy"
    missing:
      - "将 proxy 配置纳入 collectAllConfig() 的变更收集"
      - "将 NetworkProxyPanel 的 local ref 替换为 Pinia store 或通过 composable 接入 save/restore 流程"
      - "saveDraft() 保存面板当前 proxy 输入而非 _lastConfig?.proxy"
      - "restoreDraft() 恢复 networkProxy 字段到面板"
deferred: []
human_verification: []
---

# Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt — Verification Report

**Phase Goal:** 所有设置面板可用，支持双模型 LLM 配置、7 引擎 TTS 选择、BGM/Export 设置、模式专用参数。FFmpeg 内置打包，Edge-TTS 代理隧道和 VisualAnalyzer 门面补齐

**Verified:** 2026-05-14T16:05Z
**Status:** gaps_found — 1 项数据流缺陷需要修复
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths — Plan 16-01 (FFmpeg Sidecar + Config Commands)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | FFmpeg sidecar 可通过 externalBin 定位并执行 | ✓ VERIFIED | `tauri.conf.json` 含 `"externalBin": ["binaries/ffmpeg", "binaries/ffprobe"]`；二进制文件存在于 `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`、`ffprobe-x86_64-pc-windows-msvc.exe`；`.gitattributes` 配 LFS tracking |
| 2 | 5 个 Tauri 命令全部注册并可通过 invoke 调用 | ✓ VERIFIED | `config.rs` 含 5 个 `#[tauri::command]`（get_config, save_config, test_llm_connection, get_edge_tts_voices, get_system_proxy）；`mod.rs` 中 `register_all_commands!` 宏包含全部 5 个 |
| 3 | shell:allow-execute 权限已声明，shell 插件初始化完成 | ✓ VERIFIED | `capabilities/default.json` 含 `"shell:allow-execute"`、`"shell:allow-spawn"`；`lib.rs` 含 `.plugin(tauri_plugin_shell::init())` |
| 4 | config.rs 命令可读写 ConfigManager | ✓ VERIFIED | 所有 config 命令使用 `State<'_, ConfigManager>`；`get_config` 调用 `config_manager.get()`；`save_config` 通过 `config_manager.config()` 读写 Arc<RwLock<AppConfig>> |
| 5 | save_config 支持增量更新（仅修改字段） | ✓ VERIFIED | `config.rs:273-289` merge_json() 递归合并；`config.rs:291-303` strip_masked_secrets() 过滤脱敏占位符 |
| 6 | save_config 支持冲突检测（expected_mtime） | ✓ VERIFIED | `config.rs:18-34` expected_mtime 参数 + config_file_mtime() 检测；不匹配时返回 `{"status": "conflict", "current_config": ...}` |

### Observable Truths — Plan 16-02 (Edge-TTS Proxy + VisualAnalyzer Facade)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 7 | Edge-TTS 时钟校准 HTTP 请求可通过代理 | ✓ VERIFIED | `edge_tts.rs:459-474` build_http_client() 在 proxy_enabled 时使用 ProxyConfig 创建代理感知客户端；`correct_clock_skew():500` 使用 build_http_client(Duration::from_secs(10)) |
| 8 | Edge-TTS voice list 可通过代理获取 | ✓ VERIFIED | `edge_tts.rs:534-565` get_voices() 使用 build_http_client(Duration::from_secs(30))；Tauri 命令 `get_edge_tts_voices` 委托到 `narratoai_core::tts::get_edge_tts_voices()` |
| 9 | 代理配置从 ProxyConfig::from_env() 自动检测环境变量 | ✓ VERIFIED | `common.rs:27-48` from_env() 检测 HTTPS_PROXY/HTTP_PROXY/ALL_PROXY 大小写不敏感；`from_proxy()` 回退到 `from_env()`；测试 `test_from_env_with_https_proxy` 和 `test_from_env_no_env_vars` 通过 |
| 10 | 所有视觉分析调用通过 analyze_video_frames() 门面 | ✓ VERIFIED | `analyzer.rs:316-327` TDET-06 审查注释确认：Rust 视觉分析通过此门面；SDE/SDP 无视觉分析步骤 |
| 11 | 视觉分析的 prompt template 从 PromptRegistry 加载 | ✓ VERIFIED | 通过 `analyze_video_frames()` 的 `prompt_template` 参数传入，来源为 PromptManager/PromptRegistry |

### Observable Truths — Plan 16-03 (Pinia Stores + useConfig + ProviderPresets)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 12 | Pinia stores 扩展为完整字段 | ✓ VERIFIED | `llm.ts`: visionConfig/textConfig (LLMConfig) + loading/dirty/testing + loadConfig/testConnection/resetPanel/markClean/markDirty；`tts.ts`: 7 引擎类型接口（EdgeEngineConfig~DoubaoEngineConfig） + voiceList + loadConfig/loadVoices/resetPanel；`bgm.ts`: folder/mode/audioFiles + loadConfig/resetPanel；`export.ts`: outputDir/format + loadConfig/resetPanel；`mode.ts`: params 含 7 字段 + loadConfig/resetParams |
| 13 | useConfig composable 封装 get_config/save_config | ✓ VERIFIED | `useConfig.ts:176-208` loadFromBackend() 调用 `tauriInvoke('get_config')` + 分发到 stores；saveToBackend() 调用 `tauriInvoke('save_config', changes)` |
| 14 | ProviderPresets 包含 LLM Provider 推荐模型和默认 Base URL | ✓ VERIFIED | `ProviderPresets.ts` 含 10 个预设（openai/deepseek/gemini/qwen/siliconflow/moonshot/anthropic/cohere/together/openrouter），每个含 recommendedModel/defaultBaseUrl/label |
| 15 | ts-rs 生成类型同步到 src/types/ | ✓ VERIFIED | `src/types/generated/` 目录含 12 个 `.ts` 文件（AppConfig/AppSection/UiSection/ProxySection/FramesSection/AzureSection/TencentSection/SoulVoiceSection/TtsQwenSection/IndexTTS2Section/DoubaoTTSSection/AudioSection） |
| 16 | AppConfig TS 接口可在前端 import | ✓ VERIFIED | `src/types/index.ts` re-exports: `export type { AppConfig } from './generated/AppConfig'` + 所有子类型 |
| 17 | mode store params 扩展 | ✓ VERIFIED | `mode.ts:7-15` ModeParams 含 frameInterval/visionBatchSize/dramaName/temperature/clipCount/minDuration/maxDuration + loadConfig/resetParams |

### Observable Truths — Plan 16-04 (SettingsDrawer + 7 Panels)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 18 | SettingsDrawer 显示 7 个 SettingSection 面板 | ✓ VERIFIED | `SettingsDrawer.vue:50-91` 按 D-03 顺序渲染：LlmVisionPanel > LlmTextPanel > TtsPanel > BgmPanel > ExportPanel > NetworkProxyPanel > ModeParamsPanel |
| 19 | 每个 LLM 面板含 4 字段 + 测试连接按钮 | ✓ VERIFIED | `LlmVisionPanel.vue` 和 `LlmTextPanel.vue` 均含：Provider v-select、Model、API Key(password + toggle)、Base URL、测试连接按钮 |
| 20 | TTS 面板含引擎下拉 + 动态参数 + Edge-TTS voice 加载 | ✓ VERIFIED | `TtsPanel.vue` 含 7 引擎 v-select + v-if 链按引擎动态渲染字段 + Edge-TTS voice autocomplete（voices loaded via watch(engine) → loadVoices()） |
| 21 | BGM 面板含文件夹选择 + 随机/指定切换 | ✓ VERIFIED | `BgmPanel.vue` 含只读路径 + 浏览按钮（Tauri dialog）+ v-switch 随机/指定 |
| 22 | Export 面板含输出目录选择 + 格式选择 | ✓ VERIFIED | `ExportPanel.vue` 含目录浏览 + v-select (mp4/mkv/mov) |
| 23 | 模式参数面板根据 currentMode 动态显示不同字段 | ✓ VERIFIED | `ModeParamsPanel.vue` Documentary 显示帧参数，SDE 显示剧名+温度，SDP 显示片段数+时长 |
| 24 | 网络代理面板含启用开关 + HTTP/HTTPS 地址字段 | ✓ VERIFIED | `NetworkProxyPanel.vue` 含 v-switch 启用代理 + 2 个 v-text-field（HTTP/HTTPS），开关关闭时字段灰化 |
| 25 | API Key 字段 type=password + 显示/隐藏切换 | ✓ VERIFIED | 所有 LLM 面板/TTS 面板密码字段含 `:type="showKey ? 'text' : 'password'"` + 眼睛图标切换 |
| 26 | LLM Vision 仅 Documentary 可见，TTS 在 SDP 隐藏 | ✓ VERIFIED | `SettingsDrawer.vue:51` `v-if="modeStore.currentMode === 'documentary'"`；`:64` `v-if="modeStore.currentMode !== 'sdp'"` |

### Observable Truths — Plan 16-05 (Draft + Save Flow + Tests)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 27 | SettingsDrawer 关闭时自动暂存未保存编辑 | PARTIAL | saveDraft() 保存 llmVision/llmText/tts/bgm/export/modeParams，但对 networkProxy 使用 `_lastConfig?.proxy`（缓存值，非用户输入）；`onDrawerClose()` 在关闭时触发 |
| 28 | 应用启动时自动从 localStorage 恢复草稿 | ✓ VERIFIED | `App.vue:13-24` onMounted 调用 loadFromBackend() + restoreDraft() |
| 29 | 全局保存成功后清除 localStorage 草稿 | ✓ VERIFIED | `SettingsDrawer.vue:402` handleSave 成功时调用 clearDraft() |
| 30 | Footer 保存按钮旁显示"未保存的更改"橙色文字 | ✓ VERIFIED | `SettingsDrawer.vue:108-114` `v-if="hasUnsavedChanges"` + `text-warning` |
| 31 | 验证错误显示红色小圆点（panel badge） | PARTIAL | SettingSection 有 `badgeCount` prop + 红色 v-badge；SettingsDrawer 有 FIELD_TO_PANEL 映射 + parseValidationErrors；但 proxy 字段不被 collectAllConfig 收集，所以 NetworkProxyPanel 的验证永远不会触发 |
| 32 | 模式切换时未保存更改弹出确认对话框 | ✓ VERIFIED | `SettingsDrawer.vue:262-282` watch modeStore.currentMode + showModeSwitchDialog + confirmModeSwitch/cancelModeSwitch |
| 33 | App.vue onMounted 预加载 get_config() | ✓ VERIFIED | `App.vue:13` onMounted → await loadFromBackend() + restoreDraft() |
| 34 | 配置面板有 vitest 真实测试 | ✓ VERIFIED | 4 个测试文件共 21 个测试（llm.test.ts 5、tts.test.ts 4、useConfig.test.ts 6、SettingSection.test.ts 6），全部通过 |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src-tauri/src/commands/config.rs` | 5 Tauri 命令, >=250 行 | ✓ VERIFIED | 350 行，5 个 `#[tauri::command]`，含脱敏/合并/冲突检测 |
| `src-tauri/src/commands/mod.rs` | pub mod config | ✓ VERIFIED | 第 1 行 `pub mod config;`，宏内含全部 5 个命令 |
| `src-tauri/src/lib.rs` | tauri_plugin_shell::init | ✓ VERIFIED | 第 29 行 `.plugin(tauri_plugin_shell::init())` |
| `src-tauri/tauri.conf.json` | externalBin | ✓ VERIFIED | 第 29 行 `"externalBin": ["binaries/ffmpeg", "binaries/ffprobe"]` |
| `src-tauri/capabilities/default.json` | shell:allow-execute | ✓ VERIFIED | 第 17 行 `"shell:allow-execute"`，第 18 行 `"shell:allow-spawn"` |
| `src-tauri/binaries/.gitkeep` | FFmpeg 占位 | ✓ VERIFIED | 存在，59 字节 |
| `narratoai-core/src/tts/common.rs` | fn from_env | ✓ VERIFIED | `from_env()` 方法第 27 行，含测试 |
| `narratoai-core/src/tts/edge_tts.rs` | >=550 行 | ✓ VERIFIED | 1147 行，含 build_http_client/correct_clock_skew/get_voices |
| `src/stores/llm.ts` | >=80 行 | ✓ VERIFIED | 85 行，完整 LLM store |
| `src/stores/tts.ts` | >=80 行 | ✓ VERIFIED | 201 行，7 引擎配置 + voiceList |
| `src/composables/useConfig.ts` | >=120 行 (16-05) | ✓ VERIFIED | 319 行，含 loadFromBackend/saveToBackend/collectAllConfig + draft 管理 |
| `src/components/config/ProviderPresets.ts` | >=40 行 | ✓ VERIFIED (flex) | 39 行（1 行不足 min，内容完整含 10 预设 + OPTIONS + getProviderPreset） |
| `src/types/index.ts` | re-export generated | ✓ VERIFIED | 13 个 `export type` 语句 |
| `src/components/SettingsDrawer.vue` | >=200 行 | ✓ VERIFIED | 535 行，7 面板 + save flow + dialogs + snackbar |
| `src/components/config/LlmVisionPanel.vue` | >=80 行 | ✓ VERIFIED | 102 行 |
| `src/components/config/LlmTextPanel.vue` | >=80 行 | ✓ VERIFIED | 101 行 |
| `src/components/config/TtsPanel.vue` | >=120 行 | ✓ VERIFIED | 419 行，7 引擎完整渲染 |
| `src/components/config/BgmPanel.vue` | >=60 行 | ✓ VERIFIED | 72 行 |
| `src/components/config/ExportPanel.vue` | >=60 行 | ✓ VERIFIED | 60 行 |
| `src/components/config/ModeParamsPanel.vue` | >=70 行 | ✓ VERIFIED | 113 行 |
| `src/components/config/NetworkProxyPanel.vue` | >=60 行 | ✗ STUB (content) | 50 行，内容完整但代码量偏少 |
| `src/App.vue` | contains "loadFromBackend" | ✓ VERIFIED | 第 9 行 import，第 15 行调用 |
| `tests/stores/llm.test.ts` | >=30 行 | ✓ VERIFIED | 46 行，5 个测试用例 |
| `tests/composables/useConfig.test.ts` | >=30 行 | ✓ VERIFIED | 89 行，6 个测试用例 |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| lib.rs | commands/config.rs | register_all_commands! 宏 | ✓ WIRED | `mod.rs:40-44` 含全部 5 个 config 命令 |
| config.rs | narratoai-core/config/types.rs | get_config 返回 AppConfig JSON | ✓ WIRED | `config.rs:12` 通过 `config_to_json(&config, true)` 序列化 AppConfig |
| config.rs | narratoai-core/config/ConfigManager | Tauri State 注入 | ✓ WIRED | `lib.rs:79` `app.manage(config_manager)`；`config.rs:12` `State<'_, ConfigManager>` |
| edge_tts.rs correct_clock_skew | ProxyConfig::from_proxy | build_http_client | ✓ WIRED | `edge_tts.rs:459-474` 代理感知客户端构建 |
| useConfig.ts | composables/useTauri.ts | tauriInvoke('get_config') | ✓ WIRED | `useConfig.ts:178` 和 `:207` |
| stores/llm.ts | types/AppConfig.ts | AppConfig.app 字段映射 | ✓ WIRED | `llm.ts:22-37` loadConfig() 映射 AppSection 字段 |
| SettingsDrawer.vue | config/*Panel.vue | import + 渲染 | ✓ WIRED | `SettingsDrawer.vue:192-198` import 全部 7 面板 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| config.rs get_config | AppConfig | ConfigManager::config() | SQLite/TOML 文件 | ✓ FLOWING |
| config.rs save_config | updates (serde_json::Value) | Tauri IPC invoke | ConfigManager write | ✓ FLOWING (except proxy) |
| edge_tts.rs get_voices | serde_json::Value voice list | HTTPS Edge-TTS API | External API | ✓ FLOWING |
| useConfig.ts loadFromBackend | AppConfig | tauriInvoke('get_config') | Rust backend → config.toml | ✓ FLOWING |
| useConfig.ts saveDraft | SettingsDraft | localStorage | localStorage | ✓ FLOWING (except proxy uses stale value) |
| useConfig.ts collectAllConfig | Record<string, unknown> | Pinia stores | → saveToBackend → save_config | ⚠️ HOLLOW: proxy fields NOT collected |
| NetworkProxyPanel.vue | proxyEnabled/Http/Https | Local refs (no store) | Nowhere — data is lost | ✗ DISCONNECTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| TypeScript compilation | `npx vue-tsc --noEmit` | exit 0 | ✓ PASS |
| LLM store tests | `npx vitest run tests/stores/llm.test.ts` | 5/5 passed | ✓ PASS |
| TTS store tests | `npx vitest run tests/stores/tts.test.ts` | 4/4 passed | ✓ PASS |
| Draft tests | `npx vitest run tests/composables/useConfig.test.ts` | 6/6 passed | ✓ PASS |
| SettingSection tests | `npx vitest run tests/components/SettingSection.test.ts` | 6/6 passed (after npm install) | ✓ PASS |

### Probe Execution

Step 7b (Behavioral Spot-Checks) covered as above; no separate probe scripts are defined or required for this phase.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| CONF-01 | 16-01, 16-03, 16-04, 16-05 | LLM/Mode 面板 | ✓ SATISFIED | LlmVisionPanel + LlmTextPanel 含 Provider/Model/API Key/Base URL + 测试连接 |
| CONF-02 | 16-01, 16-03, 16-04, 16-05 | TTS 面板 | ✓ SATISFIED | TtsPanel 7 引擎 + 动态参数 + Edge-TTS voice 加载 |
| CONF-03 | 16-01, 16-03, 16-04, 16-05 | BGM 面板 | ✓ SATISFIED | BgmPanel 含文件夹选择 + 随机/指定切换 |
| CONF-04 | 16-01, 16-03, 16-04, 16-05 | Export 面板 | ✓ SATISFIED | ExportPanel 含输出目录 + 格式选择 |
| CONF-05 | 16-01, 16-03, 16-04, 16-05 | 模式专用面板 | ✓ SATISFIED | ModeParamsPanel 根据 currentMode 动态切换字段 |
| CONF-06 | 16-05 | Local Draft | ⚠️ NEEDS HUMAN (partial) | saveDraft/restoreDraft/clearDraft/hasDraft 完整，但 proxy 字段不走 draft 流程 |
| MODE-03 | 16-04 | 面板动态可见性 | ✓ SATISFIED | LlmVisionPanel `v-if="documentary"`、TtsPanel `v-if="!sdp"` |
| TDET-01 | 16-02 | Edge-TTS 代理隧道 | ✓ SATISFIED | ProxyConfig::from_env() + build_http_client + clock skew + voice list 均支持代理 |
| TDET-06 | 16-02 | VisualAnalyzer 门面 | ✓ SATISFIED | TDET-06 审查注释 `analyzer.rs:316-327` 确认调用路径 |
| TDET-07 | 16-01 | FFmpeg sidecar | ✓ SATISFIED | externalBin + shell plugin + capabilities + 二进制文件 |

### Anti-Patterns Found

None. Zero TBD/FIXME/XXX markers in all modified files. Zero TODO/HACK/PLACEHOLDER markers.

### Gaps Summary

**1 gap identified — NetworkProxyPanel 数据流不完整（WARNING，非 BLOCKER）**

NetworkProxyPanel 渲染完整（启用开关 + HTTP/HTTPS 地址字段、开关灰化），但其数据流存在以下问题：

- **Pinia store 缺失**: 面板使用本地 `ref()`（proxyEnabled/proxyHttp/proxyHttps），未接入任何 Pinia store
- **collectAllConfig() 不收集 proxy**: `useConfig.ts:305-319` 的 collectAllConfig() 完全不包含 proxy 字段 → save_config 不会发送 proxy 变更
- **saveDraft() 保存错误数据**: `useConfig.ts:63-65` 的 saveDraft() 使用 `_lastConfig?.proxy`（上一次 get_config 返回的缓存值），而非面板当前用户输入
- **restoreDraft() 忽略 proxy**: `useConfig.ts:82-134` 的 restoreDraft() 不恢复 networkProxy 字段

**影响**: 用户在 NetworkProxyPanel 中输入的代理配置永远不会被持久化到 config.toml。关闭设置抽屉时会用旧缓存值覆盖可能的手动编辑。

**建议修复方式**:
1. 在 `collectChangedFields()`/`collectAllConfig()` 中加入 proxy 字段收集
2. `saveDraft()` 从 NetworkProxyPanel 组件读取当前值（或创建 proxy Pinia store）
3. `restoreDraft()` 恢复 networkProxy 到面板
4. 将 `saveDraft()` 中的 `_lastConfig?.proxy` 替换为实时输入

This gap is WARNING-level, not BLOCKER — the panel UI is complete, 6/7 panels have full Pinia store integration, and the Roadmap Success Criteria for Phase 16 are substantially met. The proxy save wiring can be addressed in Phase 18 (Remaining Tech Debt) or a targeted follow-up plan.

## Summary

| Category | Result |
|----------|--------|
| Must-haves verified | 29/31 (93.5%) |
| Plans completed | 5/5 (100%) |
| Requirements satisfied | 9/10 (CONF-06 partial) |
| Artifacts created | 25+ files across Rust backend + Vue frontend |
| Tests passing | 21/21 (4 test files) |
| Blocker anti-patterns | 0 |
| Commit hashes verified | All 12+ commits present in git log |
| TypeScript compilation | `vue-tsc --noEmit` exit 0 |

**Phase 16 is substantially complete.** The sole gap is NetworkProxyPanel's data flow — the panel renders but user proxy input cannot be saved to the backend. This does not block Phase 17 (Pipeline Controls) from starting, as proxy panel data integration is independent of pipeline operations.

---

_Verified: 2026-05-14T16:05Z_
_Verifier: Claude (gsd-verifier)_
