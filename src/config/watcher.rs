use std::path::Path;
use std::sync::{Arc, RwLock};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, Config as NotifyConfig};
use crate::config::types::AppConfig;
use crate::error::ConfigError;

/// ConfigWatcher 监听配置文件变更并自动重加载
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
            move |res: Result<Event, notify::Error>| {
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
        )
        .map_err(|e| ConfigError::WatchError(e.to_string()))?;

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::WatchError(e.to_string()))?;

        Ok(Self { _watcher: watcher })
    }

    /// 在监听线程中重加载配置
    /// 注意（Pitfall 5 防护）：在写锁外解析新配置，获取写锁后仅做指针替换
    fn reload_file(path: &Path) -> Result<AppConfig, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
