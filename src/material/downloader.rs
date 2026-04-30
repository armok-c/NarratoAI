//! 素材下载与缓存模块
//!
//! 下载素材视频到本地缓存，使用 MD5 URL 去重（去掉 query string），
//! 写入时使用 temp 文件 + 原子重命名，下载后通过 ffprobe 验证。

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;

use md5::{Md5, Digest};

use crate::config::types::ProxySection;
use crate::material::searcher::MaterialError;

// ---------------------------------------------------------------------------
// Stubs — RED phase, will be implemented in GREEN phase
// ---------------------------------------------------------------------------

/// 计算 URL 的 MD5 缓存 key（去掉 query string，与 Python 版行为一致）
pub fn compute_cache_key(url: &str) -> String {
    let url_without_query = url.split('?').next().unwrap_or(url);
    let mut hasher = Md5::new();
    hasher.update(url_without_query.as_bytes());
    let result = hasher.finalize();
    format!("vid-{:x}", result)
}

/// 下载并保存单个视频，支持缓存检查、temp 文件、ffprobe 验证
pub fn save_video(
    _video_url: &str,
    _save_dir: &Path,
    _proxy_section: &ProxySection,
) -> Result<PathBuf, MaterialError> {
    unimplemented!("save_video not yet implemented")
}

/// 编排搜索 + 下载 + 时长汇总
pub fn download_videos(
    _task_id: &str,
    _search_terms: &[String],
    _source: &str,
    _target_width: u32,
    _target_height: u32,
    _audio_duration: f64,
    _max_clip_duration: u32,
    _api_keys_pexels: &[String],
    _api_keys_pixabay: &[String],
    _key_counter_pexels: &AtomicUsize,
    _key_counter_pixabay: &AtomicUsize,
    _proxy_section: &ProxySection,
) -> Result<Vec<PathBuf>, MaterialError> {
    unimplemented!("download_videos not yet implemented")
}

// ---------------------------------------------------------------------------
// Tests — RED phase: compute_cache_key passes, save_video/download_videos fail
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_compute_cache_key() {
        // MD5("https://example.com/video.mp4") = 已知值
        let key = compute_cache_key("https://example.com/video.mp4");
        assert!(key.starts_with("vid-"), "缓存 key 应以 vid- 开头: {}", key);
        assert_eq!(key.len(), 36, "vid- + 32 hex chars = 36: {}", key);
    }

    #[test]
    fn test_compute_cache_key_removes_query_string() {
        // 两个 URL 只有 query 不同，缓存 key 应相同
        let key1 = compute_cache_key("https://example.com/video.mp4?token=abc");
        let key2 = compute_cache_key("https://example.com/video.mp4?token=xyz");
        assert_eq!(key1, key2, "query string 不应影响缓存 key");
    }

    #[test]
    fn test_compute_cache_key_different_urls() {
        let key1 = compute_cache_key("https://example.com/video1.mp4");
        let key2 = compute_cache_key("https://example.com/video2.mp4");
        assert_ne!(key1, key2, "不同 URL 应产生不同缓存 key");
    }

    #[test]
    fn test_save_video_invalid_url() {
        // 无效 URL 下载应返回错误
        let save_dir = std::env::temp_dir().join("narratoai_test_material");
        let proxy = ProxySection::default();

        let result = save_video(
            "https://invalid.example.com/nonexistent.mp4",
            &save_dir,
            &proxy,
        );
        // 预期出错（网络不通或 404）
        assert!(result.is_err());
    }

    #[test]
    fn test_download_videos_no_terms() {
        let result = download_videos(
            "test",
            &[],
            "pexels",
            1920,
            1080,
            30.0,
            10,
            &[],
            &[],
            &AtomicUsize::new(0),
            &AtomicUsize::new(0),
            &ProxySection::default(),
        );
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_empty(),
            "无搜索词应返回空列表"
        );
    }
}
