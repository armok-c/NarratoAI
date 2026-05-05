use crate::sde::error::SdeError;

/// SDE 音频合并——直接调用纪录片对应步骤
///
/// 与纪录片逻辑完全一致，无 SDE 特有逻辑。
/// 实际调用在 pipeline.rs 的步骤函数中直接调用纪录片 `crate::documentary::audio::merge_audio_files()`
/// 和 `crate::documentary::audio::merge_subtitle_files()`。
pub async fn sde_step_merge_audio_subtitle() -> Result<(), SdeError> {
    // 逻辑直接在 pipeline.rs 的步骤函数中实现
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sde_step_merge_audio_subtitle_ok() {
        let result = sde_step_merge_audio_subtitle().await;
        assert!(result.is_ok(), "空实现应始终返回 Ok");
    }
}
