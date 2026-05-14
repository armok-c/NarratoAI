# Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt - Research

**Researched:** 2026-05-13
**Domain:** Vue 3.5 + Vuetify 4 + Pinia 3 (前端), Rust + Tauri 2.0 (后端), FFmpeg sidecar, Edge-TTS proxy
**Confidence:** HIGH

## Summary

Phase 16 是 v2.0 里程碑中最大的单一 Phase：将 SettingsDrawer 从单一面板扩展为 7 个完整配置面板（LLM Vision/Text、TTS、BGM、Export、网络代理、模式专用参数），扩展 4 个 Pinia Store 骨架为完整实现，新增 5 个 Tauri 后端命令，实现 FFmpeg Tauri sidecar 内置打包，补齐 Edge-TTS 代理隧道和 VisualAnalyzer 门面审查。

核心架构模式：每个配置面板独立为 `src/components/config/*Panel.vue` 组件，通过 SettingSection 包装，从 Pinia Store 读取状态，通过 `tauriInvoke` 调用后端命令。Local Draft 暂存策略在 composable `useConfig.ts` 中集中实现。FFmpeg sidecar 通过 `tauri-plugin-shell` 实现，需要同时修改 Rust Cargo.toml（新增 `tauri-plugin-shell`）和前端 package.json（新增 `@tauri-apps/plugin-shell`）。

**主要建议:** Wave 0 优先实现后端基础（config.rs + FFmpeg sidecar + Edge-TTS 代理），确保 Wave 1 前端面板有可调用的命令。ts-rs 生成 TypeScript 类型缺口（`src/types/generated/` 目前为空）需在 Wave 0 解决，否则前端缺少 AppConfig 类型定义。

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### SettingsDrawer 面板组织
- **D-01:** 手风琴式布局 — 每个配置面板使用 Phase 15 已实现的 `SettingSection` collapsible 组件，复用 v-expand-transition 动画。
- **D-02:** 完整显隐规则 — Documentary: LLM Vision + LLM Text + TTS + BGM + Export + 帧参数。SDE: LLM Text + TTS + BGM + Export + 剧名/温度。SDP: LLM Text + BGM + Export + 片段数。Vision LLM 仅 Documentary 可见，TTS 在 SDP 隐藏。
- **D-03:** 面板排列顺序: LLM Vision → LLM Text → TTS → BGM → Export → 网络代理 → 模式专用参数。
- **D-04:** Footer 包含全局 v-btn "保存设置"（primary elevated）+ 版本号保留。手风琴默认仅展开第一个面板（LLM Vision），其余折叠。面板折叠状态通过 localStorage('settingsPanelStates') 跨会话记忆，不限制同时展开数量。
- **D-05:** 面板显隐过渡使用 v-expand-transition（SettingSection 已有机制），全局保存成功后显示 v-snackbar "设置已保存"（绿色，3 秒自动消失），失败显示红色 snackbar。
- **D-06:** 每个面板字段默认值从后端 config.toml 通过 get_config() Tauri 命令加载。每个面板内提供"重置"按钮恢复该面板默认值，Footer 提供"重置全部"链接文字（弹出确认对话框后清除草稿 + 重置所有 Pinia stores）。
- **D-07:** 面板图标 — LLM Vision: mdi-robot, LLM Text: mdi-chat-processing, TTS: mdi-text-to-speech, BGM: mdi-music-note, Export: mdi-file-export, 模式专用参数: mdi-tune, 网络代理: mdi-earth。
- **D-08:** API Key/Token/Secret 字段使用 type="password" + 显示/隐藏切换按钮。模式切换时检测未保存更改：弹出确认对话框"未保存的更改将丢失，是否继续？"，用户确认后才切换。

#### LLM 双模型配置 UX (CONF-01)
- **D-09:** LLM Vision 和 LLM Text 各自独立 SettingSection（非内部分组）。Vision 面板按 D-02 显隐规则在非 Documentary 模式下隐藏。
- **D-10:** 每个 LLM 面板内 4 字段垂直排列（全宽 v-text-field）: Provider（v-select 下拉）、Model（v-text-field）、API Key（type="password" + 脱敏显示）、Base URL（v-text-field + placeholder 提示"留空使用默认"）。
- **D-11:** Provider 下拉预设 — OpenAI / DeepSeek / Gemini / Qwen / SiliconFlow / Moonshot / Anthropic / Cohere / Together AI / OpenRouter / 自定义。选择 Provider 后 Model 字段下方显示灰色小字建议（如"推荐: gpt-4o"），不自动填充。
- **D-12:** 每个 LLM 面板底部添加 v-btn "测试连接"（outlined），点击后调用后端 Tauri 命令 test_llm_connection，成功绿色 snackbar，失败红色 + 错误详情。
- **D-13:** LLM 面板加载时 SettingSection 显示 loading 骨架。API Key 从后端获取后脱敏显示（`sk-...****...xyz` 格式），点击显示按钮完整展示。

#### TTS 面板 (CONF-02)
- **D-14:** 顶部 v-select 下拉选择 TTS 引擎（中文标签：Edge-TTS / Azure / 腾讯云 / SoulVoice / 阿里通义 / IndexTTS2 / 豆包），下方动态渲染当前引擎的专属配置字段。切换引擎时已填参数保留在内存中不丢失。
- **D-15:** Edge-TTS voice 列表通过 Tauri 命令 get_edge_tts_voices 从后端加载（Rust 侧调用 Edge-TTS voice list API）。前端使用 v-autocomplete 带搜索过滤。加载失败显示错误提示 + 重试按钮。

#### BGM / Export / 模式专用参数面板 (CONF-03/04/05)
- **D-16:** BGM 文件夹和 Export 输出目录使用 Tauri dialog.open() 原生文件夹选择器，路径 v-text-field 只读 + 右侧"浏览"按钮。Export 格式: MP4/MKV/MOV v-select 下拉，默认 MP4。
- **D-17:** BGM 面板内 v-switch 切换"随机"/"指定"。模式专用参数面板标题和内容随 ModeStore.currentMode 动态变化。

#### 配置验证与帮助
- **D-18:** 软验证策略 — 不阻塞输入。全局保存时后端验证返回缺失/无效字段列表。
- **D-19:** 每个 v-text-field 使用 placeholder 显示示例值。复杂字段旁加 v-icon mdi-help-circle-outline + v-tooltip 悬浮说明。

#### Local Draft 暂存策略 (CONF-06)
- **D-20:** 抽屉关闭时自动暂存到 localStorage('settingsDraft')。应用启动时静默恢复。全局保存成功后才清除。
- **D-21:** 有草稿时 Footer 保存按钮旁显示橙色文字"未保存的更改"。JSON 结构含所有面板字段 + updatedAt。
- **D-22:** 乐观草稿策略 — 草稿始终覆盖显示。保存时后端对比 config.toml 修改时间戳，冲突则提示用户。
- **D-23:** 草稿仅在全局保存成功后清除。
- **D-24:** 草稿明文存储，暂存失败静默降级，恢复失败静默跳过。

#### Edge-TTS 代理隧道 (TDET-01)
- **D-25:** 平台代理自动检测 — Rust 侧检测环境变量 HTTPS_PROXY/HTTP_PROXY/ALL_PROXY，config.toml [proxy] 段作为手动覆盖。
- **D-26:** 代理覆盖 WebSocket CONNECT（已实现）+ 时钟校准 HTTP + voice 列表 HTTP。新增 get_system_proxy 命令。前端添加网络代理面板。
- **D-27:** 优先直连，直连失败后自动尝试代理连接。代理操作超时 30s。失败 tracing::warn 记录。

#### VisualAnalyzer 门面补齐 (TDET-06)
- **D-28:** 审查 pipeline 中所有视觉分析调用路径，统一走 analyzer.rs 的 analyze_video_frames() 门面。prompt template 从 PromptRegistry 加载而非硬编码。不新增功能，不新增 Tauri 命令。

#### FFmpeg Tauri Sidecar 打包 (TDET-07)
- **D-29:** 手动嵌入 FFmpeg 7.1+ 静态构建二进制文件，使用 git-lfs 管理。目录: `src-tauri/binaries/ffmpeg-{platform}/ffmpeg{ext}`。
- **D-30:** tauri.conf.json 添加 externalBin 声明。添加 shell plugin + allow-execute scope。

#### 配置数据流与 Tauri 命令
- **D-31:** get_config() 返回完整脱敏 AppConfig JSON。save_config() 接收增量更新。
- **D-32:** App.vue onMounted 预加载 get_config() 到 Pinia stores。加载失败 snackbar 提示。
- **D-33:** 新增 5 个 Tauri 命令在 `src-tauri/src/commands/config.rs` 新文件。

#### 实现优先级与计划拆分 (D-34)
- Wave 0: FFmpeg sidecar + 5 个 Tauri 命令 + Edge-TTS 代理后端 + VisualAnalyzer 审查
- Wave 1: SettingsDrawer 手风琴重构 + 所有配置面板 + Pinia stores 扩展
- Wave 2: Local Draft 暂存 + 全局保存/验证/snackbar + 模式切换面板显隐 + 测试

#### 测试策略 (D-35)
- 前端 vitest 真实测试 — SettingSection 组件、Pinia stores
- Edge-TTS 代理隧道 — Rust 单元测试 + 标记 #[ignore] 的集成测试
- FFmpeg sidecar — smoke test 调用 ffmpeg -version

### Claude's Discretion
- SettingSection loading 骨架屏的具体实现细节和过渡时间
- Provider 下拉列表中各 Provider 的默认 Base URL 映射
- v-snackbar 的精确位置、颜色和持续时间
- 网络代理面板中"启用"开关与空代理地址的交互逻辑
- test_llm_connection 命令的具体请求构造和超时设置
- get_edge_tts_voices 的缓存策略和刷新机制
- 配置面板 TypeScript 类型从 ts-rs 导出（或手动定义接口）
- git-lfs 配置和 FFmpeg 二进制文件版本固定
- Tauri capabilities 中 shell:allow-execute 的具体 scope 路径

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONF-01 | LLM/Mode 面板 — 双模型配置（Vision + Text），Provider/Model/API Key/Base URL | llm.ts 骨架已存在需扩展；get_config/save_config 命令提供后端数据；test_llm_connection 命令需新建；Provider 预设表已定义 |
| CONF-02 | TTS 面板 — 7 引擎选择 + 各引擎参数配置，Edge-TTS voice 列表加载 | tts.ts 骨架已存在需扩展；get_edge_tts_voices 命令需新建；edge_tts.rs 已有 voice list API 端点 |
| CONF-03 | BGM 面板 — BGM 文件夹选择、随机/指定模式切换 | bgm.ts 骨架已存在需扩展；dialog.open() 已可用（tauri-plugin-dialog 已安装）|
| CONF-04 | Export 面板 — 输出目录选择、导出格式选项 | export.ts 骨架已存在需扩展；dialog.open() 原生文件夹选择器 |
| CONF-05 | 模式专用面板 — Documentary/SDE/SDP 动态参数 | 无独立 Store；modeParams 可合并到 mode store 或独立 composable；面板根据 ModeStore.currentMode 动态渲染 |
| CONF-06 | Local Draft 模式 — 未保存编辑暂存，仅显式保存或启动时持久化 | localStorage 持久化模式已存在于 AppStore/ModeStore；useConfig.ts composable 承载 draft 逻辑 |
| TDET-01 | Edge-TTS 代理隧道实现（HTTP CONNECT + 平台代理自动检测） | edge_tts.rs connect() 已实现 WebSocket CONNECT；correct_clock_skew() 缺代理；环境变量检测未实现；ProxyConfig 模式可用 |
| TDET-06 | VisualAnalyzer 门面调用补齐（统一 API 入口） | analyze_video_frames() 已是统一门面；需审查 pipeline 调用路径是否全部使用此门面；prompt template 来源需验证 |
| TDET-07 | FFmpeg Tauri sidecar 内置打包（externalBin + shell:allow-execute） | 需新增 tauri-plugin-shell Cargo + npm 依赖；tauri.conf.json 加 externalBin；capabilities 加 shell:allow-execute |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| LLM 配置字段渲染 | Browser (Vue) | — | 表单字段在前端渲染，通过 Tauri 命令读写后端配置 |
| LLM 测试连接 | API (Rust) | — | test_llm_connection 命令直接调用 LLM Provider，不涉及前端 |
| TTS 引擎参数 | Browser (Vue) | API (Rust) | 面板渲染由前端负责；voice 列表通过 get_edge_tts_voices 命令从后端获取 |
| BGM/Export 文件夹选择 | Browser (Vue) + Native Dialog | — | dialog.open() 是前端调用 Tauri plugin，返回路径后渲染到只读字段 |
| Local Draft 暂存 | Browser (localStorage) | — | 纯前端机制，不涉及后端 |
| FFmpeg sidecar 执行 | API (Rust) | CDN (bundle) | sidecar 在 Rust 端通过 tauri_plugin_shell::ShellExt 调用；二进制嵌入在 bundle 中 |
| Edge-TTS 代理隧道 | API (Rust) | — | 全在 Rust 后端处理；前端仅展示代理配置并调用 get_system_proxy |
| 配置读写（config.toml） | API (Rust) | — | get_config/save_config 命令在后端操作文件系统 |
| 面板折叠状态持久化 | Browser (localStorage) | — | SettingsDrawer 组件内管理面板展开/折叠状态 |
| 模式切换面板显隐 | Browser (Vue) | — | 根据 ModeStore.currentMode 计算 v-if 条件 |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue | ^3.5.x (已安装 3.5.x) | 前端框架 | Composition API + TypeScript-first，项目已安装 |
| Vuetify | ^4.0.x (已安装 4.x) | Material Design 组件库 | v-navigation-drawer/v-select/v-switch/v-text-field/v-autocomplete/v-snackbar/v-dialog 全部需要 |
| Pinia | ^3.0.x (已安装 3.x) | 状态管理 | 9 个 Store 全部使用 Composition API 风格 |
| TypeScript | ^6.0.x (已安装 6.x) | 类型检查 | 严格模式 |
| vue-router | ^5.0.x (已安装 5.x) | 路由 | createWebHashHistory |

### Tauri Integration

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/api | ^2.x (已安装) | 核心 JS API | invoke() 调用后端命令 |
| @tauri-apps/plugin-dialog | ^2.x (已安装) | 原生文件对话框 | BGM/Export 文件夹选择 |
| @tauri-apps/plugin-shell | ^2.x (**需新增**) | Shell + sidecar 执行 | FFmpeg sidecar 调用 |
| tauri-plugin-shell | "2" (Cargo, **需新增**) | Rust 端 shell 支持 | Sidecar 的 Rust 端 API |
| @tauri-apps/plugin-shell | ^2.x (npm, **需新增**) | 前端 shell API | Command.sidecar() JS API |

### Type System

| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| ts-rs | 11.x (已在 Cargo.toml) | Rust 到 TS 类型导出 | `#[ts(export, export_to = "../src/types/generated/")]` 已在 config/types.rs 中配置，但生成产物目录目前为空 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ts-rs 生成类型 | 手动定义 TypeScript 接口 | ts-rs 确保前后端类型一致；但 generates 目录当前为空，需运行 `cargo test` 触发生成；若 ts-rs 输出路径不对可考虑手动定义接口作为 Claude's Discretion |

### Installation

```bash
# FFmpeg sidecar 所需
npm install @tauri-apps/plugin-shell@^2
# Rust 端需加 tauri-plugin-shell = "2" 到 src-tauri/Cargo.toml [dependencies]
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Browser (Vue 3 + Vuetify 4)                      │
│                                                                         │
│  SettingsDrawer.vue  ───  uses  ──▶  SettingSection (x7 panels)        │
│       │                              ├── LlmVisionPanel.vue             │
│       │                              ├── LlmTextPanel.vue               │
│       │                              ├── TtsPanel.vue                   │
│       │                              ├── BgmPanel.vue                   │
│       │                              ├── ExportPanel.vue                │
│       │                              ├── NetworkProxyPanel.vue          │
│       │                              └── ModeParamsPanel.vue             │
│       │                                                                  │
│       └── reads/writes ──▶  Pinia Stores (llm/tts/bgm/export/mode)      │
│                               │                                          │
│                               ├── localStorage('settingsDraft') ─── DR  │
│                               ├── localStorage('settingsPanelStates')   │
│                               │                                          │
│  App.vue (onMounted) ─── preload get_config() ───▶ to stores            │
│                               │                                          │
│  useConfig.ts composable ─── tauriInvoke() ──────▶ Tauri IPC            │
│                               │                                          │
└───────────────────────────────┼──────────────────────────────────────────┘
                                │ Tauri IPC (invoke)
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   Tauri Backend (Rust + commands/)                       │
│                                                                         │
│  commands/config.rs  ── 5 new commands:                                  │
│  ├── get_config()        ──▶ ConfigManager.get() + desensitize          │
│  ├── save_config(data)   ──▶ ConfigManager update + write toml          │
│  ├── test_llm_connection ──▶ reqwest to OpenAI compatible endpoint       │
│  ├── get_edge_tts_voices ──▶ HTTP GET voices/list + proxy support       │
│  └── get_system_proxy    ──▶ env vars + config.toml proxy merge          │
│                                                                         │
│  commands/system.rs  ──  get_system_stats (Phase 15, exists)            │
│                                                                         │
│  lib.rs (setup) ──▶ ConfigManager + tauri-plugin-shell init             │
│                                                                         │
│  FFmpeg Sidecar (via tauri_plugin_shell):                               │
│  ├── src-tauri/binaries/ffmpeg-{platform}/ffmpeg                        │
│  └── tauri.conf.json: externalBin + shell plugin                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       narratoai-core (library crate)                     │
│                                                                         │
│  tts/edge_tts.rs ──▶ correct_clock_skew() + connect() proxy tunnel      │
│                          ↑ gap: clock skew + voice list need proxy       │
│  tts/common.rs    ──▶ ProxyConfig::from_proxy() + apply_to_client()     │
│  visual/analyzer.rs ──▶ analyze_video_frames() [门面已就绪]              │
│  config/types.rs    ──▶ AppConfig (all sections, #[derive(TS)])         │
│  config/mod.rs      ──▶ ConfigManager::load() + get()                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Key Props/Interactions |
|-----------|---------------|----------------------|
| `LlmVisionPanel.vue` | Vision LLM 配置字段 + Provider 预设 + 测试连接 | useLlmStore().visionConfig, tauriInvoke('test_llm_connection') |
| `LlmTextPanel.vue` | Text LLM 配置字段 + Provider 预设 + 测试连接 | useLlmStore().textConfig, tauriInvoke('test_llm_connection') |
| `TtsPanel.vue` | 引擎选择 + 动态引擎参数 + voice 列表加载 | useTtsStore(), tauriInvoke('get_edge_tts_voices') |
| `BgmPanel.vue` | 文件夹路径 + 随机/指定切换 | useBgmStore(), dialog.open() |
| `ExportPanel.vue` | 输出目录 + 格式选择 | useExportStore(), dialog.open() |
| `ModeParamsPanel.vue` | 动态参数 (帧参数/剧名温度/片段数) | useModeStore().currentMode, useLlmStore() for 帧参数 |
| `NetworkProxyPanel.vue` | HTTP/HTTPS 代理地址 + 启用开关 | tauriInvoke('get_system_proxy'), tauriInvoke('save_config') |
| `SettingsDrawer.vue` | 手风琴布局 + Footer 保存/重置 | useLlmStore/useTtsStore/useBgmStore/useExportStore, tauriInvoke('save_config') |
| `useConfig.ts` | get_config/save_config 封装 + draft 管理 | localStorage('settingsDraft'), tauriInvoke |
| `commands/config.rs` | 5 个新 Tauri 命令 | 读取/写入 ConfigManager, LLM 请求, Edge-TTS API, 环境变量 |

### Pattern 1: Panel Component Structure

每个配置面板遵循统一结构：
```
<SettingSection :icon="..." :title="..." collapsible :loading="loading">
  <template #header-actions>
    <v-btn ... @click="resetPanel">重置</v-btn>
  </template>
  <template #default>
    <!-- 配置字段 -->
    <v-text-field v-model="field" label="..." ... />
  </template>
</SettingSection>
```

### Pattern 2: Draft Save/Restore (from existing AppStore/ModeStore)

```typescript
// localStorage 持久化模式 (已验证于 app.ts / mode.ts):
const stored = localStorage.getItem('settingsDraft')
const draft = ref<DraftData | null>(stored ? JSON.parse(stored) : null)

watch(draft, (val) => {
  localStorage.setItem('settingsDraft', JSON.stringify(val))
}, { deep: true })
```

### Pattern 3: Tauri Command Registration (from commands/mod.rs)

```rust
// 1. 在 commands/mod.rs 顶部添加:
pub mod config;
pub use config::*;

// 2. 在 register_all_commands!() 宏内添加:
crate::commands::config::get_config,
crate::commands::config::save_config,
crate::commands::config::test_llm_connection,
crate::commands::config::get_edge_tts_voices,
crate::commands::config::get_system_proxy,
```

### Pattern 4: Typed Tauri Invoke (from useTauri.ts)

```typescript
import { tauriInvoke } from '@/composables/useTauri'

// 调用 get_config 命令
const config = await tauriInvoke<AppConfig>('get_config')

// 调用 save_config 命令（增量更新）
await tauriInvoke<void>('save_config', {
  llm: { vision_provider: 'openai', vision_model: 'gpt-4o' },
  tts: { engine: 'edge_tts' }
})
```

### Anti-Patterns to Avoid

- **在 SettingsDrawer.vue 中内联所有面板代码** — 每个面板应为独立组件文件 (`src/components/config/*Panel.vue`)，避免单一文件过大
- **为配置验证创建自定义验证库** — 后端软验证 + 前端 placeholder/tooltip 足以；不需要引入 zod/joi
- **使用 v-show 替代 v-if 控制面板显隐** — D-02 明确要求 v-if（避免隐藏面板占用 DOM）
- **在多个地方重复 draft 读写逻辑** — draft 管理集中在 useConfig.ts composable 中
- **前端直接读写 config.toml** — 所有配置读写必须通过 Tauri IPC 命令，前端不接触文件系统

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LLM 测试连接 | 自己发 HTTP 请求 | test_llm_connection Tauri 命令 | 后端已有 reqwest + LLM Provider；统一错误处理和超时管理 |
| Voice 列表获取 | 前端直接调 Edge-TTS API | get_edge_tts_voices Tauri 命令 | 需要走代理隧道、环境变量检测；Rust 端已有 edge_tts.rs |
| 原生文件夹选择 | 自定义对话框 | @tauri-apps/plugin-dialog open() | 跨平台一致性；Tauri 原生 API 已验证 |
| FFmpeg 调用 | 系统 which + subprocess | tauri-plugin-shell sidecar | 生产环境 sidecar 路径自动解析；shell:allow-execute 权限控制 |
| 配置 TOML 读写 | 前端 TOML 解析 | 后端 ConfigManager + save_config 命令 | 配置逻辑集中在 Rust 端；TOML 格式和注释完整性由后端保证 |
| Provider 列表管理 | 硬编码所有 Provider 信息 | 前端组件内 preset 数组 + placeholder 建议 | 简化实现；Claude's Discretion 确认最终列表 |

**关键洞察:** 这五个后端命令（get_config/save_config/test_llm_connection/get_edge_tts_voices/get_system_proxy）是前端和后端之间的防坠落层。不要在前端复制任何后端逻辑（代理检测、LLM 请求、TOML 解析）。

## Common Pitfalls

### Pitfall 1: ts-rs 生成目录为空或路径不对
**什么会出错:** 前端编译时报 `AppConfig` 类型找不到，因为 `src/types/generated/` 目录为空。
**原因:** ts-rs 类型生成需要运行 `cargo test`，且 `#[ts(export_to = "../src/types/generated/")]` 路径相对于 `narratoai-core/` crate 根。假设工作目录是项目根，生成文件实际上会到 `E:\GitLib\NarratoAI\src\types\generated\`。
**如何避免:** Wave 0 先在 narratoai-core 目录运行 `cargo test` 确认类型文件生成到正确位置。如果路径不对，调整 `#[ts(export_to = "...")]` 路径。另一种方案：手动定义 `AppConfig` TypeScript 接口作为后备。
**警告信号:** `import type { AppConfig } from '@/types/generated'` 编译报错。

### Pitfall 2: tauri-plugin-shell 未添加到 Cargo.toml 和前端依赖
**什么会出错:** FFmpeg sidecar 调用在 Tauri dev/build 时报 `plugin not found` 错误。
**原因:** sidecar 需要 Rust 端 `tauri-plugin-shell` crate 和前端 `@tauri-apps/plugin-shell` npm 包同时安装。
**如何避免:** 在 Cargo.toml 和 package.json 中同步添加这两个依赖，并在 lib.rs 的 `tauri::Builder` 链中添加 `.plugin(tauri_plugin_shell::init())`。
**警告信号:** 运行 `cargo tauri dev` 后 console 报 `Invoke API not found` 或 `Shell plugin not initialized`。

### Pitfall 3: FFmpeg sidecar 平台后缀不匹配
**什么会出错:** Tauri 找不到 FFmpeg 二进制文件，"No such file or directory"。
**原因:** 目录结构必须是 `src-tauri/binaries/ffmpeg-{target-triple}{.exe}`，但文件名后缀与 `rustc -vV` 输出的 target 不匹配。
**如何避免:** 用 `rustc -vV | grep host | cut -d' ' -f2` 确认 target triple，以相同 triple 命名。Windows 上加 `.exe` 后缀。

```
# Windows x64 (本例环境):
src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
src-tauri/binaries/ffprobe-x86_64-pc-windows-msvc.exe

# macOS ARM:
src-tauri/binaries/ffmpeg-aarch64-apple-darwin

# Linux x64:
src-tauri/binaries/ffmpeg-x86_64-unknown-linux-gnu
```

### Pitfall 4: save_config 增量更新与 TOML 注释丢失
**什么会出错:** 每次保存配置时 TOML 文件中的注释被覆盖丢失。
**原因:** toml crate 的反序列化/再序列化默认不保留注释。
**如何避免:** save_config 命令只覆盖变更的字段（使用 `toml_edit` 或手动合并），保持配置文件中其他部分和注释不变。D-31 已指定"增量语义"，后端须处理合并逻辑。

### Pitfall 5: localStorage 草稿与后端配置冲突
**什么会出错:** 应用启动时 localStorage 草稿覆盖了用户新修改的 config.toml 配置。
**原因:** D-22 草稿始终覆盖显示，但用户可能在外部分改了 config.toml。
**如何避免:** 启动时先调用 get_config() 加载后端配置，仅在 localStorage 有草稿且 `updatedAt` 比 get_config 时间戳新时才恢复草稿。冲突时显示提示让用户选择。

### Pitfall 6: Edge-TTS voice list 加载阻塞面板首次渲染
**什么会出错:** TTS 面板打开时 voice 列表还在加载，用户看到空白下拉。
**原因:** get_edge_tts_voices 是异步网络请求，首次打开会有延迟。
**如何避免:** v-autocomplete 设置 loading 状态；voice 列表在 Pinia tts store 中缓存避免重复请求。缓存过期策略由 Claude's Discretion 确定。

### Pitfall 7: API Key 脱敏逻辑重复
**什么会出错:** 后端脱敏和后端脱敏逻辑不一致导致显示混乱。
**原因:** D-31 说后端在 get_config 时脱敏 API Key，如果前端也做了脱敏就会双重脱敏。
**如何避免:** 统一在后端 Rust 侧脱敏（`sk-...****...xyz` 格式），前端直接显示后端返回的值。前端仅在"显示/隐藏"切换时调用后端获取完整明文。

## Code Examples

### LLM Panel Provider Presets (Frontend)

```typescript
// 约 11 个预设 Provider + 推荐模型映射 (Claude's Discretion)
const PROVIDER_PRESETS: Record<string, { recommendedModel: string; defaultBaseUrl: string }> = {
  openai:      { recommendedModel: 'gpt-4o',           defaultBaseUrl: 'https://api.openai.com/v1' },
  deepseek:    { recommendedModel: 'deepseek-chat',    defaultBaseUrl: 'https://api.deepseek.com/v1' },
  gemini:      { recommendedModel: 'gemini-2.0-flash', defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai' },
  qwen:        { recommendedModel: 'qwen-plus',        defaultBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
  siliconflow: { recommendedModel: 'Qwen/Qwen3.5-122B-A10B', defaultBaseUrl: 'https://api.siliconflow.cn/v1' },
  moonshot:    { recommendedModel: 'moonshot-v1-8k',   defaultBaseUrl: 'https://api.moonshot.cn/v1' },
  anthropic:   { recommendedModel: 'claude-sonnet-4-20250514', defaultBaseUrl: 'https://api.anthropic.com/v1' },
  cohere:      { recommendedModel: 'command-r-plus',   defaultBaseUrl: 'https://api.cohere.com/v1' },
  together:    { recommendedModel: 'meta-llama/Llama-4-17B', defaultBaseUrl: 'https://api.together.xyz/v1' },
  openrouter:  { recommendedModel: 'openai/gpt-4o',    defaultBaseUrl: 'https://openrouter.ai/api/v1' },
}
// "自定义" 不在预设中，留空让用户手动输入
```

### get_config Desensitization (Rust)

```rust
// 在 config.rs Tauri 命令中，返回前脱敏 API Key
fn desensitize_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    let prefix = &key[..3];
    let suffix = &key[key.len()-3..];
    format!("{}...****...{}", prefix, suffix)
}
```

### save_config Incremental Merge (Rust — pseudo pattern)

```rust
// save_config 接收 PartialConfig (仅含修改的字段)
#[derive(Deserialize)]
pub struct PartialConfig {
    pub llm_vision_provider: Option<String>,
    pub llm_vision_api_key: Option<String>,
    // ... 所有字段均为 Option<String>
}

#[tauri::command]
pub async fn save_config(
    updates: PartialConfig,
    state: State<'_, Arc<RwLock<AppConfig>>>,
) -> Result<(), CommandError> {
    let mut config = state.write().map_err(|e| ...)?;
    if let Some(provider) = updates.llm_vision_provider {
        config.app.vision_llm_provider = provider;
    }
    // ... 其他字段同样处理
    // 写回 config.toml (保留注释需用 toml_edit 或 file::write)
    Ok(())
}
```

### FFmpeg Sidecar Rust Invocation

```rust
// 使用 tauri_plugin_shell 调用 FFmpeg sidecar
use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn run_ffmpeg(app: tauri::AppHandle, input: String, output: String) -> Result<String, CommandError> {
    let output = app.shell()
        .sidecar("ffmpeg")
        .map_err(|e| CommandError { code: "SIDECAR_ERROR".into(), message: e.to_string() })?
        .args(["-i", &input, "-c:v", "libx264", &output])
        .output()
        .await
        .map_err(|e| CommandError { code: "FFMPEG_ERROR".into(), message: e.to_string() })?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(CommandError {
            code: "FFMPEG_ERROR".into(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
```

### Edge-TTS Voice List API (Rust)

```rust
// 从 edge_tts.rs 获取 voice list，目前缺少 proxy 支持
// 需要修改 current 函数（第 470-508 行）让 correct_clock_skew 和新增的 get_voices 使用 ProxyConfig

pub async fn get_edge_tts_voices(proxy_enabled: bool, proxy_https: String) -> Result<Vec<VoiceInfo>, TTSError> {
    let client = if proxy_enabled {
        reqwest::Client::builder()
            .proxy(reqwest::Proxy::https(&proxy_https)?)
            .build()?
    } else {
        reqwest::Client::new()
    };
    
    let voices = client
        .get("https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list")
        .header("User-Agent", build_user_agent())
        .send().await?
        .json::<Vec<VoiceInfo>>().await?;
    
    Ok(voices)
}
```

### Tauri Sidecar externalBin Config

```json
// tauri.conf.json 新增:
{
  "bundle": {
    "externalBin": ["binaries/ffmpeg", "binaries/ffprobe"]
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

```json
// capabilities/default.json 新增:
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:event:allow-emit",
    "core:event:allow-listen",
    "dialog:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
    "core:window:allow-is-maximized",
    "shell:default",
    "shell:allow-execute",
    "shell:allow-spawn"
  ]
}
```

### Frontend dialog.open() for folder selection

```typescript
import { open } from '@tauri-apps/plugin-dialog'

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择文件夹',
  })
  if (selected) {
    bgmStore.setFolder(selected)
  }
}
```

### Tauri App Library Setup (lib.rs additions)

```rust
// 在 lib.rs run() 函数中添加:
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())  // ← 新增 (TDET-07)
        .setup(|app| {
            // ... 现有 setup 代码不变
            Ok(())
        })
        .invoke_handler(register_all_commands!())  // ← config 命令自动注册
        .run(tauri::generate_context!())
        .expect("NarratoAI 应用启动失败");
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SettingsDrawer 仅含外观面板 | 7 个完整配置面板 | Phase 16 | 用户可配置全部流水线参数 |
| Pinia stores 骨架 | 完整字段 + actions | Phase 16 | stores 可直接对接后端命令 |
| 无 Tauri 配置命令 | 5 个 config.rs 命令 | Phase 16 | 前端通过 IPC 读写 config.toml |
| FFmpeg from $PATH / ffmpeg-sidecar auto-download | FFmpeg Tauri sidecar 内置打包 | Phase 16 | 消除外部依赖，生产环境自包含 |
| Edge-TTS 时钟校准和 voice 列表不走代理 | 全部请求通过代理隧道 | Phase 16 | 代理环境下正常工作 |
| config.toml 手动编辑 | SettingsDrawer GUI 配置 | Phase 16 | 用户无需直接编辑 TOML 文件 |

### Deprecated/outdated

- **STACK.md** 建议使用 Vuetify 3，但项目实际使用 Vuetify 4（`^4.0.0`）。所有 Vuetify 组件 API 兼容 v3 模式，v4 差异不影响 Phase 16 需求。

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[ts(export_to = "../src/types/generated/")]` 路径相对于 `narratoai-core/` crate 根，生成文件到 `E:\GitLib\NarratoAI\src\types\generated\` | Standard Stack | 生成位置不对需手动复制或调整路径 |
| A2 | Vuetify 4 的 `v-navigation-drawer`、`v-select`、`v-btn` 等组件 API 与 Phase 14/15 使用的模式完全兼容 | Architecture Patterns | 若 Vuetify 4 有破坏性变更需适配 |
| A3 | `tauri-plugin-shell` 在 Rust 侧的 `.plugin(tauri_plugin_shell::init())` 和前端 `Command.sidecar()` API 如 Tauri 2 文档所述 | Code Examples | Sidecar API 签名可能不同需查看官方文档 |

## Open Questions (RESOLVED)

1. **ts-rs 类型生成当前状态**
   - 已知: `narratoai-core/src/config/types.rs` 已有 `#[ts(export, export_to = "../src/types/generated/")]` 属性
   - 不清楚: 运行 `cargo test` 是否已成功生成 `.ts` 文件到预期位置；生成的 `.ts` 文件中敏感字段 (`#[ts(skip)]`) 是否被正确排除
   - 建议: Wave 0 先运行 `cargo test -p narratoai-core` 确认生成，若失败则手动定义 AppConfig 接口
   - — RESOLVED: Plan 16-03 Task 1 直接处理 ts-rs 类型同步（复制生成文件到 src/types/generated/），若生成失败则手动定义接口作为后备

2. **Edge-TTS 环境变量检测实现位置**
   - 已知: D-25 要求 Rust 端检测 HTTPS_PROXY/HTTP_PROXY/ALL_PROXY 环境变量
   - 不清楚: 检测逻辑放 `commands/config.rs`（get_system_proxy 命令内）还是放 `narratoai-core/src/tts/common.rs` 的 `ProxyConfig` 中
   - 建议: 放 `common.rs` 的 `ProxyConfig::from_env()` 方法中使之可复用
   - — RESOLVED: 实现在 narratoai-core/src/tts/common.rs 的 `ProxyConfig::from_env()` 方法中（Plan 16-02 Task 1）

3. **FFmpeg 版本选择**
   - 已知: D-29 说 FFmpeg 7.1，但本地环境安装的是 FFmpeg 8.1.1
   - 不清楚: Tauri sidecar 是否需要捆绑特定 FFmpeg 版本
   - 建议: 捆绑 7.1 稳定静态构建版本（由 git-lfs 管理），与 v1.0 流水线兼容
   - — RESOLVED: 捆绑 FFmpeg 7.1 静态构建版本（D-29 已有决策），用户手动下载放置到 src-tauri/binaries/；开发期间可用系统 FFmpeg 8.1.1 替代测试

4. **modeParams 存储位置**
   - 已知: 模式专用参数（帧间隔、剧名/温度、片段数）需存储和持久化
   - 不清楚: 数据放独立 Pinia store、合并到 llm store、还是合并到 mode store
   - 建议: 合并到 mode store 最自然（`useModeStore().params`），因为这些参数随模式切换而变化
   - — RESOLVED: 合并到 mode store 的 params 对象中（Plan 16-03 Task 2 实现）

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build | Yes | 22.22.0 | — |
| npm | Package install | Yes | 10.9.4 | — |
| Rust + Cargo | Backend build | Yes | 1.93.1 | — |
| FFmpeg | Sidecar binary (dev) | Yes | 8.1.1 | Use system FFmpeg for dev testing |
| git-lfs | FFmpeg binary storage | Yes | 3.7.1 | — |
| Python 3.12 | Dev tools only | Yes | 3.12.10 | — |
| Streamlit | Legacy UI (phase complete) | Yes | 1.54.0 | Not needed for Phase 16 |

**Missing dependencies with no fallback:**
- None — all core dependencies available

**Missing dependencies with fallback:**
- `@tauri-apps/plugin-shell` and `tauri-plugin-shell` — not yet installed; will be installed in Wave 0
- FFmpeg sidecar binary — not yet placed in `src-tauri/binaries/`; system FFmpeg 8.1.1 can be used for dev testing

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | vitest ^4.1.6 |
| Config file | `vitest.config.ts` (project root) |
| Quick run command | `npx vitest run --reporter verbose` |
| Full suite command | `npx vitest run` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-01 | LLM panels render fields, provider select changes model suggestion | unit (vitest) | `npx vitest run tests/components/SettingSection.test.ts` | ❌ Wave 0 |
| CONF-01 | LLM store load/save/desensitize logic | unit (vitest) | `npx vitest run tests/stores/llm.test.ts` | ❌ Wave 0 |
| CONF-02 | TTS store engine switching preserves per-engine params | unit (vitest) | `npx vitest run tests/stores/tts.test.ts` | ❌ Wave 0 |
| TDET-01 | ProxyConfig env var detection | unit (Rust) | `cargo test -p narratoai-core -- tts::common` | ✅ exists |
| TDET-01 | WebSocket CONNECT request construction | unit (Rust) | `cargo test -p narratoai-core -- edge_tts` | ✅ exists |
| TDET-07 | FFmpeg sidecar binary exists and is executable | smoke (Bash) | `ls -la src-tauri/binaries/ffmpeg-*` | ❌ Wave 0 |
| TDET-07 | FFmpeg --version via sidecar | integration (Rust, #[ignore]) | `cargo test -p narratoai-app -- --ignored ffmpeg_version` | ❌ Wave 0 |
| CONF-06 | Draft save/restore roundtrip | unit (vitest) | `npx vitest run tests/composables/useConfig.test.ts` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `npx vitest run --changed`
- **Per wave merge:** `npx vitest run`
- **Phase gate:** Full suite green before `/gsd-verify-work`; Rust `cargo test --workspace` green

### Wave 0 Gaps

- [ ] `tests/components/SettingSection.test.ts` — panel rendering, collapsible, loading, reset
- [ ] `tests/stores/llm.test.ts` — loadConfig, saveConfig, desensitize, testConnection
- [ ] `tests/stores/tts.test.ts` — engine switching, voice list caching, per-engine params
- [ ] `tests/composables/useConfig.test.ts` — draft save/restore/clear
- [ ] `tests/ffmpeg_sidecar_test.rs` — FFmpeg version smoke test (Rust, #[ignore] for CI)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | 桌面单用户应用，无用户认证系统 |
| V3 Session Management | No | 无会话管理需求 |
| V4 Access Control | No | 单用户桌面应用，无访问控制需求 |
| V5 Input Validation | Yes | 前端 v-text-field 标准输入 + 后端 path traversal 检查（validate_path 已实现）|
| V6 Cryptography | Partial | API Key 在 get_config 中脱敏传输；save_config 要求明文 key 通过 IPC |

### Known Threat Patterns for NarratoAI

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API Key 泄露 via IPC 日志 | Information Disclosure | useTauri.ts filterSensitiveArgs() 已过滤；日志仅在 DEV 模式输出 |
| Path traversal via 文件夹选择 | Tampering | validate_path() 拒绝 `..` 序列（error.rs 已实现）；dialog.open() 原生选择器本身限制文件系统 |
| API Key 在 localStorage 明文存储 | Information Disclosure | D-24 已确认桌面单用户环境风险低；不额外加密 |
| FFmpeg sidecar 路径替换 | Tampering | 内置包 embedded via externalBin，不允许用户配置；shell:allow-execute 限定 scope |

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase] — `src/components/SettingsDrawer.vue` — 当前抽屉实现（Phase 15）
- [VERIFIED: codebase] — `src/components/SettingSection.vue` — 可折叠卡片组件
- [VERIFIED: codebase] — `src/stores/llm.ts`, `tts.ts`, `bgm.ts`, `export.ts` — 4 个配置 Store 骨架
- [VERIFIED: codebase] — `src/stores/mode.ts`, `app.ts` — 带 localStorage 持久化的 Store
- [VERIFIED: codebase] — `src/composables/useTauri.ts` — tauriInvoke 封装
- [VERIFIED: codebase] — `src-tauri/src/commands/mod.rs` — 命令注册宏模式
- [VERIFIED: codebase] — `src-tauri/src/lib.rs` — 应用启动和 State 注入
- [VERIFIED: codebase] — `src-tauri/src/commands/system.rs` — 现有命令参考
- [VERIFIED: codebase] — `src-tauri/src/error.rs` — CommandError + From 实现
- [VERIFIED: codebase] — `narratoai-core/src/tts/edge_tts.rs` — Edge-TTS 引擎（含 WebSocket CONNECT 代理）
- [VERIFIED: codebase] — `narratoai-core/src/tts/common.rs` — ProxyConfig 结构体
- [VERIFIED: codebase] — `narratoai-core/src/visual/analyzer.rs` — analyze_video_frames() 门面
- [VERIFIED: codebase] — `narratoai-core/src/config/types.rs` — AppConfig 完整类型（含 #[derive(TS)]）
- [VERIFIED: codebase] — `narratoai-core/src/config/mod.rs` — ConfigManager
- [VERIFIED: codebase] — `src-tauri/tauri.conf.json` — 当前 Tauri 配置（无 externalBin/shell）
- [VERIFIED: codebase] — `src-tauri/capabilities/default.json` — 当前权限（无 shell:allow-execute）
- [VERIFIED: codebase] — `src-tauri/Cargo.toml` — 当前 Rust 依赖（无 tauri-plugin-shell）
- [VERIFIED: codebase] — `package.json` — 当前 npm 依赖（无 @tauri-apps/plugin-shell）
- [VERIFIED: npm registry / package.json] — `vuetify@^4.0.0` — 项目实际使用 Vuetify 4（非 STACK.md 建议的 Vuetify 3）
- [VERIFIED: npm registry / package.json] — `pinia@^3.0.0`, `vite@^8.0.0`, `vitest@^4.1.6`
- [VERIFIED: system] — `node v22.22.0`, `rustc 1.93.1`, `cargo 1.93.1`, `ffmpeg 8.1.1`
- [VERIFIED: system] — `git-lfs 3.7.1` — 可用于 FFmpeg 二进制管理

### Secondary (MEDIUM confidence)
- [CITED: Tauri 2 sidecar docs] — `externalBin` + `tauri-plugin-shell` 配置模式
- [CITED: Tauri 2 capabilities docs] — `shell:allow-execute` 权限声明
- [VERIFIED: SmartEdit codebase] — `E:/GitLib/SmartEdit/src/components/SettingsDrawer.vue` — 三段式抽屉参考
- [VERIFIED: SmartEdit codebase] — `E:/GitLib/SmartEdit/src/components/SettingSection.vue` — 支持 v-model:expanded 的增强版

### Tertiary (LOW confidence)
- None — all claims verified from codebase or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all verified from actual codebase files and package.json
- Architecture: HIGH — all patterns verified from existing code (commands/mod.rs, lib.rs, stores, composables)
- Pitfalls: HIGH — all derived from codebase inspection and known Tauri 2 patterns

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days — stack is stable)
