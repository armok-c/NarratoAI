use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::documentary::error::PipelineError;
use crate::documentary::timestamp::{
    format_clip_filename, parse_time_to_secs, secs_to_ffmpeg_time,
};
use crate::documentary::types::TtsResult;
use crate::ffmpeg::hwaccel::{recommend_profile, HwAccelProfile};
use crate::script::types::{OstType, ScriptClip};

/// OST=0 裁剪: 仅解说音，移除原始音频
///
/// 裁剪范围从 timestamp 起始到 TTS 时长结束（不含额外秒数）。
pub async fn clip_ost0_narration_only(
    video_path: &Path,
    clip: &ScriptClip,
    tts_result: &TtsResult,
    task_dir: &Path,
    profile: HwAccelProfile,
) -> Result<PathBuf, PipelineError> {
    let start_secs = parse_time_to_secs(&clip.timestamp.split('-').next().unwrap_or(&clip.timestamp))?;
    let end_secs = start_secs + tts_result.duration;
    let duration = tts_result.duration;

    let filename = format_clip_filename(OstType::NarrationOnly, &secs_to_ffmpeg_time(start_secs), &secs_to_ffmpeg_time(end_secs));
    let output_path = task_dir.join(&filename);

    run_clip_ffmpeg(video_path, &output_path, start_secs, duration, true, profile).await?;
    Ok(output_path)
}

/// OST=1 裁剪: 保留原声，按 timestamp 精确裁剪
pub async fn clip_ost1_original_sound(
    video_path: &Path,
    clip: &ScriptClip,
    task_dir: &Path,
    profile: HwAccelProfile,
) -> Result<PathBuf, PipelineError> {
    let ts_parts: Vec<&str> = clip.timestamp.split('-').collect();
    if ts_parts.len() != 2 {
        return Err(PipelineError::Timestamp(format!(
            "OST=1 片段需要完整时间范围: {}",
            clip.timestamp
        )));
    }
    let start_secs = parse_time_to_secs(ts_parts[0])?;
    let end_secs = parse_time_to_secs(ts_parts[1])?;
    let duration = end_secs - start_secs;

    let filename = format_clip_filename(OstType::OriginalSound, &secs_to_ffmpeg_time(start_secs), &secs_to_ffmpeg_time(end_secs));
    let output_path = task_dir.join(&filename);

    run_clip_ffmpeg(video_path, &output_path, start_secs, duration, false, profile).await?;
    Ok(output_path)
}

/// OST=2 裁剪: 混合模式，保留原始音频+叠加 TTS
///
/// 裁剪范围从 timestamp 起始到 TTS 时长结束，保留原始音频轨道。
pub async fn clip_ost2_mixed(
    video_path: &Path,
    clip: &ScriptClip,
    tts_result: &TtsResult,
    task_dir: &Path,
    profile: HwAccelProfile,
) -> Result<PathBuf, PipelineError> {
    let start_secs = parse_time_to_secs(&clip.timestamp.split('-').next().unwrap_or(&clip.timestamp))?;
    let end_secs = start_secs + tts_result.duration;
    let duration = tts_result.duration;

    let filename = format_clip_filename(OstType::Mixed, &secs_to_ffmpeg_time(start_secs), &secs_to_ffmpeg_time(end_secs));
    let output_path = task_dir.join(&filename);

    run_clip_ffmpeg(video_path, &output_path, start_secs, duration, false, profile).await?;
    Ok(output_path)
}

/// 批量裁剪所有脚本片段，按 OST 类型分发到对应函数
pub async fn clip_all_videos(
    video_path: &Path,
    script_clips: &[ScriptClip],
    tts_results: &HashMap<i64, TtsResult>,
    task_dir: &Path,
) -> Result<HashMap<i64, PathBuf>, PipelineError> {
    let profile = recommend_profile();
    let mut video_clips = HashMap::new();

    for clip in script_clips {
        let output = match clip.ost {
            OstType::NarrationOnly => {
                let tts = tts_results.get(&clip._id).ok_or_else(|| {
                    PipelineError::VideoClip {
                        details: format!("OST=0 片段 {} 缺少 TTS 结果", clip._id),
                    }
                })?;
                clip_ost0_narration_only(video_path, clip, tts, task_dir, profile).await?
            }
            OstType::OriginalSound => {
                clip_ost1_original_sound(video_path, clip, task_dir, profile).await?
            }
            OstType::Mixed => {
                let tts = tts_results.get(&clip._id).ok_or_else(|| {
                    PipelineError::VideoClip {
                        details: format!("OST=2 片段 {} 缺少 TTS 结果", clip._id),
                    }
                })?;
                clip_ost2_mixed(video_path, clip, tts, task_dir, profile).await?
            }
        };
        video_clips.insert(clip._id, output);
    }

    Ok(video_clips)
}

/// 执行 FFmpeg 裁剪命令
async fn run_clip_ffmpeg(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    no_audio: bool,
    profile: HwAccelProfile,
) -> Result<(), PipelineError> {
    let profile_info = crate::ffmpeg::hwaccel::get_profile(profile);
    let encoder = profile_info.map(|p| p.encoder).unwrap_or("libx264");

    let input_str = input.to_string_lossy().to_string();
    let output_str = output.to_string_lossy().to_string();
    let start_str = format!("{:.3}", start);
    let duration_str = format!("{:.3}", duration);
    let encoder_str = encoder.to_string();

    crate::ffmpeg::command::run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.seek(&start_str)
            .input(&input_str)
            .duration(&duration_str)
            .overwrite();

        if no_audio {
            cmd.arg("-an");
        } else {
            cmd.codec_audio("aac");
        }

        cmd.codec_video(encoder_str.as_str())
            .arg("-pix_fmt")
            .arg("yuv420p")
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

        let mut had_errors = false;
        for event in iter {
            match event {
                ffmpeg_sidecar::event::FfmpegEvent::Error(e) => {
                    tracing::error!("FFmpeg clip error: {}", e);
                    had_errors = true;
                }
                _ => {}
            }
        }

        if had_errors {
            return Err(crate::error::FFmpegError::ExecutionError(
                "FFmpeg clip reported errors".into(),
            ));
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(format!(
                "FFmpeg clip exited with code {:?}",
                status.code()
            )));
        }

        Ok(())
    })
    .await
    .map_err(PipelineError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clip_filename_ost0() {
        let name = format_clip_filename(OstType::NarrationOnly, "00:00:00.000", "00:00:20.250");
        assert!(name.starts_with("ost0_"));
        assert!(name.ends_with(".mp4"));
    }

    #[test]
    fn test_parse_single_timestamp() {
        let ts = "00:00:05,500-00:00:15,750";
        let start: &str = ts.split('-').next().unwrap();
        let secs = parse_time_to_secs(start).unwrap();
        assert!((secs - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timestamp_range_extraction() {
        let ts = "00:00:05-00:00:15";
        let parts: Vec<&str> = ts.split('-').collect();
        assert_eq!(parts.len(), 2);
        let start = parse_time_to_secs(parts[0]).unwrap();
        let end = parse_time_to_secs(parts[1]).unwrap();
        assert_eq!(start, 5.0);
        assert_eq!(end, 15.0);
        assert_eq!(end - start, 10.0);
    }
}
