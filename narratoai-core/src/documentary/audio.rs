use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::documentary::error::PipelineError;
use crate::documentary::subtitle::{merge_srt_files, SubtitleSegment};
use crate::documentary::types::TtsResult;
use crate::script::types::{OstType, ScriptClip};

/// 合并所有 TTS 音频到单一文件
///
/// 创建 `total_duration` 秒的静音基底，然后将每个 TTS 音频叠加到正确的时间偏移位置。
pub async fn merge_audio_files(
    script_clips: &[ScriptClip],
    tts_results: &HashMap<i64, TtsResult>,
    total_duration: f64,
    output_path: &Path,
) -> Result<PathBuf, PipelineError> {
    if total_duration <= 0.0 {
        return Err(PipelineError::AudioMerge {
            details: "总时长必须大于 0".to_string(),
        });
    }

    let duration_str = format!("{:.3}", total_duration);
    let output_str = output_path.to_string_lossy().to_string();

    let mut input_args: Vec<String> = Vec::new();
    let mut filter_parts = Vec::new();
    let mut mix_inputs: Vec<String> = Vec::new();

    // Input 0: silent base
    input_args.extend(["-f", "lavfi", "-i"].iter().map(|s| s.to_string()));
    // Use colon separator for lavfi filter options (comma is filter separator in FFmpeg 6+)
    input_args.push(format!("anullsrc=r=44100:cl=stereo:duration={}", duration_str));

    mix_inputs.push("[0:a]".to_string());

    // Add each TTS audio with delay
    let mut cumulative_offset = 0.0f64;
    for clip in script_clips {
        let clip_duration = calculate_clip_duration(clip, tts_results);
        if clip.ost == OstType::OriginalSound {
            cumulative_offset += clip_duration;
            continue;
        }

        let tts = tts_results.get(&clip._id).ok_or_else(|| {
            PipelineError::AudioMerge {
                details: format!(
                    "OST={:?} 片段 {} 缺少 TTS 结果，无法合并音频",
                    clip.ost, clip._id
                ),
            }
        })?;
        let delay_ms = (cumulative_offset * 1000.0).round() as u64;
        let input_idx = mix_inputs.len();

        input_args.push("-i".to_string());
        input_args.push(tts.audio_path.to_string_lossy().to_string());

        let label = format!("[{}:a]", input_idx);
        // Use single delay value — FFmpeg pads unspecified channels with zero
        filter_parts.push(format!("{}adelay={}[tts{}]", label, delay_ms, clip._id));
        mix_inputs.push(format!("[tts{}]", clip._id));

        cumulative_offset += clip_duration;
    }

    let amix_inputs = mix_inputs.join("");
    let input_count = mix_inputs.len();
    if input_count > 1 {
        filter_parts.push(format!(
            "{}amix=inputs={}:duration=longest,volume={}[aout]",
            amix_inputs, input_count, input_count
        ));
    } else {
        filter_parts.push(format!("{}volume=1[aout]", amix_inputs));
    }

    let filter = filter_parts.join(";");

    crate::ffmpeg::command::run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();

        for arg in &input_args {
            cmd.arg(arg);
        }

        cmd.arg("-filter_complex")
            .arg(&filter)
            .arg("-map")
            .arg("[aout]")
            .arg("-y")
            .output(&output_str);

        let mut child = cmd
            .spawn()
            .map_err(|e| crate::error::FFmpegError::SpawnFailed(e.to_string()))?;

        let iter = match child.iter() {
            Ok(iter) => iter,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(crate::error::FFmpegError::SpawnFailed(e.to_string()));
            }
        };

        // Log FFmpeg events but don't treat non-fatal messages as errors
        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::warn!("Audio merge (non-fatal): {}", e);
            }
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(format!(
                "Audio merge exited with code {:?}",
                status.code()
            )));
        }

        Ok(())
    })
    .await
    .map_err(PipelineError::from)?;

    Ok(output_path.to_path_buf())
}

/// 合并所有 SRT 字幕片段，应用时间偏移
pub async fn merge_subtitle_files(
    script_clips: &[ScriptClip],
    tts_results: &HashMap<i64, TtsResult>,
    task_dir: &Path,
) -> Result<Option<PathBuf>, PipelineError> {
    let mut segments = Vec::new();
    let mut cumulative_offset = 0.0f64;

    for clip in script_clips {
        let clip_duration = calculate_clip_duration(clip, tts_results);

        if clip.ost != OstType::OriginalSound {
            if let Some(tts) = tts_results.get(&clip._id) {
                if !tts.word_boundaries.is_empty() {
                    let srt_content = crate::documentary::subtitle::generate_srt_from_word_boundaries(
                        &tts.word_boundaries,
                        0.0, // offset applied via SubtitleSegment during merge
                    );
                    segments.push(SubtitleSegment {
                        srt_content,
                        offset_secs: cumulative_offset,
                    });
                }
            }
        }

        cumulative_offset += clip_duration;
    }

    if segments.is_empty() {
        return Ok(None);
    }

    let merged = merge_srt_files(&segments)?;
    let output_path = task_dir.join("merged_subtitle.srt");
    tokio::fs::write(&output_path, &merged)
        .await
        .map_err(PipelineError::from)?;

    Ok(Some(output_path))
}

/// 计算单个片段的时长（与 pipeline.rs 共享的统一函数）
pub fn calculate_clip_duration(clip: &ScriptClip, tts_results: &HashMap<i64, TtsResult>) -> f64 {
    // For OST 0/2: TTS duration first (clip sized to match narration)
    if clip.ost != OstType::OriginalSound {
        if let Some(tts) = tts_results.get(&clip._id) {
            if tts.duration > 0.0 {
                return tts.duration;
            }
        }
    }

    // Explicit duration override
    if let Some(dur) = clip.duration {
        if dur > 0.0 {
            return dur;
        }
    }

    // source_time_range
    if let Some(ref tr) = clip.source_time_range {
        if let Ok((start, end)) = crate::documentary::timestamp::parse_timestamp_range(tr) {
            let dur = end - start;
            if dur > 0.0 {
                return dur;
            }
        }
    }

    // timestamp range
    if let Ok((start, end)) = crate::documentary::timestamp::parse_timestamp_range(&clip.timestamp) {
        let dur = end - start;
        if dur > 0.0 {
            return dur;
        }
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::WordBoundary;

    fn make_tts_result(clip_id: i64, duration: f64) -> TtsResult {
        TtsResult {
            clip_id,
            audio_path: PathBuf::from(format!("tts_{}.mp3", clip_id)),
            subtitle_path: PathBuf::from(format!("subtitle_{}.srt", clip_id)),
            duration,
            word_boundaries: vec![WordBoundary {
                start_offset: 0,
                end_offset: (duration * 10_000_000.0) as u64,
                text: "测试文本".to_string(),
            }],
        }
    }

    fn make_clip(id: i64, ts: &str, ost: OstType, dur: Option<f64>) -> ScriptClip {
        ScriptClip {
            _id: id,
            timestamp: ts.to_string(),
            picture: "test".to_string(),
            narration: "test".to_string(),
            ost,
            duration: dur,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        }
    }

    #[test]
    fn test_calculate_clip_duration_from_timestamp_range() {
        let tts_results = HashMap::new();
        let clip = make_clip(1, "00:00:00-00:00:10", OstType::NarrationOnly, None);
        let dur = calculate_clip_duration(&clip, &tts_results);
        assert!((dur - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_clip_duration_explicit() {
        let tts_results = HashMap::new();
        let clip = make_clip(1, "00:00:00-00:00:10", OstType::OriginalSound, Some(7.5));
        let dur = calculate_clip_duration(&clip, &tts_results);
        assert!((dur - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_clip_duration_from_tts_fallback() {
        let tts_results = HashMap::from([(1, make_tts_result(1, 5.0))]);
        let clip = make_clip(1, "00:00:00", OstType::NarrationOnly, None);
        let dur = calculate_clip_duration(&clip, &tts_results);
        assert!((dur - 5.0).abs() < f64::EPSILON);
    }
}
