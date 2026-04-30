//! 素材搜索模块（Pexels + Pixabay）
//!
//! 通过 reqwest HTTP 客户端调用 Pexels 和 Pixabay 的 REST API 搜索视频素材。
//! 支持多 API key 轮换（AtomicUsize 原子自增）、代理配置、分辨率过滤。

use std::sync::atomic::AtomicUsize;

use serde::Deserialize;
use reqwest::Client;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// 素材操作错误类型
#[derive(Error, Debug)]
pub enum MaterialError {
    #[error("API 请求失败: {0}")]
    ApiRequest(String),

    #[error("API 认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("文件验证失败: {0}")]
    ValidationFailed(String),

    #[error("API key 未配置: {0}")]
    MissingApiKey(String),

    #[error("IO 错误: {0}")]
    IoError(String),
}

// ---------------------------------------------------------------------------
// Pexels API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PexelsSearchResponse {
    pub page: u32,
    pub per_page: u32,
    pub total_results: u32,
    pub videos: Vec<PexelsVideo>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideo {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    pub url: String,
    pub video_files: Vec<PexelsVideoFile>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoFile {
    pub id: u64,
    pub quality: String,
    pub file_type: String,
    pub width: u32,
    pub height: u32,
    pub link: String,
}

// ---------------------------------------------------------------------------
// Pixabay API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PixabaySearchResponse {
    pub total: u32,
    pub total_hits: u32,
    pub hits: Vec<PixabayHit>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayHit {
    pub id: u64,
    pub duration: u32,
    pub videos: PixabayVideos,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideos {
    pub large: Option<PixabayVideoVariant>,
    pub medium: Option<PixabayVideoVariant>,
    pub small: Option<PixabayVideoVariant>,
    pub tiny: Option<PixabayVideoVariant>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoVariant {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

// ---------------------------------------------------------------------------
// Unified result type
// ---------------------------------------------------------------------------

/// 搜索结果的统一表示
#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub id: u64,
    pub duration: u32,
    pub width: u32,
    pub height: u32,
    pub video_url: String,
    pub source: String, // "pexels" | "pixabay"
}

// ---------------------------------------------------------------------------
// Stubs — RED phase, will be implemented in GREEN phase
// ---------------------------------------------------------------------------

/// 从 key 数组中轮询返回 API key（AtomicUsize 原子自增，防止异步竞态）
pub fn get_api_key(_keys: &[String], _counter: &AtomicUsize) -> Result<String, MaterialError> {
    unimplemented!("get_api_key not yet implemented")
}

/// 构建 reqwest Client，按需配置代理
pub(crate) fn build_client(
    _proxy_section: &crate::config::types::ProxySection,
) -> Result<Client, MaterialError> {
    unimplemented!("build_client not yet implemented")
}

/// 简单 URL 编码（仅编码空格和特殊字符）
fn urlencoding(s: &str) -> String {
    // 简单实现，测试可以验证
    s.replace(' ', "+")
}

/// 搜索 Pexels 视频素材
pub fn search_videos_pexels(
    _search_term: &str,
    _minimum_duration: u32,
    _target_width: u32,
    _target_height: u32,
    _api_keys: &[String],
    _key_counter: &AtomicUsize,
    _proxy_section: &crate::config::types::ProxySection,
) -> Result<Vec<MaterialInfo>, MaterialError> {
    unimplemented!("search_videos_pexels not yet implemented")
}

/// 搜索 Pixabay 视频素材
pub fn search_videos_pixabay(
    _search_term: &str,
    _minimum_duration: u32,
    _target_width: u32,
    _target_height: u32,
    _api_keys: &[String],
    _key_counter: &AtomicUsize,
    _proxy_section: &crate::config::types::ProxySection,
) -> Result<Vec<MaterialInfo>, MaterialError> {
    unimplemented!("search_videos_pixabax not yet implemented")
}

// ---------------------------------------------------------------------------
// Tests — RED phase: tests exist but fail because functions are stubbed
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_error_chinese_display() {
        let err = MaterialError::ApiRequest("timeout".into());
        assert!(err.to_string().contains("API 请求失败"));

        let err = MaterialError::AuthenticationFailed("bad key".into());
        assert!(err.to_string().contains("API 认证失败"));

        let err = MaterialError::DownloadFailed("network".into());
        assert!(err.to_string().contains("下载失败"));

        let err = MaterialError::MissingApiKey("pexels".into());
        assert!(err.to_string().contains("API key 未配置"));
    }

    #[test]
    fn test_get_api_key_single() {
        let keys = vec!["key1".to_string()];
        let counter = AtomicUsize::new(0);
        let key = get_api_key(&keys, &counter).unwrap();
        assert_eq!(key, "key1");
    }

    #[test]
    fn test_get_api_key_rotation() {
        let keys = vec![
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ];
        let counter = AtomicUsize::new(0);

        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key1");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key2");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key3");
        assert_eq!(get_api_key(&keys, &counter).unwrap(), "key1"); // wrap
    }

    #[test]
    fn test_get_api_key_empty() {
        let keys: Vec<String> = vec![];
        let counter = AtomicUsize::new(0);
        let result = get_api_key(&keys, &counter);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key 未配置"));
    }

    #[test]
    fn test_pexels_response_deserialize() {
        let json = r#"{
            "page": 1,
            "per_page": 20,
            "total_results": 1,
            "videos": [{
                "id": 12345,
                "width": 1920,
                "height": 1080,
                "duration": 15,
                "url": "https://www.pexels.com/video/test/",
                "video_files": [{
                    "id": 1,
                    "quality": "hd",
                    "file_type": "video/mp4",
                    "width": 1920,
                    "height": 1080,
                    "link": "https://www.pexels.com/video/download/12345/"
                }]
            }]
        }"#;
        let resp: PexelsSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_results, 1);
        assert_eq!(resp.videos.len(), 1);
        assert_eq!(resp.videos[0].id, 12345);
        assert_eq!(resp.videos[0].video_files[0].quality, "hd");
    }

    #[test]
    fn test_pixabay_response_deserialize() {
        let json = r#"{
            "total": 1,
            "total_hits": 1,
            "hits": [{
                "id": 67890,
                "duration": 10,
                "videos": {
                    "large": {
                        "url": "https://pixabay.com/video/download/67890_large.mp4",
                        "width": 1920,
                        "height": 1080,
                        "size": 5000000
                    },
                    "medium": null,
                    "small": null,
                    "tiny": null
                }
            }]
        }"#;
        let resp: PixabaySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_hits, 1);
        assert_eq!(resp.hits[0].id, 67890);
        assert!(resp.hits[0].videos.large.is_some());
        assert!(resp.hits[0].videos.medium.is_none());
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("nature"), "nature");
        assert_eq!(urlencoding("mountain river"), "mountain+river");
    }

    #[test]
    fn test_material_info_structure() {
        let info = MaterialInfo {
            id: 1,
            duration: 30,
            width: 1920,
            height: 1080,
            video_url: "https://example.com/video.mp4".into(),
            source: "pexels".into(),
        };
        assert_eq!(info.source, "pexels");
        assert_eq!(info.duration, 30);
    }
}
