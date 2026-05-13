# Requirements: NarratoAI v2.0

**Defined:** 2026-05-12
**Core Value:** 高性能 Rust 后端驱动完整视频自动化生产流水线，前端模仿 SmartEdit 布局和操作逻辑

## v2.0 Requirements

Requirements for v2.0: Tauri + Vue 3 Frontend + v1.0 Tech Debt Cleanup.

### Foundation (FNDT)

- [ ] **FNDT-01**: 项目初始化 — Vite 8 + Vue 3.5 + TypeScript 6.0，集成 Tauri 2.0
- [ ] **FNDT-02**: Vuetify 3 插件安装，@mdi/font 图标集成，zhHans 本地化
- [ ] **FNDT-03**: vue-router 5 Hash 路由配置（createWebHashHistory），单页架构
- [ ] **FNDT-04**: Pinia 3 状态管理 — 9 个 store（app/mode/video/pipeline/script/llm/tts/bgm）
- [ ] **FNDT-05**: API 层 — tauriInvoke() 封装（BigInt 序列化、敏感字段过滤、调试日志）
- [ ] **FNDT-06**: ts-rs 自动生成 TypeScript 类型，与 Rust 后端类型同步

### Layout (LOUT)

- [ ] **LOUT-01**: AppHeader 自定义标题栏（Logo + "NarratoAI" + 设置齿轮 + 主题切换 + 窗口控制按钮）
- [ ] **LOUT-02**: DefaultLayout 三区域布局（AppHeader + v-main + SettingsDrawer）
- [ ] **LOUT-03**: HomeView 上下分区布局（上部：VideoTable | 设置卡片，下部：操作按钮行 + LogPanel）
- [ ] **LOUT-04**: SettingsDrawer 右侧临时抽屉（420px），包含 LLM 双模型配置
- [ ] **LOUT-05**: 深色/浅色主题切换，localStorage 持久化
- [ ] **LOUT-06**: SystemMonitorBar — CPU/RAM 实时使用率监控条

### Mode Management (MODE)

- [ ] **MODE-01**: ModeStore — 当前工作模式状态管理，模式切换时重置 pipeline 状态
- [ ] **MODE-02**: ModeSelector — 设置卡片内下拉切换（纪录片解说 / 短剧解说 / 短剧混剪）
- [ ] **MODE-03**: 设置面板动态可见性 — Documentary 显示 Vision LLM，SDP 隐藏 TTS 面板

### Configuration Panels (CONF)

- [ ] **CONF-01**: LLM/Mode 面板 — 双模型配置（Vision + Text），Provider/Model/API Key/Base URL
- [ ] **CONF-02**: TTS 面板 — 7 引擎选择 + 各引擎参数配置，Edge-TTS voice 列表加载
- [ ] **CONF-03**: BGM 面板 — BGM 文件夹选择、随机/指定模式切换
- [ ] **CONF-04**: Export 面板 — 输出目录选择、导出格式选项
- [ ] **CONF-05**: 模式专用面板 — Documentary（帧间隔/视觉批处理）、SDE（剧名/温度）、SDP（自定义片段数）
- [ ] **CONF-06**: Local Draft 模式 — 未保存编辑暂存，仅显式保存或启动时持久化到后端

### Pipeline UI (PIPE)

- [ ] **PIPE-01**: VideoTable — 视频列表数据表（名称/主题/字幕/状态/操作列）
- [ ] **PIPE-02**: 操作按钮行 — Upload / Start / Stop / Clear / 输出目录
- [ ] **PIPE-03**: 模式感知 Start — 根据当前模式分发到正确的 Tauri 命令（run_documentary / run_sde / run_sdp）
- [ ] **PIPE-04**: 流水线进度事件流 — 按步骤显示进度（6/9/3 步），兼容 ProgressPayload
- [ ] **PIPE-05**: ScriptPreviewDialog — 脚本预览弹窗，按模式不同渲染内容
- [ ] **PIPE-06**: LogPanel — 滚动日志查看器，支持导出和清空

### Tech Debt (TDET)

- [ ] **TDET-01**: Edge-TTS 代理隧道实现（HTTP CONNECT + 平台代理自动检测）
- [ ] **TDET-02**: Tauri State 读锁优化（extract before .await，缩短锁持有时间）
- [ ] **TDET-03**: 集成测试桩替换为真实测试（assert!(true) → 实际断言）
- [ ] **TDET-04**: YouTube/Pexels Tauri 命令暴露 + 前端 UI 集成
- [ ] **TDET-05**: ConfigManager 热加载激活（TOML 文件监视器 + 前端通知）
- [ ] **TDET-06**: VisualAnalyzer 门面调用补齐（统一 API 入口）
- [x] **TDET-07**: FFmpeg Tauri sidecar 内置打包（externalBin + shell:allow-execute 权限）

## Out of Scope

| Feature | Reason |
|---------|--------|
| FFmpeg 管理面板 | 打包时内置，不需要用户管理 FFmpeg |
| Streamlit Web UI | 已被 Tauri + Vue 3 替代 |
| Redis 状态后端 | 桌面单用户不需要分布式状态 |
| Docker 部署 | 桌面应用打包为原生安装包 |
| 多用户/并发任务管理 | 桌面单用户场景 |
| FFmpeg 自定义滤镜图 UI | 复杂度高，非核心需求 |
| 多语言 UI | v2.0 仅简体中文 |
| 视频实时预览播放器 | Tauri 原生播放器复杂度高，延后 |
| 内嵌字幕编辑器 | 延后到 v2.1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FNDT-01 | Phase 14 | Pending |
| FNDT-02 | Phase 14 | Pending |
| FNDT-03 | Phase 14 | Pending |
| FNDT-04 | Phase 14 | Pending |
| FNDT-05 | Phase 14 | Pending |
| FNDT-06 | Phase 14 | Pending |
| LOUT-01 | Phase 15 | Pending |
| LOUT-02 | Phase 15 | Pending |
| LOUT-03 | Phase 15 | Pending |
| LOUT-04 | Phase 15 | Pending |
| LOUT-05 | Phase 15 | Pending |
| LOUT-06 | Phase 15 | Pending |
| MODE-01 | Phase 15 | Pending |
| MODE-02 | Phase 15 | Pending |
| MODE-03 | Phase 15 | Pending |
| CONF-01 | Phase 16 | Pending |
| CONF-02 | Phase 16 | Pending |
| CONF-03 | Phase 16 | Pending |
| CONF-04 | Phase 16 | Pending |
| CONF-05 | Phase 16 | Pending |
| CONF-06 | Phase 16 | Pending |
| PIPE-01 | Phase 17 | Pending |
| PIPE-02 | Phase 17 | Pending |
| PIPE-03 | Phase 17 | Pending |
| PIPE-04 | Phase 17 | Pending |
| PIPE-05 | Phase 17 | Pending |
| PIPE-06 | Phase 17 | Pending |
| TDET-01 | Phase 16 | Pending |
| TDET-02 | Phase 18 | Pending |
| TDET-03 | Phase 14 | Pending |
| TDET-04 | Phase 18 | Pending |
| TDET-05 | Phase 18 | Pending |
| TDET-06 | Phase 16 | Pending |
| TDET-07 | Phase 16 | Complete |

**Coverage:**
- v2.0 requirements: 34 total
- Mapped to phases: 34 (roadmap created)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-12*
*Last updated: 2026-05-12 — traceability updated with Phase 14-18 mappings*
