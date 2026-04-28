mod edge_tts;

use async_trait::async_trait;
use std::path::Path;
use std::path::PathBuf;
use crate::error::TTSError;
use self::edge_tts::EdgeTtsEngine;

/// 词边界，单位 100 纳秒（与 Python 版一致，Phase 6 字幕生成使用）
///
/// 对应 D-08: start_offset/end_offset 单位 100 纳秒，与 Python 版 edge-tts 的 SubMaker 行为一致
#[derive(Debug, Clone)]
pub struct WordBoundary {
    pub start_offset: u64,
    pub end_offset: u64,
    pub text: String,
}

/// TTS 合成输出结果
///
/// 对应 D-06: 调用方从返回值获取所有必要数据，无需额外查询
/// 对应 D-09: 词边界直接内嵌在 word_boundaries 字段中，无独立 SubMaker 结构体
#[derive(Debug, Clone)]
pub struct TtsOutput {
    /// 生成的音频文件路径（MP3 格式）
    pub audio_file_path: PathBuf,
    /// 词边界时间序列，为空表示引擎不支持词边界
    pub word_boundaries: Vec<WordBoundary>,
    /// 音频总时长（秒）
    pub duration: f64,
}

/// TTS 引擎统一接口
///
/// 对应 D-05: 使用 #[async_trait] 宏
/// 对应 D-07: 独立参数 text/voice_name/rate/pitch/output_path
/// 每个具体引擎（Edge-TTS、Azure 等）实现此 trait
#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// 将文本合成为语音音频文件
    ///
    /// # 参数
    /// - text: 要合成的文本
    /// - voice_name: TTS 音色名
    /// - rate: 语速（1.0 为标准速度）
    /// - pitch: 音调（0 为标准音调）
    /// - output_path: 输出音频文件路径
    ///
    /// # 返回值
    /// - Ok(TtsOutput): 包含音频文件路径、词边界时序和时长
    /// - Err(TTSError): 合成失败
    async fn synthesize(
        &self,
        text: &str,
        voice_name: &str,
        rate: f64,
        pitch: f64,
        output_path: &Path,
    ) -> Result<TtsOutput, TTSError>;
}

/// TTS 路由器：按引擎名字符串分发到对应实现
///
/// 对应 D-11: 字符串匹配分发
/// 对应 D-12: 模块级 pub async fn，不包装为结构体
/// 对应 D-13: 未知引擎返回 TTSError::UnknownEngine
/// 对应 D-14: Phase 3 只实现 Edge-TTS 分支
///
/// # 参数
/// - engine: 引擎名称（如 "edge_tts"）
/// - 其余参数同 TtsProvider::synthesize
pub async fn synthesize(
    engine: &str,
    text: &str,
    voice_name: &str,
    rate: f64,
    pitch: f64,
    output_path: &Path,
    proxy: Option<&crate::config::types::ProxySection>,
) -> Result<TtsOutput, TTSError> {
    match engine {
        "edge_tts" => {
            let (proxy_enabled, proxy_http, proxy_https) = match proxy {
                Some(p) => (p.enabled, p.http.clone(), p.https.clone()),
                None => (false, String::new(), String::new()),
            };
            let edge_engine = EdgeTtsEngine::new(proxy_enabled, proxy_http, proxy_https);
            edge_engine.synthesize(text, voice_name, rate, pitch, output_path).await
        }
        _ => Err(TTSError::UnknownEngine {
            engine: engine.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::path::Path;

    /// Mock TTS 引擎——验证 TtsProvider trait 可被实现
    struct MockTtsEngine;

    #[async_trait]
    impl TtsProvider for MockTtsEngine {
        async fn synthesize(
            &self,
            _text: &str,
            _voice_name: &str,
            _rate: f64,
            _pitch: f64,
            output_path: &Path,
        ) -> Result<TtsOutput, TTSError> {
            Ok(TtsOutput {
                audio_file_path: output_path.to_path_buf(),
                word_boundaries: vec![],
                duration: 1.0,
            })
        }
    }

    #[tokio::test]
    async fn test_mock_engine_implements_trait() {
        let engine = MockTtsEngine;
        let result = engine
            .synthesize("hello", "voice", 1.0, 0.0, Path::new("test.mp3"))
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.duration, 1.0);
        assert!(output.word_boundaries.is_empty());
    }

    #[tokio::test]
    async fn test_synthesize_unknown_engine() {
        let result = synthesize(
            "nonexistent_engine",
            "hello",
            "voice",
            1.0,
            0.0,
            Path::new("test.mp3"),
        )
        .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::UnknownEngine { engine } => {
                assert_eq!(engine, "nonexistent_engine");
            }
            _ => panic!("应为 UnknownEngine 错误"),
        }
    }

    #[tokio::test]
    async fn test_synthesize_unknown_engine_message() {
        let result = synthesize(
            "bad_engine",
            "hello",
            "voice",
            1.0,
            0.0,
            Path::new("test.mp3"),
        )
        .await;
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("未知 TTS 引擎"), "消息应包含中文: {}", msg);
        assert!(msg.contains("bad_engine"), "消息应包含引擎名: {}", msg);
    }
}
