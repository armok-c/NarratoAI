use std::path::PathBuf;

use narratoai_core::jianying::builder::{export_draft, ExportRequest};
use narratoai_core::script::types::Script;

use crate::error::{validate_path, CommandError};

/// 导出项目时间线为剪映草稿 JSON 格式
#[tauri::command]
pub async fn export_jianying_draft(
    script: Script,
    video_origin_path: PathBuf,
    draft_path: PathBuf,
    draft_name: String,
    width: u32,
    height: u32,
) -> Result<PathBuf, CommandError> {
    validate_path(&video_origin_path)?;
    validate_path(&draft_path)?;
    // IPC 边界校验 draft_name，防止路径遍历
    if draft_name.contains('/')
        || draft_name.contains('\\')
        || draft_name.contains("..")
        || draft_name.contains('\0')
    {
        return Err(CommandError {
            code: "INVALID_PATH".into(),
            message: "草稿名称包含非法字符".into(),
        });
    }
    let request = ExportRequest {
        script,
        video_origin_path,
        draft_path,
        draft_name,
        width,
        height,
    };
    let result = tokio::task::spawn_blocking(move || export_draft(&request))
        .await
        .map_err(|e| CommandError {
            code: "INTERNAL_ERROR".into(),
            message: format!("任务执行失败: {}", e),
        })??;
    Ok(result)
}
