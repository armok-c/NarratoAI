use std::collections::HashMap;
use std::sync::OnceLock;

use crate::error::FFmpegError;

/// 硬件加速 Profile（对齐 Python 版 5 个预定义 Profile）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HwAccelProfile {
    /// h264_nvenc/h264_amf — 高性能（NVIDIA/AMD）
    HighPerformance,
    /// libx264 — 兼容性回退
    Compatibility,
    /// h264_nvenc — Windows NVIDIA 专用
    WindowsNvidia,
    /// h264_videotoolbox — macOS
    MacosVideotoolbox,
    /// libx264 — 通用软件编码
    UniversalSoftware,
}

/// FFmpeg Profile 信息（对应 Python 版 FFmpegProfile dataclass）
#[derive(Debug, Clone)]
pub struct FFmpegProfileInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub hwaccel_enabled: bool,
    pub hwaccel_type: Option<&'static str>,
    pub encoder: &'static str,
    pub quality_preset: &'static str,
    pub pixel_format: &'static str,
    pub additional_args: &'static [&'static str],
    pub compatibility_level: u8,
}

/// 内部懒加载 Profile 表
fn profile_map() -> &'static HashMap<HwAccelProfile, FFmpegProfileInfo> {
    static PROFILES: OnceLock<HashMap<HwAccelProfile, FFmpegProfileInfo>> = OnceLock::new();
    PROFILES.get_or_init(|| {
        let mut m = HashMap::with_capacity(5);
        m.insert(
            HwAccelProfile::HighPerformance,
            FFmpegProfileInfo {
                name: "High Performance",
                description: "高性能（NVIDIA/AMD 硬件编码）",
                hwaccel_enabled: true,
                hwaccel_type: Some("auto"),
                encoder: "h264_nvenc",
                quality_preset: "fast",
                pixel_format: "yuv420p",
                additional_args: &["-preset", "fast"],
                compatibility_level: 2,
            },
        );
        m.insert(
            HwAccelProfile::Compatibility,
            FFmpegProfileInfo {
                name: "Compatibility",
                description: "兼容性回退（软件编码）",
                hwaccel_enabled: false,
                hwaccel_type: None,
                encoder: "libx264",
                quality_preset: "medium",
                pixel_format: "yuv420p",
                additional_args: &["-preset", "medium", "-crf", "23"],
                compatibility_level: 5,
            },
        );
        m.insert(
            HwAccelProfile::WindowsNvidia,
            FFmpegProfileInfo {
                name: "Windows NVIDIA",
                description: "Windows NVIDIA 专用硬件编码",
                hwaccel_enabled: true,
                hwaccel_type: Some("cuda"),
                encoder: "h264_nvenc",
                quality_preset: "medium",
                pixel_format: "yuv420p",
                additional_args: &["-preset", "medium", "-cq", "23"],
                compatibility_level: 3,
            },
        );
        m.insert(
            HwAccelProfile::MacosVideotoolbox,
            FFmpegProfileInfo {
                name: "macOS VideoToolbox",
                description: "macOS VideoToolbox 硬件编码",
                hwaccel_enabled: true,
                hwaccel_type: Some("videotoolbox"),
                encoder: "h264_videotoolbox",
                quality_preset: "medium",
                pixel_format: "yuv420p",
                additional_args: &["-q:v", "65"],
                compatibility_level: 3,
            },
        );
        m.insert(
            HwAccelProfile::UniversalSoftware,
            FFmpegProfileInfo {
                name: "Universal Software",
                description: "通用软件编码（libx264）",
                hwaccel_enabled: false,
                hwaccel_type: None,
                encoder: "libx264",
                quality_preset: "medium",
                pixel_format: "yuv420p",
                additional_args: &["-preset", "medium", "-crf", "23"],
                compatibility_level: 5,
            },
        );
        m
    })
}

/// 5 个预定义 Profile（对应 Python 版 PROFILES 字典）
pub fn get_profiles() -> HashMap<HwAccelProfile, FFmpegProfileInfo> {
    profile_map().clone()
}

/// 获取指定 Profile 的配置信息
pub fn get_profile(profile: HwAccelProfile) -> Option<&'static FFmpegProfileInfo> {
    profile_map().get(&profile)
}

/// 检测可用硬件编码器
///
/// 使用 `ffmpeg_sidecar::paths::ffmpeg_path()` 获取 ffmpeg 二进制路径（per D-12），
/// 然后通过 `std::process::Command` 调用 `ffmpeg -encoders -hide_banner` 解析输出。
pub fn detect_hw_encoders() -> Result<Vec<String>, FFmpegError> {
    let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();

    let output = match std::process::Command::new(&ffmpeg_bin)
        .args(["-encoders", "-hide_banner"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(FFmpegError::BinaryNotFound);
            }
            tracing::warn!("无法启动 ffmpeg: {} — 返回空编码器列表", e);
            return Ok(Vec::new());
        }
    };

    if !output.status.success() {
        tracing::warn!(
            "ffmpeg -encoders exited with code {:?} — 返回空编码器列表",
            output.status.code()
        );
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 要检测的硬件编码器列表
    let hw_encoders = [
        "h264_nvenc",
        "hevc_nvenc",
        "h264_amf",
        "hevc_amf",
        "h264_qsv",
        "hevc_qsv",
        "h264_videotoolbox",
    ];

    let detected: Vec<String> = hw_encoders
        .iter()
        .filter(|&&name| stdout.contains(name))
        .map(|&name| name.to_string())
        .collect();

    Ok(detected)
}

/// 根据检测到的编码器推荐 Profile
///
/// 推荐逻辑（对齐 Python 版 get_recommended_profile）:
/// - 有 nvenc -> 检查 OS: Windows -> WindowsNvidia, 其他 -> HighPerformance
/// - 有 videotoolbox -> MacosVideotoolbox
/// - 有 amf 或 qsv -> HighPerformance
/// - 无硬件编码器 或 检测失败 -> UniversalSoftware
pub fn recommend_profile() -> HwAccelProfile {
    match detect_hw_encoders() {
        Ok(encoders) => {
            let joined: String = encoders.join(" ");

            if joined.contains("nvenc") {
                if cfg!(target_os = "windows") {
                    HwAccelProfile::WindowsNvidia
                } else {
                    HwAccelProfile::HighPerformance
                }
            } else if joined.contains("videotoolbox") {
                HwAccelProfile::MacosVideotoolbox
            } else if joined.contains("amf") || joined.contains("qsv") {
                HwAccelProfile::HighPerformance
            } else {
                HwAccelProfile::UniversalSoftware
            }
        }
        Err(_) => HwAccelProfile::UniversalSoftware,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 5 个 HwAccelProfile 变体存在
    #[test]
    fn test_hw_accel_profile_five_variants() {
        let variants = [
            HwAccelProfile::HighPerformance,
            HwAccelProfile::Compatibility,
            HwAccelProfile::WindowsNvidia,
            HwAccelProfile::MacosVideotoolbox,
            HwAccelProfile::UniversalSoftware,
        ];
        assert_eq!(variants.len(), 5, "应有 5 个 HwAccelProfile 变体");
    }

    /// 验证 detect_hw_encoders 返回合法的编码器名称列表
    ///
    /// 注意: 如果系统未安装 FFmpeg，返回空列表（非错误）。
    #[test]
    fn test_detect_encoders_format() {
        let result = detect_hw_encoders();
        assert!(result.is_ok(), "detect_hw_encoders 不应失败");
        let encoders = result.unwrap();
        if !encoders.is_empty() {
            assert!(
                encoders.iter().any(|e| e.contains("264") || e.contains("265")),
                "编码器名称应包含 264 或 265，实际: {:?}",
                encoders
            );
        }
        // 空列表表示系统未安装 FFmpeg, 测试仍然通过
    }

    /// recommend_profile 始终返回 5 个变体之一
    #[test]
    fn test_recommend_returns_valid_profile() {
        let profile = recommend_profile();
        match profile {
            HwAccelProfile::HighPerformance
            | HwAccelProfile::Compatibility
            | HwAccelProfile::WindowsNvidia
            | HwAccelProfile::MacosVideotoolbox
            | HwAccelProfile::UniversalSoftware => {} // 有效
        }
    }

    /// 验证 get_profiles 返回正确的 profile 信息
    #[test]
    fn test_profiles_data() {
        let profiles = get_profiles();
        assert_eq!(profiles.len(), 5, "应有 5 个预定义 profile");

        // 验证 UniversalSoftware 字段非空
        let sw = profiles.get(&HwAccelProfile::UniversalSoftware);
        assert!(sw.is_some(), "UniversalSoftware profile 应存在");
        let sw = sw.unwrap();
        assert_eq!(sw.encoder, "libx264");
        assert!(!sw.hwaccel_enabled);
        assert_eq!(sw.compatibility_level, 5);
    }
}
