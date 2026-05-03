use crate::error::{FFmpegError, LLMError, TTSError};
use crate::script::error::ScriptError;

/// 纪录片流水线错误——覆盖 6 步流水线各环节 + IO + 时间戳 + 字幕
#[derive(thiserror::Error, Debug)]
pub enum PipelineError {
    #[error("脚本加载失败: {source}")]
    ScriptLoad {
        #[source]
        source: ScriptError,
    },

    #[error("TTS 语音生成失败: {source}")]
    TtsGeneration {
        #[source]
        source: TTSError,
    },

    #[error("视频裁剪失败: {details}")]
    VideoClip { details: String },

    #[error("音频合并失败: {details}")]
    AudioMerge { details: String },

    #[error("视频拼接失败: {details}")]
    Concat { details: String },

    #[error("最终合成失败: {details}")]
    Composite { details: String },

    #[error("IO 操作失败: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },

    #[error("时间戳解析失败: {0}")]
    Timestamp(String),

    #[error("字幕生成失败: {0}")]
    SrtGeneration(String),

    #[error("帧提取失败: {details}")]
    FrameExtraction { details: String },

    #[error("FFmpeg 操作失败: {source}")]
    FFmpeg {
        #[source]
        source: FFmpegError,
    },

    #[error("LLM 调用失败: {source}")]
    Llm {
        #[source]
        source: LLMError,
    },
}

impl From<ScriptError> for PipelineError {
    fn from(source: ScriptError) -> Self {
        PipelineError::ScriptLoad { source }
    }
}

impl From<TTSError> for PipelineError {
    fn from(source: TTSError) -> Self {
        PipelineError::TtsGeneration { source }
    }
}

impl From<std::io::Error> for PipelineError {
    fn from(source: std::io::Error) -> Self {
        PipelineError::Io { source }
    }
}

impl From<FFmpegError> for PipelineError {
    fn from(source: FFmpegError) -> Self {
        PipelineError::FFmpeg { source }
    }
}

impl From<LLMError> for PipelineError {
    fn from(source: LLMError) -> Self {
        PipelineError::Llm { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_error_script_load_chinese() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let script_err = ScriptError::Io(io_err);
        let err = PipelineError::from(script_err);
        let msg = err.to_string();
        assert!(msg.contains("脚本加载失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_tts_generation_chinese() {
        let tts_err = TTSError::SynthesisFailed("boom".into());
        let err = PipelineError::from(tts_err);
        let msg = err.to_string();
        assert!(msg.contains("TTS 语音生成失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_video_clip_chinese() {
        let err = PipelineError::VideoClip {
            details: "clip 42 OST=0".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("视频裁剪失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_audio_merge_chinese() {
        let err = PipelineError::AudioMerge {
            details: "mix failed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("音频合并失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_concat_chinese() {
        let err = PipelineError::Concat {
            details: "concat failed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("视频拼接失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_composite_chinese() {
        let err = PipelineError::Composite {
            details: "composite failed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("最终合成失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_io_chinese() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = PipelineError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("IO 操作失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_timestamp_chinese() {
        let err = PipelineError::Timestamp("bad format".into());
        let msg = err.to_string();
        assert!(msg.contains("时间戳解析失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_srt_generation_chinese() {
        let err = PipelineError::SrtGeneration("parse error".into());
        let msg = err.to_string();
        assert!(msg.contains("字幕生成失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_frame_extraction_chinese() {
        let err = PipelineError::FrameExtraction {
            details: "提取关键帧失败".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("帧提取失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_ffmpeg_chinese() {
        let ffmpeg_err = FFmpegError::ExecutionError("failed".into());
        let err = PipelineError::from(ffmpeg_err);
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg 操作失败"), "消息应包含中文: {}", msg);
    }

    #[test]
    fn test_pipeline_error_llm_chinese() {
        let llm_err = LLMError::APICall("timeout".into());
        let err = PipelineError::from(llm_err);
        let msg = err.to_string();
        assert!(msg.contains("LLM 调用失败"), "消息应包含中文: {}", msg);
    }
}
