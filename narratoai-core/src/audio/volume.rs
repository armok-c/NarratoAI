//! 智能音量控制模块
//!
//! 提供类型安全的音量配置和基于内容类型的预设音量 profile。
//! 1:1 对齐 Python 版 `audio_config.py`。
//!
//! ## 函数概览
//!
//! - `get_optimized_volumes()` — 按视频类型返回推荐的音量配置
//! - `apply_volume_profile()` — 应用命名 profile（balanced / voice_focused / ...）
//! - `get_recommended_volumes_for_content()` — 按内容类型推荐音量
//! - `validate_volume()` — 将音量值 clamp 到 [0.0, 2.0] 范围

use crate::config::types::AudioSection;

/// 类型安全的音量配置
///
/// 包含 TTS、原声、BGM 三路音量值，编译时字段检查。
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VolumeConfig {
    pub tts_volume: f64,
    pub original_volume: f64,
    pub bgm_volume: f64,
}

impl VolumeConfig {
    /// 验证所有音量值在 [0.0, 2.0] 范围内
    pub fn validate(&self) -> Result<(), super::normalizer::AudioError> {
        for (name, val) in [
            ("tts_volume", self.tts_volume),
            ("original_volume", self.original_volume),
            ("bgm_volume", self.bgm_volume),
        ] {
            if val.is_nan() || val < 0.0 || val > 2.0 {
                return Err(super::normalizer::AudioError::InvalidVolume(
                    format!("{}={} 不在 [0.0, 2.0] 范围内", name, val),
                ));
            }
        }
        Ok(())
    }
}

/// 音频处理配置
///
/// 控制是否启用智能音量 / LUFS 标准化，以及目标响度参数。
pub struct ProcessingConfig {
    pub enable_smart_volume: bool,
    pub enable_audio_normalization: bool,
    pub target_lufs: f64,
    pub max_peak: f64,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            enable_smart_volume: false,
            enable_audio_normalization: false,
            target_lufs: -14.0,
            max_peak: -1.0,
        }
    }
}

impl ProcessingConfig {
    /// 从配置段构建处理配置
    pub fn from_section(section: &AudioSection) -> Self {
        Self {
            enable_smart_volume: section.enable_smart_volume,
            enable_audio_normalization: section.enable_audio_normalization,
            target_lufs: section.target_lufs,
            max_peak: section.max_peak,
        }
    }
}

/// 音频混合配置
///
/// 仅定义类型，FFmpeg 交叉淡化实现在 Phase 6。
pub struct MixingConfig {
    pub crossfade_duration: f64,
    pub bgm_fade_out: f64,
    pub dynamic_range_compression: bool,
}

impl Default for MixingConfig {
    fn default() -> Self {
        Self {
            crossfade_duration: 0.1,
            bgm_fade_out: 3.0,
            dynamic_range_compression: false,
        }
    }
}

impl MixingConfig {
    /// 从配置段构建混合配置
    pub fn from_section(section: &AudioSection) -> Self {
        Self {
            crossfade_duration: section.crossfade_duration,
            bgm_fade_out: section.bgm_fade_out,
            // dynamic_range_compression 尚未暴露到配置层
            dynamic_range_compression: false,
        }
    }
}

/// 按视频类型返回优化的音量配置
///
/// 预定义四种视频类型，默认值对应 entertainment 类。
pub fn get_optimized_volumes(video_type: &str) -> VolumeConfig {
    match video_type {
        "educational" => VolumeConfig {
            tts_volume: 0.9,
            original_volume: 0.8,
            bgm_volume: 0.2,
        },
        "entertainment" => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.2,
            bgm_volume: 0.4,
        },
        "news" => VolumeConfig {
            tts_volume: 1.0,
            original_volume: 0.6,
            bgm_volume: 0.1,
        },
        "documentary" => VolumeConfig {
            tts_volume: 1.0,
            original_volume: 0.5,
            bgm_volume: 0.2,
        },
        "sde" => VolumeConfig {
            tts_volume: 0.9,
            original_volume: 0.8,
            bgm_volume: 0.3,
        },
        "sdp" => VolumeConfig {
            tts_volume: 0.0,
            original_volume: 1.0,
            bgm_volume: 0.3,
        },
        _ => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
        }, // default
    }
}

/// 应用命名音量 profile
///
/// 四个硬编码 profile：balanced, voice_focused, original_focused, quiet_background。
pub fn apply_volume_profile(profile_name: &str) -> VolumeConfig {
    match profile_name {
        "balanced" => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.2,
            bgm_volume: 0.3,
        },
        "voice_focused" => VolumeConfig {
            tts_volume: 1.0,
            original_volume: 0.7,
            bgm_volume: 0.2,
        },
        "original_focused" => VolumeConfig {
            tts_volume: 0.7,
            original_volume: 1.5,
            bgm_volume: 0.2,
        },
        "quiet_background" => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.1,
        },
        _ => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
        }, // default
    }
}

/// 按内容类型推荐音量配置
///
/// 四种内容类型：mixed, voice_only, original_heavy, music_video。
pub fn get_recommended_volumes_for_content(content_type: &str) -> VolumeConfig {
    match content_type {
        "mixed" => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
        },
        "voice_only" => VolumeConfig {
            tts_volume: 1.0,
            original_volume: 0.5,
            bgm_volume: 0.2,
        },
        "original_heavy" => VolumeConfig {
            tts_volume: 0.6,
            original_volume: 1.6,
            bgm_volume: 0.1,
        },
        "music_video" => VolumeConfig {
            tts_volume: 0.7,
            original_volume: 1.8,
            bgm_volume: 0.0,
        },
        _ => VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
        },
    }
}

/// 验证并调整音量值到 [0.0, 2.0] 范围
///
/// 超出范围时记录 warning 并 clamp。
pub fn validate_volume(volume: f64, name: &str) -> f64 {
    if volume.is_nan() || volume.is_infinite() {
        tracing::warn!("{} 音量值为 NaN 或 Infinity，已调整为 0.0", name);
        return 0.0;
    }
    let min = 0.0;
    let max = 2.0;
    if volume < min {
        tracing::warn!("{} 音量 {} 低于最小值 {}，已调整", name, volume, min);
        min
    } else if volume > max {
        tracing::warn!("{} 音量 {} 超过最大值 {}，已调整", name, volume, max);
        max
    } else {
        volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_config_validate_ok() {
        let v = VolumeConfig {
            tts_volume: 0.8,
            original_volume: 1.3,
            bgm_volume: 0.3,
        };
        assert!(v.validate().is_ok());
    }

    #[test]
    fn test_volume_config_validate_out_of_range() {
        let v = VolumeConfig {
            tts_volume: 2.5,
            original_volume: 1.3,
            bgm_volume: 0.3,
        };
        assert!(v.validate().is_err());
    }

    #[test]
    fn test_get_optimized_volumes_default() {
        let v = get_optimized_volumes("default");
        assert!((v.tts_volume - 0.8).abs() < 0.01);
        assert!((v.original_volume - 1.3).abs() < 0.01);
        assert!((v.bgm_volume - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_get_optimized_volumes_educational() {
        let v = get_optimized_volumes("educational");
        assert!((v.tts_volume - 0.9).abs() < 0.01);
        assert!((v.original_volume - 0.8).abs() < 0.01);
        assert!((v.bgm_volume - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_apply_volume_profile_balanced() {
        let v = apply_volume_profile("balanced");
        assert!((v.tts_volume - 0.8).abs() < 0.01);
        assert!((v.original_volume - 1.2).abs() < 0.01);
        assert!((v.bgm_volume - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_apply_volume_profile_voice_focused() {
        let v = apply_volume_profile("voice_focused");
        assert!((v.tts_volume - 1.0).abs() < 0.01);
        assert!((v.original_volume - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_apply_volume_profile_original_focused() {
        let v = apply_volume_profile("original_focused");
        assert!((v.original_volume - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_apply_volume_profile_quiet_background() {
        let v = apply_volume_profile("quiet_background");
        assert!((v.bgm_volume - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_apply_volume_profile_unknown_returns_default() {
        let v = apply_volume_profile("unknown_profile");
        assert!((v.tts_volume - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_get_recommended_volumes_for_content() {
        let v = get_recommended_volumes_for_content("mixed");
        assert!((v.tts_volume - 0.8).abs() < 0.01);
        assert!((v.original_volume - 1.3).abs() < 0.01);

        let v = get_recommended_volumes_for_content("voice_only");
        assert!((v.tts_volume - 1.0).abs() < 0.01);

        let v = get_recommended_volumes_for_content("music_video");
        assert!((v.bgm_volume - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_volume_clamps() {
        assert!((validate_volume(1.5, "test") - 1.5).abs() < 0.01);
        assert!((validate_volume(-1.0, "test") - 0.0).abs() < 0.01);
        assert!((validate_volume(3.0, "test") - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_processing_config_default() {
        let cfg = ProcessingConfig::default();
        assert!(!cfg.enable_smart_volume);
        assert!(!cfg.enable_audio_normalization);
        assert!((cfg.target_lufs - (-14.0)).abs() < 0.01);
    }

    #[test]
    fn test_get_optimized_volumes_documentary() {
        let v = get_optimized_volumes("documentary");
        assert!((v.tts_volume - 1.0).abs() < 0.01);
        assert!((v.original_volume - 0.5).abs() < 0.01);
        assert!((v.bgm_volume - 0.2).abs() < 0.01);
    }
    #[test]
    fn test_get_optimized_volumes_sde() {
        let v = get_optimized_volumes("sde");
        assert!((v.tts_volume - 0.9).abs() < 0.01);
        assert!((v.original_volume - 0.8).abs() < 0.01);
        assert!((v.bgm_volume - 0.3).abs() < 0.01);
    }
    #[test]
    fn test_get_optimized_volumes_sdp() {
        let v = get_optimized_volumes("sdp");
        assert!((v.tts_volume - 0.0).abs() < 0.01);
        assert!((v.original_volume - 1.0).abs() < 0.01);
        assert!((v.bgm_volume - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_mixing_config_default() {
        let cfg = MixingConfig::default();
        assert!((cfg.crossfade_duration - 0.1).abs() < 0.01);
        assert!((cfg.bgm_fade_out - 3.0).abs() < 0.01);
    }
}
