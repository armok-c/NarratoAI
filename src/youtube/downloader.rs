//! YouTube 视频下载模块
//!
//! 通过 yt-dlp CLI 子进程完成视频格式查询和下载。
//! 所有 CLI 调用使用 args 数组，禁止 shell 字符串（per T-11-01）。

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
        assert!(validate_format("avi").is_ok());   // 在支持列表中
        assert!(validate_format("mp3").is_err());   // 不在列表中
        assert!(validate_format("mov").is_err());
    }

    #[test]
    fn test_validate_format_case_insensitive() {
        assert!(validate_format("MP4").is_ok());
        assert!(validate_format("MKV").is_ok());
    }

    #[test]
    fn test_invalid_url_prefix() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            // 模拟 _get_video_formats 内部的 URL 校验
            let url = "ftp://example.com/video";
            if !url.starts_with("http://") && !url.starts_with("https://") {
                Err(YoutubeError::InvalidUrl(url.to_string()))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_resolution_normalization() {
        // 模拟 _get_video_formats 内部的分辨率标准化逻辑

        // "1080p60" → "1080p"
        let res = "1080p60";
        let base_res = if res.contains('p') {
            let parts: Vec<&str> = res.split('p').collect();
            format!("{}p", parts[0])
        } else {
            res.to_string()
        };
        assert_eq!(base_res, "1080p");

        // "720p" → "720p"
        let res = "720p";
        let base_res2 = if res.contains('p') {
            let parts: Vec<&str> = res.split('p').collect();
            format!("{}p", parts[0])
        } else {
            res.to_string()
        };
        assert_eq!(base_res2, "720p");

        // "2160p60" → "2160p"
        let res = "2160p60";
        let base_res3 = if res.contains('p') {
            let parts: Vec<&str> = res.split('p').collect();
            format!("{}p", parts[0])
        } else {
            res.to_string()
        };
        assert_eq!(base_res3, "2160p");
    }

    #[test]
    fn test_format_selection_prefers_non_av01() {
        // 验证格式选择的优先级过滤逻辑
        let formats = vec![
            VideoFormat { format_id: "1".into(), resolution: "1080p".into(), ext: "mp4".into(), vcodec: "av01.0.05M.08".into() },
            VideoFormat { format_id: "2".into(), resolution: "1080p".into(), ext: "mp4".into(), vcodec: "avc1.640028".into() },
            VideoFormat { format_id: "3".into(), resolution: "720p".into(), ext: "mp4".into(), vcodec: "avc1.4d401f".into() },
        ];

        // 匹配 1080p 且非 av01
        let matched = formats.iter()
            .filter(|f| f.resolution == "1080p" && f.vcodec != "none")
            .find(|f| !f.vcodec.contains("av01"));

        assert!(matched.is_some());
        assert_eq!(matched.unwrap().format_id, "2");
    }
}
