use std::path::PathBuf;

use narratoai_core::script::types::{OstType, Script};
use narratoai_core::script::edit;

use crate::error::{validate_path, CommandError};
use crate::models::progress::ScriptInfo;

/// 从 JSON 文件加载脚本，自动校验
#[tauri::command]
pub async fn load_script(
    path: PathBuf,
) -> Result<Script, CommandError> {
    validate_path(&path)?;
    if path.extension().map_or(true, |ext| ext != "json") {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "仅支持 .json 文件".into(),
        });
    }
    let script = narratoai_core::script::load_script(&path)?;
    Ok(script)
}

/// 将脚本保存为美化 JSON 文件
#[tauri::command]
pub async fn save_script(
    script: Script,
    path: PathBuf,
) -> Result<(), CommandError> {
    validate_path(&path)?;
    if path.extension().map_or(true, |ext| ext != "json") {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "仅支持 .json 文件".into(),
        });
    }
    narratoai_core::script::save_script(&script, &path)?;
    Ok(())
}

/// 校验脚本——所有错误一次性返回
#[tauri::command]
pub async fn validate_script(
    script: Script,
) -> Result<(), CommandError> {
    narratoai_core::script::validate(&script)?;
    Ok(())
}

/// 获取脚本基本信息
#[tauri::command]
pub async fn get_script_info(
    script: Script,
) -> Result<ScriptInfo, CommandError> {
    // 计算总时长（累加每个片段的实际持续时间），统计无法解析的片段数
    let mut total_duration_secs = 0.0;
    let mut unparseable_clips = 0;

    for clip in &script {
        let ts = &clip.timestamp;
        let parts: Vec<&str> = ts.splitn(2, '-').collect();
        if parts.len() != 2 {
            unparseable_clips += 1;
            continue;
        }
        let Some(start_ts) = parts.get(0) else { unparseable_clips += 1; continue; };
        let Some(end_ts) = parts.get(1) else { unparseable_clips += 1; continue; };
        let parse_time = |time_str: &str| -> Option<f64> {
            let time_parts: Vec<&str> = time_str.split(':').collect();
            if time_parts.len() != 3 { return None; }
            let h: f64 = time_parts[0].parse().ok()?;
            let m: f64 = time_parts[1].parse().ok()?;
            let s: f64 = {
                let sec_parts: Vec<&str> = time_parts[2].split(',').collect();
                let sec: f64 = sec_parts[0].parse().ok()?;
                let ms: f64 = sec_parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                sec + ms / 1000.0
            };
            Some(h * 3600.0 + m * 60.0 + s)
        };
        match (parse_time(start_ts), parse_time(end_ts)) {
            (Some(start), Some(end)) => {
                let dur = end - start;
                if dur < 0.0 {
                    unparseable_clips += 1;
                } else {
                    total_duration_secs += dur;
                }
            }
            _ => unparseable_clips += 1,
        }
    }

    Ok(ScriptInfo {
        clip_count: script.len(),
        total_duration_secs,
        unparseable_clips,
    })
}

/// 更新指定片段的解说文案
#[tauri::command]
pub async fn update_narration(
    script: Script,
    index: usize,
    text: String,
) -> Result<Script, CommandError> {
    let new_script = edit::update_narration(&script, index, &text)?;
    Ok(new_script)
}

/// 设置指定片段的 OST 类型
#[tauri::command]
pub async fn set_ost(
    script: Script,
    index: usize,
    ost: OstType,
) -> Result<Script, CommandError> {
    let new_script = edit::set_ost(&script, index, ost)?;
    Ok(new_script)
}

/// 更新指定片段的时间戳
#[tauri::command]
pub async fn update_timestamp(
    script: Script,
    index: usize,
    timestamp: String,
) -> Result<Script, CommandError> {
    let new_script = edit::update_timestamp(&script, index, &timestamp)?;
    Ok(new_script)
}
