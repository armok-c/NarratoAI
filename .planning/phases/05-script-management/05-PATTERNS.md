# Phase 5: Script Management - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 7 (5 new + 2 modified)
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/script/types.rs` | model | transform | `src/config/types.rs` | exact |
| `src/script/error.rs` | model | transform | `src/error.rs` | exact |
| `src/script/mod.rs` | service | file-I/O | `src/config/mod.rs` | exact |
| `src/script/edit.rs` | service | transform | `src/config/mod.rs` (immutable get) | role-match |
| `src/lib.rs` | config | — | `src/lib.rs` (self) | exact |
| `Cargo.toml` | config | — | `Cargo.toml` (self) | exact |
| `tests/script_test.rs` | test | file-I/O | `tests/config_test.rs` | exact |

## Pattern Assignments

### `src/script/types.rs` (model, transform)

**Analog:** `src/config/types.rs`

ScriptClip 结构体的 serde derive 模式直接复制 `src/config/types.rs` 中 `AppConfig` 和各 Section 结构体的写法。区别在于 ScriptClip 使用 `#[serde(rename = "...")]` 映射 camelCase JSON 字段名，以及 `#[serde(skip_serializing_if = "Option::is_none")]` 控制保存输出。

**Imports pattern** (参考 `src/config/types.rs` line 1):
```rust
use serde::{Deserialize, Serialize};
```

**Struct derive + serde 属性 pattern** (参考 `src/config/types.rs` lines 3-6):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSection,
```

ScriptClip 的 adaptation -- 核心字段不用 `#[serde(default)]`（必填），管道扩展字段用 `Option<T>` + `skip_serializing_if`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptClip {
    pub _id: i64,
    pub timestamp: String,
    pub picture: String,
    pub narration: String,
    pub OST: OstType,
    #[serde(rename = "sourceTimeRange", skip_serializing_if = "Option::is_none")]
    pub source_time_range: Option<String>,
    // ... 其他 Option 字段
}
```

**OstType 枚举 pattern** -- 使用 `serde_repr` 将 C-like enum 序列化为整数 0/1/2:
```rust
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OstType {
    NarrationOnly = 0,
    OriginalSound = 1,
    Mixed = 2,
}
```

**Type alias pattern** (参考 Python 版纯数组格式):
```rust
pub type Script = Vec<ScriptClip>;
```

**内联测试 pattern** (参考 `src/config/types.rs` lines 205-362):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_full_config() {
        let toml_str = r#"... "#;
        let config: AppConfig = toml::from_str(toml_str).expect("...");
        assert_eq!(config.app.project_version, "0.7.8");
    }
}
```

ScriptClip 的 adaptation -- 用 JSON 字符串 + `serde_json::from_str`:
```rust
#[test]
fn test deserialize script clip from json() {
    let json = r#"[{"_id":1,"timestamp":"00:00:00,001-00:01:15,001","picture":"...","narration":"...","OST":0}]"#;
    let script: Script = serde_json::from_str(json).unwrap();
    assert_eq!(script[0]._id, 1);
    assert_eq!(script[0].OST, OstType::NarrationOnly);
}
```

---

### `src/script/error.rs` (model, transform)

**Analog:** `src/error.rs`

ScriptError 枚举的 thiserror derive 模式直接复制 `src/error.rs` 中 `ConfigError` 和 `FFmpegError` 的写法。中文错误消息、`#[from]` 自动转换、结构化变体。

**Imports pattern** (参考 `src/error.rs` line 1):
```rust
use thiserror::Error;
```

**Error enum derive pattern** (参考 `src/error.rs` lines 4-20):
```rust
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

#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg 进程启动失败: {0}")]
    SpawnFailed(String),

    #[error("未找到 FFmpeg 二进制文件")]
    BinaryNotFound,
}
```

**From trait impl pattern** (参考 `src/error.rs` lines 42-46):
```rust
impl From<notify::Error> for ConfigError {
    fn from(err: notify::Error) -> Self {
        ConfigError::WatchError(err.to_string())
    }
}
```

ScriptError 的 adaptation -- 使用 `#[from]` 属性替代手写 From impl（更简洁）:
```rust
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

**测试 pattern** (参考 `src/error.rs` lines 49-121):
```rust
#[test]
fn test_config_error_io_error_message_chinese() {
    let err = ConfigError::IoError("test".into());
    let msg = err.to_string();
    assert!(msg.contains("配置文件读取失败"), "消息应包含中文: {}", msg);
}
```

---

### `src/script/mod.rs` (service, file-I/O)

**Analog:** `src/config/mod.rs`

模块入口文件 + 文件加载/保存逻辑。`load_script()` 和 `save_script()` 直接复制 `ConfigManager::load()` 的文件 I/O + 解析 + 错误映射模式。

**Module structure pattern** (参考 `src/config/mod.rs` lines 1-3):
```rust
pub mod defaults;
pub mod types;
pub mod watcher;
```

Script module 的 adaptation:
```rust
pub mod edit;
pub mod error;
pub mod types;
```

**File load pattern** (参考 `src/config/mod.rs` lines 21-39):
```rust
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

    Ok(Self { ... })
}
```

load_script 的 adaptation:
```rust
pub fn load_script(path: &Path) -> Result<Script, ScriptError> {
    let content = std::fs::read_to_string(path)
        .map_err(ScriptError::Io)?;
    let script: Script = serde_json::from_str(&content)
        .map_err(ScriptError::JsonParse)?;
    validate(&script)?;
    Ok(script)
}
```

**save_script pattern** -- 无直接 analog（ConfigManager 没有 save），但对齐 Python 版 `json.dumps(indent=2, ensure_ascii=False)`:
```rust
pub fn save_script(script: &Script, path: &Path) -> Result<(), ScriptError> {
    let json = serde_json::to_string_pretty(script)
        .map_err(|e| ScriptError::JsonParse(e.to_string()))?;
    std::fs::write(path, json.as_bytes())
        .map_err(ScriptError::Io)?;
    Ok(())
}
```

注意: `save_script` 中 `serde_json::Error` 不能直接用 `#[from]`（因为 `serde_json::to_string_pretty` 的错误需要映射），使用 `.map_err(|e| ScriptError::JsonParse(e.to_string()))` 或定义辅助错误变体。

**validate pattern** -- 参考 Python 版 `app/utils/check_script.py:check_format()` (lines 5-110):
```python
# Python 版校验逻辑:
# 1. json.loads() 解析 -> serde 自动处理
# 2. isinstance(data, list) -> serde 自动处理
# 3. len(data) == 0 -> Rust validate 补充
# 4. 循环每个 clip:
#    a. isinstance(clip, dict) -> serde 自动处理
#    b. required_fields 检查 -> serde 自动处理（缺字段报错）
#    c. _id > 0 -> Rust validate 补充
#    d. timestamp 正则 -> Rust validate 补充
#    e. picture.strip() 非空 -> Rust validate 补充
#    f. narration.strip() 非空 -> Rust validate 补充
#    g. OST 为整数 -> serde_repr 自动处理
```

**内联测试 pattern** (参考 `src/config/mod.rs` lines 78-111):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_load_nonexistent_file() {
        let temp_path = std::env::temp_dir().join("narratoai_test_nonexistent_config.toml");
        let result = ConfigManager::load(&temp_path);
        assert!(result.is_err());
        match result {
            Err(ConfigError::NotFound { path: _ }) => { /* 正确 */ }
            Err(e) => panic!("应为 NotFound 错误, 得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    #[test]
    fn test_config_manager_load_valid_file() {
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, toml_content).expect("写入临时文件失败");
        let config_manager = ConfigManager::load(&path).expect("有效 TOML 文件应加载成功");
        // assertions...
    }
}
```

---

### `src/script/edit.rs` (service, transform)

**Analog:** `src/config/mod.rs` (不可变 get 模式) + 项目规则（不可变更新）

项目编码规则要求所有更新方法返回新实例，不修改原始数据。`ConfigManager::get()` (line 60-62) 展示了 `.clone()` 模式。

**Immutable update pattern** (参考项目规则 + `src/config/mod.rs` line 60-62):
```rust
// ConfigManager::get() 返回 clone
pub fn get(&self) -> AppConfig {
    self.config.read().unwrap_or_else(|e| e.into_inner()).clone()
}
```

edit.rs 的 adaptation -- 每个方法 clone 整个 Script，修改指定元素，返回新 Script:
```rust
use crate::script::error::ScriptError;
use crate::script::types::{OstType, Script};

fn update_narration(script: &Script, index: usize, text: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    let mut new_script = script.clone();
    new_script[index].narration = text.to_string();
    Ok(new_script)
}

fn set_ost(script: &Script, index: usize, ost: OstType) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    let mut new_script = script.clone();
    new_script[index].OST = ost;
    Ok(new_script)
}

fn update_timestamp(script: &Script, index: usize, ts: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    // 可选: 在此校验 ts 格式
    let mut new_script = script.clone();
    new_script[index].timestamp = ts.to_string();
    Ok(new_script)
}

fn update_picture(script: &Script, index: usize, pic: &str) -> Result<Script, ScriptError> {
    if index >= script.len() {
        return Err(ScriptError::IndexOutOfBounds);
    }
    let mut new_script = script.clone();
    new_script[index].picture = pic.to_string();
    Ok(new_script)
}
```

---

### `src/lib.rs` (config, --)

**Analog:** `src/lib.rs` (self -- 修改现有文件)

**Current pattern** (lines 1-8):
```rust
pub mod config;
pub mod ffmpeg;
pub mod error;
```

**Required change:** 添加 `pub mod script;` 行:
```rust
pub mod config;
pub mod ffmpeg;
pub mod error;
pub mod script;      // Phase 5 新增
```

---

### `Cargo.toml` (config, --)

**Analog:** `Cargo.toml` (self -- 修改现有文件)

**Current dependencies** (lines 6-18):
```toml
[dependencies]
tokio = { version = "1.52.1", features = ["full"] }
serde = { version = "1.0.228", features = ["derive"] }
toml = "1.1.2"
thiserror = "2.0.18"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }
ffmpeg-sidecar = { version = "2.5.1", features = ["download_ffmpeg"] }
notify = "9.0.0-rc.3"
serde_json = "1.0.140"

[dev-dependencies]
tempfile = "3.27.0"
```

**Required change:** 添加 `serde_repr` 到 `[dependencies]`:
```toml
serde_repr = "0.1.20"
```

`regex` 不添加 -- RESEARCH.md 推荐手写 timestamp 校验函数，避免为单一正则引入整个 regex crate。

---

### `tests/script_test.rs` (test, file-I/O)

**Analog:** `tests/config_test.rs`

集成测试模式: 使用 `tempfile::TempDir` 创建临时文件，测试端到端的加载/保存流程。

**Test file pattern** (参考 `tests/config_test.rs` 的组织方式):
```rust
// tests/script_test.rs
use narratoai_core::script::{load_script, save_script};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_load_save_roundtrip() {
    // 创建临时 JSON 文件
    // 加载
    // 编辑
    // 保存
    // 重新加载
    // 验证一致性
}
```

---

## Shared Patterns

### Serde Derive 模式
**Source:** `src/config/types.rs` lines 1-6
**Apply to:** `src/script/types.rs` (ScriptClip, OstType)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptClip { ... }
```
所有数据模型结构体统一使用 `Debug, Clone, Serialize, Deserialize` derive 宏。

### Thiserror 错误枚举模式
**Source:** `src/error.rs` lines 1-46
**Apply to:** `src/script/error.rs` (ScriptError)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("...中文消息: {0}")]
    Variant(Type),
}
```
规则: (1) 中文错误消息 (2) `#[from]` 用于 std::io::Error 和 serde_json::Error (3) 结构化变体带命名字段。

### 文件 I/O + 错误映射模式
**Source:** `src/config/mod.rs` lines 21-39
**Apply to:** `src/script/mod.rs` (load_script, save_script)
```rust
let content = std::fs::read_to_string(path)
    .map_err(DomainError::Io)?;
let data: T = parse(&content)
    .map_err(DomainError::Parse)?;
```
规则: 用 `std::fs::read_to_string` 读取 UTF-8 文件，错误通过 `map_err` 映射到领域错误。

### 不可变更新模式
**Source:** 项目规则 (CLAUDE.md + `.claude/rules/common/coding-style.md`)
**Apply to:** `src/script/edit.rs` (所有 update_* 函数)
```rust
fn update_xxx(data: &Data, ...) -> Result<Data, Error> {
    let mut new_data = data.clone();
    new_data.field = new_value;
    Ok(new_data)
}
```
规则: 永远返回新实例，不修改原始数据。

### 内联测试 + tempfile 模式
**Source:** `src/config/mod.rs` lines 78-111, `src/config/types.rs` lines 205-362
**Apply to:** 所有 `src/script/*.rs` 文件的内联 `#[cfg(test)] mod tests`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_xxx() { ... }
}
```
规则: (1) 每个源文件内联 `#[cfg(test)] mod tests` (2) 用 `tempfile::TempDir` 管理临时文件 (3) 测试数据用原始字符串字面量 `r#"..."#` (4) 中文注释说明测试意图。

### 校验规则对齐
**Source:** `app/utils/check_script.py` lines 5-110
**Apply to:** `src/script/mod.rs` (validate 函数)
```python
# Python 版 check_format() 的权威规则:
# 1. JSON 数组 (serde 自动处理)
# 2. 非空数组 (validate 补充)
# 3. 每个 clip 是 dict (serde 自动处理)
# 4. 5 个必需字段全部存在 (serde 自动处理)
# 5. _id > 0 正整数 (validate 补充)
# 6. timestamp 匹配正则 (validate 补充)
# 7. picture 非空字符串 (validate 补充)
# 8. narration 非空字符串 (validate 补充)
# 9. OST 为 0/1/2 整数 (serde_repr 自动处理)
```
注意: Python 版在第一个错误即停（逐个 return），但 D-13 要求收集所有错误。Rust 版用 `Vec<ValidationError>` 累积所有错误后一次性返回。

## No Analog Found

所有文件均有明确 analog，无需使用 RESEARCH.md 中的代码示例替代。

## Python 参考文件映射

| Rust 文件 | Python 参考 | 映射关系 |
|-----------|-------------|----------|
| `src/script/types.rs` | `app/models/schema.py:VideoClipParams` (line 160) | Rust ScriptClip 对齐 Python VideoClipParams 的 JSON 字段 |
| `src/script/mod.rs` (validate) | `app/utils/check_script.py:check_format()` (lines 5-110) | Rust validate() 1:1 对齐 Python 校验规则 |
| `src/script/mod.rs` (load/save) | `app/services/script_service.py` | load_script 对齐 Python 版脚本加载流程 |
| `src/script/edit.rs` | `app/services/update_script.py:update_script_timestamps()` (lines 90-197) | Rust 不可变更新对齐 Python 的 item.copy() 模式 |
| 测试 JSON fixtures | `resource/scripts/2026-0416-112716.json` | 实际脚本样例，用于集成测试 |

## Metadata

**Analog search scope:** `src/` (Rust 代码), `app/` (Python 参考), `tests/` (测试)
**Files scanned:** 14
**Pattern extraction date:** 2026-04-28
