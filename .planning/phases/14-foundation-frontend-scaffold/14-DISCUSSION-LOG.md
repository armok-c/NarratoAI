# Phase 14: Foundation — Frontend Scaffold - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 14-foundation-frontend-scaffold
**Areas discussed:** 目录结构, Store 粒度, 集成测试桩, Vite 版本, ts-rs 类型同步, 目录组织维度

---

## 目录结构

| Option | Description | Selected |
|--------|-------------|----------|
| 照搬 SmartEdit | 完全复用 SmartEdit 的目录布局和命名约定 | |
| 照搬 + 扩展 | 在 SmartEdit 结构基础上新增 mode/、pipeline/ 等目录 | |
| 自行设计 | 不强制对齐，以 NarratoAI 需求出发重新设计 | ✓ |

**User's choice:** 自行设计 — 后续追问中确认按类型分组

---

## Store 粒度

| Option | Description | Selected |
|--------|-------------|----------|
| 严格 9 Store | 每个 Store 职责单一：app/mode/video/pipeline/script/llm/tts/bgm | ✓ |
| 合并相近 Store | 参照 SmartEdit 合并部分 Store | |
| 先建骨架再细化 | Phase 14 只创建骨架，后续细调 | |

**User's choice:** 严格 9 Store

---

## 集成测试桩替换

| Option | Description | Selected |
|--------|-------------|----------|
| 真实断言 + #[ignore] | 可验证行为用真实断言，需外部依赖用 #[ignore] | ✓ |
| 移除桩文件 | 直接删除桩文件，后续重写 | |
| 最小替换 | 仅替换当前可运行的，其他保持现状 | |

**User's choice:** 真实断言 + #[ignore]

---

## Vite 版本

| Option | Description | Selected |
|--------|-------------|----------|
| Vite 6（推荐） | 最新稳定版，生态成熟 | ✓ |
| 对齐 SmartEdit Vite 5 | 保持与参考项目一致 | |
| Vite 7 beta | 接近规范要求的 Vite 8 | |

**User's choice:** Vite 6

---

## 目录组织维度

| Option | Description | Selected |
|--------|-------------|----------|
| 按类型分组（推荐） | components/stores/views/types 等 | ✓ |
| 按功能域分组 | documentary/sde/sdp/shared | |
| 混合模式 | 通用按类型，三大模式按域 | |

**User's choice:** 按类型分组

---

## ts-rs 类型同步

| Option | Description | Selected |
|--------|-------------|----------|
| ts-rs 自动导出（推荐） | Rust #[derive(TS)] 自动生成 .ts 文件 | ✓ |
| 手写类型定义 | 手动维护 TypeScript 类型 | |
| 混合模式 | ts-rs 基础类型 + 手写复杂类型 | |

**User's choice:** ts-rs 自动导出

---

## Claude's Discretion

- Vite 具体配置（端口、manualChunks、Tauri host）
- tsconfig 严格度级别
- ESLint/Prettier 规则集
- Vuetify 主题色和本地化配置
- 各 Store 接口方法签名
- API 层模块拆分粒度
- ts-rs 导出路径和文件组织
- Vite + Tauri dev/build 命令集成

## Deferred Ideas

None — discussion stayed within phase scope.
