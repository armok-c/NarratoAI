# Phase 9: JianYing Export - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 09-jianying-export
**Areas discussed:** 剪映格式逆向策略, 导出 API 输入设计, 时间线轨道映射, 草稿元数据配置

---

## 剪映格式逆向策略

| Option | Description | Selected |
|--------|-------------|----------|
| pyJianYingDraft 源码逆向 | 阅读 pyJianYingDraft Python 源码，理解数据结构到 JSON 的映射关系 | ✓ |
| 样本分析为主 | 用 Python 版生成草稿，手动分析输出 JSON 结构 | |
| 两者结合 | 先读源码理解结构，再用样本验证 | |

**User's choice:** pyJianYingDraft 源码逆向（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 跟随 pyJianYingDraft 最新版 | 确保与当前剪映专业版兼容 | ✓ |
| 最低兼容版本 | 更稳定但可能缺少新特性 | |

**User's choice:** 跟随 pyJianYingDraft 最新版（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 完整复制 pyJianYingDraft API | 在 Rust 中实现对等的 DraftFolder/VideoSegment/AudioSegment 等 builder 类型 | ✓ |
| 只实现导出功能 | 直接流程，不抽象出 builder API | |

**User's choice:** 完整复制 pyJianYingDraft API（推荐）

---

## 导出 API 输入设计

| Option | Description | Selected |
|--------|-------------|----------|
| 仅处理后的数据 | 导出函数接收处理后的 Script + 视频源路径 + 配置，不做 TTS/裁剪 | ✓ (Claude) |
| 包含完整流水线 | 像 Python 版一样包含 TTS + 裁剪 + 导出 | |
| 两种都提供 | 提供纯导出函数和完整流水线+导出函数 | |

**User's choice:** You decide → Claude 选定"仅处理后的数据"
**Notes:** Phase 9 依赖链只有 Phase 5，不包含 TTS/FFmpeg 流水线。导出是纯格式转换职责。

| Option | Description | Selected |
|--------|-------------|----------|
| 新建 ExportRequest 结构体 | 包含 Script、视频源路径、草稿配置等 | ✓ |
| 多参数函数 | 直接传多个参数 | |

**User's choice:** 新建 ExportRequest 结构体（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 严格校验 | 检查所有 Option 字段有值，缺失则报错 | ✓ |
| 宽松处理 | Option 字段缺失时跳过对应轨道 | |

**User's choice:** 严格校验（推荐）

---

## 时间线轨道映射

| Option | Description | Selected |
|--------|-------------|----------|
| 视频 + 音频双轨 | 与 Python 版一致 | ✓ |
| 视频 + 音频 + 字幕三轨 | 字幕直接烧录到剪映中 | |
| 四轨分离 | 视频轨 + 解说音频轨 + 原声轨 + BGM 轨 | |

**User's choice:** 视频 + 音频双轨（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 与 Python 版 1:1 对齐 | OST=0/2 加音频轨，OST=1 不加 | ✓ |
| You decide | Claude 根据实际场景优化 | |

**User's choice:** 与 Python 版 1:1 对齐（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 智能回退 | 有处理后视频用它，没有则用原始视频+source_timerange | ✓ |
| 仅处理后的文件 | 没有处理后视频则报错 | |

**User's choice:** 智能回退（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| ffprobe 精确时长 + min 策略 | 获取音频实际时长，取 min(音频时长, 视频时长) | ✓ |
| 用脚本 duration 字段 | 简单但可能不精确 | |

**User's choice:** ffprobe 精确时长 + min 策略（推荐）

---

## 草稿元数据配置

| Option | Description | Selected |
|--------|-------------|----------|
| 配置化 + 默认 1080p | ExportRequest 中包含分辨率参数，默认 1920x1080 | ✓ |
| 硬编码 1080p | 与 Python 版一致 | |
| 从 config.toml 读取 | 与 Phase 1 配置系统集成 | |

**User's choice:** 配置化 + 默认 1080p（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 用户指定 + 自动回退 | 用户指定名称，为空用 NarratoAI_{timestamp} | ✓ |
| 始终自动生成 | 始终 NarratoAI_{timestamp} | |

**User's choice:** 用户指定 + 自动回退（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 使用剪映默认帧率 | 减少逆向工程复杂度 | ✓ |
| 配置化帧率 | ExportRequest 中添加 framerate 字段 | |

**User's choice:** 使用剪映默认帧率（推荐）

| Option | Description | Selected |
|--------|-------------|----------|
| 由调用方传入 | ExportRequest 结构体的字段 | ✓ |
| 从 config.toml 读取 | 与 Phase 1 配置系统集成 | |

**User's choice:** 由调用方传入（推荐）

---

## Claude's Discretion

- 导出 API 输入设计：用户选择"You decide"，Claude 选定"仅处理后的数据"——Phase 9 只做格式转换，管道编排属于 Phase 6/7/8

## Deferred Ideas

None — discussion stayed within phase scope
