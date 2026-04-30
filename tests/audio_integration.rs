//! 音频标准化集成测试
//!
//! 需要系统安装 FFmpeg。使用 `cargo test -- --ignored` 运行。

use std::process::Command;

/// 获取 FFmpeg 二进制路径（复用 tests/ffmpeg_test.rs 模式）
fn find_ffmpeg() -> String {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return "ffmpeg".to_string();
        }
    }
    panic!("FFmpeg 不可用。请安装 FFmpeg 或等待 ffmpeg-sidecar 自动下载。");
}

#[test]
#[ignore]
fn test_normalize_lufs_two_pass() {
    let _ffmpeg = find_ffmpeg();
    // 集成测试：使用 ffmpeg 生成测试音频 → normalize_lufs → 验证输出
    // 完整实现在执行阶段补充
    assert!(true, "存根 — 执行阶段填充具体测试逻辑");
}

#[test]
#[ignore]
fn test_get_audio_rms_with_real_file() {
    let _ffmpeg = find_ffmpeg();
    // 集成测试：生成已知响度的测试音频 → get_audio_rms → 验证 RMS 值
    assert!(true, "存根 — 执行阶段填充具体测试逻辑");
}
