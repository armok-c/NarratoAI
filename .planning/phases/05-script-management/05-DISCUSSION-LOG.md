# Phase 5: Script Management - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in 05-CONTEXT.md — this log preserves the discussion process.

**Date:** 2026-04-28
**Phase:** 05-script-management
**Mode:** discuss (default, interactive)
**Areas discussed:** 数据模型设计, 格式兼容性, 校验策略, 编辑 API

## Discussion Flow

### 数据模型设计

| Question | Options | User Selection |
|----------|---------|----------------|
| 核心字段与管道扩展字段的组织方式 | 单结构体 + Option / 双结构体分离 / 你来决定 | 单结构体 + Option |
| OST 字段类型 | Rust 枚举 (serde_repr 0/1/2) / 整数 u8 | Rust 枚举 |
| timestamp 存储 | 字符串保留原格式 / 解析为 Duration 类型 | 字符串保留原格式 |
| 顶层结构 | Vec\<ScriptClip\> / 包装结构 + 元数据 | Vec\<ScriptClip\> |

### 格式兼容性

| Question | Options | User Selection |
|----------|---------|----------------|
| LLM items 信封处理 | 仅脚本 JSON（信封由管道处理）/ 两种都支持 | 仅脚本 JSON |
| 保存字段范围 | 仅核心字段（忽略 Option）/ 全部字段 | 仅核心字段 |
| JSON 输出格式 | 美化 JSON (indent=2) / 紧凑 JSON | 美化 JSON |

### 校验策略

| Question | Options | User Selection |
|----------|---------|----------------|
| 校验规则严格程度 | Python 版对齐 / 严格校验（timestamp 范围、_id 连续性等） | Python 版对齐 |
| 校验时机 | 加载时自动校验 / 显式调用 validate() | 加载时自动校验 |
| 错误报告方式 | 收集所有错误 / 第一个错误即停 | 收集所有错误 |

### 编辑 API

| Question | Options | User Selection |
|----------|---------|----------------|
| API 风格 | 不可变更新方法（返回新 Script）/ Builder 模式 | 不可变更新方法 |
| clip 定位方式 | 数组下标 usize / 按 _id 查找 | 数组下标 usize |
| 编辑范围 | 单字段更新 / 包含增删改排序 | 单字段更新 |
| JSON 保存格式 | 美化 JSON / 紧凑 JSON | 美化 JSON |

## Corrections Made

No corrections — all recommendations accepted.

## Deferred Ideas

None — discussion stayed within phase scope.
