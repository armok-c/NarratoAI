//! Phase 01 CONF-03 验证：配置热加载
//!
//! 验证 ConfigWatcher 通过 notify 检测文件变更后，
//! ConfigManager.get() 返回更新后的配置。

use std::time::Duration;

use narratoai_core::config::ConfigManager;
use tempfile::TempDir;

/// 热加载：修改文件后 ConfigManager.get() 返回更新后的值
#[test]
fn test_hot_reload() {
    let dir = TempDir::new().expect("创建临时目录失败");
    let config_path = dir.path().join("config.toml");

    std::fs::write(
        &config_path,
        r#"[app]
project_version = "0.1.0"
"#,
    )
    .expect("写入初始配置失败");

    let mut manager = ConfigManager::load(&config_path).expect("初始配置加载失败");
    assert_eq!(manager.get().app.project_version, "0.1.0");

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    manager
        .start_watching_with_ready(ready_tx)
        .expect("启动热加载监听失败");
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("watcher 应在超时前就绪");

    std::fs::write(
        &config_path,
        r#"[app]
project_version = "0.7.8"
"#,
    )
    .expect("写入更新配置失败");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut updated = false;
    while std::time::Instant::now() < deadline {
        if manager.get().app.project_version == "0.7.8" {
            updated = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(updated, "热加载应在超时前更新 project_version");
}

/// 热加载：损坏的 TOML 不应覆盖当前有效配置
#[test]
fn test_hot_reload_invalid_toml_preserves_config() {
    let dir = TempDir::new().expect("创建临时目录失败");
    let config_path = dir.path().join("config.toml");

    std::fs::write(
        &config_path,
        r#"[app]
project_version = "original"
"#,
    )
    .expect("写入初始配置失败");

    let mut manager = ConfigManager::load(&config_path).expect("初始配置加载失败");
    assert_eq!(manager.get().app.project_version, "original");

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    manager
        .start_watching_with_ready(ready_tx)
        .expect("启动热加载监听失败");
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("watcher 应在超时前就绪");

    // 写入无效 TOML
    std::fs::write(&config_path, "this is not valid toml [[[").expect("写入无效 TOML 失败");

    // 等待 notify 检测到变更
    std::thread::sleep(Duration::from_millis(500));

    // 配置应保持不变（watcher 在解析失败时仅记录日志，不替换）
    assert_eq!(
        manager.get().app.project_version,
        "original",
        "无效 TOML 不应覆盖有效配置"
    );
}
