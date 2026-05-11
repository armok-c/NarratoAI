//! Level 2: 纪录片流水线真实全链路测试
//!
//! 创建最小脚本 → TTS 合成 → 视频裁剪 → 音频字幕合并 → 拼接 → 最终合成
//!
//! 成本控制：
//! - LLM API 调用：0 次（脚本手动创建）
//! - TTS 调用：1 次（edge_tts，免费）
//! - FFmpeg 处理：本地
//!
//! 测试产出在 E:\GitLib\NarratoAI\test_output\pipeline\ 目录

use std::path::Path;
use std::time::Instant;

use narratoai_core::config::ConfigManager;
use narratoai_core::documentary::pipeline::run_documentary;
use narratoai_core::documentary::types::DocumentaryRequest;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

const VIDEO_PATH: &str = "E:/GitLib/视频/华星大酒店.mp4";
const TEST_OUTPUT_DIR: &str = "E:/GitLib/NarratoAI/test_output/pipeline";

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init();
}

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

/// 创建测试脚本 JSON 文件
/// 包含 1 个 NarrationOnly 片段（OST=0），前 6 秒
fn create_test_script(output_dir: &Path) -> std::path::PathBuf {
    let script_path = output_dir.join("test_script.json");

    let script_json = r#"[
        {
            "_id": 1,
            "timestamp": "00:00:00,000-00:00:06,000",
            "picture": "华星大酒店外观全景，灯火辉煌的夜晚",
            "narration": "欢迎来到华星大酒店，这里是城市中最繁华的地段，为您提供最优质的服务。",
            "OST": 0
        }
    ]"#;

    std::fs::write(&script_path, script_json).expect("创建测试脚本文件失败");
    eprintln!("测试脚本已创建: {:?}", script_path);
    script_path
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

/// 纪录片流水线全链路测试
///
/// 1. 创建测试脚本（1 个 NarrationOnly 片段）
/// 2. 构建 DocumentaryRequest
/// 3. 运行 run_documentary()
/// 4. 验证输出视频存在且大于最小尺寸
#[tokio::test]
async fn test_documentary_pipeline() {
    init_tracing();

    // 准备输出目录
    let output_dir = Path::new(TEST_OUTPUT_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("创建输出目录失败");
    }

    // Step 1: 创建测试脚本
    let script_path = create_test_script(output_dir);

    // Step 2: 加载配置
    let config_path = get_config_path();
    let config_manager = ConfigManager::load(&config_path).expect("配置文件加载失败");
    let config = config_manager.get();

    let proxy = if config.proxy.enabled && !config.proxy.http.is_empty() {
        Some(config.proxy.clone())
    } else {
        None
    };

    // Step 3: 构建请求
    let request = DocumentaryRequest {
        video_path: Path::new(VIDEO_PATH).to_path_buf(),
        script_path: script_path.clone(),
        tts_engine: "edge_tts".to_string(),
        voice_name: "zh-CN-XiaoyiNeural".to_string(),
        voice_rate: 1.0,
        voice_pitch: 0.0,
        tts_volume: 1.0,
        original_volume: 0.7,
        bgm_volume: 0.3,
        bgm_path: None,
        subtitle_enabled: true,
        subtitle_font: None,
        subtitle_font_size: 40,
        subtitle_color: "#FFFFFF".to_string(),
        subtitle_position: "bottom".to_string(),
        output_dir: Some(output_dir.to_path_buf()),
        threads: 4,
    };

    tracing::info!("开始纪录片流水线...");
    let start = Instant::now();

    // Step 4: 运行流水线
    let result = run_documentary(request, &config, proxy.as_ref(), None).await;

    let elapsed = start.elapsed();
    tracing::info!("流水线完成, 耗时 {:.2}s", elapsed.as_secs_f64());

    match result {
        Ok(output_path) => {
            tracing::info!(
                "纪录片输出: {:?} (耗时 {:.2}s)",
                output_path,
                elapsed.as_secs_f64()
            );

            assert!(
                output_path.exists(),
                "输出视频文件应存在: {:?}",
                output_path
            );

            let file_size = std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0);
            assert!(
                file_size > 100_000,
                "输出视频应大于 100KB, 实际: {} bytes",
                file_size
            );

            tracing::info!(
                "输出文件: {:?} ({} bytes, {:.1}s)",
                output_path,
                file_size,
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            tracing::error!("纪录片流水线失败: {}", e);
            // 输出中间产物目录以便排查
            if output_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(output_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            eprintln!("  中间产物: {:?} ({} bytes)", path, size);
                        }
                    }
                }
            }
            panic!("纪录片流水线测试失败: {}", e);
        }
    }
}
