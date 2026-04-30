//! YouTube 下载集成测试
//!
//! 需要系统安装 yt-dlp。使用 `cargo test -- --ignored` 运行。

use std::process::Command;

/// 检测 yt-dlp 是否可用
fn find_ytdlp() -> String {
    if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return "yt-dlp".to_string();
        }
    }
    // macOS/Linux: `which yt-dlp`, Windows: `where yt-dlp`
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg("yt-dlp").output() {
            if output.status.success() {
                return "yt-dlp".to_string();
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(output) = Command::new("which").arg("yt-dlp").output() {
            if output.status.success() {
                return "yt-dlp".to_string();
            }
        }
    }
    panic!("yt-dlp 不可用。请访问 https://github.com/yt-dlp/yt-dlp/releases 下载安装。");
}

#[test]
#[ignore]
fn test_ytdlp_version() {
    let yt_dlp = find_ytdlp();
    let output = Command::new(&yt_dlp)
        .arg("--version")
        .output()
        .expect("yt-dlp 启动失败");
    assert!(output.status.success(), "yt-dlp --version 应返回 0");
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!version.is_empty(), "版本号不应为空");
}

#[test]
#[ignore]
fn test_get_video_formats() {
    let _yt_dlp = find_ytdlp();
    // 集成测试：对已知公开视频 URL 调用 get_video_formats → 验证格式列表非空
    // 完整实现在执行阶段补充
    assert!(true, "存根 — 执行阶段填充具体测试逻辑");
}
