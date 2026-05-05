# Phase 7: SDE Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 07-sde-pipeline
**Areas discussed:** 模块结构, LLM 编排, 数据模型

---

## 模块结构

### Round 1: 模块组织方式

| Option | Description | Selected |
|--------|-------------|----------|
| 独立 src/sde/ 模块 | 创建独立模块，视频处理步骤调用 src/documentary/ 已有函数 | ✓ |
| 独立模块 + 立即抽取 | 创建 src/sde/ 同时抽取 common.rs | |
| 复用纪录片模块 | 在 src/documentary/ 下扩展 SDE 路径 | |

**Notes:** 用户选择与 Phase 6 D-05 决策一致（先做能用的，发现重复再抽取）。

### Round 2: 文件拆分 / 步骤复用 / 请求参数 / 进度步骤

| Question | Selected | Alternatives |
|----------|----------|--------------|
| 文件拆分 | 镜像纪录片 8 文件 | 精简 4 文件 / 单文件 mod.rs |
| 步骤复用方式 | 暴露 Phase 6 私有函数 (pub(crate)) | 新增公开入口 / 完全独立实现 |
| 请求参数 | 新建 SdeRequest | 复用 DocumentaryRequest / 组合模式 |
| 进度步骤 | 独立 9 步 | 复用 6 步 / SDE 专属 5 步 |

### Round 3: 入口设计 / SdeRequest 输入 / 字幕解析位置 / 参数对齐

| Question | Selected | Alternatives |
|----------|----------|--------------|
| 入口设计 | 原子入口 run_sde() | 两阶段 / 多入口 |
| SdeRequest 输入 | 含字幕路径 | 仅脚本路径 / 混合可选 |
| 字幕解析位置 | SDE 内部实现 | 抽取公共模块 / 仅 UTF-8 |
| 参数对齐 | 完全对齐 DocumentaryRequest | 嵌入 / 分离 |

### Round 4: 文件职责 / 错误类型 / JSON 修复 / 状态结构体

| Question | Selected | Alternatives |
|----------|----------|--------------|
| 文件职责映射 | 按职责对齐 (script_gen/subtitle/pipeline/error/types/clip/audio/timestamp) | 最简 3 文件 / 按 SDE 特有逻辑 |
| 错误类型 | SdeError + 嵌入 PipelineError | 完全独立 / 复用已有 |
| JSON 修复 | SDE 内部实现 (script_gen.rs 私有函数) | 抽取公共工具 / 不实现修复 |
| 状态结构体 | 独立 SdePipelineState | 扩展 DocumentaryPipelineState / 泛型抽象 |

---

## LLM 编排

### Round 1: 模型选择 / 中间数据类型 / 返回值 / 中间产物

| Question | Selected | Alternatives |
|----------|----------|--------------|
| 模型选择 | 统一使用 text 模型 | 两步不同模型 / 新增 SDE 专属配置 |
| 中间数据类型 | 纯字符串传递 | 定义中间类型 / JSON 中间格式 |
| 生成结果返回值 | 返回原始 LLM 输出字符串 | 函数内解析返回 Script / 两种返回模式 |
| 中间产物 | 保存所有中间产物到 task_dir | 仅内存传递 / 临时文件 |

### Round 2: Prompt 加载 / Temperature / 错误处理 / 原子入口串联

| Question | Selected | Alternatives |
|----------|----------|--------------|
| Prompt 加载 | 通过 PromptManager | 硬编码 / 外部文件 |
| Temperature | 统一 temperature 控制 | 分步独立 / 硬编码 |
| LLM 错误处理 | Fail-fast | 自动重试 / 分步恢复 |
| 原子入口编排 | run_sde 内部串联 | 仅处理视频 / 智能检测输入 |

---

## 数据模型

### Round 1: 字段映射 / OST=1 检测 / 脚本校验 / 时间戳格式

| Question | Selected | Alternatives |
|----------|----------|--------------|
| 字段映射 | Python 版 1:1 映射 | 中间类型 + From / 自定义映射 |
| OST=1 检测 | 字符串前缀 "播放原片" | 扩展 ScriptClip / 新增字段 |
| 脚本校验 | 复用 Phase 5 校验 | SDE 专属校验 / 不做额外校验 |
| 时间戳格式 | 与 ScriptClip 一致直接映射 | 标准化 / 字幕时间戳校正 |

### Round 2: OST 类型范围 / 运行时字段 / 字幕信息保留 / 原声裁剪

| Question | Selected | Alternatives |
|----------|----------|--------------|
| OST 类型范围 | 仅 OST=0/1 | 允许全部 / 仅 OST=0 |
| 运行时字段 | 视频处理步骤计算回写 | 不回写 / LLM 估算 |
| 字幕信息保留 | 辅助结构 (SubtitleSegment) | 信任 LLM / 扩展 ScriptClip |
| 原声裁剪 | 按字幕精确时间戳 | LLM 近似值 / 固定时长 |

---

## Claude's Discretion

- SdePipelineState 具体字段定义
- step 函数拆分粒度和内部实现
- SdeError 完整变体列表和中文错误消息
- JSON 修复函数 repair_json() 的 6 步回退策略
- SRT 时间戳解析工具实现
- 字幕编码探测链实现
- 进度百分比分配
- mod.rs 公开 API 设计
- lib.rs 的 pub mod sde 声明

## Deferred Ideas

None

---

*Discussion completed: 2026-05-05*
*Decisions: 29 total across 3 areas*
