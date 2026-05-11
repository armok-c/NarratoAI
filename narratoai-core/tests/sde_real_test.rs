//! Level 3: SDE 短剧解说流水线全链路测试（含真实 LLM）
//!
//! 9 步顺序：ParseSubtitle → PlotAnalysis → ScriptGenerate → LoadScript
//!            → Tts → Clip → MergeAudio → Concat → Composite
//!
//! 成本控制：
//! - LLM API 调用：2 次（剧情分析 + 脚本生成）
//! - TTS 调用：按生成脚本片段数量（edge_tts 免费）
//! - FFmpeg 处理：本地
//!
//! 测试产出在 E:\GitLib\NarratoAI\test_output\sde\ 目录

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use narratoai_core::config::ConfigManager;
use narratoai_core::llm::openai_compatible::{OpenAiCompatibleProvider, ProviderConfig};
use narratoai_core::llm::registry::Registry;
use narratoai_core::prompt::manager::PromptManager;
use narratoai_core::prompt::register::register_all_prompts;
use narratoai_core::prompt::registry::PromptRegistry;
use narratoai_core::sde::pipeline::run_sde;
use narratoai_core::sde::types::SdeRequest;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

const VIDEO_PATH: &str = "E:/GitLib/视频/华星大酒店.mp4";
const SUBTITLE_PATH: &str = "E:/GitLib/视频/华星大酒店.srt";
const TEST_OUTPUT_DIR: &str = "E:/GitLib/NarratoAI/test_output/sde";

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
    let base = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    if base.join("config.toml").exists() {
        base.join("config.toml")
    } else {
        Path::new("E:/GitLib/NarratoAI/config.toml").to_path_buf()
    }
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

/// SDE 短剧解说流水线全链路测试
///
/// 使用华星大酒店字幕和视频，通过 Gemini 2.5 Pro 分析剧情并生成解说脚本。
/// 然后执行完整的 TTS → 裁剪 → 合并 → 拼接 → 合成流程。
///
/// 共 2 次 LLM 调用（步骤 1 + 步骤 2），每个步骤约 2K-4K tokens。
#[tokio::test]
async fn test_sde_full_pipeline() {
    init_tracing();

    let output_dir = Path::new(TEST_OUTPUT_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("创建 SDE 输出目录失败");
    }

    // ── 1. 加载配置 ────────────────────────────────────────────
    let config_path = get_config_path();
    let config_manager = ConfigManager::load(&config_path).expect("配置文件加载失败");
    let config = config_manager.get();

    let proxy = if config.proxy.enabled && !config.proxy.http.is_empty() {
        Some(config.proxy.clone())
    } else {
        None
    };

    // ── 2. 初始化 LLM Provider ──────────────────────────────────
    let provider = OpenAiCompatibleProvider::new(ProviderConfig {
        api_key: config.app.text_openai_api_key.clone(),
        model_name: config.app.text_openai_model_name.clone(),
        base_url: config.app.text_openai_base_url.clone(),
        max_retries: 1,
        timeout_secs: 120,
        proxy_http: proxy.as_ref().and_then(|p| {
            if !p.http.is_empty() { Some(p.http.clone()) } else { None }
        }),
        proxy_https: proxy.as_ref().and_then(|p| {
            if !p.https.is_empty() { Some(p.https.clone()) } else { None }
        }),
    })
    .expect("LLM Provider 创建失败");

    // ── 3. 初始化 Registry ──────────────────────────────────────
    let mut registry = Registry::new();
    registry.register("openai", Arc::new(provider));

    // ── 4. 初始化 PromptManager ─────────────────────────────────
    let mut prompt_registry = PromptRegistry::new();
    register_all_prompts(&mut prompt_registry).expect("Prompt 注册失败");
    let prompt_manager = PromptManager::new(Arc::new(std::sync::RwLock::new(prompt_registry)));

    // ── 5. 构建 SDE 请求 ────────────────────────────────────────
    let request = SdeRequest {
        subtitle_path: Path::new(SUBTITLE_PATH).to_path_buf(),
        video_path: Path::new(VIDEO_PATH).to_path_buf(),
        drama_name: "华星大酒店".to_string(),
        temperature: 0.7,
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

    request.validate().expect("SDE 请求参数校验失败");

    tracing::info!("开始 SDE 全链路流水线 (9 步)...");
    let start = Instant::now();

    // ── 6. 运行流水线 ───────────────────────────────────────────
    let result = run_sde(request, &config, proxy.as_ref(), &registry, &prompt_manager, None).await;

    let elapsed = start.elapsed();
    tracing::info!("SDE 流水线完成, 耗时 {:.2}s", elapsed.as_secs_f64());

    match result {
        Ok(output_path) => {
            tracing::info!(
                "SDE 输出: {:?} (耗时 {:.2}s)",
                output_path,
                elapsed.as_secs_f64()
            );

            assert!(output_path.exists(), "SDE 输出视频应存在: {:?}", output_path);

            let file_size = std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0);
            assert!(file_size > 100_000, "输出视频应 > 100KB, 实际: {} bytes", file_size);

            tracing::info!(
                "SDE 输出文件: {:?} ({} bytes, {:.1}s)",
                output_path,
                file_size,
                elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            tracing::error!("SDE 流水线失败: {}", e);
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
            panic!("SDE 流水线测试失败: {}", e);
        }
    }
}
