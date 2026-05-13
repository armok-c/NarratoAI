# Phase 16: Configuration Panels + FFmpeg Sidecar + Core Tech Debt - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
**Areas discussed:** SettingsDrawer 面板组织, LLM 双模型配置 UX, TTS 面板, BGM/Export/模式参数, 配置验证与帮助, Local Draft 暂存策略, Edge-TTS 代理隧道架构, VisualAnalyzer 门面, FFmpeg Sidecar 打包, 配置数据流与 Tauri 命令, 实现优先级与计划拆分, 测试策略

---

## SettingsDrawer 面板组织

| Option | Description | Selected |
|--------|-------------|----------|
| 手风琴式 | 每个面板一个 collapsible SettingSection，复用 Phase 15 组件 | ✓ |
| v-tabs 标签页 | 顶部 v-tabs 分组切换，每组一个配置页 | |
| 垂直滚动列表 | 全部平铺，v-divider 分隔 | |

### 面板可见性 (MODE-03)

| Option | Description | Selected |
|--------|-------------|----------|
| 完整显隐规则 | Documentary: LLM Vision+Text+TTS+BGM+Export+帧参数; SDE: LLM Text+TTS+BGM+Export+剧名/温度; SDP: LLM Text+BGM+Export+片段数 | ✓ |
| 所有面板全模式可见 | 所有面板始终可见，用户自行选择 | |
| 仅隐藏不适用的字段 | 面板可见但字段灰化 | |

### 面板排列顺序

| Option | Description | Selected |
|--------|-------------|----------|
| LLM → TTS → BGM → Export → 模式参数 | 按配置依赖链排序 | ✓ |
| 按优先级 | 模式参数 → LLM → TTS → BGM → Export | |
| 字母/功能分组 | 通用配置在前，模式参数在最后 | |

### SettingsDrawer Footer + 默认展开

| Option | Description | Selected |
|--------|-------------|----------|
| 全局保存按钮 | Footer 添加保存按钮，默认仅展开第一个面板，版本号保留 | ✓ |
| 仅版本号 + 自动保存 | 不变，debounced 自动暂存，全部默认展开 | |
| 保存按钮 + 全部展开 | Footer 保存按钮，所有面板默认展开 | |

### 面板交互

| Option | Description | Selected |
|--------|-------------|----------|
| v-expand-transition + snackbar | 过渡动画 + 绿色 snackbar "设置已保存" 3s | ✓ |
| 即时切换 + 静默保存 | 无动画，无提示 | |
| 淡入淡出 + 底部状态 | fade 过渡 + Footer 状态文字 | |

### 面板重置

| Option | Description | Selected |
|--------|-------------|----------|
| config.toml 驱动默认值 | 后端加载默认值，面板级重置按钮 + 全局重置链接 | ✓ |
| 前端硬编码 | 前端 Store 中硬编码默认值 | |
| 无重置功能 | 不提供重置 | |

### 面板图标

| Option | Description | Selected |
|--------|-------------|----------|
| 具体图标分配 | LLM Vision=mdi-robot, LLM Text=mdi-chat-processing, TTS=mdi-text-to-speech, BGM=mdi-music-note, Export=mdi-file-export, 模式参数=mdi-tune | ✓ |
| 统一样式 | 全部 mdi-cog/mdi-tune | |
| 颜色编码 | 图标 + 颜色小竖条 | |

### 敏感字段 + 模式切换

| Option | Description | Selected |
|--------|-------------|----------|
| 密码遮罩 + 确认丢弃 | type="password" + 显示/隐藏切换，模式切换弹出确认对话框 | ✓ |
| 明文 + 自动暂存 | 明文显示，自动暂存 | |
| 密码遮罩 + 自动暂存 | 遮罩 + 静默暂存 | |

### 折叠记忆

| Option | Description | Selected |
|--------|-------------|----------|
| localStorage 记忆 + 无限制 | 折叠状态持久化，不限制同时展开数 | ✓ |
| 不记忆 + 手风琴 | 仅一个展开 | |
| 会话记忆 + 无限制 | 仅当前会话 | |

---

## LLM 双模型配置 UX (CONF-01)

| Option | Description | Selected |
|--------|-------------|----------|
| 两个独立 SettingSection | Vision + Text 各自独立面板 | ✓ |
| 一个 SettingSection + 内部分组 | 内部 v-card 分组 | |
| 一个 SettingSection + 下拉切换 | 顶部 v-select 切换 | |

### LLM 字段布局

| Option | Description | Selected |
|--------|-------------|----------|
| 垂直排列 + Provider 下拉 | 4 字段垂直全宽，Provider v-select 预设 | ✓ |
| 两列网格 + 自由输入 | 一行两列，全部自由输入 | |
| 垂直排列 + 自动填充 Base URL | Provider 选择后自动填充默认 Base URL | |

### Provider 预设 + 测试连接

| Option | Description | Selected |
|--------|-------------|----------|
| 全量预设 + 自动建议 | OpenAI/DeepSeek/Gemini/Qwen/SiliconFlow/Moonshot/Anthropic/Cohere/Together AI/OpenRouter/自定义 + Model 建议提示 + 测试连接按钮 | ✓ |
| 仅常用 + 无建议 | OpenAI/DeepSeek/Gemini/自定义 | |
| 从后端加载 + 自动填充 | 后端加载 Provider 列表 + 自动填充 Model | |

### LLM 面板状态

| Option | Description | Selected |
|--------|-------------|----------|
| 加载骨架 + 脱敏 | SettingSection loading + API Key sk-...****...xyz | ✓ |
| 无加载态 + 完整显示 | 即时渲染，完整显示 | |
| 加载遮罩 + 默认隐藏 | 加载条 + 完全隐藏 | |

---

## TTS 面板 (CONF-02)

| Option | Description | Selected |
|--------|-------------|----------|
| v-select + 动态参数 | 下拉引擎选择 + 动态专属参数 + 保留已填参数 | ✓ |
| v-radio-group 单选 | 垂直排列 radio 列表 | |
| v-select + 子 SettingSection | 嵌套折叠卡片 | |

### Edge-TTS voice 列表

| Option | Description | Selected |
|--------|-------------|----------|
| 后端加载 + v-autocomplete | Tauri 命令加载 + 搜索过滤 + 失败重试 + 其他引擎按 v1.0 字段 | ✓ |
| 前端加载 + 手动输入 | 前端直接调用 API | |
| 硬编码预设 | 前端硬编码 | |

---

## BGM/Export/模式参数 (CONF-03/04/05)

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri 原生对话框 | dialog.open() + 只读路径 + 浏览按钮 + Export MP4/MKV/MOV | ✓ |
| 手动输入 | v-text-field 手动输入路径 | |
| 多格式 | 全格式 MP4/MKV/MOV/AVI/WEBM | |

### BGM 模式 + 模式专用参数

| Option | Description | Selected |
|--------|-------------|----------|
| v-switch + 动态面板 | 随机/指定 v-switch + 模式参数随 ModeStore 变化 | ✓ |
| v-select + 独立 SettingSection | 下拉选择 + 独立面板 | |
| 双按钮 + 参数分散 | 按钮切换 + 参数分散到其他面板 | |

---

## 配置验证与帮助

| Option | Description | Selected |
|--------|-------------|----------|
| 软验证 + 保存时汇总 | 不阻塞输入，后端验证 + 标题红点标记 | ✓ |
| 前端硬验证 | 实时 Vuetify rules 校验 | |
| 无验证 | 不验证 | |

### 字段帮助

| Option | Description | Selected |
|--------|-------------|----------|
| placeholder + 悬浮提示 | placeholder 示例 + mdi-help-circle-outline v-tooltip | ✓ |
| 仅 placeholder | 弱提示 | |
| 底部帮助卡片 | caption 样式说明 | |

---

## Local Draft 暂存策略 (CONF-06)

| Option | Description | Selected |
|--------|-------------|----------|
| 抽屉关闭暂存 + 启动静默恢复 | 关闭时 localStorage + 启动时静默恢复 + 全局保存后清除 | ✓ |
| 实时 debounced + 询问 | 500ms debounced + 启动时对话框 | |
| 仅内存 + 不跨会话 | Pinia 内存，重启丢失 | |

### 草稿标记

| Option | Description | Selected |
|--------|-------------|----------|
| Footer 状态指示 + 分面板存储 | "未保存的更改"橙色文字 + 保存按钮 elevated + JSON 结构 | ✓ |
| 仅按钮变化 | disabled → enabled + "*" 标记 | |
| 面板级标记 + 扁平存储 | 每个面板标题橙色圆点 | |

### 草稿冲突

| Option | Description | Selected |
|--------|-------------|----------|
| 乐观草稿 + 模式独立 | 覆盖显示 + 跨模式保留 + 保存时时间戳冲突检测 | ✓ |
| 保守策略 + 按模式 | 检查 TOML 时间 + 按模式分离草稿 | |
| 无冲突检测 | 不检测 | |

### 草稿清理

| Option | Description | Selected |
|--------|-------------|----------|
| 保存清除 + 重置保留 | 仅保存成功后清除 + 面板重置不修改草稿 + 全局重置确认后清除 | ✓ |
| 保存和重置都清除 | 两者都清除 | |
| 仅手动清除 | 永不清除 | |

### 草稿安全

| Option | Description | Selected |
|--------|-------------|----------|
| 明文存储 + 静默降级 | localStorage 明文 + 暂存/恢复失败静默降级 | ✓ |
| Base64 编码 + 警告 | 编码混淆 + 失败 snackbar | |
| 不存储敏感字段 | 草稿不含 API Key | |

---

## Edge-TTS 代理隧道 (TDET-01)

| Option | Description | Selected |
|--------|-------------|----------|
| 环境变量 + 手动配置 | HTTPS_PROXY/HTTP_PROXY/ALL_PROXY + config.toml 覆盖 + 前端面板 | ✓ |
| 全平台自动检测 | Windows/macOS/Linux 原生系统代理 API | |
| 仅 config.toml | 无自动检测，无前端面板 | |

### 代理覆盖范围

| Option | Description | Selected |
|--------|-------------|----------|
| 全部覆盖 + 简化面板 | WebSocket CONNECT/时钟校准/voice 列表 + HTTP/HTTPS 地址 + 启用开关 | ✓ |
| 仅 WebSocket | 不扩展 | |
| 全部覆盖 + 详细面板 | 含测试代理按钮 + 认证字段 | |

### 代理降级

| Option | Description | Selected |
|--------|-------------|----------|
| 先直连后代理 + 30s | 优先直连 + 失败自动代理 + 30s 超时 + tracing::warn | ✓ |
| 仅代理 | 启用则强行走代理 | |
| 并行竞速 | 同时直连和代理 | |

---

## VisualAnalyzer 门面 (TDET-06)

| Option | Description | Selected |
|--------|-------------|----------|
| 检查调用方 + 缺失功能 | 审查 pipeline 调用路径 + PromptRegistry 加载 | ✓ |
| 仅代码审查 | 仅审查不做改动 | |
| 重构为新 API | 新 VisualAnalyzer 结构体 | |

---

## FFmpeg Sidecar 打包 (TDET-07)

| Option | Description | Selected |
|--------|-------------|----------|
| 手动嵌入 + 固定版本 | git-lfs 管理 FFmpeg 二进制，externalBin 声明 | ✓ |
| npm 脚本下载 | 构建时从 ffmpeg.org 下载 | |
| 首次启动下载 | CDN 自动下载 | |

### FFmpeg 版本 + 平台

| Option | Description | Selected |
|--------|-------------|----------|
| FFmpeg 7.1 + 平台目录 + sidecar API | 平台子目录 + Command::new_sidecar() | ✓ |
| FFmpeg 6.1 + 统一目录 | LTS 版本 + 根目录 | |
| 预编译嵌入 + 最小二进制 | 最小静态编译 | |

---

## 配置数据流与 Tauri 命令

| Option | Description | Selected |
|--------|-------------|----------|
| 统一 get_config/save_config | 两个命令 + 脱敏 JSON + 前端分发到 Pinia | ✓ |
| 面板级独立命令 | 每面板一对命令 | |
| Pinia 为主 + 后端同步 | Pinia 主数据源 + 后端从同步 | |

### 敏感字段

| Option | Description | Selected |
|--------|-------------|----------|
| 脱敏返回 + 增量保存 | get_config 返回脱敏 + save_config 仅发送修改字段 | ✓ |
| 完整返回 + 全量保存 | 不脱敏 + 全量覆盖 | |
| 不返回 + 哨兵值 | 敏感字段不返回 + 'KEEP_EXISTING' 哨兵 | |

### 加载时序

| Option | Description | Selected |
|--------|-------------|----------|
| 应用启动预加载 + 即时渲染 | App.vue onMounted + Pinia 缓存 + 加载失败 snackbar+重试 | ✓ |
| 抽屉打开懒加载 | 首次打开加载 | |
| 启动同步阻塞 | await 阻塞 UI | |

### 新 Tauri 命令

| Option | Description | Selected |
|--------|-------------|----------|
| 5 个新命令 | get_config/save_config/test_llm_connection/get_edge_tts_voices/get_system_proxy → config.rs | ✓ |
| 3 个命令 | 仅 get/save/voices | |
| 2 个命令 | 仅 get/save | |

---

## 实现优先级与计划拆分

| Option | Description | Selected |
|--------|-------------|----------|
| Wave 0 + Wave 1 + Wave 2 | 后端基础 → 前端面板 → 整合，约 5~6 plans | ✓ |
| 前端优先 | 全部前端 → 技术债 | |
| 全部一个 plan | 合并为一个 | |

---

## 测试策略

| Option | Description | Selected |
|--------|-------------|----------|
| vitest 真实 + 集成 smoke | 面板/Pinia 真实测试 + Rust 单元测试 + FFmpeg smoke test | ✓ |
| 仅存根测试 | 与 Phase 15 一致的 8 行存根 | |
| 全真实测试 | 含代理测试环境搭建 | |

---

## Claude's Discretion

- SettingSection loading 骨架屏的实现细节和过渡时间
- Provider 下拉列表中各 Provider 的默认 Base URL 映射
- v-snackbar 的精确位置、颜色和持续时间
- 网络代理面板中"启用"开关与空代理地址的交互逻辑
- test_llm_connection 命令的具体请求构造和超时设置
- get_edge_tts_voices 的缓存策略和刷新机制
- 配置面板 TypeScript 类型从 ts-rs 导出（或手动定义接口）
- git-lfs 配置和 FFmpeg 二进制文件版本固定
- Tauri capabilities 中 shell:allow-execute 的具体 scope 路径

## Deferred Ideas

None — all discussion stayed within phase scope.
