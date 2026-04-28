pub mod defaults;
pub mod types;
pub mod watcher;

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
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

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: path.to_path_buf(),
            _watcher: None,
        })
    }

    /// 启动文件变更监听（热加载）
    pub fn start_watching(&mut self) -> Result<(), ConfigError> {
        let watcher = ConfigWatcher::new(&self.config_path, Arc::clone(&self.config), None)?;
        self._watcher = Some(watcher);
        tracing::info!("配置热加载监听已启动: {}", self.config_path.display());
        Ok(())
    }

    /// 启动热加载监听，并通过 `ready` 信号通知调用者监听已建立
    ///
    /// 测试使用：替代固定延时，等待信号后再写入文件，避免竞态。
    pub fn start_watching_with_ready(&mut self, ready: Sender<()>) -> Result<(), ConfigError> {
        let watcher = ConfigWatcher::new(&self.config_path, Arc::clone(&self.config), Some(ready))?;
        self._watcher = Some(watcher);
        tracing::info!("配置热加载监听已启动: {}", self.config_path.display());
        Ok(())
    }

    /// 获取配置快照（读锁）
    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// 返回 Arc clone 供其他模块使用
    pub fn config(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config)
    }
}

impl AppConfig {
    /// 预留的配置校验方法
    /// Phase 1 不要求 API key 必填，校验逻辑在后续 Phase 添加
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_load_nonexistent_file() {
        let temp_path = std::env::temp_dir().join("narratoai_test_nonexistent_config.toml");
        let result = ConfigManager::load(&temp_path);
        assert!(result.is_err(), "不存在的文件应返回错误");
        match result {
            Err(ConfigError::NotFound { path: _ }) => { /* 正确 */ }
            Err(e) => panic!("应为 NotFound 错误, 得到: {}", e),
            Ok(_) => panic!("应失败"),
        }
    }

    #[test]
    fn test_config_manager_load_valid_file() {
        let toml_content = r#"
[app]
project_version = "0.7.8"
vision_llm_provider = "openai"
"#;
        let dir = TempDir::new().expect("创建临时目录失败");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, toml_content).expect("写入临时文件失败");
        let config_manager = ConfigManager::load(&path)
            .expect("有效 TOML 文件应加载成功");
        let config = config_manager.get();
        assert_eq!(config.app.project_version, "0.7.8");
        assert_eq!(config.app.vision_llm_provider, "openai");
    }
}
