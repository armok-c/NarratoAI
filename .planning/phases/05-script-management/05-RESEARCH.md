# Phase 5: Script Management - Research

**Researched:** 2026-04-28
**Domain:** JSON 脚本数据模型 / serde 反序列化 / 不可变编辑 API / 校验规则
**Confidence:** HIGH

## Summary

Phase 5 为 NarratoAI Rust 重写项目建立脚本管理基础设施。脚本是一个 JSON 数组，每个元素包含 5 个核心字段（`_id`, `timestamp`, `picture`, `narration`, `OST`）和 6 个管道扩展字段（`duration`, `sourceTimeRange`, `editedTimeRange`, `audio`, `video`, `subtitle`）。Python 版的权威实现在 `app/utils/check_script.py:check_format()`、`app/models/schema.py:VideoClipParams` 和 `app/services/update_script.py:update_script_timestamps()` 中。

Rust 实现需要三个核心组件：(1) `ScriptClip` 结构体通过 serde 从 JSON 反序列化，OST 用 `serde_repr` 序列化为 0/1/2 整数；(2) `Script` 类型别名（`Vec<ScriptClip>`）提供不可变更新方法；(3) `validate()` 函数对齐 Python 版校验规则，收集所有错误一次性返回。所有 serde 模式与 Phase 1 `config/types.rs` 的 `#[derive(Serialize, Deserialize)]` + `#[serde(default)]` 模式一致。

**Primary recommendation:** 使用 serde + serde_json + serde_repr 三个 crate 实现 JSON 脚本的完整往返（round-trip），用 `#[serde(skip_serializing_if = "Option::is_none")]` 控制保存时仅输出核心字段。serde_json 默认保留非 ASCII 字符，与 Python 版 `ensure_ascii=False` 行为天然一致。

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** 单个 `ScriptClip` 结构体包含所有字段（5 个核心 + 6 个管道扩展），管道扩展字段用 `Option<T>`。加载时核心字段必须有值，管道字段为 None。
- **D-02:** OST 字段用 Rust 枚举 `OstType { NarrationOnly, OriginalSound, Mixed }`，通过 `serde_repr` 序列化为 0/1/2 整数。
- **D-03:** timestamp 字段保留原始字符串格式（`HH:MM:SS,mmm-HH:MM:SS,mmm`），不解析为 Duration 类型。通过校验函数验证格式正确性。
- **D-04:** 顶层脚本类型为 `type Script = Vec<ScriptClip>`（类型别名），不包装元数据结构。与 Python 版纯数组格式一致。
- **D-05:** `_id` 字段类型为 `i64`（非 usize），与 JSON 数值兼容。
- **D-06:** `picture` 和 `narration` 为 `String` 类型。管道扩展字段：`duration: Option<f64>`, `source_time_range: Option<String>`, `edited_time_range: Option<String>`, `audio: Option<PathBuf>`, `video: Option<PathBuf>`, `subtitle: Option<PathBuf>`。
- **D-07:** 只支持加载纯 JSON 数组格式（`[{...}, {...}]`）。不处理 LLM 输出的 `{"items": [...]}` 信封——信封由各管道模块在调用 Script 加载前自行剥离。
- **D-08:** 保存时只输出核心字段（`_id`, `timestamp`, `picture`, `narration`, `OST`），忽略 Option 管道扩展字段。与 Python 版脚本保存行为一致。
- **D-09:** 保存输出美化 JSON（indent=2），非 ASCII 字符不转义（`ensure_ascii=false` 行为）。与 Python 版 `json.dumps(indent=2, ensure_ascii=False)` 一致。
- **D-10:** 加载/保存使用 UTF-8 编码。与 Python 版 `encoding="utf-8"` 一致。
- **D-11:** 校验级别与 Python 版 `check_script.py` 对齐：JSON 数组、每个元素有全部 5 个核心字段、`_id` 为正整数、timestamp 格式匹配正则 `^\d{2}:\d{2}:\d{2},\d{3}-\d{2}:\d{2}:\d{2},\d{3}$`、picture/narration 非空字符串、OST 为 0/1/2 有效值。
- **D-12:** 加载时自动校验（`load_script()` 内部调用 `validate()`），校验失败返回 `ScriptError`。不需要显式调用校验方法（但可提供 `validate()` 公开方法供复用）。
- **D-13:** 校验失败时收集所有错误（`Vec<ValidationError>`），一次性返回。不采用第一个错误即停策略。
- **D-14:** 用户可见的错误信息用中文（如"脚本第 2 段缺少必要字段: timestamp"），与 Phase 1 D-18 决策一致。
- **D-15:** Script 提供不可变更新方法，每个方法返回新的 `Script` 实例：`update_narration()`, `set_ost()`, `update_timestamp()`, `update_picture()`。
- **D-16:** 用数组下标 `usize` 定位 clip，不用 `_id` 查找。
- **D-17:** Phase 5 只实现单字段更新。不包含增/删/重排操作。
- **D-18:** 索引越界时返回 `ScriptError::IndexOutOfBounds`。
- **D-19:** `ScriptError` 枚举包含变体：`JsonParse(json Error)`, `Validation(Vec<ValidationError>)`, `Io(io::Error)`, `IndexOutOfBounds`, `InvalidTimestamp`。使用 thiserror 派生。

### Claude's Discretion
- ScriptClip 的 serde 属性命名策略（snake_case vs 原始 camelCase 字段名如 sourceTimeRange）
- ValidationError 的具体结构和 Display 实现
- 是否提供 `ScriptClip::builder()` 便利方法
- 模块内部文件拆分方式

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCRP-01 | JSON 脚本加载/保存——与 Python 版格式兼容 | serde + serde_json 反序列化/序列化；serde_repr 处理 OST 整数枚举；serde_json::to_string_pretty 实现 indent=2；serde_json 默认保留非 ASCII 字符（等同于 ensure_ascii=False） |
| SCRP-02 | 脚本编辑——修改段落文案、OST 类型、时间戳 | 不可变更新方法（返回新的 Script 实例）；OstType 枚举提供类型安全的 OST 修改；usize 下标定位 + 越界保护 |
| SCRP-03 | 脚本校验——检查必要字段、时间戳合理性 | validate() 函数对齐 Python 版 check_format() 规则；regex 校验 timestamp 格式；Vec<ValidationError> 收集所有错误 |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JSON 反序列化 | API / Backend | — | 数据解析是后端库的核心职责 |
| 脚本数据模型 | API / Backend | — | 领域模型属于后端库 crate |
| 校验逻辑 | API / Backend | — | 业务规则校验在后端执行 |
| 文件 I/O（加载/保存） | API / Backend | — | 文件系统操作由后端库处理 |
| 编辑 API | API / Backend | — | 不可变更新方法属于后端库公开接口 |
| Tauri 命令调用 | Frontend Server | API / Backend | Phase 10 时 Tauri 命令层调用后端编辑 API |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0.228 (已有) | Serialize/Deserialize 派生宏 | Phase 1 已引入，Rust 序列化生态标准 |
| serde_json | 1.0.140 (已有) | JSON 序列化/反序列化 | Phase 1 已引入，serde 生态的 JSON 实现 |
| serde_repr | 0.1.20 | OST 枚举序列化为整数 | C-like enum 序列化为数字的标准方案 [VERIFIED: crates.io] |
| thiserror | 2.0.18 (已有) | 错误枚举派生宏 | Phase 1 已引入，ScriptError 遵循相同模式 |
| regex | 最新 | timestamp 格式校验正则匹配 | Rust 标准正则库（如不用 regex，也可手写校验） |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tempfile | 3.27.0 (已有 dev-dep) | 测试临时文件 | 单元测试和集成测试 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| serde_repr | 手写 serde serialize/deserialize | serde_repr 是一行 derive 的事，手写易出错且不必要 |
| regex crate | 手写 timestamp 格式校验函数 | timestamp 格式固定（`HH:MM:SS,mmm-HH:MM:SS,mmm`），手写拆分+数值范围校验也可行，避免引入 regex 依赖 |
| serde_with skip_serializing_none | 逐字段 skip_serializing_if | serde_with 需额外依赖，逐字段写更明确且只有 6 个 Option 字段 |

**Installation:**
```bash
# 仅需添加 serde_repr 和 regex（如果选用）
cargo add serde_repr
# 可选：如果选择用 regex 校验 timestamp
cargo add regex
```

**版本验证:**
- serde_repr 0.1.20 [VERIFIED: crates.io search 2026-04-28]
- serde_json 1.0.140 已在 Cargo.toml [VERIFIED: Cargo.toml]
- serde 1.0.228 已在 Cargo.toml [VERIFIED: Cargo.toml]

## Architecture Patterns

### System Architecture Diagram

```
调用方 (Pipeline / Tauri)
    |
    v
+------------------+
| script/mod.rs    | --- 公开 API 入口
| load_script()    | --- 从文件加载 JSON -> Script
| save_script()    | --- Script -> 美化 JSON -> 文件
| validate()       | --- Script -> Vec<ValidationError>
+--------+---------+
         |
         v
+------------------+
| script/types.rs  | --- ScriptClip 结构体
| OstType 枚举     | --- serde_repr 0/1/2
| Script 类型别名   | --- Vec<ScriptClip>
| ValidationError  | --- 校验错误详情
+--------+---------+
         |
         v
+------------------+
| script/error.rs  | --- ScriptError 枚举
| thiserror 派生   | --- 中文错误信息
+------------------+
         |
         v
+------------------+
| script/edit.rs   | --- 不可变更新方法
| update_narration | --- 返回新 Script
| set_ost          | --- 返回新 Script
| update_timestamp | --- 返回新 Script
| update_picture   | --- 返回新 Script
+------------------+
```

### Recommended Project Structure
```
src/
├── script/              # Phase 5 新增模块
│   ├── mod.rs           # 模块入口 + load_script/save_script/validate
│   ├── types.rs         # ScriptClip, OstType, ValidationError, Script 类型别名
│   ├── error.rs         # ScriptError 枚举
│   └── edit.rs          # 不可变编辑方法（impl Script 的 update_* 方法）
├── config/              # Phase 1 已有
├── ffmpeg/              # Phase 1 已有
├── error.rs             # Phase 1 已有（ConfigError, FFmpegError）
└── lib.rs               # 添加 pub mod script;
```

### Pattern 1: Serde 反序列化 + serde_repr 枚举
**What:** ScriptClip 通过 serde derive 从 JSON 反序列化，OstType 用 serde_repr 序列化为整数
**When to use:** 所有 JSON 脚本的加载/保存场景
**Example:**
```rust
// 来源: [CITED: serde.rs/enum-number.html] + [CITED: docs.rs/serde_repr]
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// OST 类型——决定视频裁剪行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OstType {
    /// 仅保留解说音频，根据 TTS 时长裁剪
    NarrationOnly = 0,
    /// 保留原声，按脚本 timestamp 精确裁剪
    OriginalSound = 1,
    /// 保留解说+原声（混合），根据 TTS 时长裁剪
    Mixed = 2,
}

/// 脚本片段——与 Python 版 VideoClipParams 对齐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptClip {
    pub _id: i64,
    pub timestamp: String,
    pub picture: String,
    pub narration: String,
    pub OST: OstType,
    // 管道扩展字段——加载时为 None，管道处理时填充
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(
        rename = "sourceTimeRange",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_time_range: Option<String>,
    #[serde(
        rename = "editedTimeRange",
        skip_serializing_if = "Option::is_none"
    )]
    pub edited_time_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<std::path::PathBuf>,
}

/// 脚本 = 片段数组（与 Python 版纯数组格式一致）
pub type Script = Vec<ScriptClip>;
```

### Pattern 2: 不可变更新方法
**What:** 编辑方法返回新的 Script 实例，不修改原始数据
**When to use:** 所有脚本编辑操作
**Example:**
```rust
// 基于项目规则：不可变更新模式
fn update_narration(script: &Script, index: usize, text: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    let mut new_script = script.clone();
    new_script[index].narration = text.to_string();
    Ok(new_script)
}
```

### Pattern 3: 校验错误收集
**What:** 遍历所有片段，收集所有校验错误后一次性返回
**When to use:** 脚本校验场景
**Example:**
```rust
fn validate(script: &Script) -> Result<(), Vec<ValidationError>> {
    let errors: Vec<ValidationError> = script.iter().enumerate()
        .flat_map(|(i, clip)| validate_clip(i, clip))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Anti-Patterns to Avoid
- **直接反序列化不校验:** JSON 可能缺少核心字段或 OST 值为无效整数（如 5），必须在 serde 之外做业务校验
- **可变更新方法:** `&mut Script` 方法违反项目不可变原则，必须返回新实例
- **手工拼接 JSON 字符串:** 必须用 serde_json，不要用 format! 拼接
- **Option 字段序列化为 null:** 保存时必须省略 Option::None 字段，不能输出 `"audio": null`

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON 解析 | 手写 JSON 解析器 | serde_json | 边界情况无穷（转义、Unicode、嵌套），serde_json 已处理所有 edge case |
| 枚举整数序列化 | 手写 custom serde serializer | serde_repr derive | 一行 derive 解决，手写易出错 |
| timestamp 格式校验 | 手写正则引擎 | regex crate 或手写简单校验函数 | 正则引擎复杂度高，但 timestamp 格式足够简单，两种方案皆可 |
| JSON 美化输出 | 手写缩进/格式化 | serde_json::to_string_pretty | 处理所有缩进、换行、转义细节 |
| 非 ASCII 处理 | 手写 Unicode 转义/不转义逻辑 | serde_json 默认行为 | serde_json 默认不转义非 ASCII 字符 [VERIFIED: serde_json 默认行为 + GitHub issue #907] |

**Key insight:** serde_json 的默认行为恰好与 Python 版 `json.dumps(ensure_ascii=False)` 一致——非 ASCII 字符原样输出。不需要额外配置。

## Common Pitfalls

### Pitfall 1: serde_repr 反序列化遇到无效整数值
**What goes wrong:** JSON 中 OST 字段为 5（超出 0/1/2 范围），serde_repr 反序列化直接失败，错误信息不友好
**Why it happens:** serde_repr 的 Deserialize_repr 严格检查值是否匹配枚举变体
**How to avoid:** 在 serde 反序列化之前或之后添加 OstType 值范围校验。可以考虑用 `try_from` 或在 validate() 中检查原始整数值。或者让 serde_repr 反序列化失败的消息包含上下文（第几段、当前值）。
**Warning signs:** 反序列化错误信息缺少片段位置信息

### Pitfall 2: 管道扩展字段的 camelCase 命名
**What goes wrong:** Python 版 JSON 使用 `sourceTimeRange` 和 `editedTimeRange`（camelCase），Rust 惯例是 snake_case
**Why it happens:** serde 默认使用 Rust 字段名作为 JSON key
**How to avoid:** 使用 `#[serde(rename = "sourceTimeRange")]` 映射字段名。这是 Claude's Discretion 区域，但推荐使用 rename 以保持 JSON 兼容性。
**Warning signs:** 反序列化后 source_time_range 为 None（字段未匹配）

### Pitfall 3: 保存时 Option::None 序列化为 JSON null
**What goes wrong:** 未加 `skip_serializing_if` 时，`None` 字段序列化为 `"audio": null`，Python 版不输出这些字段
**Why it happens:** serde 默认序列化 Option::None 为 null
**How to avoid:** 所有 Option 字段加 `#[serde(skip_serializing_if = "Option::is_none")]` [VERIFIED: serde.rs/field-attrs.html]
**Warning signs:** 保存的 JSON 包含 `"audio": null, "video": null` 等字段

### Pitfall 4: serde_json 序列化顺序不确定
**What goes wrong:** struct 字段的序列化顺序依赖 Rust 结构体定义顺序，如果字段顺序与 Python 版不同，JSON diff 会很大
**Why it happens:** serde 按结构体字段定义顺序序列化
**How to avoid:** 保持 ScriptClip 字段定义顺序与 Python 版一致：`_id`, `timestamp`, `picture`, `narration`, `OST`, 然后管道扩展字段
**Warning signs:** 保存的 JSON 字段顺序与 Python 版不同

### Pitfall 5: 大文件加载的性能
**What goes wrong:** 脚本文件可能包含几十到上百个片段，每个片段的 narration 文本可能很长（数百字中文），整个 JSON 文件可能达到 MB 级别
**Why it happens:** serde_json 一次性反序列化整个文件到内存
**How to avoid:** 对于桌面应用场景，MB 级别的 JSON 加载没有性能问题。serde_json 的性能远超 Python 版。无需优化。
**Warning signs:** 不适用于本阶段——脚本文件不会特别大

### Pitfall 6: _id 字段名以 _ 开头
**What goes wrong:** Rust 命名规范不推荐字段以 `_` 开头，但 JSON key 是 `_id`
**Why it happens:** Python 版使用了 MongoDB 风格的 `_id` 字段名
**How to avoid:** 使用 `#[serde(rename = "_id")]` 将 Rust 字段 `id` 映射为 JSON 的 `_id`。或者直接在 Rust 中保留 `_id` 字段名（D-05 已确定）。推荐直接使用 `_id`，虽然不符合 Rust 命名惯例，但减少 rename 的认知负担。
**Warning signs:** 如果用 rename，需要记住 Rust 端用 `clip.id` 还是 `clip._id`

## Code Examples

### 加载脚本
```rust
// 来源: 对齐 Python 版 app/utils/check_script.py + app/services/script_service.py
use std::path::Path;
use crate::script::error::ScriptError;
use crate::script::types::{Script, ScriptClip};

/// 从 JSON 文件加载脚本，内部自动校验
pub fn load_script(path: &Path) -> Result<Script, ScriptError> {
    let content = std::fs::read_to_string(path)
        .map_err(ScriptError::Io)?;
    let script: Script = serde_json::from_str(&content)
        .map_err(ScriptError::JsonParse)?;
    validate(&script)?;
    Ok(script)
}
```

### 保存脚本（美化 JSON，非 ASCII 不转义）
```rust
// 来源: 对齐 Python 版 json.dumps(indent=2, ensure_ascii=False)
// serde_json 默认不转义非 ASCII 字符，与 ensure_ascii=False 行为一致
// [VERIFIED: serde_json 默认行为]
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    let json = serde_json::to_string_pretty(script)
        .map_err(|e| ScriptError::JsonParse(e.to_string()))?;
    std::fs::write(path, json.as_bytes())
        .map_err(ScriptError::Io)?;
    Ok(())
}
```

### timestamp 格式校验（两种方案）
```rust
// 方案 A: 手写校验函数（推荐——避免引入 regex 依赖）
fn validate_timestamp(ts: &str) -> bool {
    // 格式: HH:MM:SS,mmm-HH:MM:SS,mmm
    let parts: Vec<&str> = ts.split('-').collect();
    if parts.len() != 2 { return false; }
    fn check_time(s: &str) -> bool {
        let time_parts: Vec<&str> = s.split(',').collect();
        if time_parts.len() != 2 { return false; }
        let hms: Vec<&str> = time_parts[0].split(':').collect();
        if hms.len() != 3 { return false; }
        // 检查每部分长度和数值范围
        hms.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_digit()))
            && time_parts[1].len() == 3 && time_parts[1].chars().all(|c| c.is_ascii_digit())
    }
    check_time(parts[0]) && check_time(parts[1])
}

// 方案 B: regex（需要引入 regex crate）
// let ts_re = Regex::new(r"^\d{2}:\d{2}:\d{2},\d{3}-\d{2}:\d{2}:\d{2},\d{3}$").unwrap();
// ts_re.is_match(ts)
```

### 校验对齐 Python 版 check_format()
```rust
// 来源: app/utils/check_script.py check_format() 函数
// Python 版检查: (1) JSON 数组 (2) 非空数组 (3) 每个元素是 dict
// (4) 5 个核心字段全部存在 (5) _id 为正整数 (6) timestamp 格式
// (7) picture 非空字符串 (8) narration 非空字符串 (9) OST 为整数
//
// Rust 版 serde 已自动处理 (1)(3)(4)(9)，validate() 需补充:
// - 数组非空检查
// - _id > 0
// - timestamp 格式正则
// - picture/narration trim 后非空
// - OST 值在 0/1/2 范围内（serde_repr 可能已拒绝无效值）
```

### ScriptError 定义
```rust
// 来源: 对齐 Phase 1 error.rs 的 thiserror 模式 + CONTEXT.md D-19
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("JSON 解析失败: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("脚本校验失败")]
    Validation(Vec<ValidationError>),

    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("索引越界")]
    IndexOutOfBounds,

    #[error("时间戳格式无效: {0}")]
    InvalidTimestamp(String),
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Python json.dumps | serde_json | N/A（新项目） | Rust serde 性能远超 Python json |
| Python Pydantic 校验 | serde + 手写 validate() | N/A | serde 反序列化本身已做类型校验，手写补充业务规则 |
| Python enum (int) | Rust enum + serde_repr | N/A | 类型安全的枚举，编译时保证值合法 |

**Deprecated/outdated:**
- 无——这是全新 Rust 实现

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | serde_json 默认不转义非 ASCII 字符（等同于 Python ensure_ascii=False） | 保存行为 | 保存的 JSON 中文变成 \uXXXX 转义 |
| A2 | serde_repr 0.1.20 与 serde 1.0.228 兼容 | Standard Stack | 反序列化编译失败 |
| A3 | serde_json::to_string_pretty 使用 2 空格缩进 | 保存行为 | 缩进风格与 Python 版不同 |
| A4 | Option<T> 字段在反序列化时 JSON 缺失该字段自动为 None | 反序列化行为 | 缺失字段导致反序列化失败 |

**Assumption A1 和 A3 的验证结果:**
- A1: 已验证 [VERIFIED: serde_json 默认行为，搜索结果确认 serde_json 保留非 ASCII 字符]
- A3: serde_json::to_string_pretty 使用 2 空格缩进 [ASSUMED]——需要确认。Python 版 `json.dumps(indent=2)` 也是 2 空格，两者应一致。
- A4: 已验证 [VERIFIED: serde.rs field-attrs.html + GitHub issue #2753，Option<T> 缺失字段自动为 None]

## Open Questions (RESOLVED)

1. **RESOLVED:** serde_json::to_string_pretty 的缩进字符 — serde_json 默认使用 2 空格缩进，与 Python 版一致。Plan 已采用。

2. **RESOLVED:** 是否引入 regex crate — 决定不引入，手写 timestamp 校验函数更轻量。Plan 01 Task 2 已采用。

3. **RESOLVED:** _id 字段在 Rust 中保留下划线前缀 — 直接使用 `_id`，与 JSON 字段名一致，减少认知负担。Plan 01 Task 1 已采用。

## Environment Availability

Step 2.6: SKIPPED (no external dependencies identified)
本阶段仅依赖 Rust 编译器和已有的 crate 依赖，无外部工具/服务/运行时需求。

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust 内置 #[test] + serde_json 测试 |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --lib script` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCRP-01 | 加载 Python 版 JSON 脚本，所有字段正确反序列化 | unit | `cargo test --lib script::tests::test_load_python_script` | Wave 0 |
| SCRP-01 | 保存脚本输出美化 JSON，非 ASCII 不转义 | unit | `cargo test --lib script::tests::test_save_pretty_json` | Wave 0 |
| SCRP-01 | Option 管道字段保存时省略 | unit | `cargo test --lib script::tests::test_save_skips_none_fields` | Wave 0 |
| SCRP-02 | 修改 narration 返回新 Script | unit | `cargo test --lib script::tests::test_update_narration` | Wave 0 |
| SCRP-02 | 修改 OST 类型（OstType 枚举） | unit | `cargo test --lib script::tests::test_set_ost` | Wave 0 |
| SCRP-02 | 修改 timestamp | unit | `cargo test --lib script::tests::test_update_timestamp` | Wave 0 |
| SCRP-02 | 索引越界返回 IndexOutOfBounds | unit | `cargo test --lib script::tests::test_index_out_of_bounds` | Wave 0 |
| SCRP-03 | 校验检测缺失核心字段 | unit | `cargo test --lib script::tests::test_validate_missing_fields` | Wave 0 |
| SCRP-03 | 校验检测无效 timestamp 格式 | unit | `cargo test --lib script::tests::test_validate_invalid_timestamp` | Wave 0 |
| SCRP-03 | 校验检测空 picture/narration | unit | `cargo test --lib script::tests::test_validate_empty_strings` | Wave 0 |
| SCRP-03 | 校验收集所有错误一次性返回 | unit | `cargo test --lib script::tests::test_validate_collects_all_errors` | Wave 0 |
| SCRP-01 | 端到端：加载实际 Python 版脚本 JSON 文件 | integration | `cargo test --test script_test` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib script`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/script/mod.rs` tests module — 覆盖 SCRP-01/02/03 单元测试
- [ ] `tests/script_test.rs` — 覆盖 SCRP-01 集成测试（加载实际 JSON 文件）
- [ ] `serde_repr` dev-dependency 或 dependency 添加到 Cargo.toml
- [ ] 实际 JSON 样例文件用于集成测试（可复制 `resource/scripts/2026-0416-112716.json` 到测试 fixtures）

## Security Domain

> 本阶段安全域评估：数据处理/反序列化模块

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | 无认证需求 |
| V3 Session Management | no | 无会话管理 |
| V4 Access Control | no | 无访问控制 |
| V5 Input Validation | yes | serde 反序列化 + validate() 校验 |
| V6 Cryptography | no | 无加密需求 |

### Known Threat Patterns for serde/JSON

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed JSON causing panic | Denial of Service | serde_json 返回 Result，不 panic |
| Extremely large JSON file | Denial of Service | 桌面应用场景，文件由用户本地选择，风险极低 |
| JSON with unexpected types | Tampering | serde 强类型反序列化拒绝类型不匹配 |

## Sources

### Primary (HIGH confidence)
- `app/utils/check_script.py` — Python 版校验规则的权威来源（直接阅读源代码）
- `app/models/schema.py` — Python 版数据模型定义（直接阅读源代码）
- `app/services/update_script.py` — 管道扩展字段的语义参考（直接阅读源代码）
- `resource/scripts/2026-0416-112716.json` — 实际脚本 JSON 样例（直接阅读文件）
- `src/config/types.rs` — Phase 1 serde 模式参考（直接阅读源代码）
- `src/error.rs` — Phase 1 错误枚举模式参考（直接阅读源代码）
- `Cargo.toml` — 已有依赖和版本（直接阅读文件）

### Secondary (MEDIUM confidence)
- [serde.rs/enum-number.html](https://serde.rs/enum-number.html) — serde_repr 用法官方文档
- [docs.rs/serde_repr](https://docs.rs/serde_repr) — serde_repr API 文档
- [serde.rs/field-attrs.html](https://serde.rs/field-attrs.html) — skip_serializing_if 属性文档
- [crates.io/serde_repr](https://crates.io/crates/serde_repr) — serde_repr 版本 0.1.20

### Tertiary (LOW confidence)
- [GitHub serde-rs/json #907](https://github.com/serde-rs/json/issues/907) — serde_json 非 ASCII 行为讨论

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — serde/serde_json 已在项目中使用，serde_repr 是标准方案
- Architecture: HIGH — Python 版参考代码完整，数据模型清晰
- Pitfalls: HIGH — serde 序列化的常见陷阱已有充分文档
- 校验规则: HIGH — Python 版 check_format() 是权威参考，逐行对齐即可

**Research date:** 2026-04-28
**Valid until:** 30 天（Rust serde 生态稳定，serde_repr 无重大变更）
