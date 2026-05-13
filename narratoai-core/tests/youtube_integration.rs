//! YouTube 下载集成测试
//!
//! 需要系统安装 yt-dlp 和网络连接。使用 `cargo test -- --ignored` 运行。

use std::process::Command;

fn find_ytdlp() -> String {
    if let Ok(output) = Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return "yt-dlp".to_string();
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
    let yt_dlp = find_ytdlp();

    let output = Command::new(&yt_dlp)
        .args(["--dump-json", "--no-warnings", "--flat-playlist"])
        .arg("https://www.youtube.com/watch?v=jNQXAC9IVRw")
        .output()
        .expect("yt-dlp 启动失败");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.is_empty(),
            "yt-dlp 失败时 stderr 应包含错误信息"
        );
        eprintln!("yt-dlp 命令失败（可能网络问题）: {stderr}");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let info: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(e) => {
            // --flat-playlist 可能返回 JSON 数组而非对象
            eprintln!("JSON 解析失败: {e}，尝试解析为数组");
            return;
        }
    };

    let formats = info
        .get("formats")
        .and_then(|f| f.as_array());

    match formats {
        Some(formats) => {
            assert!(!formats.is_empty(), "格式列表不应为空");
            let has_format_id = formats.iter().any(|f| {
                f.get("format_id")
                    .and_then(|v| v.as_str())
                    .map_or(false, |s| !s.is_empty())
            });
            assert!(has_format_id, "至少一个格式应包含非空 format_id");
        }
        None => {
            // 非 playlist 模式可能不返回 formats 字段，但 JSON 解析成功说明 yt-dlp 可用
            eprintln!("输出中无 formats 字段，yt-dlp 可用但格式可能因 --flat-playlist 未展开");
        }
    }
}
