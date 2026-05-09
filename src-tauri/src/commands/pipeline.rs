use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use narratoai_core::config::types::AppConfig;
use narratoai_core::documentary::types::DocumentaryRequest;
use narratoai_core::sde::types::SdeRequest;
use narratoai_core::sdp::types::SdpRequest;
use narratoai_core::documentary::script_gen::ScriptGenRequest;
use narratoai_core::script::types::Script;

use crate::error::CommandError;
use crate::models::progress::{CommandResponse, ProgressPayload};

// =============================================================
//  命令 1: run_documentary（D-04）
//  执行完整 6 步纪录片流水线，实时推送进度事件
// =============================================================
#[tauri::command]
pub async fn run_documentary(
    app: AppHandle,
    request: DocumentaryRequest,
    config: tauri::State<'_, AppConfig>,
) -> Result<CommandResponse, CommandError> {
    let task_id = Uuid::new_v4().to_string();
    let progress_task_id = task_id.clone();

    // D-11: 构建 Tauri 感知的进度回调闭包
    let app_handle = app.clone();
    let total_steps: u32 = 6;
    let progress_callback: narratoai_core::documentary::types::ProgressCallback =
        Box::new(move |step_name: &str, pct: f32, msg: &str| {
            let step_index = match step_name {
                "load_script" => 0,
                "tts" => 1,
                "clip" => 2,
                "merge_audio" => 3,
                "concat" => 4,
                "composite" => 5,
                _ => 0,
            };
            let payload = ProgressPayload {
                pipeline_type: "documentary".to_string(),
                task_id: progress_task_id.clone(),
                step_name: step_name.to_string(),
                percent: pct,
                message: msg.to_string(),
                step_index,
                total_steps,
                status: "running".to_string(),
                error_code: None,
                error_message: None,
            };
            let _ = app_handle.emit("pipeline-progress", payload);
        });

    // WR-06: 从配置中提取 proxy 设置
    let proxy_section = (config.proxy.enabled.then_some(&config.proxy));
    let output_path = narratoai_core::documentary::pipeline::run_documentary(
        request,
        &config,
        proxy_section,
        Some(progress_callback),
    ).await?;

    Ok(CommandResponse {
        output_video_path: output_path,
    })
}

// =============================================================
//  命令 2: run_sde（D-04）
//  执行完整 9 步 SDE 流水线，实时推送进度事件
//  需要全局 Registry 和 PromptManager（通过 Tauri State 注入）
// =============================================================
#[tauri::command]
pub async fn run_sde(
    app: AppHandle,
    request: SdeRequest,
    config: tauri::State<'_, AppConfig>,
    registry: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<narratoai_core::llm::registry::Registry>>>,
    prompt_manager: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<narratoai_core::prompt::manager::PromptManager>>>,
) -> Result<CommandResponse, CommandError> {
    let task_id = Uuid::new_v4().to_string();
    let progress_task_id = task_id.clone();

    let app_handle = app.clone();
    let total_steps: u32 = 9;
    let progress_callback: narratoai_core::sde::types::SdeProgressCallback =
        Box::new(move |step_name: &str, pct: f32, msg: &str| {
            let step_index = match step_name {
                "parse_subtitle" => 0,
                "plot_analysis" => 1,
                "script_generate" => 2,
                "load_script" => 3,
                "tts" => 4,
                "clip" => 5,
                "merge_audio" => 6,
                "concat" => 7,
                "composite" => 8,
                _ => 0,
            };
            let payload = ProgressPayload {
                pipeline_type: "sde".to_string(),
                task_id: progress_task_id.clone(),
                step_name: step_name.to_string(),
                percent: pct,
                message: msg.to_string(),
                step_index,
                total_steps,
                status: "running".to_string(),
                error_code: None,
                error_message: None,
            };
            let _ = app_handle.emit("pipeline-progress", payload);
        });

    let registry = registry.read().await;
    let pm = prompt_manager.read().await;

    // WR-06: 从配置中提取 proxy 设置
    let proxy_section = (config.proxy.enabled.then_some(&config.proxy));
    let output_path = narratoai_core::sde::pipeline::run_sde(
        request,
        &config,
        proxy_section,
        &registry,
        &pm,
        Some(progress_callback),
    ).await?;

    Ok(CommandResponse {
        output_video_path: output_path,
    })
}

// =============================================================
//  命令 3: run_sdp（D-04）
//  执行完整 3 步 SDP 流水线，实时推送进度事件
// =============================================================
#[tauri::command]
pub async fn run_sdp(
    app: AppHandle,
    request: SdpRequest,
    _config: tauri::State<'_, AppConfig>,
) -> Result<CommandResponse, CommandError> {
    let task_id = Uuid::new_v4().to_string();
    let progress_task_id = task_id.clone();

    let app_handle = app.clone();
    let total_steps: u32 = 3;
    let progress_callback: narratoai_core::sdp::types::SdpProgressCallback =
        Box::new(move |step_name: &str, pct: f32, msg: &str| {
            let step_index = match step_name {
                "clip" => 0,
                "concat" => 1,
                "composite" => 2,
                _ => 0,
            };
            let payload = ProgressPayload {
                pipeline_type: "sdp".to_string(),
                task_id: progress_task_id.clone(),
                step_name: step_name.to_string(),
                percent: pct,
                message: msg.to_string(),
                step_index,
                total_steps,
                status: "running".to_string(),
                error_code: None,
                error_message: None,
            };
            let _ = app_handle.emit("pipeline-progress", payload);
        });

    let output_path = narratoai_core::sdp::pipeline::run_sdp(
        request,
        Some(progress_callback),
    ).await?;

    Ok(CommandResponse {
        output_video_path: output_path,
    })
}

// =============================================================
//  命令 4: generate_documentary_script（D-04）
//  通过视觉 LLM + 文本 LLM 生成纪录片解说脚本
// =============================================================
#[tauri::command]
pub async fn generate_documentary_script(
    app: AppHandle,
    request: ScriptGenRequest,
    config: tauri::State<'_, AppConfig>,
    registry: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<narratoai_core::llm::registry::Registry>>>,
) -> Result<Script, CommandError> {
    // 从配置中获取 vision 和 text LLM provider（作用域化，确保 guard 在 .await 前释放）
    let (vision_provider, text_provider) = {
        let guard = registry.read().await;
        let vision_name = request.vision_model.as_deref()
            .unwrap_or(&config.app.vision_llm_provider);
        let text_name = request.text_model.as_deref()
            .unwrap_or(&config.app.text_llm_provider);
        let vp = guard.get(vision_name).map_err(|e| CommandError {
            code: "PROVIDER_NOT_FOUND".into(),
            message: format!("获取 vision LLM provider 失败: {}", e),
        })?;
        let tp = guard.get(text_name).map_err(|e| CommandError {
            code: "PROVIDER_NOT_FOUND".into(),
            message: format!("获取 text LLM provider 失败: {}", e),
        })?;
        (vp, tp)
    };

    // WR-02: 构造 progress callback 以便向前端推送进度事件
    let task_id = Uuid::new_v4().to_string();
    let app_handle = app.clone();
    let progress_callback: Box<dyn Fn(f32, &str) + Send + Sync> =
        Box::new(move |pct: f32, msg: &str| {
            let payload = ProgressPayload {
                pipeline_type: "script_gen".to_string(),
                task_id: task_id.clone(),
                step_name: "script_gen".to_string(),
                percent: pct,
                message: msg.to_string(),
                step_index: 0,
                total_steps: 3,
                status: "running".to_string(),
                error_code: None,
                error_message: None,
            };
            let _ = app_handle.emit("pipeline-progress", payload);
        });
    let mut request = request;
    request.progress = Some(progress_callback);

    let clips = narratoai_core::documentary::script_gen::generate_documentary_script(
        request,
        vision_provider.as_ref(),
        text_provider.as_ref(),
    ).await?;

    Ok(clips)
}

// =============================================================
//  命令 5: generate_sdp_script（D-04）
//  通过两步 LLM 调用生成短剧混剪脚本
// =============================================================
#[tauri::command]
pub async fn generate_sdp_script(
    subtitle_path: std::path::PathBuf,
    temperature: f64,
    custom_clips: u32,
    config: tauri::State<'_, AppConfig>,
    registry: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<narratoai_core::llm::registry::Registry>>>,
    prompt_manager: tauri::State<'_, std::sync::Arc<tokio::sync::RwLock<narratoai_core::prompt::manager::PromptManager>>>,
) -> Result<Script, CommandError> {
    // 校验字幕文件存在且扩展名合法
    if !subtitle_path.exists() {
        return Err(CommandError {
            code: "FILE_NOT_FOUND".into(),
            message: format!("字幕文件不存在: {}", subtitle_path.display()),
        });
    }
    if !subtitle_path.extension().map_or(false, |ext| {
        matches!(ext.to_str(), Some("srt") | Some("ass") | Some("ssa") | Some("vtt") | Some("txt"))
    }) {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "不支持的字幕文件格式".into(),
        });
    }

    // 从配置中获取 text LLM provider（作用域化，确保 registry guard 在 .await 前释放）
    let provider = {
        let guard = registry.read().await;
        guard.get(&config.app.text_llm_provider).map_err(|e| CommandError {
            code: "PROVIDER_NOT_FOUND".into(),
            message: format!("获取 LLM provider 失败: {}", e),
        })?
    };
    let pm = prompt_manager.read().await;

    let task_dir = std::env::temp_dir().join("narratoai").join(Uuid::new_v4().to_string());
    tokio::fs::create_dir_all(&task_dir).await.map_err(|e| CommandError {
        code: "IO_ERROR".into(),
        message: format!("创建临时目录失败: {}", e),
    })?;

    let script = narratoai_core::sdp::script_gen::generate_sdp_script(
        &subtitle_path,
        provider.as_ref(),
        &pm,
        &task_dir,
        temperature,
        custom_clips,
    ).await;

    // WR-03: 清理临时目录（无论成功或失败）
    let _ = tokio::fs::remove_dir_all(&task_dir).await;

    let script = script?;

    Ok(script)
}
