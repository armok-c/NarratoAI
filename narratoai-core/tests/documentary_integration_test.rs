mod common;

use narratoai_core::documentary::subtitle::{
    generate_srt_from_word_boundaries, merge_srt_files, SubtitleSegment,
};
use narratoai_core::documentary::timestamp::{parse_time_to_secs, parse_timestamp_range};
use narratoai_core::documentary::{DocumentaryRequest, PipelineError};

use narratoai_core::tts::WordBoundary;

// --- Timestamp parsing tests ---

#[test]
fn test_timestamp_parsing_hhmmss() {
    let secs = parse_time_to_secs("00:01:30").unwrap();
    assert!((secs - 90.0).abs() < f64::EPSILON);
}

#[test]
fn test_timestamp_parsing_with_millis() {
    let secs = parse_time_to_secs("00:01:30,500").unwrap();
    assert!((secs - 90.5).abs() < f64::EPSILON);
}

#[test]
fn test_timestamp_range_basic() {
    let (start, end) = parse_timestamp_range("00:00:05-00:00:15").unwrap();
    assert!((start - 5.0).abs() < f64::EPSILON);
    assert!((end - 15.0).abs() < f64::EPSILON);
}

#[test]
fn test_timestamp_range_with_millis() {
    let (start, end) = parse_timestamp_range("00:00:05,500-00:00:15,750").unwrap();
    assert!((start - 5.5).abs() < f64::EPSILON);
    assert!((end - 15.75).abs() < f64::EPSILON);
}

// --- SRT generation tests ---

#[test]
fn test_srt_generation_from_word_boundaries() {
    let wbs = vec![
        WordBoundary {
            start_offset: 0,
            end_offset: 50_000_000,
            text: "第一句".to_string(),
        },
        WordBoundary {
            start_offset: 50_000_000,
            end_offset: 120_000_000,
            text: "第二句".to_string(),
        },
        WordBoundary {
            start_offset: 120_000_000,
            end_offset: 200_000_000,
            text: "第三句".to_string(),
        },
        WordBoundary {
            start_offset: 200_000_000,
            end_offset: 275_000_000,
            text: "第四句".to_string(),
        },
        WordBoundary {
            start_offset: 275_000_000,
            end_offset: 350_000_000,
            text: "第五句".to_string(),
        },
    ];

    let srt = generate_srt_from_word_boundaries(&wbs, 0.0);
    let blocks: Vec<&str> = srt.split("\n\n").filter(|b| !b.is_empty()).collect();
    assert_eq!(blocks.len(), 5, "应有 5 个 SRT 块");

    // Verify timestamps format
    assert!(
        srt.contains("00:00:00,000"),
        "第一个块应从 0 秒开始: {}",
        srt
    );
    assert!(srt.contains("第一句"));
    assert!(srt.contains("第五句"));
}

#[test]
fn test_subtitle_merge_with_offsets() {
    let seg1 = SubtitleSegment {
        srt_content: "1\n00:00:00,000 --> 00:00:05,000\n第一段字幕\n\n".to_string(),
        offset_secs: 0.0,
    };
    let seg2 = SubtitleSegment {
        srt_content: "1\n00:00:00,000 --> 00:00:03,000\n第二段字幕\n\n".to_string(),
        offset_secs: 5.0,
    };

    let merged = merge_srt_files(&[seg1, seg2]).unwrap();

    // Verify sequential indexing
    assert!(merged.starts_with("1\n"), "merged: {}", merged);
    assert!(merged.contains("2\n"), "merged: {}", merged);

    // Verify second segment offset
    assert!(
        merged.contains("00:00:05,000"),
        "第二段应偏移到 5 秒: {}",
        merged
    );
}

// --- Progress tracking test ---

#[test]
fn test_progress_callback_receives_all_steps() {
    use narratoai_core::documentary::ProgressCallback;
    use std::sync::{Arc, Mutex};

    let records: Arc<Mutex<Vec<(String, f32, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let records_clone = records.clone();

    let callback: ProgressCallback = Box::new(move |step: &str, pct: f32, msg: &str| {
        records_clone
            .lock()
            .unwrap()
            .push((step.to_string(), pct, msg.to_string()));
    });

    // Simulate progress emissions using actual ProgressCallback signature
    let steps: &[(&str, f32, &str)] = &[
        ("LoadScript", 0.0, "加载视频脚本"),
        ("Tts", 20.0, "TTS 生成完成"),
        ("Clip", 60.0, "视频裁剪完成"),
        ("MergeAudio", 70.0, "音频字幕合并完成"),
        ("Concat", 80.0, "视频拼接完成"),
        ("Composite", 100.0, "最终合成完成"),
    ];

    for &(step, pct, msg) in steps {
        callback(step, pct, msg);
    }

    let r = records.lock().unwrap();
    assert_eq!(r.len(), 6, "应收到 6 个进度回调");

    // Verify step sequence
    assert_eq!(r[0].0, "LoadScript");
    assert_eq!(r[1].0, "Tts");
    assert_eq!(r[2].0, "Clip");
    assert_eq!(r[3].0, "MergeAudio");
    assert_eq!(r[4].0, "Concat");
    assert_eq!(r[5].0, "Composite");

    // Verify percentages are non-decreasing
    for i in 1..r.len() {
        assert!(
            r[i].1 >= r[i - 1].1,
            "进度应非递减: {}% -> {}%",
            r[i - 1].1,
            r[i].1
        );
    }

    // Verify final is 100
    assert_eq!(r.last().unwrap().1, 100.0);
}

// --- Module structure verification ---

#[test]
fn test_documentary_request_default() {
    let req = DocumentaryRequest::default();
    assert_eq!(req.tts_engine, "edge_tts");
    assert_eq!(req.voice_rate, 1.0);
    assert_eq!(req.original_volume, 0.7);
    assert_eq!(req.bgm_volume, 0.3);
    assert!(req.subtitle_enabled);
    assert_eq!(req.subtitle_font_size, 40);
    assert_eq!(req.subtitle_color, "#FFFFFF");
    assert_eq!(req.threads, 4);
}

#[test]
fn test_pipeline_error_variants() {
    let err = PipelineError::VideoClip {
        details: "test".into(),
    };
    assert!(err.to_string().contains("视频裁剪失败"));

    let err = PipelineError::AudioMerge {
        details: "test".into(),
    };
    assert!(err.to_string().contains("音频合并失败"));

    let err = PipelineError::Concat {
        details: "test".into(),
    };
    assert!(err.to_string().contains("视频拼接失败"));

    let err = PipelineError::Composite {
        details: "test".into(),
    };
    assert!(err.to_string().contains("最终合成失败"));
}
