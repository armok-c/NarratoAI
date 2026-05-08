use std::path::Path;

use crate::documentary::timestamp::secs_to_srt_time;
use crate::documentary::types::TtsResult;
use crate::sde::timestamp::{find_precise_range, parse_srt_timestamp};
use crate::sdp::error::SdpError;
use crate::sdp::types::SdpPipelineState;

/// SDP 视频裁剪——复用 SDE 的 sde_step_clip() 模式
///
/// 1. OST=1 精确时间戳校正（find_precise_range）
/// 2. 调用 documentary clip_all_videos 执行 FFmpeg 裁剪
/// 3. 计算每个片段的 duration 和 time_range
///
/// 与 SDE 裁剪的区别：
/// - SDE 的 SdePipelineState 有 TTS 相关字段，SDP 无需
/// - SDP 全部 OST=1，timezone 校正逻辑相同
/// - clip_all_videos 需要 tts_results 参数，SDP 传空 HashMap
pub async fn sdp_step_clip(
    state: &mut SdpPipelineState,
    video_path: &Path,
) -> Result<(), SdpError> {
    use crate::script::types::OstType;

    // OST=1 精确时间戳校正（等价于 sde_step_clip 中 OST=1 分支）
    for clip in &mut state.script {
        if clip.ost == OstType::OriginalSound {
            if let Some((precise_start, precise_end)) =
                find_precise_range(&clip.timestamp, &state.subtitle_segments)
            {
                let start_str = secs_to_srt_time(precise_start);
                let end_str = secs_to_srt_time(precise_end);
                clip.timestamp = format!("{}-{}", start_str, end_str);
            }
            // fallback: 如果找不到精确匹配，保留 LLM 原始时间戳（不崩溃）
        }
    }

    // 调用 documentary 裁剪（传空 tts_results 因为 SDP 全部 OST=1，不读取 tts）
    let empty_tts: std::collections::HashMap<i64, TtsResult> = std::collections::HashMap::new();
    let video_clips = crate::documentary::clip::clip_all_videos(
        video_path,
        &state.script,
        &empty_tts,
        &state.task_dir,
    )
    .await
    .map_err(|e| SdpError::VideoProcess(e))?;

    state.video_clips = video_clips;

    // 计算每个片段的 duration 和 source_time_range/edited_time_range
    let mut cumulative_time = 0.0f64;
    for clip in &mut state.script {
        if let Some(clip_path) = state.video_clips.get(&clip._id) {
            clip.video = Some(clip_path.clone());
        }

        // 计算片段时长：从 timestamp 中解析起止秒数
        let ts_parts: Vec<&str> = clip.timestamp.splitn(2, '-').collect();
        if ts_parts.len() == 2 {
            let start_secs = parse_srt_timestamp(ts_parts[0])
                .map_err(|e| SdpError::ParseSubtitle {
                    details: format!("片段 {} 开始时间戳解析失败: {}", clip._id, e),
                })?;
            let end_secs = parse_srt_timestamp(ts_parts[1])
                .map_err(|e| SdpError::ParseSubtitle {
                    details: format!("片段 {} 结束时间戳解析失败: {}", clip._id, e),
                })?;
            if end_secs < start_secs {
                return Err(SdpError::Validation {
                    details: format!(
                        "片段 {} 结束时间 ({}) 早于开始时间 ({})",
                        clip._id, end_secs, start_secs
                    ),
                });
            }
            let clip_duration = end_secs - start_secs;

            if clip_duration <= 0.0 {
                return Err(SdpError::Validation {
                    details: format!("片段 {} 时长计算为零", clip._id),
                });
            }

            clip.duration = Some(clip_duration);
            clip.source_time_range = Some(format!(
                "{}-{}",
                secs_to_srt_time(start_secs),
                secs_to_srt_time(end_secs)
            ));
            clip.edited_time_range = Some(format!(
                "{}-{}",
                secs_to_srt_time(cumulative_time),
                secs_to_srt_time(cumulative_time + clip_duration)
            ));
            cumulative_time += clip_duration;
        } else {
            return Err(SdpError::Validation {
                details: format!(
                    "片段 {} 时间戳格式无效（缺少 '-' 分隔符）: {}",
                    clip._id, clip.timestamp
                ),
            });
        }
    }

    state.total_duration = cumulative_time;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sde::types::SubtitleSegment;
    use crate::script::types::{OstType, ScriptClip};

    fn make_segment(index: usize, start: f64, end: f64, text: &str) -> SubtitleSegment {
        SubtitleSegment {
            index,
            start_secs: start,
            end_secs: end,
            text: text.to_string(),
        }
    }

    #[test]
    fn test_sdp_step_clip_ost1_precise_correction() {
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
            let start_str = secs_to_srt_time(precise_start);
            let end_str = secs_to_srt_time(precise_end);
            clip.timestamp = format!("{}-{}", start_str, end_str);
        }

        assert_eq!(
            clip.timestamp, "00:00:07,000-00:00:12,000",
            "OST=1 时间戳应被校正为精确字幕时间"
        );
    }

    #[test]
    fn test_sdp_step_clip_nonexistent_match_preserves_original() {
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

        let segments = vec![make_segment(0, 0.0, 5.0, "第一段")];

        let original = clip.timestamp.clone();
        if clip.ost == OstType::OriginalSound {
            if let Some((precise_start, precise_end)) =
                find_precise_range(&clip.timestamp, &segments)
            {
                let start_str = secs_to_srt_time(precise_start);
                let end_str = secs_to_srt_time(precise_end);
                clip.timestamp = format!("{}-{}", start_str, end_str);
            }
        }

        assert_eq!(
            clip.timestamp, original,
            "未匹配时应保留原始时间戳"
        );
    }
}
