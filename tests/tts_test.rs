use narratoai_core::error::TTSError;
use narratoai_core::tts::{self, TtsOutput, WordBoundary};
use std::path::Path;

// ============================================================
// TTS 核心抽象测试
// ============================================================

/// WordBoundary 结构体字段完整性验证
#[test]
fn test_word_boundary_fields() {
    let wb = WordBoundary {
        start_offset: 0,
        end_offset: 5000000,   // 500ms in 100ns units
        text: "测试".to_string(),
    };
    assert_eq!(wb.start_offset, 0);
    assert_eq!(wb.end_offset, 5000000);
    assert_eq!(wb.text, "测试");
}

/// TtsOutput 结构体字段完整性验证
#[test]
fn test_tts_output_fields() {
    let output = TtsOutput {
        audio_file_path: Path::new("test.mp3").to_path_buf(),
        word_boundaries: vec![],
        duration: 3.5,
    };
    assert_eq!(output.audio_file_path, Path::new("test.mp3"));
    assert!(output.word_boundaries.is_empty());
    assert_eq!(output.duration, 3.5);
}

/// TtsOutput 可包含多个 WordBoundary
#[test]
fn test_tts_output_with_word_boundaries() {
    let output = TtsOutput {
        audio_file_path: Path::new("test.mp3").to_path_buf(),
        word_boundaries: vec![
            WordBoundary { start_offset: 0, end_offset: 30000000, text: "你好".to_string() },
            WordBoundary { start_offset: 30000000, end_offset: 60000000, text: "世界".to_string() },
        ],
        duration: 6.0,
    };
    assert_eq!(output.word_boundaries.len(), 2);
    assert_eq!(output.word_boundaries[0].text, "你好");
    assert_eq!(output.word_boundaries[1].text, "世界");
}

// ============================================================
// TTSError 集成测试
// ============================================================

/// TTSError 所有变体的 Display 包含中文
#[test]
fn test_tts_error_all_variants_chinese() {
    let errors: Vec<TTSError> = vec![
        TTSError::UnknownEngine { engine: "x".into() },
        TTSError::ConnectionFailed("timeout".into()),
        TTSError::AuthenticationFailed("token expired".into()),
        TTSError::SynthesisFailed("bad request".into()),
        TTSError::RetryExhausted("all 3 failed".into()),
    ];
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    assert!(messages[0].contains("未知 TTS 引擎"));
    assert!(messages[1].contains("TTS 连接失败"));
    assert!(messages[2].contains("TTS 认证失败"));
    assert!(messages[3].contains("TTS 合成失败"));
    assert!(messages[4].contains("TTS 重试耗尽"));
}

// ============================================================
// TTS 数据流完整性验证
// ============================================================

/// WordBoundary 的时间单位语义：100 纳秒 = 0.1 微秒
/// 5 秒 = 5_000_000_000 微秒 = 50_000_000 单位（100ns 单位）
#[test]
fn test_word_boundary_time_unit_semantics() {
    // 5 秒在 100ns 单位下的值
    let five_seconds_in_100ns_units = 50_000_000u64;
    let wb = WordBoundary {
        start_offset: 0,
        end_offset: five_seconds_in_100ns_units,
        text: "test".to_string(),
    };
    // 验证数值在合理范围
    assert!(wb.end_offset > wb.start_offset);
    assert_eq!(wb.end_offset - wb.start_offset, 50_000_000);
}

// ============================================================
// TTS 路由器函数签名校验（编译时测试）
// ============================================================

/// Verify `tts::synthesize` function signature at compile time.
///
/// Calls the function and assigns the returned future to `_`, which
/// causes a compile error if the parameter types or return type change.
/// The future is never polled, so there are no runtime side effects.
#[allow(clippy::let_underscore_future)]
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize("", "", "", 1.0, 0.0, Path::new(""), None);
}
