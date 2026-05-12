use std::path::{Path, PathBuf};

use crate::audio::normalizer::{normalize_audio_for_mixing, AudioError};
use crate::audio::volume::{get_optimized_volumes, VolumeConfig};
use crate::config::types::AudioSection;

/// 标准化合并后的音频文件
///
/// 1. 检查 `config.enable_audio_normalization` — 关闭时直接返回 input
/// 2. 开启时调用 `normalize_audio_for_mixing()` 生成标准化文件
/// 3. 原地覆盖（std::fs::rename）原始文件
///
/// # 参数
///
/// * `input` — 合并后的音频文件路径（如 merger_audio.mp3）
/// * `config` — AudioSection 配置（target_lufs, sample_rate, channels 等）
pub fn normalize_merged_audio(
    input: &Path,
    config: &AudioSection,
) -> Result<PathBuf, AudioError> {
    if !config.enable_audio_normalization {
        tracing::info!("音频标准化已关闭，跳过");
        return Ok(input.to_path_buf());
    }

    let output_dir = input.parent().unwrap_or(Path::new("."));
    let temp = normalize_audio_for_mixing(
        input,
        output_dir,
        config.target_lufs,
        config.sample_rate,
        config.channels,
    )?;

    // 原地覆盖（D-03）
    std::fs::rename(&temp, input)
        .map_err(|e| AudioError::IoError(format!("标准化文件重命名失败: {}", e)))?;

    tracing::info!(
        "音频标准化完成 (target_lufs={})，原地覆盖: {}",
        config.target_lufs,
        input.display()
    );

    Ok(input.to_path_buf())
}

/// 解析智能音量覆盖
///
/// 1. 检查 `config.enable_smart_volume` — 关闭时返回 request_volumes 原值
/// 2. 开启时调用 `get_optimized_volumes(video_type)` 获取优化值
/// 3. 通过 `tracing::info!` 输出最终生效的音量值和来源（D-13）
///
/// # 参数
///
/// * `request_volumes` — 请求结构体中的原始音量值（DocumentaryRequest / SdeRequest / SdpRequest）
/// * `video_type` — 视频类型：`"documentary"` / `"sde"` / `"sdp"`
/// * `config` — AudioSection 配置（含 enable_smart_volume 开关）
pub fn resolve_volumes(
    request_volumes: &VolumeConfig,
    video_type: &str,
    config: &AudioSection,
) -> VolumeConfig {
    if !config.enable_smart_volume {
        tracing::info!(
            "智能音量已关闭，使用请求值: tts={:.2}, original={:.2}, bgm={:.2} (来源=manual)",
            request_volumes.tts_volume,
            request_volumes.original_volume,
            request_volumes.bgm_volume,
        );
        return *request_volumes;
    }

    let optimized = get_optimized_volumes(video_type);

    tracing::info!(
        "智能音量生效: video_type={}, tts={:.2}, original={:.2}, bgm={:.2} (来源=smart)",
        video_type,
        optimized.tts_volume,
        optimized.original_volume,
        optimized.bgm_volume,
    );

    optimized
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_audio_section(
        enable_normalization: bool,
        enable_smart_volume: bool,
    ) -> AudioSection {
        AudioSection {
            enable_audio_normalization: enable_normalization,
            enable_smart_volume,
            target_lufs: -14.0,
            sample_rate: 44100,
            channels: 2,
            ..Default::default()
        }
    }

    #[test]
    fn test_normalize_merged_audio_disabled() {
        let config = make_audio_section(false, false);
        // 当开关关闭时，应直接返回原路径（不报错）
        let input = Path::new("/tmp/nonexistent_audio.mp3");
        let result = normalize_merged_audio(input, &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input.to_path_buf());
    }

    #[test]
    fn test_resolve_volumes_disabled_returns_request() {
        let config = make_audio_section(false, false);
        let req = VolumeConfig {
            tts_volume: 0.5,
            original_volume: 1.0,
            bgm_volume: 0.1,
        };
        let result = resolve_volumes(&req, "documentary", &config);
        assert!((result.tts_volume - 0.5).abs() < 0.01);
        assert!((result.original_volume - 1.0).abs() < 0.01);
        assert!((result.bgm_volume - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_resolve_volumes_enabled_returns_optimized() {
        let config = make_audio_section(false, true);
        let req = VolumeConfig {
            tts_volume: 0.5,
            original_volume: 1.0,
            bgm_volume: 0.1,
        };
        let result = resolve_volumes(&req, "documentary", &config);
        // get_optimized_volumes("documentary") D-09: tts=1.0, original=0.5, bgm=0.2
        assert!((result.tts_volume - 1.0).abs() < 0.01, "documentary 默认 tts 应为 1.0");
        assert!((result.original_volume - 0.5).abs() < 0.01, "documentary original 应为 0.5");
        assert!((result.bgm_volume - 0.2).abs() < 0.01, "documentary bgm 应为 0.2");
    }

    #[test]
    fn test_resolve_volumes_unknown_type_uses_default() {
        let config = make_audio_section(false, true);
        let req = VolumeConfig::default();
        let result = resolve_volumes(&req, "unknown_type", &config);
        // 应返回 get_optimized_volumes 的 fallback 分支
        assert!((result.bgm_volume - 0.3).abs() < 0.01);
    }
}
