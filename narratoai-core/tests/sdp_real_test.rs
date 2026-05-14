//! Level 2: SDP 短剧混剪流水线真实全链路测试
//!
//! 包含 1 个测试用例，无 LLM 消耗（仅 FFmpeg 操作）。
//!
//! 成本控制：
//! - LLM API 调用：0 次
//! - TTS 调用：0 次（SDP 全部 OST=1，使用原始音频）
//! - FFmpeg 处理：本地
//!
//! 测试产出在 E:\GitLib\NarratoAI\test_output\sdp\ 目录

use std::path::Path;
use std::time::Instant;

use narratoai_core::config::types::AppConfig;
use narratoai_core::sdp::pipeline::run_sdp;
use narratoai_core::sdp::types::SdpRequest;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

const VIDEO_PATH: &str = "E:/GitLib/视频/华星大酒店.mp4";
const SUBTITLE_PATH: &str = "E:/GitLib/视频/华星大酒店.srt";
const TEST_OUTPUT_DIR: &str = "E:/GitLib/NarratoAI/test_output/sdp";

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .try_init();
}

/// 创建 SDP 测试脚本 JSON（2 个 OST=1 片段，基于真实字幕时间戳）
fn create_sdp_test_script(output_dir: &Path) -> std::path::PathBuf {
    let script_path = output_dir.join("sdp_test_script.json");

    let script_json = r#"[
        {
            "_id": 1,
            "timestamp": "00:00:04,000-00:00:12,000",
            "picture": "华星大酒店内景",
            "narration": "hey，what you do，win me ay",
            "OST": 1
        },
        {
            "_id": 2,
            "timestamp": "00:00:14,000-00:00:25,000",
            "picture": "华星大酒店走廊",
            "narration": "you got me to say like eye a smoking",
            "OST": 1
        }
    ]"#;

    std::fs::write(&script_path, script_json).expect("创建 SDP 测试脚本失败");
    eprintln!("SDP 测试脚本已创建: {:?}", script_path);
    script_path
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

/// 本地集成测试 — 需要真实媒体文件。
/// 运行方式: cargo test -p narratoai-core --ignored test_sdp_pipeline
///
/// 1. 创建测试脚本（2 个 OriginalSound 片段）
/// 2. 构建 SdpRequest
/// 3. 运行 run_sdp()
/// 4. 验证输出视频存在且大于最小尺寸
#[tokio::test]
#[ignore]
async fn test_sdp_pipeline() {
    init_tracing();

    let output_dir = Path::new(TEST_OUTPUT_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("创建 SDP 输出目录失败");
    }

    // Step 1: 创建测试脚本
    let script_path = create_sdp_test_script(output_dir);

    // Step 2: 构建请求
    let request = SdpRequest {
        subtitle_path: Path::new(SUBTITLE_PATH).to_path_buf(),
        video_path: Path::new(VIDEO_PATH).to_path_buf(),
        script_path: Some(script_path.clone()),
        temperature: 0.1,
        custom_clips: 2,
        original_volume: 0.7,
        bgm_volume: 0.3,
        bgm_path: None,
        output_dir: Some(output_dir.to_path_buf()),
        threads: 4,
    };

    // Step 3: 校验参数
    request.validate().expect("SDP 请求参数校验失败");

    tracing::info!("开始 SDP 流水线...");
    let start = Instant::now();

    // Step 4: 运行流水线
    let config = AppConfig::default();
    let result = run_sdp(request, &config, None).await;

    let elapsed = start.elapsed();
    tracing::info!("SDP 流水线完成, 耗时 {:.2}s", elapsed.as_secs_f64());

    match result {
        Ok(output_path) => {
            tracing::info!(
                "SDP 输出: {:?} (耗时 {:.2}s)",
                output_path,
                elapsed.as_secs_f64()
            );

            assert!(
                output_path.exists(),
                "SDP 输出视频文件应存在: {:?}",
                output_path
            );

            let file_size = std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0);
            assert!(
                file_size > 100_000,
                "SDP 输出视频应大于 100KB, 实际: {} bytes",
                file_size
            );

            tracing::info!(
                "SDP 输出文件: {:?} ({} bytes, {:.1}s)",
                output_path,
                file_size,
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            tracing::error!("SDP 流水线失败: {}", e);
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
            panic!("SDP 流水线测试失败: {}", e);
        }
    }
}
