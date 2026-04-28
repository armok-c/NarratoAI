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
