use std::path::Path;
use std::time::Instant;

/// Edge-TTS 真实连接集成测试
///
/// 从 config.toml 加载配置，通过边缘 TTS WebSocket 合成中文语音，
/// 验证连接、合成、文件输出全链路。
///
/// 依赖：
/// - E:\GitLib\NarratoAI\config.toml 配置文件（含可选的 proxy 段）
/// - 可访问互联网（连接微软 Edge TTS 服务）
#[tokio::test]
async fn test_edge_tts_real_connection() {
    // 初始化日志（仅在测试失败时显示）
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init();

    // ── 1. 加载配置 ──────────────────────────────────────────────
    let config_path_str =
        std::env::var("NARRATAI_CONFIG").unwrap_or_else(|_| {
            let base = std::env::current_dir()
                .unwrap_or_else(|_| Path::new(".").to_path_buf());
            // 如果 CWD 已经是项目根，直接追加
            if base.join("config.toml").exists() {
                base.join("config.toml").to_string_lossy().to_string()
            } else {
                // 备选：硬编码路径（CWD 可能是 narratoai-core/）
                "E:/GitLib/NarratoAI/config.toml".to_string()
            }
        });
    let config_path = Path::new(&config_path_str);

    assert!(
        config_path.exists(),
        "配置文件不存在: {} (可通过环境变量 NARRATAI_CONFIG 指定)",
        config_path.display()
    );

    let config_manager = narratoai_core::config::ConfigManager::load(config_path)
        .expect("配置文件加载失败");

    let config = config_manager.get();
    tracing::info!("配置加载成功: {}", config_path.display());

    let proxy_enabled = config.proxy.enabled;
    tracing::info!("代理状态: enabled={}", proxy_enabled);

    // ── 2. 准备输出目录 ──────────────────────────────────────────
    let output_dir = Path::new("E:/GitLib/NarratoAI/test_output/tts");
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .expect("创建输出目录失败");
    }
    let output_path = output_dir.join("edge_tts_test.mp3");

    // ── 3. 合成中文语音 ──────────────────────────────────────────
    let text = "这是一段来自华星大酒店的全链路测试语音，用于验证Edge-TTS引擎是否正常工作。";
    let voice_name = "zh-CN-XiaoyiNeural";
    let rate = 1.0;
    let pitch = 0.0;

    tracing::info!(
        "开始 Edge-TTS 合成: text_len={}, voice={}, output={}",
        text.len(),
        voice_name,
        output_path.display()
    );

    let start = Instant::now();

    let result = narratoai_core::tts::synthesize(
        "edge_tts",
        text,
        voice_name,
        rate,
        pitch,
        &output_path,
        Some(&config.proxy),
        Some(&config),
    )
    .await;

    let elapsed = start.elapsed();

    // ── 4. 验证结果 ──────────────────────────────────────────────
    match result {
        Ok(output) => {
            tracing::info!(
                "Edge-TTS 合成成功: duration={:.2}s, word_boundaries={}, elapsed={:.2}s",
                output.duration,
                output.word_boundaries.len(),
                elapsed.as_secs_f64(),
            );

            // 验证文件存在且大于 1KB
            assert!(
                output.audio_file_path.exists(),
                "输出音频文件应存在: {}",
                output.audio_file_path.display()
            );
            let file_size = std::fs::metadata(&output.audio_file_path)
                .expect("读取文件元数据失败")
                .len();
            assert!(
                file_size > 1024,
                "音频文件大小应大于 1KB, 实际: {} bytes",
                file_size
            );

            // 验证 duration 合理
            assert!(
                output.duration > 0.0,
                "音频时长应大于 0, 实际: {}",
                output.duration
            );

            // 验证词边界非空
            assert!(
                !output.word_boundaries.is_empty(),
                "应收到词边界数据"
            );

            tracing::info!(
                "输出文件: {} ({} bytes, {:.2}s)",
                output.audio_file_path.display(),
                file_size,
                output.duration,
            );
        }
        Err(e) => {
            panic!(
                "Edge-TTS 合成失败 (耗时 {:.2}s): {}",
                elapsed.as_secs_f64(),
                e
            );
        }
    }
}
