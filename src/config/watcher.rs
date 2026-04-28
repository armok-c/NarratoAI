use std::path::Path;
use std::sync::{Arc, RwLock};
use crate::config::types::AppConfig;
use crate::error::ConfigError;

/// ConfigWatcher 监听配置文件变更并自动重加载
pub struct ConfigWatcher;

impl ConfigWatcher {
    /// 启动配置文件监听
    /// RED phase: stub that always fails
    pub fn new(
        _path: &Path,
        _config: Arc<RwLock<AppConfig>>,
    ) -> Result<Self, ConfigError> {
        // RED phase: always returns error to make tests fail
        Err(ConfigError::WatchError("stub not implemented".into()))
    }
}
