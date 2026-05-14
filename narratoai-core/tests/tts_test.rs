use narratoai_core::error::TTSError;
use narratoai_core::tts::{self, TtsOutput, WordBoundary};
use std::path::Path;

// ============================================================
// TTS-01: Core Types & Error Verification
// ============================================================

#[test]
fn test_word_boundary_fields() {
    let wb = WordBoundary {
        start_offset: 0,
        end_offset: 5000000,
        text: "测试".to_string(),
    };
    assert_eq!(wb.start_offset, 0);
    assert_eq!(wb.end_offset, 5000000);
    assert_eq!(wb.text, "测试");
}

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

#[test]
fn test_tts_output_with_word_boundaries() {
    let output = TtsOutput {
        audio_file_path: Path::new("test.mp3").to_path_buf(),
        word_boundaries: vec![
            WordBoundary {
                start_offset: 0,
                end_offset: 30000000,
                text: "你好".to_string(),
            },
            WordBoundary {
                start_offset: 30000000,
                end_offset: 60000000,
                text: "世界".to_string(),
            },
        ],
        duration: 6.0,
    };
    assert_eq!(output.word_boundaries.len(), 2);
    assert_eq!(output.word_boundaries[0].text, "你好");
    assert_eq!(output.word_boundaries[1].text, "世界");
}

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
// TTS-02: Router Dispatch Verification
// ============================================================

#[tokio::test]
async fn test_synthesize_unknown_engine_error() {
    let result = tts::synthesize(
        "invalid_engine",
        "测试文本",
        "zh-CN-XiaoyiNeural",
        1.0,
        0.0,
        Path::new("output.mp3"),
        None,
        None,
    )
    .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        TTSError::UnknownEngine { ref engine } => {
            assert_eq!(engine, "invalid_engine");
        }
        _ => panic!("应为 UnknownEngine 错误"),
    }
}

#[tokio::test]
async fn test_synthesize_unknown_engine_message_contains_name() {
    let result = tts::synthesize(
        "bad_engine_name",
        "测试",
        "voice",
        1.0,
        0.0,
        Path::new("out.mp3"),
        None,
        None,
    )
    .await;
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("bad_engine_name"));
    assert!(err_msg.contains("未知 TTS 引擎"));
}

// ============================================================
// TTS-03: WordBoundary Semantics
// ============================================================

#[test]
fn test_word_boundary_time_unit_semantics() {
    let five_seconds_in_100ns_units = 50_000_000u64;
    let wb = WordBoundary {
        start_offset: 0,
        end_offset: five_seconds_in_100ns_units,
        text: "test".to_string(),
    };
    assert!(wb.end_offset > wb.start_offset);
    assert_eq!(wb.end_offset - wb.start_offset, 50_000_000);
}

// ============================================================
// TTS-02: Compile-time Signature Verification
// ============================================================

#[allow(clippy::let_underscore_future)]
#[test]
fn test_synthesize_function_signature() {
    let _ = tts::synthesize(
        "edge_tts",
        "",
        "",
        1.0,
        0.0,
        Path::new("/tmp/_narratoai_sig_check.mp3"),
        None,
        None,
    );
}
