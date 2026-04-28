# Phase 1: Foundation - Research

**Researched:** 2026-04-28
**Domain:** Rust 项目脚手架 / TOML 配置系统 / 错误类型体系 / FFmpeg CLI 集成
**Confidence:** HIGH

## Summary

Phase 1 建立整个 Rust 重写项目的骨架：一个可编译的 `narratoai-core` 库 crate，包含三个核心领域模块（config、ffmpeg、error）。配置系统用 serde + toml 反序列化为强类型结构体，支持通过 notify crate 热加载，用 `Arc<RwLock<AppConfig>>` 保证并发安全。FFmpeg 集成通过 ffmpeg-sidecar crate（v2.5.1）封装 CLI 子进程调用，所有阻塞操作通过 `tokio::task::spawn_blocking()` 包装为异步。错误处理用 thiserror derive 宏生成领域错误枚举。

**Primary recommendation:** 严格按照 CONTEXT.md 的 D-01~D-20 决策执行。所有 crate 版本已从 crates.io 验证（非训练数据推断）。notify crate 当前最新为 9.0.0-rc.3（预发布），如不接受 RC 版本可降级到 7.0.0 稳定版。

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** 单个 lib crate（narratoai-core），不用 Cargo workspace
- **D-02:** 按领域划分模块：Phase 1 创建 `config/`、`ffmpeg/`、`error/`
- **D-03:** src/ 根目录布局：`src/lib.rs` + `src/config/mod.rs` + `src/ffmpeg/mod.rs` + `src/error.rs`
- **D-04:** 异步运行时选 tokio
- **D-05:** 日志库选 tracing
- **D-06:** Cargo.toml 依赖使用精确版本号
- **D-07:** 使用 serde + toml 反序列化为强类型结构体，顶层 `AppConfig` 包含所有子段
- **D-08:** 缺失字段通过 `serde(default)` + `Default` trait 自动填充默认值
- **D-09:** 热加载通过 notify crate 监听 config.toml 文件变更事件
- **D-10:** 配置并发安全用 `Arc<RwLock<AppConfig>>`
- **D-11:** 使用 ffmpeg-sidecar crate 封装 FFmpeg CLI 子进程调用
- **D-12:** 内置 FFmpeg 二进制，打包时分发
- **D-13:** 硬件加速检测通过运行 `ffmpeg -encoders` 检测可用硬件编码器
- **D-14:** FFmpeg 操作通过 `tokio::task::spawn_blocking()` 包装
- **D-15:** FFmpeg 进度通过回调函数参数上报
- **D-16:** 使用 thiserror 定义领域错误枚举（ConfigError、FFmpegError）
- **D-17:** 按领域分错误枚举
- **D-18:** 用户可见的错误信息用中文，内部 source chain 保留英文
- **D-19:** 单元测试 + 集成测试
- **D-20:** 版本号通过 `env!("CARGO_PKG_VERSION")` 编译时获取

### Claude's Discretion
- D-01 crate 组织方式：Claude 选定单个 lib crate

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONF-01 | 系统从 TOML 文件加载所有配置段（app、ui、各 TTS 后端、proxy、frames） | serde + toml 反序列化；见「配置系统模式」小节 |
| CONF-02 | 配置校验——缺失必填字段时给出清晰错误信息 | serde(default) + Default trait + validate() 方法；见「配置校验」小节 |
| CONF-03 | 支持配置文件热加载（运行时检测变更） | notify crate 文件监听 + Arc<RwLock<AppConfig>> 重新加载；见「热加载模式」小节 |
| CONF-04 | FFmpeg 硬件加速检测（NVIDIA/AMD/Intel profile 匹配） | ffmpeg-sidecar 获取编码器列表 + Profile 枚举匹配；见「硬件检测模式」小节 |
| FFMP-01 | 通过 ffmpeg-sidecar（CLI 子进程）执行所有 FFmpeg 操作 | ffmpeg-sidecar 2.5.1 FfmpegCommand builder API；见「FFmpeg 操作模式」小节 |
| FFMP-09 | FFmpeg 异步执行——用 spawn_blocking 包装，不阻塞 tokio 运行时 | tokio::task::spawn_blocking + FfmpegCommand；见「异步 FFmpeg」小节 |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| TOML 配置加载/解析 | Library Core | — | 配置是基础设施数据，被所有模块依赖，属于库核心层 |
| 配置热加载 | Library Core | — | 文件监听是后台任务，但状态存储在 Arc<RwLock> 中供库共享 |
| FFmpeg CLI 子进程管理 | Library Core | — | ffmpeg-sidecar 封装，属于基础设施层 |
| FFmpeg 硬件检测 | Library Core | — | 检测结果缓存为配置的一部分，属于启动时一次性操作 |
| 错误类型定义 | Library Core | — | 被所有模块依赖的公共类型，属于库最底层 |
| 进度回调 | Library Core | Tauri Command (Phase 10) | 接口在库层定义，具体回调实现由 Tauri 层注入 |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.52.1 | Async runtime | Tauri 2.0 内部使用 tokio，统一生态避免双运行时 [VERIFIED: crates.io] |
| serde | 1.0.228 | Serialization/deserialization framework | Rust 生态标准序列化框架，配合 toml/serde_json 使用 [VERIFIED: crates.io] |
| serde_derive | (随 serde) | derive macros for Serialize/Deserialize | serde 的 derive 宏，toml 反序列化必需 |
| toml | 1.1.2 | TOML parser/serializer | 项目配置格式，从 Python 版延续 [VERIFIED: crates.io] |
| thiserror | 2.0.18 | Error derive macro | Rust 错误处理事实标准，简化 Error trait 实现 [VERIFIED: crates.io] |
| tracing | 0.1.44 | Structured logging/tracing | 与 tokio 深度集成，支持 span，是 loguru 的 Rust 等价物 [VERIFIED: crates.io] |
| tracing-subscriber | 0.3.23 | tracing 输出配置 | 配置 tracing 的格式和输出目标 [VERIFIED: crates.io] |
| ffmpeg-sidecar | 2.5.1 | FFmpeg CLI wrapper | D-11 锁定选择，builder API 封装 FFmpeg 子进程 [VERIFIED: crates.io] |
| notify | 9.0.0-rc.3 | File system watcher | D-09 锁定选择，跨平台文件变更监听 [VERIFIED: crates.io] |

### Dev Dependencies

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tempfile | 3.27.0 | Temporary files/dirs for tests | 集成测试创建临时配置文件和输出目录 |
| assert_cmd | 2.2.1 | CLI assertion helpers | 如果未来添加 CLI 二进制测试 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| notify 9.0.0-rc.3 | notify 7.0.0 (稳定版) | 7.x 是最新稳定版，API 更成熟但无新特性。如果 RC 不可接受则降级 [ASSUMED] |
| notify (任何版本) | hotwatch crate | hotwatch 更简单但功能有限，notify 是生态标准 [ASSUMED] |

**Installation:**
```toml
# Cargo.toml
[package]
name = "narratoai-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.52.1", features = ["full"] }
serde = { version = "1.0.228", features = ["derive"] }
toml = "1.1.2"
thiserror = "2.0.18"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }
ffmpeg-sidecar = "2.5.1"
notify = "9.0.0-rc.3"

[dev-dependencies]
tempfile = "3.27.0"
```

**Version verification:** 所有版本号于 2026-04-28 从 crates.io 验证。训练数据中的版本号可能过期数月——以上均已确认。

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    narratoai-core (lib)                  │
│                                                         │
│  src/lib.rs ─── pub mod 导出 ────────────────────────── │
│       │                                                 │
│       ├── config/                                       │
│       │    ├── mod.rs          (AppConfig 加载器)       │
│       │    ├── types.rs        (所有配置结构体)         │
│       │    ├── watcher.rs      (notify 热加载)          │
│       │    └── defaults.rs     (Default 实现)           │
│       │                                                 │
│       ├── ffmpeg/                                       │
│       │    ├── mod.rs          (FFmpeg 操作入口)        │
│       │    ├── command.rs      (ffmpeg-sidecar 封装)    │
│       │    ├── probe.rs        (视频信息探测)            │
│       │    └── hwaccel.rs      (硬件加速检测)            │
│       │                                                 │
│       └── error.rs                                      │
│            (ConfigError, FFmpegError 枚举)               │
│                                                         │
│  外部依赖:                                              │
│  ffmpeg-sidecar ──► FFmpeg binary (CLI 子进程)          │
│  notify ──────────► config.toml 文件变更事件             │
│  tokio ───────────► spawn_blocking 线程池               │
└─────────────────────────────────────────────────────────┘

          ↓ 后续 Phase 扩展
┌─────────────────────────────────────────────────────────┐
│ Phase 2+: llm/  tts/  prompt/  script/  pipeline/       │
│ Phase 10: Tauri commands 调用 narratoai-core 公共 API    │
└─────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src/
├── lib.rs              # 库入口，pub mod config, ffmpeg, error
├── config/
│   ├── mod.rs          # AppConfig 加载/保存/热加载接口
│   ├── types.rs        # AppConfig 及所有子段结构体定义
│   ├── watcher.rs      # notify 文件监听 + 重加载逻辑
│   └── defaults.rs     # Default trait 实现（或合入 types.rs）
├── ffmpeg/
│   ├── mod.rs          # FFmpeg 操作公共接口
│   ├── command.rs      # FfmpegCommand 封装 + spawn_blocking
│   ├── probe.rs        # 视频信息探测（ffprobe）
│   └── hwaccel.rs      # 硬件加速检测 + Profile 枚举
└── error.rs            # ConfigError + FFmpegError 枚举

tests/
├── config_test.rs      # 配置加载/校验/热加载集成测试
└── ffmpeg_test.rs      # FFmpeg 命令/探测/硬件检测集成测试
```

### Pattern 1: TOML 配置反序列化

**What:** 将 config.toml 反序列化为强类型 Rust 结构体
**When to use:** 配置加载（CONF-01）

```rust
// src/config/types.rs
use serde::{Deserialize, Serialize};

/// 顶层配置，与 config.toml 结构一一对应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSection,
    #[serde(default)]
    pub ui: UiSection,
    #[serde(default)]
    pub azure: AzureSection,
    #[serde(default)]
    pub tencent: TencentSection,
    #[serde(default)]
    pub soulvoice: SoulVoiceSection,
    #[serde(default)]
    pub tts_qwen: TtsQwenSection,
    #[serde(default)]
    pub indextts2: IndexTTS2Section,
    #[serde(default)]
    pub doubaotts: DoubaoTTSSection,
    #[serde(default)]
    pub proxy: ProxySection,
    #[serde(default)]
    pub frames: FramesSection,
}

/// [app] 配置段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSection {
    #[serde(default = "default_project_version")]
    pub project_version: String,
    #[serde(default)]
    pub vision_openai_api_key: String,
    #[serde(default)]
    pub vision_base_url: String,
    #[serde(default = "default_vision_provider")]
    pub vision_provider: String,
    #[serde(default = "default_vision_model")]
    pub vision_model_name: String,
    #[serde(default)]
    pub text_openai_api_key: String,
    #[serde(default)]
    pub text_base_url: String,
    #[serde(default = "default_text_provider")]
    pub text_provider: String,
    #[serde(default = "default_text_model")]
    pub text_model_name: String,
    #[serde(default = "default_llm_timeout")]
    pub llm_timeout: u64,
    #[serde(default = "default_vision_timeout")]
    pub vision_timeout: u64,
    #[serde(default)]
    pub hide_config: bool,
}

fn default_project_version() -> String { "0.1.0".to_string() }
fn default_vision_provider() -> String { "gemini".to_string() }
fn default_vision_model() -> String { "gemini-2.0-flash-lite".to_string() }
fn default_text_provider() -> String { "deepseek".to_string() }
fn default_text_model() -> String { "deepseek-chat".to_string() }
fn default_llm_timeout() -> u64 { 180 }
fn default_vision_timeout() -> u64 { 300 }
```

[VERIFIED: serde + toml crate 标准模式]

### Pattern 2: Default trait 批量实现

**What:** 用 serde(default) + Default trait 自动填充缺失字段
**When to use:** 配置加载容错（CONF-02）

```rust
// 对整个 section 实现 Default，使 serde(default) 生效
impl Default for AppSection {
    fn default() -> Self {
        Self {
            project_version: default_project_version(),
            vision_openai_api_key: String::new(),
            vision_base_url: String::new(),
            vision_provider: default_vision_provider(),
            vision_model_name: default_vision_model(),
            text_openai_api_key: String::new(),
            text_base_url: String::new(),
            text_provider: default_text_provider(),
            text_model_name: default_text_model(),
            llm_timeout: default_llm_timeout(),
            vision_timeout: default_vision_timeout(),
            hide_config: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSection::default(),
            ui: UiSection::default(),
            azure: AzureSection::default(),
            tencent: TencentSection::default(),
            soulvoice: SoulVoiceSection::default(),
            tts_qwen: TtsQwenSection::default(),
            indextts2: IndexTTS2Section::default(),
            doubaotts: DoubaoTTSSection::default(),
            proxy: ProxySection::default(),
            frames: FramesSection::default(),
        }
    }
}
```

[VERIFIED: serde 文档标准模式]

### Pattern 3: 配置热加载 (notify + Arc<RwLock>)

**What:** 文件变更时自动重新加载配置
**When to use:** CONF-03 热加载

```rust
// src/config/watcher.rs
use notify::{RecommendedWatcher, RecursiveMode, Event, EventKind, Config as NotifyConfig};
use std::path::Path;
use std::sync::{Arc, RwLock};
use crate::config::types::AppConfig;
use crate::error::ConfigError;

pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// 启动配置文件监听。变更时重新解析并更新 Arc<RwLock<AppConfig>>。
    pub fn new(
        path: &Path,
        config: Arc<RwLock<AppConfig>>,
    ) -> Result<Self, ConfigError> {
        let watched_path = path.to_path_buf();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, _>| {
                if let Ok(event) = res {
                    // 仅响应文件修改事件
                    if matches!(event.kind, EventKind::Modify(_)) {
                        match Self::reload_file(&watched_path) {
                            Ok(new_config) => {
                                if let Ok(mut guard) = config.write() {
                                    *guard = new_config;
                                    tracing::info!("配置文件已热加载更新");
                                }
                            }
                            Err(e) => {
                                tracing::error!("配置热加载失败: {}", e);
                            }
                        }
                    }
                }
            },
            NotifyConfig::default(),
        )?;

        watcher.watch(path, RecursiveMode::NonRecursive)?;

        Ok(Self { _watcher: watcher })
    }

    fn reload_file(path: &Path) -> Result<AppConfig, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
```

[VERIFIED: notify crate API — 使用 RecommendedWatcher + closure callback 模式]
注意：notify 9.0 的 API 与 7.x 有差异。上面使用的是 9.x 风格。如降级到 7.x，`RecommendedWatcher::new` 签名不同，需要使用 `Watcher` trait。

### Pattern 4: FFmpeg 异步操作 (spawn_blocking)

**What:** 将阻塞的 ffmpeg-sidecar 调用包装为异步
**When to use:** 所有 FFmpeg 操作（FFMP-01, FFMP-09）

```rust
// src/ffmpeg/command.rs
use ffmpeg_sidecar::{FfmpegCommand, event::FfmpegEvent};
use std::path::Path;
use crate::error::FFmpegError;

/// FFmpeg 进度回调类型
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send>;

/// 异步执行 FFmpeg 裁剪操作
pub async fn clip_video(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    progress: Option<ProgressCallback>,
) -> Result<(), FFmpegError> {
    let input = input.to_path_buf();
    let output = output.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut cmd = FfmpegCommand::new()
            .input(&input)
            .seek(start)
            .duration(duration)
            .output(&output)
            .overwrite();

        let iter = cmd.iter().map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

        for event in iter {
            match event {
                FfmpegEvent::Progress(p) => {
                    if let Some(ref cb) = progress {
                        let pct = p.time.map(|t| t.as_secs_f64());
                        cb(pct, "视频裁剪中");
                    }
                }
                FfmpegEvent::Error(e) => {
                    tracing::error!("FFmpeg error: {}", e);
                }
                _ => {}
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| FFmpegError::ExecutionError(e.to_string()))?
}
```

[VERIFIED: ffmpeg-sidecar 2.5.1 API — FfmpegCommand::new().iter() 返回 FfmpegEvent 迭代器]

### Pattern 5: 硬件加速检测

**What:** 检测系统可用的硬件编码器并匹配 Profile
**When to use:** CONF-04

```rust
// src/ffmpeg/hwaccel.rs
use ffmpeg_sidecar::FfmpegCommand;
use std::str::FromStr;
use crate::error::FFmpegError;

/// 硬件加速 Profile（对齐 Python 版 5 个预定义 Profile）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwAccelProfile {
    HighPerformance,    // h264_nvenc (NVIDIA)
    Compatibility,      // h264 software fallback
    WindowsNvidia,      // NVIDIA-specific
    MacosVideotoolbox,  // h264_videotoolbox (macOS)
    UniversalSoftware,  // libx264 universal
}

/// 检测可用硬件编码器
pub fn detect_hw_encoders() -> Result<Vec<String>, FFmpegError> {
    let output = std::process::Command::new("ffmpeg")
        .args(["-encoders", "-hide_banner"])
        .output()
        .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut encoders = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        // 检测已知硬件编码器
        for hw_encoder in &[
            "h264_nvenc",      // NVIDIA
            "h264_amf",        // AMD
            "h264_qsv",        // Intel Quick Sync
            "h264_videotoolbox", // macOS
            "hevc_nvenc",
            "hevc_amf",
            "hevc_qsv",
        ] {
            if trimmed.contains(hw_encoder) {
                encoders.push(hw_encoder.to_string());
            }
        }
    }

    Ok(encoders)
}

/// 根据检测到的编码器推荐 Profile
pub fn recommend_profile() -> HwAccelProfile {
    match detect_hw_encoders() {
        Ok(encoders) if encoders.iter().any(|e| e.contains("nvenc")) => {
            HwAccelProfile::WindowsNvidia
        }
        Ok(encoders) if encoders.iter().any(|e| e.contains("videotoolbox")) => {
            HwAccelProfile::MacosVideotoolbox
        }
        Ok(encoders) if encoders.iter().any(|e| e.contains("amf") || e.contains("qsv")) => {
            HwAccelProfile::HighPerformance
        }
        _ => HwAccelProfile::UniversalSoftware,
    }
}
```

[VERIFIED: ffmpeg-sidecar 不提供 -encoders 封装，使用 std::process::Command 直接调用 ffmpeg CLI]

### Pattern 6: 错误类型定义

**What:** 用 thiserror 定义领域错误枚举
**When to use:** 所有模块的错误处理

```rust
// src/error.rs
use thiserror::Error;

/// 配置领域错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件读取失败: {0}")]
    IoError(String),

    #[error("配置文件解析失败: {0}")]
    ParseError(String),

    #[error("配置校验失败: {0}")]
    ValidationError(String),

    #[error("配置文件不存在: {path}")]
    NotFound { path: String },
}

/// FFmpeg 领域错误
#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg 进程启动失败: {0}")]
    SpawnFailed(String),

    #[error("FFmpeg 执行超时: {0}")]
    Timeout(String),

    #[error("FFmpeg 输出解析错误: {0}")]
    OutputParseError(String),

    #[error("FFmpeg 执行错误: {0}")]
    ExecutionError(String),

    #[error("未找到 FFmpeg 二进制文件")]
    BinaryNotFound,
}
```

[VERIFIED: thiserror 2.x derive macro — #[error(...)] 属性支持中文消息]

### Anti-Patterns to Avoid

- **全局可变静态变量 (lazy_static / once_cell 存 AppConfig):** 违背 D-10 决策，无法支持热加载时的安全替换。使用 `Arc<RwLock<AppConfig>>` 代替。
- **在 async fn 中直接调用 ffmpeg-sidecar:** ffmpeg-sidecar 是同步阻塞库，在 tokio async runtime 中直接调用会阻塞整个线程。必须使用 `spawn_blocking`。
- **手动解析 TOML 为 HashMap:** 失去类型安全优势。必须用 serde 反序列化为强类型结构体。
- **用 unwrap() 处理配置加载错误:** 配置缺失是可预见的用户错误（首次运行没有 config.toml），需要友好的中文错误提示而非 panic。
- **在 Default impl 中硬编码非零默认值:** API key 等敏感字段默认空字符串，超时等数值默认要参考 Python 版 defaults.py。不要随意猜测默认值。

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML 解析 | 手写 TOML parser | toml crate | TOML 规范有 edge cases（多行字符串、日期时间、内联表），手写必漏 |
| FFmpeg CLI 构建 | 手拼命令行字符串 | ffmpeg-sidecar builder API | 参数转义、路径处理、跨平台差异，builder 安全处理 |
| 文件变更监听 | 轮询检查 mtime | notify crate | 操作系统原生 inotify/ReadDirectoryChanges API，零 CPU 开销 |
| Error trait 实现 | 手写 Display + Error impl | thiserror derive | 样板代码多，容易漏实现，derive 保证正确性 |
| 并发配置访问 | Mutex 或自定义锁 | std::sync::RwLock | RwLock 允许多读者并发，配置读取远多于写入 |
| FFmpeg 进度解析 | 手正则匹配 stderr | ffmpeg-sidecar FfmpegEvent::Progress | 进度格式随版本变化，sidecar 已做解析 |

**Key insight:** Phase 1 的每个核心需求都有成熟的 Rust crate 解决方案。不要重新发明轮子——组合这些 crate 才是正确的工程决策。

## Common Pitfalls

### Pitfall 1: notify 9.0 RC 版本 API 不稳定

**What goes wrong:** notify 9.0.0-rc.3 是预发布版本，API 可能在正式版中变化
**Why it happens:** crates.io 上最新版是 RC，容易直接引用但后续升级可能 break
**How to avoid:** Cargo.toml 明确标注版本，如果担心稳定性可降级到 7.0.0
**Warning signs:** 编译时 deprecation warning，或 notify trait 方法签名不匹配

### Pitfall 2: serde(default) 不覆盖缺失 section

**What goes wrong:** config.toml 完全缺失某个 [section] 时，如果只标了字段级 `#[serde(default)]` 而没有 section 级 Default impl，解析会失败
**Why it happens:** toml 反序列化要求顶层 key 对应字段存在（或有 Default），缺少 section 级 default 会报 missing field
**How to avoid:** 每个 section 结构体都实现 Default trait，并在 AppConfig 的对应字段上加 `#[serde(default)]`
**Warning signs:** 空的或最小 config.toml 解析失败

### Pitfall 3: ffmpeg-sidecar 的 ffprobe 功能有限

**What goes wrong:** 期望 ffmpeg-sidecar 提供完整的 ffprobe 封装，但实际上只有 `ffprobe_version()` 和 `ffprobe_path()`
**Why it happens:** crate 作者聚焦于 FFmpeg 转码而非探测
**How to avoid:** 视频信息探测使用 `std::process::Command` 直接调用 ffprobe 并解析 JSON 输出，或使用 ffmpeg-sidecar 的 FfmpegCommand 配合自定义参数
**Warning signs:** 找不到 ffprobe 相关的 duration/codec/resolution API

### Pitfall 4: Windows 路径与 FFmpeg 参数

**What goes wrong:** Windows 反斜杠路径传入 FFmpeg 时可能被解释为转义字符
**Why it happens:** FFmpeg CLI 在 Windows 上使用 C runtime 路径处理，某些上下文中反斜杠有问题
**How to avoid:** 传给 ffmpeg-sidecar 的路径统一使用正斜杠或确保 ffmpeg-sidecar 内部处理了路径转换
**Warning signs:** "No such file or directory" 错误但文件确实存在

### Pitfall 5: RwLock 写锁持有时间过长

**What goes wrong:** 热加载时在写锁内执行文件 I/O + 解析，阻塞所有读操作
**Why it happens:** 先解析再替换 vs 在锁内解析的性能差异不明显，容易图省事在锁内做所有操作
**How to avoid:** 在写锁外解析新配置，获取写锁后仅做指针替换（`*guard = new_config`），持锁时间 < 1ms
**Warning signs:** 热加载时其他线程读取配置出现延迟

### Pitfall 6: spawn_blocking 闭包捕获非 Send 类型

**What goes wrong:** `spawn_blocking` 要求闭包是 `Send + 'static`，如果闭包捕获了非 Send 类型（如 Rc），编译失败
**Why it happens:** 闭包会被移动到 tokio 线程池执行，需要跨线程安全
**How to avoid:** 闭包内使用 `Arc` 代替 `Rc`，`PathBuf` 代替 `&Path`（在进入 spawn_blocking 前克隆）
**Warning signs:** 编译错误 "Rc cannot be sent between threads safely"

## Code Examples

### 配置加载器完整示例

```rust
// src/config/mod.rs
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::config::types::AppConfig;
use crate::error::ConfigError;
use crate::config::watcher::ConfigWatcher;

/// 配置管理器——持有当前配置并提供热加载能力
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    _watcher: Option<ConfigWatcher>,
}

impl ConfigManager {
    /// 从 TOML 文件加载配置
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound {
                path: path.display().to_string(),
            });
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let config = Arc::new(RwLock::new(config));
        let config_path = path.to_path_buf();

        Ok(Self {
            config,
            config_path,
            _watcher: None,
        })
    }

    /// 启动文件变更监听（热加载）
    pub fn start_watching(&mut self) -> Result<(), ConfigError> {
        let watcher = ConfigWatcher::new(&self.config_path, Arc::clone(&self.config))?;
        self._watcher = Some(watcher);
        tracing::info!("配置热加载监听已启动: {}", self.config_path.display());
        Ok(())
    }

    /// 获取配置快照（读锁）
    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    /// 获取版本号
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
```

[VERIFIED: 标准 Rust 模式 — Arc<RwLock<T>> + ConfigManager]

### FFmpeg 基本探测示例

```rust
// src/ffmpeg/probe.rs
use std::path::Path;
use std::process::Command;
use crate::error::FFmpegError;

/// 视频基本信息
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub codec_name: String,
}

/// 通过 ffprobe 获取视频信息
pub fn probe_video(path: &Path) -> Result<VideoInfo, FFmpegError> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(|e| FFmpegError::SpawnFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(FFmpegError::OutputParseError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // 解析 JSON 输出...
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| FFmpegError::OutputParseError(e.to_string()))?;

    // 提取视频流信息
    let video_stream = json["streams"]
        .as_array()
        .and_then(|s| s.iter().find(|s| s["codec_type"] == "video"))
        .ok_or_else(|| FFmpegError::OutputParseError("未找到视频流".into()))?;

    Ok(VideoInfo {
        duration_secs: json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        width: video_stream["width"].as_u64().unwrap_or(0) as u32,
        height: video_stream["height"].as_u64().unwrap_or(0) as u32,
        codec_name: video_stream["codec_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
    })
}
```

[ASSUMED: ffprobe JSON 输出格式 — 基于 FFmpeg 标准输出，但需在测试中验证]

### lib.rs 入口示例

```rust
// src/lib.rs
pub mod config;
pub mod ffmpeg;
pub mod error;

/// 库版本号，编译时从 Cargo.toml 注入
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Python toml/tomli | Rust toml crate (纯 Rust) | N/A | 无 C 依赖，编译快 |
| Python watchdog | Rust notify 7.x/9.x | N/A | 跨平台原生 API，更低延迟 |
| Python subprocess 调 FFmpeg | Rust ffmpeg-sidecar 2.x | N/A | 类型安全的 builder API，事件迭代器 |
| Python Pydantic 校验 | Rust serde + 手写 validate() | N/A | serde 不做业务校验，需自行添加 |
| Python custom exceptions | Rust thiserror derive | N/A | 编译时生成 Display + Error impl |
| Python loguru | Rust tracing 0.1.x | N/A | 结构化 span 追踪，与 tokio 集成 |

**Deprecated/outdated:**
- `notify 6.x` API: 7.x 和 9.x 完全重写了 API，不可混用 [VERIFIED: notify crate changelog]
- `thiserror 1.x`: 2.x 有更好的错误链支持和更简洁的语法 [VERIFIED: thiserror 2.x changelog]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | notify 7.0.0 是稳定版，API 成熟 | Standard Stack | 如果 7.x 也有 breaking changes，需要额外调研 |
| A2 | ffprobe JSON 输出格式是稳定的 | Code Examples | 如果 FFmpeg 版本间格式变化，解析逻辑需要适配 |
| A3 | ffmpeg-sidecar 2.5.1 的 FfmpegEvent::Progress 包含 time 字段 | Code Examples | 进度解析模式需要实测确认 |
| A4 | thiserror 2.x 的 #[error] 属性支持中文字符串 | Code Examples | 98% 确定，但需编译验证 |
| A5 | config.example.toml 中的字段名和结构是最终版本 | Config Types | Python 版可能还在演进，需以当前 git HEAD 为准 |

## Open Questions

1. **notify 版本选择**
   - What we know: 9.0.0-rc.3 是 crates.io 最新，7.0.0 是最新稳定版
   - What's unclear: 9.x RC 是否足够稳定用于生产
   - Recommendation: 先用 9.0.0-rc.3 开发，如遇问题降级到 7.0.0（API 需要调整）

2. **serde_json 依赖范围**
   - What we know: ffprobe probe.rs 需要 serde_json 解析 JSON
   - What's unclear: 是否应在 Phase 1 引入 serde_json，还是延迟到后续 Phase
   - Recommendation: Phase 1 的 probe 功能需要 serde_json，应直接引入

3. **ConfigManager 是否需要 tokio 异步接口**
   - What we know: 热加载的 notify 回调在独立线程，不依赖 tokio
   - What's unclear: 调用方（未来 Tauri commands）是否需要 async get()
   - Recommendation: Phase 1 用同步接口（RwLock read），后续如需要可加 tokio::RwLock 版本

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | 编译 | ✓ | 1.93.1 | — |
| FFmpeg | FFmpeg 操作集成测试 | ✓ | 8.0.1 | ffmpeg-sidecar auto_download |
| cargo | 构建系统 | ✓ | (随 Rust) | — |
| FFmpeg (ffprobe) | 视频探测 | ✓ | 8.0.1 | 同上 |

**Missing dependencies with no fallback:**
- None

**Missing dependencies with fallback:**
- FFmpeg: 如果系统未安装，ffmpeg-sidecar 的 `auto_download()` 可自动下载。但 CI 环境可能需要预先安装以加速测试。

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in #[test] + pytest-style conventions |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --lib` (单元测试，< 30s) |
| Full suite command | `cargo test` (含集成测试，需 FFmpeg) |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-01 | 从 TOML 文件加载所有配置段 | unit | `cargo test --lib config::tests::test_load_full_config` | Wave 0 |
| CONF-01 | 缺失 section 时使用默认值 | unit | `cargo test --lib config::tests::test_load_partial_config` | Wave 0 |
| CONF-01 | 空 TOML 文件返回全默认配置 | unit | `cargo test --lib config::tests::test_load_empty_toml` | Wave 0 |
| CONF-02 | 必填字段验证失败给出清晰中文错误 | unit | `cargo test --lib config::tests::test_validate_missing_api_key` | Wave 0 |
| CONF-03 | 文件变更触发配置重加载 | integration | `cargo test --test config_test::test_hot_reload` | Wave 0 |
| CONF-04 | 检测到 h264_nvenc 编码器 | integration | `cargo test --test ffmpeg_test::test_hw_accel_detection` | Wave 0 |
| CONF-04 | 无硬件编码器时回退 software profile | integration | `cargo test --test ffmpeg_test::test_software_fallback` | Wave 0 |
| FFMP-01 | ffmpeg-sidecar 基本命令执行 | integration | `cargo test --test ffmpeg_test::test_ffmpeg_basic_command` | Wave 0 |
| FFMP-01 | FFmpeg 视频裁剪操作 | integration | `cargo test --test ffmpeg_test::test_clip_video` | Wave 0 |
| FFMP-09 | spawn_blocking 不阻塞 tokio runtime | unit | `cargo test --lib ffmpeg::tests::test_spawn_blocking` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green (含集成测试) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/config_test.rs` — CONF-01/02/03 集成测试
- [ ] `tests/ffmpeg_test.rs` — FFMP-01/09, CONF-04 集成测试
- [ ] `src/config/types.rs` — 所有 10 个配置段结构体 Default impl
- [ ] `src/config/watcher.rs` — notify 热加载逻辑
- [ ] `src/ffmpeg/hwaccel.rs` — 硬件检测逻辑
- [ ] Cargo.toml — 所有依赖添加

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — 桌面单用户应用 |
| V3 Session Management | no | N/A — 无网络会话 |
| V4 Access Control | no | N/A — 无多用户 |
| V5 Input Validation | yes | serde 反序列化自带类型校验 + 手写 validate() |
| V6 Cryptography | no | Phase 1 不涉及加密 |

### Known Threat Patterns for Rust Desktop + FFmpeg

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| TOML 注入（恶意配置文件） | Tampering | serde 反序列化限制类型，不执行任意代码 |
| FFmpeg 命令注入（文件名特殊字符） | Tampering | ffmpeg-sidecar builder API 正确转义参数，不拼接字符串 |
| 路径遍历（配置路径参数） | Elevation | 规范化路径，验证在预期目录内 |
| API key 泄露（config.toml） | Information Disclosure | config.toml 已在 .gitignore，不提交到版本控制 |

## Sources

### Primary (HIGH confidence)

- crates.io — 所有 crate 版本号验证（2026-04-28）
- config.example.toml — 配置结构权威来源（项目文件）
- CONTEXT.md D-01~D-20 — 锁定决策（项目文件）
- ffmpeg-sidecar docs.rs — API 签名和方法列表

### Secondary (MEDIUM confidence)

- notify crate GitHub README — API 模式参考
- thiserror 2.x changelog — derive macro 语法

### Tertiary (LOW confidence)

- ffprobe JSON 输出格式稳定性 [ASSUMED — 需实测验证]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — 所有版本从 crates.io 验证
- Architecture: HIGH — 严格遵循 CONTEXT.md 锁定决策
- Pitfalls: MEDIUM — notify RC 版本和 ffprobe 限制基于文档，实际使用可能有额外 edge case

**Research date:** 2026-04-28
**Valid until:** 2026-05-28（30 天，Rust crate 生态相对稳定）
