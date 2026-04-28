use std::ffi::OsString;
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
        let parent_dir = path.parent().unwrap_or(path).to_path_buf();
        let file_name: Option<OsString> = path.file_name().map(|n| n.to_os_string());
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // 响应文件修改和创建事件（原子保存工具常用 Create 事件）
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        // 按文件名过滤：父目录监听可能收到不相关的事件
                        let is_target = file_name.as_ref().map_or(true, |name| {
                            event.paths.iter().any(|p| p.file_name() == Some(name.as_os_str()))
                        });
                        if is_target {
                            match Self::reload_file(&watched_path) {
                                Ok(new_config) => {
                                    let mut guard = config.write().unwrap_or_else(|e| e.into_inner());
                                    *guard = new_config;
                                    tracing::info!("配置文件已热加载更新");
                                }
                                Err(e) => {
                                    tracing::error!("配置热加载失败: {}", e);
                                }
                            }
                        }
                    }
                }
            },
            NotifyConfig::default(),
        )
        .map_err(|e| ConfigError::WatchError(e.to_string()))?;

        watcher
            .watch(&parent_dir, RecursiveMode::NonRecursive)
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
