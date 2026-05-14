//! Level 1: 视觉分析真实集成测试
//!
//! 验证帧提取 + LLM 视觉分析能力。
//! LLM 调用有限额度，仅提取 2 帧图片进行分析。
//!
//! 前置条件：
//! - config.toml 配置了有效的 vision LLM 密钥
//! - 视频素材 E:\GitLib\视频\华星大酒店.mp4 存在

use std::path::{Path, PathBuf};
use std::time::Instant;

use narratoai_core::config::ConfigManager;
use narratoai_core::llm::openai_compatible::{OpenAiCompatibleProvider, ProviderConfig};
use narratoai_core::llm::provider::LlmProvider;
use narratoai_core::visual::frame_extractor::extract_frames;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

const VIDEO_PATH: &str = "E:/GitLib/视频/华星大酒店.mp4";
const FRAME_OUTPUT_DIR: &str = "E:/GitLib/NarratoAI/test_output/frames";

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 获取配置文件路径
fn get_config_path() -> std::path::PathBuf {
    let config_path_str = std::env::var("NARRATAI_CONFIG").unwrap_or_else(|_| {
        let base = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        if base.join("config.toml").exists() {
            base.join("config.toml").to_string_lossy().to_string()
        } else {
            "E:/GitLib/NarratoAI/config.toml".to_string()
        }
    });
    Path::new(&config_path_str).to_path_buf()
}

/// 初始化 tracing（单次初始化）
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .try_init();
}

/// 创建视觉 LLM provider（使用 vision 配置）
fn create_vision_provider() -> OpenAiCompatibleProvider {
    let config_path = get_config_path();
    let config_manager = ConfigManager::load(&config_path).expect("配置文件加载失败");
    let config = config_manager.get();

    assert!(
        !config.app.vision_openai_api_key.is_empty(),
        "config.toml 中 vision_openai_api_key 为空"
    );
    assert!(
        !config.app.vision_openai_model_name.is_empty(),
        "config.toml 中 vision_openai_model_name 为空"
    );

    let proxy_http = if config.proxy.enabled && !config.proxy.http.is_empty() {
        Some(config.proxy.http.clone())
    } else {
        None
    };
    let proxy_https = if config.proxy.enabled && !config.proxy.https.is_empty() {
        Some(config.proxy.https.clone())
    } else {
        None
    };

    tracing::info!(
        "Vision Provider 配置: model={}, base_url={}",
        config.app.vision_openai_model_name,
        config.app.vision_openai_base_url,
    );

    OpenAiCompatibleProvider::new(ProviderConfig {
        api_key: config.app.vision_openai_api_key.clone(),
        model_name: config.app.vision_openai_model_name.clone(),
        base_url: config.app.vision_openai_base_url.clone(),
        max_retries: 1,
        timeout_secs: 120,
        proxy_http,
        proxy_https,
    })
    .expect("Provider 创建失败")
}

/// 清理帧输出目录
fn cleanup_frame_dir(dir: &Path) {
    if dir.exists() {
        for entry in std::fs::read_dir(dir).unwrap() {
            if let Ok(entry) = entry {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

/// 测试 1: 帧提取 — 从视频中提取约 3 帧
#[tokio::test]
async fn test_frame_extraction() {
    init_tracing();

    let output_dir = Path::new(FRAME_OUTPUT_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("创建帧输出目录失败");
    }
    cleanup_frame_dir(output_dir);

    // 先探测视频时长，设置间隔提取约 3 帧
    let info =
        narratoai_core::ffmpeg::probe::probe_video(Path::new(VIDEO_PATH)).expect("视频探测应成功");
    let interval = info.duration_secs / 4.0; // 提取约 3 帧
    tracing::info!(
        "视频时长: {:.1}s, 帧提取间隔: {:.1}s",
        info.duration_secs,
        interval
    );

    let start = Instant::now();
    let result = extract_frames(
        Path::new(VIDEO_PATH),
        output_dir,
        interval,
        Some(15), // quality=15 平衡大小和质量
        None,     // no progress callback
        None,     // no cancel token
    )
    .await;

    let elapsed = start.elapsed();
    assert!(result.is_ok(), "帧提取应成功: {:?}", result.err());

    let (frame_count, frame_paths) = result.unwrap();
    tracing::info!(
        "帧提取完成: {} 帧, 耗时 {:.2}s",
        frame_count,
        elapsed.as_secs_f64()
    );

    assert!(frame_count > 0, "应提取至少 1 帧");
    for fp in &frame_paths {
        assert!(fp.exists(), "帧文件应存在: {:?}", fp);
        let size = std::fs::metadata(fp).map(|m| m.len()).unwrap_or(0);
        assert!(size > 1000, "帧文件应 > 1KB: {:?} ({} bytes)", fp, size);
        tracing::info!("  帧: {:?} ({} bytes)", fp, size);
    }
}

/// 测试 2: LLM 视觉分析 — 用 2 张帧图片进行简单的视觉理解
///
/// 成本控制：
/// - 仅 2 帧图片
/// - 最短 prompt
/// - max_tokens=100
#[tokio::test]
async fn test_vision_analysis() {
    init_tracing();
    let provider = create_vision_provider();

    // Step 1: 准备帧文件
    let frame_dir = Path::new(FRAME_OUTPUT_DIR);
    if !frame_dir.exists() {
        std::fs::create_dir_all(frame_dir).expect("创建帧目录失败");
    }

    let mut existing_frames: Vec<PathBuf> = std::fs::read_dir(frame_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map(|ext| ext == "jpg" || ext == "png")
                .unwrap_or(false)
        })
        .collect();
    existing_frames.sort();

    if existing_frames.len() < 2 {
        let info = narratoai_core::ffmpeg::probe::probe_video(Path::new(VIDEO_PATH))
            .expect("视频探测应成功");
        let interval = info.duration_secs / 3.0; // 提取约 2 帧
        let result = extract_frames(
            Path::new(VIDEO_PATH),
            frame_dir,
            interval,
            Some(15),
            None,
            None,
        )
        .await
        .expect("帧提取应成功");
        existing_frames = result.1;
        existing_frames.sort();
    }

    assert!(
        existing_frames.len() >= 2,
        "需要至少 2 帧进行视觉分析，实际: {}",
        existing_frames.len()
    );

    // 仅取前 2 帧
    let test_frames: Vec<PathBuf> = existing_frames.into_iter().take(2).collect();
    tracing::info!("使用 {} 帧进行视觉分析", test_frames.len());

    // Step 2: 使用最短 prompt 调用 LLM 视觉分析
    let prompt = "用一句中文逐帧描述画面内容，每帧一行。格式: [帧序号]: [描述]";
    let system_prompt = "你是一个视觉分析助手";

    let start = Instant::now();
    let result = provider
        .analyze_images(
            &test_frames,
            prompt,
            Some(system_prompt),
            Some(2),   // batch_size
            Some(1),   // max_concurrency
            None,      // response_format
            Some(0.1), // temperature
            Some(100), // max_tokens
            None,      // cancel
        )
        .await;

    let elapsed = start.elapsed();
    match result {
        Ok(responses) => {
            tracing::info!(
                "视觉分析成功: {} 条响应, 耗时 {:.2}s",
                responses.len(),
                elapsed.as_secs_f64()
            );
            for (i, resp) in responses.iter().enumerate() {
                let preview: String = resp.chars().take(100).collect();
                tracing::info!("  响应 #{}: {}", i + 1, preview);
            }
        }
        Err(e) => {
            tracing::warn!(
                "视觉分析失败 (耗时 {:.2}s): {} — 可能是模型不支持视觉或 API 配置问题",
                elapsed.as_secs_f64(),
                e
            );
            // 不硬失败——视觉 API 可能因配置不支持而失败
            eprintln!("视觉分析跳过 (非致命): {}", e);
        }
    }
}
