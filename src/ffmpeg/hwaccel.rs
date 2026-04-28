use std::collections::HashMap;

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

/// 5 个预定义 Profile（对应 Python 版 PROFILES 字典）
pub fn get_profiles() -> HashMap<HwAccelProfile, FFmpegProfileInfo> {
    // RED 阶段: 返回空 map（测试期望 5 个 profile）
    HashMap::new()
}

/// 获取指定 Profile 的配置信息
pub fn get_profile(profile: HwAccelProfile) -> Option<&'static FFmpegProfileInfo> {
    // RED 阶段: 返回 None
    let _ = profile;
    None
}

/// 检测可用硬件编码器
///
/// 使用 `ffmpeg_sidecar::ffmpeg_path()` 获取 ffmpeg 二进制路径（per D-12），
/// 然后通过 `std::process::Command` 调用 `ffmpeg -encoders` 解析输出。
pub fn detect_hw_encoders() -> Result<Vec<String>, FFmpegError> {
    // RED 阶段: 返回空列表（测试期望包含 libx264 或硬件编码器）
    Ok(Vec::new())
}

/// 根据检测到的编码器推荐 Profile
pub fn recommend_profile() -> HwAccelProfile {
    // RED 阶段: 始终返回 UniversalSoftware
    HwAccelProfile::UniversalSoftware
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 5 个 HwAccelProfile 变体存在
    #[test]
    fn test_hw_accel_profile_five_variants() {
        let variants = vec![
            HwAccelProfile::HighPerformance,
            HwAccelProfile::Compatibility,
            HwAccelProfile::WindowsNvidia,
            HwAccelProfile::MacosVideotoolbox,
            HwAccelProfile::UniversalSoftware,
        ];
        assert_eq!(variants.len(), 5, "应有 5 个 HwAccelProfile 变体");
    }

    /// 验证 detect_hw_encoders 返回合法的编码器名称列表
    #[test]
    fn test_detect_encoders_format() {
        let result = detect_hw_encoders();
        assert!(result.is_ok(), "detect_hw_encoders 不应失败");
        let encoders = result.unwrap();
        // 至少应包含一个软件编码器或硬件编码器
        assert!(
            !encoders.is_empty(),
            "编码器列表不应为空（至少应包含 libx264）"
        );
        assert!(
            encoders.iter().any(|e| e.contains("264") || e.contains("265")),
            "编码器名称应包含 264 或 265，实际: {:?}",
            encoders
        );
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

    /// 验证 get_profile 返回正确的 profile 信息
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
