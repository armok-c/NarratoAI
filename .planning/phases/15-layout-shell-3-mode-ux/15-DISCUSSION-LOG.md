# Phase 15: Layout Shell + 3-Mode UX - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 15-layout-shell-3-mode-ux
**Areas discussed:** AppHeader 窗口控制策略, HomeView 上下分区布局比例, ModeSelector 交互位置, SettingsDrawer 内容边界, SystemMonitorBar

---

## AppHeader 窗口控制策略

### 标题栏方案
| Option | Description | Selected |
|--------|-------------|----------|
| 隐藏原生标题栏 | 与 SmartEdit 一致。40px 自定义 v-app-bar 含窗口控制按钮 + Logo + 标题，mousedown 启动 window.startDragging() | ✓ |
| 保留原生标题栏 | 不设置 decorations: false，保留操作系统原生标题栏。占用额外垂直空间 | |

**User's choice:** 隐藏原生标题栏（推荐）

### 关闭按钮行为
| Option | Description | Selected |
|--------|-------------|----------|
| 直接退出 | 关闭按钮 = getCurrentWindow().close()。与 SmartEdit 一致 | ✓ |
| 隐藏到托盘 | 关闭按钮 = getCurrentWindow().hide()，需额外实现托盘图标 + 右键菜单 | |

**User's choice:** 直接退出（推荐）

### Logo 方案
| Option | Description | Selected |
|--------|-------------|----------|
| 拿 SmartEdit 的 logo.svg | 复用 SmartEdit logo.svg 作为 logo | ✓ |
| 图标 + 文字 | mdi 图标配 "NarratoAI" 文本 | |
| 准备 SVG logo | Phase 15 临时用图标 + 文字，预留 SVG 位置 | |

**User's choice:** 拿 SmartEdit 的 logo.svg 拿来作为 logo

### 窗口控制按钮布局
| Option | Description | Selected |
|--------|-------------|----------|
| 与 SmartEdit 一致 | 最小化 \| 最大化 \| 关闭 三按钮排布 AppHeader 右侧 | ✓ |
| 简化版 | 仅最小化和关闭两个按钮，去掉最大化 | |

**User's choice:** 与 SmartEdit 一致（推荐）

### 标题栏高度
| Option | Description | Selected |
|--------|-------------|----------|
| 40px | 与 SmartEdit 一致。紧凑标题栏，留给 HomeView 更多垂直空间 | ✓ |
| 48px | 标准 Material Design 3 高度，稍宽松 | |

**User's choice:** 40px（推荐）

### 按钮排列
| Option | Description | Selected |
|--------|-------------|----------|
| 与 SmartEdit 一致 | Logo \| NarratoAI \| spacer \| 设置齿轮 \| 主题切换 \| 窗口控制 | ✓ |
| 简化：去掉设置齿轮 | 如果设置入口在 HomeView 可省略齿轮 | |
| 添加模式切换 | ModeSelector 放在 AppHeader 中 | |

**User's choice:** 与 SmartEdit 一致（推荐）

### 拖拽范围
| Option | Description | Selected |
|--------|-------------|----------|
| 完全对齐 | 整个 AppHeader 区域可拖拽，button 和 window-controls 区域排除 | ✓ |
| 仅 Logo+标题区 | 只有 Logo 和 NarratoAI 文字区域可拖拽 | |

**User's choice:** 完全对齐（推荐）

### 双击最大化
| Option | Description | Selected |
|--------|-------------|----------|
| 保留双击 + 分割线 | dblclick 触发 toggleMaximize()，v-app-bar 下方 border-bottom | ✓ |
| 保留双击，无分割线 | 纯靠颜色区分 | |

**User's choice:** 保留双击 + 分割线（推荐）

---

## HomeView 上下分区布局比例

### 上下分区比例
| Option | Description | Selected |
|--------|-------------|----------|
| 固定 60/40 | 上区占 60%（主要内容区），下区占 40%（操作+日志）。固定比例不可拖拽 | ✓ |
| 固定 50/50 | 上下各一半 | |
| 可拖拽分隔 | 用户可拖拽分隔条调整比例 | |

**User's choice:** 固定 60/40（推荐）

### 上区双列比例
| Option | Description | Selected |
|--------|-------------|----------|
| 60/40 左右 | 左列 VideoTable 主操作区占 60%，右列设置面板占 40% | ✓ |
| 70/30 左右 | 左列 70%（更宽 VideoTable），右列 30% | |

**User's choice:** 60/40 左右（推荐）

### 下区双列比例
| Option | Description | Selected |
|--------|-------------|----------|
| 与上区对齐 60/40 | 上下分区左右列完全对齐，视觉统一 | ✓ |
| 70/30 操作区更窄 | 左列 30%（按钮行），右列 70%（LogPanel 需要更多空间） | |
| 独立比例 | 下区比例与上区不同 | |

**User's choice:** 与上区对齐 60/40（推荐）

### 占位策略
| Option | Description | Selected |
|--------|-------------|----------|
| Labeled placeholder | 灰色虚线框 + 区域标题 + 「Phase 17 实现」标注 | ✓ |
| 基础布局 + 空 div | 仅 CSS/HTML 画出布局槽位，不添加标签文字 | |
| 替换现有欢迎页 | 完全替换 HomeView + 占位卡片 | |

**User's choice:** Labeled placeholder（推荐）

### 设置卡片区内容
| Option | Description | Selected |
|--------|-------------|----------|
| 仅 ModeSelector | 上区右列只有模式选择下拉。LLM/TTS/BGM/Export 面板在 SettingsDrawer 中 | ✓ |
| ModeSelector + 快速开关 | 深色模式开关 + 系统监控入口 | |

**User's choice:** 仅 ModeSelector（推荐）

### 容器宽度
| Option | Description | Selected |
|--------|-------------|----------|
| 全宽 + gap 分隔 | HomeView 内容充满整个 v-main 宽度，16px gap 分隔 | ✓ |
| 最大宽度 1200px 居中 | 内容区居中，小窗口全宽大窗口居中 | |

**User's choice:** 全宽 + gap 分隔（推荐）

### 响应式
| Option | Description | Selected |
|--------|-------------|----------|
| 固定最低宽度 | Tauri 窗口设置 minWidth（如 900px），保证双列布局始终可用 | ✓ |
| 响应式堆叠 | 窗口窄屏时双列变单列垂直堆叠 | |

**User's choice:** 固定最低宽度（推荐）

### 占位标题
| Option | Description | Selected |
|--------|-------------|----------|
| 有标题占位 | 占位卡片带标题（视频列表 / 操作按钮 / 日志面板） | ✓ |
| 无标题纯占位 | 仅灰色虚线框无文字 | |

**User's choice:** 有标题占位（推荐）

---

## ModeSelector 交互位置

### 控件样式
| Option | Description | Selected |
|--------|-------------|----------|
| v-select 下拉 | Vuetify v-select 下拉选择框，标签「工作模式」三个选项 | ✓ |
| v-btn-toggle 按钮组 | 三个并排按钮切换，视觉更突出 | |
| v-radio-group 单选 | 单选按钮垂直排列 | |

**User's choice:** v-select 下拉（推荐）

### 模式切换副作用
| Option | Description | Selected |
|--------|-------------|----------|
| 仅 reset + 模式切换 | resetPipeline() + 更新 currentMode。不处理面板显隐 | ✓ |
| reset + 动态可见性切换 | 在 ModeStore 中设置 visible panels map，为 Phase 16 做准备 | |

**User's choice:** 仅 reset + 模式切换（推荐）

### 模式标签
| Option | Description | Selected |
|--------|-------------|----------|
| 纯中文 | 纪录片解说 \| 短剧解说 \| 短剧混剪 | ✓ |
| 中英混合 | 含英文缩写（如 Documentary / SDE / SDP） | |

**User's choice:** 纯中文（推荐）

### 卡片包装
| Option | Description | Selected |
|--------|-------------|----------|
| 有标题的卡片 | v-card 包装：标题「工作模式」+ mdi 图标 + v-select | ✓ |
| 纯下拉无卡片 | 仅 v-select 无卡片包装 | |

**User's choice:** 有标题的卡片（推荐）

### 默认模式 + 持久化
| Option | Description | Selected |
|--------|-------------|----------|
| 默认纪录片 + 持久化 | 启动默认 documentary，localStorage('lastMode') 持久化 | ✓ |
| 总是默认纪录片 | 每次启动从 documentary 开始，不持久化 | |

**User's choice:** 默认纪录片 + 持久化（推荐）

### 切换保护
| Option | Description | Selected |
|--------|-------------|----------|
| 不阻止，直接重置 | 切换模式直接调用 resetPipeline()，不弹确认 | ✓ |
| 弹出确认对话框 | 若 pipeline 正在运行弹出确认 | |

**User's choice:** 不阻止，直接重置（推荐）

### 选项描述
| Option | Description | Selected |
|--------|-------------|----------|
| 仅模式名称 | 三个选项仅显示中文名称 | ✓ |
| 名称 + 简短描述 | 每个选项显示副标题描述 | |

**User's choice:** 仅模式名称（推荐）

---

## SettingsDrawer 内容边界

### 抽屉内容
| Option | Description | Selected |
|--------|-------------|----------|
| 主题 + 版本号 | 仅外观设置（深色/浅色主题切换）+ 版本号 + 页头 | ✓ |
| 主题 + 配置面板骨架 | 主题 + 所有配置面板折叠骨架（LLM/TTS/BGM/Export 占位） | |
| 完整设置骨架 + 可见性 | SmartEdit 完整设置抽屉 + MODE-03 可见性规则 | |

**User's choice:** 主题 + 版本号（推荐）

### 抽屉结构
| Option | Description | Selected |
|--------|-------------|----------|
| Header + Content + Footer 版本号 | 保留 SmartEdit 三段式：Header + Content + Footer（仅版本号） | ✓ |
| Header + Content 无 Footer | 版本号放在 Content 底部 | |

**User's choice:** Header + Content + Footer 版本号（推荐）

### 实现方式
| Option | Description | Selected |
|--------|-------------|----------|
| v-navigation-drawer | location="right" temporary width="420"，右侧滑入 + scrim 遮罩 | ✓ |
| v-dialog | 全屏右侧面板 | |

**User's choice:** v-navigation-drawer（推荐）

### 主题切换样式
| Option | Description | Selected |
|--------|-------------|----------|
| v-switch + 图标 | 左侧动态图标 + 标签 + 状态描述，右侧 v-switch | ✓ |
| v-btn 切换 | 两个按钮 active 高亮 | |

**User's choice:** v-switch + 图标（推荐）

### 滚动条样式
| Option | Description | Selected |
|--------|-------------|----------|
| 420px + 自定义滚动条 | 自定义 webkit-scrollbar 6px 宽圆角半透明 | ✓ |
| 420px + 默认滚动条 | 使用系统默认滚动条 | |

**User's choice:** 420px + 自定义滚动条（推荐）

### SettingSection 组件
| Option | Description | Selected |
|--------|-------------|----------|
| 创建 SettingSection | Phase 15 创建可复用组件，主题切换 + Phase 16 面板复用 | ✓ |
| 内联实现 | Phase 15 用 v-card 内联，不提取组件 | |

**User's choice:** 创建 SettingSection（推荐）

---

## SystemMonitorBar

### 位置
| Option | Description | Selected |
|--------|-------------|----------|
| DefaultLayout v-footer | v-footer 中所有页面共享，始终可见 | ✓ |
| HomeView 底部 | 仅 HomeView 内显示 | |

**User's choice:** DefaultLayout v-footer（推荐）

### 数据来源
| Option | Description | Selected |
|--------|-------------|----------|
| Tauri command | Rust `get_system_stats` 命令 + sysinfo crate，前端轮询 | ✓ |
| 纯前端 JS API | 浏览器 API 估算，不精确 | |

**User's choice:** Tauri command（推荐）

### 显示格式
| Option | Description | Selected |
|--------|-------------|----------|
| 图标 + 百分比文字 | mdi-chip CPU + "CPU XX%" \| mdi-memory RAM + "RAM XX%" | ✓ |
| 进度条 + 百分比 | v-progress-linear 各一个 | |
| 仅百分比文字 | 纯文字 "CPU: XX% \| RAM: XX%" | |

**User's choice:** 图标 + 百分比文字（推荐）

### 刷新频率
| Option | Description | Selected |
|--------|-------------|----------|
| 2 秒轮询 | setInterval 2 秒调用一次，onUnmounted 清理 | ✓ |
| 5 秒轮询 | 更低开销但对变化感知较慢 | |
| Tauri 事件推送 | 后端主动推送，无需轮询但需维护监控线程 | |

**User's choice:** 2 秒轮询（推荐）

---

## Claude's Discretion

- `tauri.conf.json` 的 `minWidth` 具体像素值和窗口默认大小
- AppHeader 和 WindowControls 按钮的精确 Vuetify icon 名称和 hover 样式
- v-footer 的高度和颜色 token
- HomeView 占位卡片的精确样式（虚线框颜色、文字大小）
- SystemMonitorBar `get_system_stats` 命令的具体签名和错误处理
- SettingSection.vue 的 collapsible 实现细节和过渡动画
- `isMaximized` 状态的初始化方式（`getCurrentWindow().isMaximized()` 调用时机）

## Deferred Ideas

None — discussion stayed within phase scope.
