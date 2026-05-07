use std::path::PathBuf;

use narratoai_core::jianying::builder::{export_draft, ExportRequest};
use narratoai_core::script::types::Script;

use crate::error::CommandError;

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
    let request = ExportRequest {
        script,
        video_origin_path,
        draft_path,
        draft_name,
        width,
        height,
    };
    let result = export_draft(&request)?;
    Ok(result)
}
