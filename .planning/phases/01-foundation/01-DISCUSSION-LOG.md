# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 1-Foundation
**Areas discussed:** Rust 项目结构, 配置系统实现, FFmpeg 集成方式, 错误处理体系, 配置并发安全, FFmpeg 进度回调, 测试策略, 版本号管理

---

## Rust 项目结构

### Crate 组织

| Option | Description | Selected |
|--------|-------------|----------|
| 单个 lib crate | 一个 narratoai-core 库，内部用 mod 划分模块 | |
| Cargo workspace | 多个 crate 拆分，更模块化 | |
| 让我决定 | 交给 Claude 判断 | ✓ |

**User's choice:** 让我决定
**Notes:** Claude 选定单个 lib crate

### 模块划分

| Option | Description | Selected |
|--------|-------------|----------|
| 按领域划分 | config/、ffmpeg/、error/、llm/、tts/ 等，与 Python 版对齐 | ✓ |
| 按层划分 | domain/、service/、infra/，DDD 风格 | |
| 让我决定 | 交给 Claude 判断 | |

**User's choice:** 按领域划分

### 异步运行时

| Option | Description | Selected |
|--------|-------------|----------|
| tokio | 事实标准，Tauri 2.0 依赖 tokio | ✓ |
| async-std | 轻量级，但与 Tauri 不兼容 | |
| 让我决定 | 交给 Claude 判断 | |

**User's choice:** tokio

### 日志库

| Option | Description | Selected |
|--------|-------------|----------|
| tracing | 异步生态标准，与 tokio 深度集成 | ✓ |
| log + env_logger | 更简单但缺少异步上下文追踪 | |
| 让我决定 | 交给 Claude 判断 | |

**User's choice:** tracing

### 目录布局

| Option | Description | Selected |
|--------|-------------|----------|
| src/ 根目录布局 | src/lib.rs + src/config/mod.rs 等 | ✓ |
| src/app/ 嵌套布局 | 多一层嵌套与 Python 版对齐 | |

**User's choice:** src/ 根目录布局

### 依赖版本策略

| Option | Description | Selected |
|--------|-------------|----------|
| 精确版本 | 如 tokio = "1.45.0"，可复现构建 | ✓ |
| 宽松版本范围 | 如 tokio = "1"，更灵活 | |

**User's choice:** 精确版本

---

## 配置系统实现

### 配置类型

| Option | Description | Selected |
|--------|-------------|----------|
| 强类型 serde 结构体 | 编译时检查，IDE 补全友好 | ✓ |
| 动态 HashMap | 灵活但失去编译时检查 | |

**User's choice:** 强类型 serde 结构体

### 默认值策略

| Option | Description | Selected |
|--------|-------------|----------|
| serde(default) 自动填充 | 缺失字段自动填默认值 | ✓ |
| Option<T> + 必填检查 | 运行时检查必填字段 | |
| 混合方案 | 必填用原始类型，可选用 Option | |

**User's choice:** serde(default) 自动填充

### 热加载机制

| Option | Description | Selected |
|--------|-------------|----------|
| notify 文件监听 | 实时响应，资源开销小 | ✓ |
| mtime 轮询检查 | 简单但有延迟 | |
| Phase 1 不做热加载 | 推迟到后续阶段 | |

**User's choice:** notify 文件监听

### 配置结构

| Option | Description | Selected |
|--------|-------------|----------|
| 单一顶层结构体 | AppConfig 包含所有子段，与 TOML 结构对应 | ✓ |
| 分散加载 | 各配置段独立加载 | |

**User's choice:** 单一顶层结构体

---

## FFmpeg 集成方式

### FFmpeg 绑定

| Option | Description | Selected |
|--------|-------------|----------|
| ffmpeg-sidecar crate | 类型安全命令构建器，自动下载 FFmpeg | ✓ |
| std::process::Command | 完全控制，但需自己封装 | |

**User's choice:** ffmpeg-sidecar crate

### ffprobe

| Option | Description | Selected |
|--------|-------------|----------|
| 通过 ffmpeg-sidecar | 同一封装体系内完成探测 | ✓ |
| 独立调用 ffprobe | 更底层但需额外封装 | |

**User's choice:** 通过 ffmpeg-sidecar

### 硬件加速检测

| Option | Description | Selected |
|--------|-------------|----------|
| 通过 FFmpeg 检测编码器 | 运行 ffmpeg -encoders 检测可用编码器 | ✓ |
| 系统 GPU 检测 | 用 sysinfo crate 检测 GPU | |

**User's choice:** 通过 FFmpeg 检测编码器

### FFmpeg 二进制

| Option | Description | Selected |
|--------|-------------|----------|
| 双模式：自动下载 + 系统 FFmpeg | 灵活支持两种来源 | |
| 仅系统 FFmpeg | 要求用户自行安装 | |
| 内置 ffmpeg | 用户选择内置（用户自由输入） | ✓ |

**User's choice:** 内置 ffmpeg

### 异步执行

| Option | Description | Selected |
|--------|-------------|----------|
| spawn_blocking 包装 | 移到专用线程池，不阻塞运行时 | ✓ |
| tokio::process::Command | 异步子进程执行 | |

**User's choice:** spawn_blocking 包装

---

## 错误处理体系

### 错误类型

| Option | Description | Selected |
|--------|-------------|----------|
| thiserror 领域错误 | 编译时完整性保证，适合库 | ✓ |
| anyhow 统一错误 | 简单但失去类型匹配 | |
| 混合：库层 thiserror + 应用层 anyhow | 结合两者优势 | |

**User's choice:** thiserror 领域错误

### 错误层级

| Option | Description | Selected |
|--------|-------------|----------|
| 按领域分枚举 | ConfigError、FFmpegError 等 | ✓ |
| 单一统一枚举 | 简单但会膨胀 | |

**User's choice:** 按领域分枚举

### 错误信息语言

| Option | Description | Selected |
|--------|-------------|----------|
| 中文用户信息 | 与 Python 版对齐 | ✓ |
| 全英文 | 代码一致性 | |

**User's choice:** 中文用户信息

---

## 配置并发安全

| Option | Description | Selected |
|--------|-------------|----------|
| Arc<RwLock<T>> | 读写锁，多读者并发 | ✓ |
| ArcSwap<T> | 原子指针交换，读无锁 | |

**User's choice:** Arc<RwLock<T>>

---

## FFmpeg 进度回调

| Option | Description | Selected |
|--------|-------------|----------|
| 回调函数参数 | Option<ProgressCallback>，简单直接 | ✓ |
| Channel 事件流 | tokio mpsc，更解耦但更复杂 | |

**User's choice:** 回调函数参数

---

## 测试策略

| Option | Description | Selected |
|--------|-------------|----------|
| 单元 + 真实 FFmpeg 集成测试 | CI 通过 ffmpeg-sidecar 自动下载 FFmpeg | ✓ |
| 单元 + mock 测试 | 不依赖外部工具但可能不一致 | |

**User's choice:** 单元 + 真实 FFmpeg 集成测试

---

## 版本号管理

| Option | Description | Selected |
|--------|-------------|----------|
| Cargo.toml version 字段 | Rust 标准做法，env!() 编译时获取 | ✓ |
| 保留 project_version 文件 | 与 Python 版兼容但非 Rust 惯用 | |

**User's choice:** Cargo.toml version 字段

---

## Claude's Discretion

- Crate 组织方式：用户选择"让我决定"，Claude 选定单个 lib crate

## Deferred Ideas

None — discussion stayed within phase scope
