//! YouTube 视频下载模块
//!
//! 通过 yt-dlp CLI 子进程完成视频格式查询和下载。
//! 所有 CLI 调用使用 args 数组，禁止 shell 字符串（per T-11-01）。

use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

/// YouTube 操作错误类型
///
/// 所有用户可见错误信息为中文（per D-26, D-27）。
#[derive(Error, Debug)]
pub enum YoutubeError {
    /// yt-dlp 子进程调用失败（IO 错误、非零退出码等）
    #[error("yt-dlp 子进程错误: {0}")]
    SubprocessError(String),

    /// 未找到匹配的视频格式
    #[error("未找到匹配的格式: {0}")]
    FormatNotFound(String),

    /// 提供的 URL 无效（不是 http:// 或 https:// 开头）
    #[error("无效的 URL: {0}")]
    InvalidUrl(String),

    /// 视频下载失败（yt-dlp 退出码非零）
    #[error("下载失败: {0}")]
    DownloadFailed(String),
}

/// 视频格式信息
///
/// 表示 yt-dlp --dump-json 输出中单个格式条目的摘要。
#[derive(Debug, Clone)]
pub struct VideoFormat {
    /// yt-dlp 格式 ID（如 "137", "299+140"）
    pub format_id: String,
    /// 分辨率字符串（如 "1080p", "720p"）
    pub resolution: String,
    /// 容器扩展名（如 "mp4", "webm"）
    pub ext: String,
    /// 视频编码器（如 "avc1.640028", "av01.0.05M.08", "none"）
    pub vcodec: String,
}

/// 下载结果
///
/// `download_video` 的返回值。
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// 下载到本地的文件路径
    pub output_path: PathBuf,
    /// 视频标题（从 yt-dlp --dump-json 提取）
    pub title: String,
}

/// 验证输出格式是否在支持列表中
///
/// 支持格式：`mp4`, `mkv`, `webm`, `flv`, `avi`。
/// 比较不区分大小写。
pub fn validate_format(output_format: &str) -> Result<(), YoutubeError> {
    let supported = ["mp4", "mkv", "webm", "flv", "avi"];
    let lower = output_format.to_lowercase();
    if supported.contains(&lower.as_str()) {
        Ok(())
    } else {
        Err(YoutubeError::FormatNotFound(format!(
            "不支持的输出格式: {}（支持: mp4, mkv, webm, flv, avi）",
            output_format
        )))
    }
}

/// 标准化分辨率字符串
///
/// "1080p60" → "1080p", "720p" → "720p", "N/A" → "N/A"
fn normalize_resolution(resolution: &str) -> String {
    if resolution.contains('p') {
        let parts: Vec<&str> = resolution.split('p').collect();
        format!("{}p", parts[0])
    } else {
        resolution.to_string()
    }
}

/// 获取视频可用格式列表
///
/// 使用 yt-dlp --dump-json 获取视频元数据，从中提取格式列表和视频标题。
/// 包含分辨率标准化逻辑（如 "1080p60" → "1080p"）。
///
/// # 参数
/// - `url`: YouTube 视频 URL
/// - `proxy_url`: 可选代理 URL（从 config.toml [proxy] 段读取）
///
/// # 返回
/// 元组 `(Vec<VideoFormat>, String)`，第二个元素为视频标题。
///
/// # 安全
/// - URL 前缀校验（必须以 http:// 或 https:// 开头），防止协议注入（per T-11-01, T-11-02）
/// - 所有 CLI 参数通过 args 数组传递，禁止 shell 字符串拼接（per T-11-01）
async fn get_video_formats(
    url: &str,
    proxy_url: Option<&str>,
) -> Result<(Vec<VideoFormat>, String), YoutubeError> {
    // URL 前缀校验（防止参数注入，per T-11-02）
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(YoutubeError::InvalidUrl(url.to_string()));
    }
    if url.contains('\n') || url.contains('\r') {
        return Err(YoutubeError::InvalidUrl("URL 包含非法换行符".into()));
    }

    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--dump-json", "--no-warnings", "--quiet", url]);

    if let Some(proxy) = proxy_url {
        if !proxy.is_empty() {
            if !proxy.starts_with("http://") && !proxy.starts_with("https://")
                && !proxy.starts_with("socks5://") && !proxy.starts_with("socks5h://")
            {
                return Err(YoutubeError::InvalidUrl(
                    format!("代理 URL 协议无效: {}（支持 http/https/socks5）", proxy),
                ));
            }
            cmd.args(["--proxy", proxy]);
        }
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| YoutubeError::SubprocessError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YoutubeError::SubprocessError(stderr.to_string()));
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| YoutubeError::SubprocessError(format!("JSON 解析失败: {}", e)))?;

    let formats = info["formats"]
        .as_array()
        .ok_or_else(|| YoutubeError::FormatNotFound("无格式数组".into()))?;

    let result: Vec<VideoFormat> = formats
        .iter()
        .map(|f| {
            let resolution = f["format_note"].as_str().unwrap_or("N/A").to_string();
            let base_res = normalize_resolution(&resolution);

            VideoFormat {
                format_id: f["format_id"].as_str().unwrap_or("").to_string(),
                resolution: base_res,
                ext: f["ext"].as_str().unwrap_or("").to_string(),
                vcodec: f["vcodec"].as_str().unwrap_or("none").to_string(),
            }
        })
        .collect();

    let title = info["title"]
        .as_str()
        .unwrap_or("unknown_title")
        .to_string();

    Ok((result, title))
}

/// 下载 YouTube 视频
///
/// 完整流程：
/// 1. URL 格式校验
/// 2. 输出格式校验
/// 3. 分辨率标准化（如 "1080p60" → "1080p"）
/// 4. 获取格式列表和标题（--dump-json），匹配目标分辨率
/// 5. 选择最佳格式（优先非 av01 编码）
/// 6. 通过 yt-dlp 下载
/// 7. 验证输出文件存在
///
/// # 参数
/// - `url`: YouTube 视频 URL
/// - `resolution`: 目标分辨率（如 "1080p", "720p", "1080p60"）
/// - `output_format`: 输出容器格式（如 "mp4", "mkv", "webm"）
/// - `rename`: 可选，指定输出文件名（不含扩展名）
/// - `proxy_url`: 可选代理 URL
///
/// # 安全
/// - URL 前缀校验（per T-11-01, T-11-02）
/// - 所有 CLI 参数通过 args 数组传递
/// - 禁止使用 shell 模式执行命令
///
/// # 注意
/// - 无下载进度回调（per D-10，与 Python 版行为一致）
pub async fn download_video(
    url: &str,
    resolution: &str,
    output_format: &str,
    rename: Option<&str>,
    proxy_url: Option<&str>,
) -> Result<DownloadResult, YoutubeError> {
    // 1. 校验 URL 格式
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(YoutubeError::InvalidUrl(url.to_string()));
    }
    if url.contains('\n') || url.contains('\r') {
        return Err(YoutubeError::InvalidUrl("URL 包含非法换行符".into()));
    }

    // 2. 校验输出格式
    validate_format(output_format)?;

    // 3. 标准化分辨率： "1080p60" → "1080p"
    let base_resolution = normalize_resolution(resolution);

    // 4. 获取格式列表，匹配目标分辨率
    let (formats, title) = get_video_formats(url, proxy_url).await?;

    // 5. 在匹配到目标分辨率的格式中，优先选非 av01 的
    let matched = formats
        .iter()
        .filter(|f| f.resolution == base_resolution && f.vcodec != "none")
        .find(|f| !f.vcodec.contains("av01"));

    let format_id = match matched {
        Some(f) => f.format_id.clone(),
        None => {
            // fallback: 匹配任何有视频的格式
            let fallback = formats
                .iter()
                .find(|f| f.vcodec != "none")
                .map(|f| f.format_id.clone())
                .ok_or_else(|| {
                    YoutubeError::FormatNotFound(format!(
                        "无可用视频格式（目标分辨率: {}）",
                        base_resolution
                    ))
                })?;
            if let Some(fb) = formats.iter().find(|f| f.format_id == fallback) {
                tracing::warn!(
                    "目标分辨率 {} 无匹配格式，回退到: {} ({})",
                    base_resolution,
                    fb.format_id,
                    fb.resolution
                );
            }
            fallback
        }
    };

    // 6. 下载（使用 tokio::process::Command，无进度回调 per D-10）
    let output_template = match rename {
        Some(name) => {
            if name.contains('/') || name.contains('\\') || name.contains("..") {
                return Err(YoutubeError::InvalidUrl("rename 参数包含非法路径字符".into()));
            }
            name.to_string()
        }
        None => format!("{}_video", base_resolution),
    };

    let output_ext = output_format.to_string();

    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "-f",
        &format!("{}+bestaudio", format_id),
        "--merge-output-format",
        &output_ext,
        "--no-warnings",
        "--quiet",
        "-o",
        &format!("{}.{}", output_template, output_ext),
        url,
    ]);

    if let Some(proxy) = proxy_url {
        if !proxy.is_empty() {
            if !proxy.starts_with("http://") && !proxy.starts_with("https://")
                && !proxy.starts_with("socks5://") && !proxy.starts_with("socks5h://")
            {
                return Err(YoutubeError::InvalidUrl(
                    format!("代理 URL 协议无效: {}（支持 http/https/socks5）", proxy),
                ));
            }
            cmd.args(["--proxy", proxy]);
        }
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| YoutubeError::SubprocessError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YoutubeError::DownloadFailed(stderr.to_string()));
    }

    let output_path = PathBuf::from(format!("{}.{}", output_template, output_ext));

    // 验证输出文件实际存在
    if !output_path.exists() {
        return Err(YoutubeError::DownloadFailed(format!(
            "下载完成后文件未找到: {}",
            output_path.display()
        )));
    }

    Ok(DownloadResult { output_path, title })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_error_chinese_display() {
        let err = YoutubeError::SubprocessError("test".into());
        assert!(err.to_string().contains("yt-dlp 子进程错误"));

        let err = YoutubeError::FormatNotFound("1080p".into());
        assert!(err.to_string().contains("未找到匹配的格式"));

        let err = YoutubeError::InvalidUrl("bad".into());
        assert!(err.to_string().contains("无效的 URL"));

        let err = YoutubeError::DownloadFailed("timeout".into());
        assert!(err.to_string().contains("下载失败"));
    }

    #[test]
    fn test_validate_format_supported() {
        assert!(validate_format("mp4").is_ok());
        assert!(validate_format("mkv").is_ok());
        assert!(validate_format("webm").is_ok());
    }

    #[test]
    fn test_validate_format_unsupported() {
        assert!(validate_format("avi").is_ok()); // 在支持列表中
        assert!(validate_format("mp3").is_err()); // 不在列表中
        assert!(validate_format("mov").is_err());
    }

    #[test]
    fn test_validate_format_case_insensitive() {
        assert!(validate_format("MP4").is_ok());
        assert!(validate_format("MKV").is_ok());
    }

    #[test]
    fn test_invalid_url_prefix() {
        let err = YoutubeError::InvalidUrl("ftp://example.com/video".into());
        assert!(err.to_string().contains("无效的 URL"));

        // 验证 download_video 拒绝非 http 前缀
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(download_video(
            "ftp://example.com/video",
            "1080p",
            "mp4",
            None,
            None,
        ));
        assert!(result.is_err());
        match result {
            Err(YoutubeError::InvalidUrl(_)) => {} // 预期
            _ => panic!("应返回 InvalidUrl 错误"),
        }
    }

    #[test]
    fn test_normalize_resolution() {
        assert_eq!(normalize_resolution("1080p60"), "1080p");
        assert_eq!(normalize_resolution("720p"), "720p");
        assert_eq!(normalize_resolution("2160p60"), "2160p");
        assert_eq!(normalize_resolution("N/A"), "N/A");
        assert_eq!(normalize_resolution(""), "");
    }

    #[test]
    fn test_format_selection_prefers_non_av01() {
        let formats = vec![
            VideoFormat {
                format_id: "1".into(),
                resolution: "1080p".into(),
                ext: "mp4".into(),
                vcodec: "av01.0.05M.08".into(),
            },
            VideoFormat {
                format_id: "2".into(),
                resolution: "1080p".into(),
                ext: "mp4".into(),
                vcodec: "avc1.640028".into(),
            },
            VideoFormat {
                format_id: "3".into(),
                resolution: "720p".into(),
                ext: "mp4".into(),
                vcodec: "avc1.4d401f".into(),
            },
        ];

        // 匹配 1080p 且非 av01
        let matched = formats
            .iter()
            .filter(|f| f.resolution == "1080p" && f.vcodec != "none")
            .find(|f| !f.vcodec.contains("av01"));

        assert!(matched.is_some());
        assert_eq!(matched.unwrap().format_id, "2");
    }

    #[test]
    fn test_videoformat_debug() {
        let fmt = VideoFormat {
            format_id: "137".into(),
            resolution: "1080p".into(),
            ext: "mp4".into(),
            vcodec: "avc1.640028".into(),
        };
        let debug_str = format!("{:?}", fmt);
        assert!(debug_str.contains("1080p"));
        assert!(debug_str.contains("137"));
    }

    #[test]
    fn test_downloadresult_debug() {
        let result = DownloadResult {
            output_path: PathBuf::from("test.mp4"),
            title: "Test Video".into(),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test.mp4"));
        assert!(debug_str.contains("Test Video"));
    }

    #[test]
    fn test_validate_format_all_supported() {
        let supported = ["mp4", "mkv", "webm", "flv", "avi"];
        for fmt in &supported {
            assert!(
                validate_format(fmt).is_ok(),
                "格式 '{}' 应在支持列表中",
                fmt
            );
        }
    }

    #[test]
    fn test_validate_format_rejects_nonexistent() {
        let unsupported = ["mp3", "mov", "wmv", "flac", "ogg", ""];
        for fmt in &unsupported {
            assert!(
                validate_format(fmt).is_err(),
                "格式 '{}' 应在不支持列表中",
                fmt
            );
        }
    }

    #[test]
    fn test_empty_str_validates_as_unsupported() {
        assert!(validate_format("").is_err());
    }
}
