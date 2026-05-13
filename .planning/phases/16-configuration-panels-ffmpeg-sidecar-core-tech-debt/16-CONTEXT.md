# Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

所有设置面板可用（LLM 双模型 + TTS + BGM + Export + 模式专用参数），Local Draft 暂存模式，FFmpeg Tauri sidecar 内置打包，Edge-TTS 代理隧道实现，VisualAnalyzer 门面调用补齐。

**覆盖需求:** CONF-01, CONF-02, CONF-03, CONF-04, CONF-05, CONF-06, TDET-01, TDET-06, TDET-07 + MODE-03 (Phase 15 延期)

**依赖:** Phase 15（SettingsDrawer + SettingSection + ModeStore 已就绪）
</domain>

<decisions>
## Implementation Decisions

### SettingsDrawer 面板组织
- **D-01:** 手风琴式布局 — 每个配置面板使用 Phase 15 已实现的 `SettingSection` collapsible 组件，复用 v-expand-transition 动画。
- **D-02:** 完整显隐规则 — Documentary: LLM Vision + LLM Text + TTS + BGM + Export + 帧参数。SDE: LLM Text + TTS + BGM + Export + 剧名/温度。SDP: LLM Text + BGM + Export + 片段数。Vision LLM 仅 Documentary 可见，TTS 在 SDP 隐藏。
- **D-03:** 面板排列顺序: LLM Vision → LLM Text → TTS → BGM → Export → 网络代理 → 模式专用参数。
- **D-04:** Footer 包含全局 v-btn "保存设置"（primary elevated）+ 版本号保留。手风琴默认仅展开第一个面板（LLM Vision），其余折叠。面板折叠状态通过 localStorage('settingsPanelStates') 跨会话记忆，不限制同时展开数量。
- **D-05:** 面板显隐过渡使用 v-expand-transition（SettingSection 已有机制），全局保存成功后显示 v-snackbar "设置已保存"（绿色，3 秒自动消失），失败显示红色 snackbar。
- **D-06:** 每个面板字段默认值从后端 config.toml 通过 get_config() Tauri 命令加载。每个面板内提供"重置"按钮恢复该面板默认值，Footer 提供"重置全部"链接文字（弹出确认对话框后清除草稿 + 重置所有 Pinia stores）。
- **D-07:** 面板图标 — LLM Vision: mdi-robot, LLM Text: mdi-chat-processing, TTS: mdi-text-to-speech, BGM: mdi-music-note, Export: mdi-file-export, 模式专用参数: mdi-tune, 网络代理: mdi-earth。
- **D-08:** API Key/Token/Secret 字段使用 type="password" + 显示/隐藏切换按钮。模式切换时检测未保存更改：弹出确认对话框"未保存的更改将丢失，是否继续？"，用户确认后才切换。

### LLM 双模型配置 UX (CONF-01)
- **D-09:** LLM Vision 和 LLM Text 各自独立 SettingSection（非内部分组）。Vision 面板按 D-02 显隐规则在非 Documentary 模式下隐藏。
- **D-10:** 每个 LLM 面板内 4 字段垂直排列（全宽 v-text-field）: Provider（v-select 下拉）、Model（v-text-field）、API Key（type="password" + 脱敏显示）、Base URL（v-text-field + placeholder 提示"留空使用默认"）。
- **D-11:** Provider 下拉预设 — OpenAI / DeepSeek / Gemini / Qwen / SiliconFlow / Moonshot / Anthropic / Cohere / Together AI / OpenRouter / 自定义。选择 Provider 后 Model 字段下方显示灰色小字建议（如"推荐: gpt-4o"），不自动填充。
- **D-12:** 每个 LLM 面板底部添加 v-btn "测试连接"（outlined），点击后调用后端 Tauri 命令 test_llm_connection，成功绿色 snackbar，失败红色 + 错误详情。
- **D-13:** LLM 面板加载时 SettingSection 显示 loading 骨架。API Key 从后端获取后脱敏显示（`sk-...****...xyz` 格式），点击显示按钮完整展示。

### TTS 面板 (CONF-02)
- **D-14:** 顶部 v-select 下拉选择 TTS 引擎（中文标签：Edge-TTS / Azure / 腾讯云 / SoulVoice / 阿里通义 / IndexTTS2 / 豆包），下方动态渲染当前引擎的专属配置字段。切换引擎时已填参数保留在内存中不丢失。
- **D-15:** Edge-TTS voice 列表通过 Tauri 命令 get_edge_tts_voices 从后端加载（Rust 侧调用 Edge-TTS voice list API）。前端使用 v-autocomplete 带搜索过滤。加载失败显示错误提示 + 重试按钮。各引擎专属参数按 v1.0 config.toml 对应段设计 — Azure（订阅密钥+区域）、腾讯（SecretId+SecretKey）、SoulVoice（API Key）、通义（API Key+模型名）、IndexTTS2（API Key）、豆包（API Key+音色ID）。

### BGM / Export / 模式专用参数面板 (CONF-03/04/05)
- **D-16:** BGM 文件夹和 Export 输出目录使用 Tauri dialog.open() 原生文件夹选择器，路径 v-text-field 只读 + 右侧"浏览"按钮。Export 格式: MP4/MKV/MOV v-select 下拉，默认 MP4。
- **D-17:** BGM 面板内 v-switch 切换"随机"（从文件夹随机选择）/ "指定"（v-select 列出文件夹内音频文件）。模式专用参数面板标题和内容随 ModeStore.currentMode 动态变化：Documentary → "帧参数"（帧间隔 v-text-field + 视觉批处理大小 v-text-field），SDE → "短剧参数"（剧名 v-text-field + 温度 v-slider 0~2），SDP → "混剪参数"（自定义片段数 v-text-field + 最短/最长时长）。

### 配置验证与帮助
- **D-18:** 软验证策略 — 字段不阻塞输入（允许留空）。点击全局保存时后端验证返回缺失/无效字段列表，前端 snackbar 提示"N 项配置不完整"，对应 SettingSection 标题旁显示红色小圆点标记。启动流水线时后端再次校验。
- **D-19:** 每个 v-text-field 使用 placeholder 显示示例值。复杂字段旁加 v-icon mdi-help-circle-outline + v-tooltip 悬浮说明。字段标签和错误信息使用简体中文。

### Local Draft 暂存策略 (CONF-06)
- **D-20:** 暂存触发 — SettingsDrawer 关闭时（关闭按钮/scrim/齿轮按钮 toggle）自动将所有当前编辑暂存到 localStorage('settingsDraft')。应用启动时静默恢复草稿到 Pinia stores。点击全局保存按钮成功（后端确认）后才清除 localStorage 草稿。
- **D-21:** 视觉提示 — 有草稿时 Footer 保存按钮旁显示橙色文字"未保存的更改"，保存按钮变为 elevated + primary 颜色。localStorage 键名: 'settingsDraft'，值 JSON 结构: `{ llmVision: {...}, llmText: {...}, tts: {...}, bgm: {...}, export: {...}, modeParams: {...}, networkProxy: {...}, updatedAt: ISO }`。
- **D-22:** 乐观草稿策略 — 草稿始终覆盖显示。模式切换时草稿保留不丢失（单一 'settingsDraft' 不按模式分离）。保存时后端对比 config.toml 修改时间戳，如被外部修改则返回冲突错误 + 前端提示"配置文件已被外部修改，是否覆盖？"。
- **D-23:** 草稿仅在全局保存成功后清除。面板级"重置"按钮不修改草稿（重置仅作用于 Pinia 状态，草稿下次暂存时更新）。全局重置链接确认后清除草稿 + 重置全部 Pinia stores。
- **D-24:** 草稿在 localStorage 中明文存储（桌面单用户环境安全风险低）。暂存失败（localStorage 满/禁用）静默降级不弹窗。恢复失败静默跳过、使用后端配置。

### Edge-TTS 代理隧道 (TDET-01)
- **D-25:** 平台代理自动检测 — Rust 侧检测 HTTPS_PROXY / HTTP_PROXY / ALL_PROXY 环境变量，config.toml [proxy] 段作为手动覆盖。不实现 Windows/macOS 原生系统代理 API 检测。
- **D-26:** 代理隧道覆盖范围 — WebSocket CONNECT（现有 edge_tts.rs:269-420 已实现）+ 时钟校准 HTTP（correct_clock_skew，当前未使用代理）+ voice 列表 HTTP。新增 get_system_proxy Tauri 命令返回当前生效的代理配置。前端 SettingsDrawer 添加"网络代理" SettingSection（mdi-earth，HTTP 代理地址 + HTTPS 代理地址 v-text-field + 启用 v-switch，placeholder "留空使用环境变量"）。
- **D-27:** 连接降级策略 — 优先直连（Edge-TTS WebSocket 和 HTTP 请求），直连失败后自动尝试代理连接。代理操作超时 30s。失败用 tracing::warn 记录（不弹窗），流水线运行时 TTS 合成失败通过现有错误机制反馈。

### VisualAnalyzer 门面补齐 (TDET-06)
- **D-28:** 审查 Documentary/SDE pipeline 中所有视觉分析调用路径，确保统一走 `narratoai-core/src/visual/analyzer.rs` 的 `analyze_video_frames()` 门面。确保 prompt template 从 PromptRegistry 加载而非硬编码。不新增功能，不新增 Tauri 命令。

### FFmpeg Tauri Sidecar 打包 (TDET-07)
- **D-29:** 手动嵌入 FFmpeg 7.1 静态构建二进制文件，使用 git-lfs 管理。目录结构: `src-tauri/binaries/ffmpeg-{platform}/ffmpeg{ext}`, `ffprobe{ext}`（platform = win32-x64 / darwin-arm64 / linux-x64）。
- **D-30:** tauri.conf.json 添加 externalBin 声明各平台 FFmpeg 路径。添加 shell plugin + allow-execute scope。Rust 侧使用 `tauri::process::Command::new_sidecar('ffmpeg')` 自动定位 externalBin 路径。

### 配置数据流与 Tauri 命令
- **D-31:** 统一 get_config() / save_config() 两个 Tauri 命令。get_config 返回完整脱敏 AppConfig JSON（API Key 脱敏为 `sk-...****...xyz`），前端解析后分发到各 Pinia store。save_config 接收增量更新（仅发送用户修改的字段），后端合并写入 config.toml（未发送字段保持原值，空字符串表示"清除"）。
- **D-32:** 应用启动时（App.vue onMounted）预加载 get_config() 到 Pinia stores。SettingsDrawer 打开时直接从 Pinia 读取即时渲染。加载中各面板显示 SettingSection loading 骨架。加载失败 v-snackbar "配置加载失败" + 重试按钮，使用 config.toml 默认值继续运行。
- **D-33:** 新增 5 个 Tauri 命令: get_config / save_config / test_llm_connection / get_edge_tts_voices / get_system_proxy。全部放在 `src-tauri/src/commands/config.rs` 新文件，在 `commands/mod.rs` 注册。test_llm_connection 发送简单请求验证 API Key/Base URL，get_edge_tts_voices 调用 Edge-TTS voice list API（须走代理），get_system_proxy 读取环境变量和 config.toml 合并后的代理配置。

### 实现优先级与计划拆分
- **D-34:** 按依赖关系分 3 波:
  - **Wave 0 (后端基础):** FFmpeg sidecar (TDET-07) + 5 个新 Tauri 命令 (config.rs) + Edge-TTS 代理后端（时钟校准/voice 列表代理 + 环境变量检测）+ VisualAnalyzer 审查 (TDET-06)。
  - **Wave 1 (前端面板):** SettingsDrawer 手风琴重构 + LLM Vision/Text 面板 (CONF-01) + TTS 面板 (CONF-02) + BGM 面板 (CONF-03) + Export 面板 (CONF-04) + 模式专用参数面板 (CONF-05) + 网络代理面板。面板和 Pinia stores 与后端命令接驳。
  - **Wave 2 (整合):** Local Draft 暂存 (CONF-06) + 全局保存/验证/snackbar + 模式切换面板显隐 (MODE-03) + 配置预加载时序 + 配置面板 vitest 测试。

### 测试策略
- **D-35:** 前端 vitest 真实测试 — SettingSection 组件（字段渲染、保存/重置、模式切换显隐）、Pinia stores（配置加载/保存/脱敏逻辑）。Edge-TTS 代理隧道 — Rust 侧单元测试（环境变量解析、CONNECT 请求构造）+ 集成测试标记 #[ignore]（需真实代理环境）。FFmpeg sidecar — smoke test 调用 ffmpeg -version 验证 externalBin 正确。

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
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 15 产出（SettingsDrawer + SettingSection + ModeStore 已就绪）
- `.planning/phases/15-layout-shell-3-mode-ux/15-CONTEXT.md` — Phase 15 全部决策（AppHeader/DefaultLayout/SettingsDrawer/SettingSection/SystemMonitorBar/ModeStore/HomeView）
- `.planning/phases/15-layout-shell-3-mode-ux/15-VERIFICATION.md` — MODE-03 延期到 Phase 16（面板动态可见性）
- `src/components/SettingsDrawer.vue` — 420px 右侧抽屉，三段式结构（Header/Content/Footer），自定义滚动条
- `src/components/SettingSection.vue` — 可复用设置分组卡片组件（collapsible + icon + title + slot + variant）

### Phase 14 产出（Pinia Stores + API 层 + 类型同步）
- `.planning/phases/14-foundation-frontend-scaffold/14-CONTEXT.md` — 目录结构、Store 粒度、tauriInvoke 封装、ts-rs 类型同步
- `.planning/phases/14-foundation-frontend-scaffold/14-UI-SPEC.md` — 设计体系（spacing、typography、颜色）
- `src/stores/llm.ts` — LLM Store 骨架（双模型配置）
- `src/stores/tts.ts` — TTS Store 骨架（7 引擎选择）
- `src/stores/bgm.ts` — BGM Store 骨架（文件夹路径 + 随机/指定）
- `src/stores/export.ts` — Export Store 骨架（输出目录 + 格式）
- `src/composables/useTauri.ts` — tauriInvoke 封装（BigInt 序列化、敏感字段过滤、调试日志）
- `src/plugins/vuetify.ts` — Vuetify 4 双主题配色、zhHans 本地化、@mdi/font 图标

### SmartEdit 参考项目
- `E:/GitLib/SmartEdit/src/components/SettingsDrawer.vue` — 设置抽屉三段式结构和 SettingSection 用法参考
- `E:/GitLib/SmartEdit/src/components/SettingSection.vue` — 可折叠设置卡片参考

### Rust 后端（现有实现需扩展）
- `narratoai-core/src/tts/edge_tts.rs` — Edge-TTS 引擎（WebSocket CONNECT 代理隧道已实现于 connect()，时钟校准 correct_clock_skew 缺代理）
- `narratoai-core/src/tts/common.rs` — ProxyConfig 结构体（from_proxy + apply_to_client）
- `narratoai-core/src/visual/analyzer.rs` — VisualAnalyzer 编排器（analyze_video_frames 主入口）
- `narratoai-core/src/config/types.rs` — AppConfig / ProxySection / AppSection 完整类型定义
- `src-tauri/tauri.conf.json` — 当前 Tauri 配置（需添加 externalBin + shell plugin）
- `src-tauri/src/commands/` — 现有 15 个 Tauri 命令目录（需新增 config.rs）
- `src-tauri/capabilities/default.json` — 权限声明（需添加 shell:allow-execute）

### 项目规划文档
- `.planning/ROADMAP.md` — Phase 16 定义、成功标准、需求映射
- `.planning/REQUIREMENTS.md` — CONF-01~06、TDET-01/06/07 详细要求
- `.planning/PROJECT.md` — 项目约束、技术栈、已知技术债
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets（Phase 14 + 15 已产出）
- `src/components/SettingSection.vue` — 可折叠卡片组件（props: title/icon/collapsible/defaultExpanded/variant/loading），Phase 16 配置面板统一使用
- `src/components/SettingsDrawer.vue` — 420px 三段式抽屉（Header/Content/Footer），需扩展 Content 为多个 SettingSection + Footer 添加全局保存按钮
- `src/composables/useTauri.ts` — tauriInvoke 封装，配置面板调用后端命令用
- `src/stores/llm.ts` / `src/stores/tts.ts` / `src/stores/bgm.ts` / `src/stores/export.ts` — Store 骨架需扩展为完整字段和 actions
- `src/stores/mode.ts` — ModeStore 已实现 localStorage 持久化和 resetPipeline，面板显隐逻辑读取 currentMode
- `src/stores/app.ts` — 主题持久化参考模式（localStorage + Pinia watch），Local Draft 可借鉴

### Established Patterns
- **SettingSection 用法** — Phase 15 SettingsDrawer.vue 中的外观 SettingSection 使用模式：`<SettingSection icon="..." title="..." collapsible><template #default>...</template></SettingSection>`，Phase 16 面板遵循相同模式
- **动态 import Tauri API** — 所有 @tauri-apps/api 调用使用动态 import + try/catch（浏览器开发模式降级），Tauri 命令调用通过 useTauri composable
- **localStorage 持久化** — AppStore 和 ModeStore 已实现 localStorage 读写模式，Local Draft 和面板折叠状态沿用相同机制
- **Composition API stores** — Pinia stores 使用 `defineStore('name', () => { ... })` 风格

### Integration Points
- **SettingsDrawer.vue** → 扩展 Content 区域，插入多个 SettingSection（LLM Vision/Text、TTS、BGM、Export、网络代理、模式参数）
- **SettingsDrawer Footer** → 当前仅版本号，新增全局保存按钮 + "未保存的更改" 状态文字 + 重置链接
- **DefaultLayout.vue** → 当前管理 showSettings，无需修改（SettingsDrawer 内部变更不影响外层）
- **ModeStore** → 面板显隐逻辑读取 currentMode computed，监听模式变更触发面板重新计算
- **App.vue** → onMounted 添加 get_config() 预加载调用
- **Rust 命令层** → 新增 config.rs（5 个命令），mod.rs 注册

### 新建/修改文件清单
| 文件 | 用途 |
|------|------|
| `src/components/config/LlmVisionPanel.vue` | LLM Vision 模型配置面板 |
| `src/components/config/LlmTextPanel.vue` | LLM Text 模型配置面板 |
| `src/components/config/TtsPanel.vue` | TTS 引擎选择与参数配置面板 |
| `src/components/config/BgmPanel.vue` | BGM 文件夹选择与模式切换面板 |
| `src/components/config/ExportPanel.vue` | 输出目录与格式选项面板 |
| `src/components/config/ModeParamsPanel.vue` | 模式专用参数动态面板 |
| `src/components/config/NetworkProxyPanel.vue` | 网络代理设置面板 |
| `src/stores/llm.ts` | 扩展：完整字段 + 加载/保存/重置/测试连接 actions |
| `src/stores/tts.ts` | 扩展：引擎选择 + voice 列表 + 各引擎参数 + 加载/保存 actions |
| `src/stores/bgm.ts` | 扩展：路径 + 随机/指定 + 文件列表 + 加载/保存 actions |
| `src/stores/export.ts` | 扩展：路径 + 格式 + 加载/保存 actions |
| `src/composables/useConfig.ts` | 新增：get_config()/save_config() 封装 + draft 管理 |
| `src-tauri/src/commands/config.rs` | 新增：5 个 Tauri 命令 |
| `src-tauri/tauri.conf.json` | 修改：externalBin + shell plugin |
| `src-tauri/capabilities/default.json` | 修改：shell:allow-execute 权限 |
| `src-tauri/binaries/` | 新增：FFmpeg 平台二进制文件 (git-lfs) |
| `src/components/SettingsDrawer.vue` | 修改：Content 手风琴重构 + Footer 扩展 |
</code_context>

<specifics>
## Specific Ideas

- SettingsDrawer Content 使用 `<v-expansion-panels>` 或手动管理多个 SettingSection 的折叠状态（通过 localStorage('settingsPanelStates') 持久化），不强制手风琴模式（允许多个同时展开）。
- 模式切换面板显隐使用 `v-if`（非 v-show），避免隐藏面板仍占用 DOM 和响应式开销。
- save_config 的增量语义：前端维护一个 dirtyFields Set，仅发送变更的字段。后端接收后与现有 config.toml 合并写入，保持 TOML 格式和注释。
- get_config 返回时对 API Key / Secret 字段统一脱敏处理在后端完成（Rust 侧字符串截取前3后3字符），前端无需单独实现脱敏逻辑。
- Edge-TTS voice 列表加载时机：SettingsDrawer 首次打开时（懒加载），缓存到 Pinia tts store，避免每次打开都重新请求。
- FFmpeg sidecar 的 externalBin 路径在开发模式（`cargo tauri dev`）下 Tauri 自动解析到 `src-tauri/binaries/` 目录，生产构建时嵌入 bundle。
- 网络代理面板 "启用" 开关关闭时仍显示代理地址字段（灰化），方便用户临时关闭而不丢失配置。
- SettingSection loading 状态在首次加载时显示，后续切换面板不再显示 loading（使用 Pinia 缓存判断）。
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

### Reviewed Todos (not folded)
- **Fix Tauri State read locks held across long pipelines** — TDET-02，Phase 18 范围，不在本 Phase 处理。
- **Replace integration test stubs using assert!(true)** — TDET-03，Phase 14 已完成，不适用于 Phase 16。
</deferred>

---

*Phase: 16-configuration-panels-ffmpeg-sidecar-core-tech-debt*
*Context gathered: 2026-05-13*
