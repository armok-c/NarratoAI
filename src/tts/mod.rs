use std::path::PathBuf;
use crate::error::TTSError;

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
            text: &str,
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
