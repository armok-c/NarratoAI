use std::path::Path;
use std::time::Duration;

use narratoai_core::config::ConfigManager;
use narratoai_core::config::types::AppConfig;
use narratoai_core::error::ConfigError;
use tempfile::{NamedTempFile, TempDir};

/// 从 config.example.toml 加载完整配置，验证所有 10 个 section 的值
#[test]
fn test_load_example_config() {
    let config_manager = ConfigManager::load(Path::new("config.example.toml"))
        .expect("config.example.toml 应加载成功");
    let config = config_manager.get();

    // [app] section
    assert_eq!(config.app.project_version, "0.7.8");
    assert_eq!(config.app.llm_vision_timeout, 120);
    assert_eq!(config.app.llm_text_timeout, 180);
    assert_eq!(config.app.llm_max_retries, 3);
    assert!(config.app.hide_config);

    // [ui] section
    assert_eq!(config.ui.tts_engine, "edge_tts");
    assert_eq!(config.ui.edge_voice_name, "zh-CN-XiaoyiNeural-Female");

    // [azure] section 存在
    assert_eq!(config.azure.speech_key, "");
    assert_eq!(config.azure.speech_region, "");

    // [tencent] section 存在
    assert_eq!(config.tencent.region, "ap-beijing");

    // [soulvoice] section 存在
    assert_eq!(config.soulvoice.api_url, "https://tts.scsmtech.cn/tts");
    assert_eq!(config.soulvoice.model, "FunAudioLLM/CosyVoice2-0.5B");

    // [tts_qwen] section 存在
    assert_eq!(config.tts_qwen.model_name, "qwen3-tts-flash");

    // [indextts2] section 存在
    assert_eq!(config.indextts2.api_url, "http://127.0.0.1:8081/tts");

    // [doubaotts] section 存在
    assert_eq!(config.doubaotts.cluster, "volcano_tts");
    assert_eq!(config.doubaotts.volume, 1.0);

    // [proxy] section
    assert!(!config.proxy.enabled);

    // [frames] section
    assert_eq!(config.frames.frame_interval_input, 3);
    assert_eq!(config.frames.vision_batch_size, 10);
    assert_eq!(config.frames.vision_max_concurrency, 2);
}

/// 加载只含 [app] section 的最小配置，缺失的 section 应使用默认值
#[test]
fn test_load_minimal_config() {
    let file = NamedTempFile::new().expect("创建临时文件失败");
    std::fs::write(
        file.path(),
        r#"[app]
project_version = "test"
"#,
    )
    .expect("写入临时文件失败");

    let config_manager =
        ConfigManager::load(file.path()).expect("含 [app] 的 TOML 应加载成功");
    let config = config_manager.get();

    assert_eq!(config.app.project_version, "test");
    // 缺失的 section 应使用默认值
    assert_eq!(config.ui.tts_engine, "edge_tts");
    assert_eq!(config.azure.speech_key, "");
    assert_eq!(config.frames.frame_interval_input, 3);
    assert!(!config.proxy.enabled);
}

/// 加载空 TOML 文件，全部 section 使用默认值
#[test]
fn test_load_empty_config() {
    let file = NamedTempFile::new().expect("创建临时文件失败");
    std::fs::write(file.path(), "").expect("写入临时文件失败");

    let config_manager =
        ConfigManager::load(file.path()).expect("空 TOML 应加载成功");
    let loaded = config_manager.get();
    let default = AppConfig::default();

    // 验证空文件加载结果与默认值一致
    assert_eq!(loaded.app.project_version, default.app.project_version);
    assert_eq!(loaded.app.llm_vision_timeout, default.app.llm_vision_timeout);
    assert_eq!(loaded.ui.tts_engine, default.ui.tts_engine);
    assert_eq!(loaded.ui.edge_voice_name, default.ui.edge_voice_name);
    assert_eq!(loaded.azure.speech_key, default.azure.speech_key);
    assert_eq!(loaded.tencent.region, default.tencent.region);
    assert_eq!(loaded.proxy.enabled, default.proxy.enabled);
    assert_eq!(
        loaded.frames.frame_interval_input,
        default.frames.frame_interval_input
    );
}

/// 加载不存在的配置文件，应返回 ConfigError::NotFound
#[test]
fn test_load_nonexistent() {
    let result = ConfigManager::load(Path::new("/nonexistent/config.toml"));
    assert!(result.is_err(), "不存在的文件应返回错误");
    match result {
        Err(ConfigError::NotFound { path }) => {
            assert!(path.contains("nonexistent"), "路径信息应包含文件路径: {}", path);
        }
        Err(e) => panic!("应为 NotFound 错误, 得到: {}", e),
        Ok(_) => panic!("应失败"),
    }
}

/// 验证 AppConfig::default() 的关键字段值与 config.example.toml 一致
#[test]
fn test_default_values() {
    let config = AppConfig::default();

    // AppSection 默认值
    assert_eq!(config.app.llm_vision_timeout, 120);
    assert_eq!(config.app.llm_text_timeout, 180);
    assert_eq!(config.app.llm_max_retries, 3);

    // UiSection 默认值
    assert_eq!(config.ui.edge_voice_name, "zh-CN-XiaoyiNeural-Female");

    // FramesSection 默认值
    assert_eq!(config.frames.vision_batch_size, 10);

    // ProxySection 默认值
    assert!(!config.proxy.enabled);
}

/// 热加载：修改文件后 ConfigManager.get() 返回更新后的值
///
/// 注意：依赖 notify 文件监听，在 Windows 上可能有延迟。
/// 如果通知系统不稳定，可标记 #[ignore]。
#[test]
fn test_hot_reload() {
    let dir = TempDir::new().expect("创建临时目录失败");
    let config_path = dir.path().join("config.toml");

    // 写入初始配置
    std::fs::write(
        &config_path,
        r#"[app]
project_version = "0.1.0"
"#,
    )
    .expect("写入初始配置失败");

    let mut manager =
        ConfigManager::load(&config_path).expect("初始配置加载失败");

    assert_eq!(manager.get().app.project_version, "0.1.0");

    // 启动热加载监听
    manager.start_watching().expect("启动热加载监听失败");

    // 等待 watcher 初始化
    std::thread::sleep(Duration::from_millis(200));

    // 修改配置文件内容
    std::fs::write(
        &config_path,
        r#"[app]
project_version = "0.7.8"
"#,
    )
    .expect("写入更新配置失败");

    // 等待 notify 检测到变更（Windows 上可能需要更长时间）
    std::thread::sleep(Duration::from_millis(1500));

    // 验证配置已更新
    let updated = manager.get();
    assert_eq!(
        updated.app.project_version, "0.7.8",
        "热加载应更新 project_version"
    );
}
