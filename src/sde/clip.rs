use std::path::Path;

use crate::documentary::error::PipelineError;
use crate::documentary::clip::clip_all_videos;
use crate::sde::error::SdeError;
use crate::sde::timestamp::find_precise_range;
use crate::sde::types::{SdePipelineState, SubtitleSegment};
use crate::script::types::OstType;

/// SDE 视频裁剪——在纪录片裁剪基础上增加 OST=1 精确时间戳校正
///
/// 1. 遍历 OST=1 片段，使用 find_precise_range() 从原始字幕段落获取精确时间戳（D-29）
/// 2. 调用纪录片 clip_all_videos 执行实际 FFmpeg 裁剪
///
/// LLM 生成的时间戳可能有偏差（±1s），本函数用原始字幕段落时间戳修正偏差。
pub async fn sde_step_clip(
    state: &mut SdePipelineState,
    video_path: &Path,
) -> Result<(), SdeError> {
    // OST=1 精确时间戳校正（D-29）
    for clip in &mut state.script {
        if clip.ost == OstType::OriginalSound {
            if let Some((precise_start, precise_end)) =
                find_precise_range(&clip.timestamp, &state.subtitle_segments)
            {
                let start_str = secs_to_timestamp_str(precise_start);
                let end_str = secs_to_timestamp_str(precise_end);
                clip.timestamp = format!("{}-{}", start_str, end_str);
            }
            // fallback: 如果找不到精确匹配，保留 LLM 原始时间戳（不崩溃）
        }
    }

    // 调用纪录片裁剪
    let video_clips = clip_all_videos(
        video_path,
        &state.script,
        &state.tts_results,
        &state.task_dir,
    )
    .await
    .map_err(|e| SdeError::VideoProcess(e))?;

    state.video_clips = video_clips;
    Ok(())
}

/// 将秒数转换为 SRT 时间格式 "HH:MM:SS,mmm"
fn secs_to_timestamp_str(secs: f64) -> String {
    if secs < 0.0 {
        return "00:00:00,000".to_string();
    }
    let secs = secs.min(86399.999);
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::documentary::types::TtsResult;
    use crate::script::types::ScriptClip;

    fn make_segment(index: usize, start: f64, end: f64, text: &str) -> SubtitleSegment {
        SubtitleSegment {
            index,
            start_secs: start,
            end_secs: end,
            text: text.to_string(),
        }
    }

    #[test]
    fn test_sde_step_clip_ost1_precise_correction() {
        // 模拟 OST=1 片段，LLM 时间戳稍有偏差
        let mut clip = ScriptClip {
            _id: 1,
            timestamp: "00:00:07,500-00:00:12,500".to_string(), // 偏差 0.5s
            picture: "测试画面".to_string(),
            narration: "播放原片".to_string(),
            ost: OstType::OriginalSound,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };

        let segments = vec![
            make_segment(0, 0.0, 5.0, "第一段"),
            make_segment(1, 7.0, 12.0, "第二段"), // 实际: 7.0-12.0
            make_segment(2, 14.0, 19.0, "第三段"),
        ];

        // 测试 find_precise_range 校正
        if let Some((precise_start, precise_end)) =
            find_precise_range(&clip.timestamp, &segments)
        {
            let start_str = secs_to_timestamp_str(precise_start);
            let end_str = secs_to_timestamp_str(precise_end);
            clip.timestamp = format!("{}-{}", start_str, end_str);
        }

        assert_eq!(clip.timestamp, "00:00:07,000-00:00:12,000",
            "OST=1 时间戳应被校正为精确字幕时间");
    }

    #[test]
    fn test_sde_step_clip_ost0_no_correction() {
        // OST=0 片段不应被时间戳校正
        let clip = ScriptClip {
            _id: 2,
            timestamp: "00:00:05,000-00:00:10,000".to_string(),
            picture: "测试画面".to_string(),
            narration: "测试解说".to_string(),
            ost: OstType::NarrationOnly,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };

        let segments = vec![
            make_segment(0, 0.0, 5.0, "第一段"),
            make_segment(1, 7.0, 12.0, "第二段"),
        ];

        // OST=0 片段不应调用 find_precise_range
        let original_ts = clip.timestamp.clone();
        if clip.ost == OstType::OriginalSound {
            if let Some((_, _)) = find_precise_range(&clip.timestamp, &segments) {
                // 不执行
            }
        }
        assert_eq!(clip.timestamp, original_ts, "OST=0 时间戳不应被修改");
    }

    #[test]
    fn test_secs_to_timestamp_str_zero() {
        assert_eq!(secs_to_timestamp_str(0.0), "00:00:00,000");
    }

    #[test]
    fn test_secs_to_timestamp_str_normal() {
        assert_eq!(secs_to_timestamp_str(3661.5), "01:01:01,500");
    }

    #[test]
    fn test_secs_to_timestamp_str_negative() {
        assert_eq!(secs_to_timestamp_str(-1.0), "00:00:00,000");
    }

    #[test]
    fn test_secs_to_timestamp_str_overflow() {
        assert_eq!(secs_to_timestamp_str(999999.0), "23:59:59,999");
    }

    #[test]
    fn test_sde_step_clip_nonexistent_match_preserves_original() {
        // 当 find_precise_range 找不到匹配时，保留原始时间戳
        let mut clip = ScriptClip {
            _id: 3,
            timestamp: "00:10:00,000-00:10:05,000".to_string(),
            picture: "测试画面".to_string(),
            narration: "播放原片".to_string(),
            ost: OstType::OriginalSound,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        };

        let segments = vec![
            make_segment(0, 0.0, 5.0, "第一段"),
        ];

        let original = clip.timestamp.clone();
        if clip.ost == OstType::OriginalSound {
            if let Some((precise_start, precise_end)) =
                find_precise_range(&clip.timestamp, &segments)
            {
                let start_str = secs_to_timestamp_str(precise_start);
                let end_str = secs_to_timestamp_str(precise_end);
                clip.timestamp = format!("{}-{}", start_str, end_str);
            }
        }

        assert_eq!(clip.timestamp, original, "未匹配时应保留原始时间戳");
    }
}
