use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::documentary::audio::{calculate_clip_duration, merge_audio_files, merge_subtitle_files};
use crate::documentary::clip::clip_all_videos;
use crate::documentary::error::PipelineError;
use crate::documentary::subtitle::generate_srt_from_word_boundaries;
use crate::documentary::timestamp::{parse_time_to_secs, secs_to_ffmpeg_time};
use crate::documentary::types::{DocumentaryRequest, ProgressCallback, TtsResult};
use crate::script::types::OstType;

/// 流水线中间状态——贯穿 6 步
pub struct PipelineState {
    pub task_id: String,
    pub task_dir: PathBuf,
    pub script: crate::script::types::Script,
    pub tts_results: HashMap<i64, TtsResult>,
    pub video_clips: HashMap<i64, PathBuf>,
    pub merged_audio_path: Option<PathBuf>,
    pub merged_subtitle_path: Option<PathBuf>,
    pub combined_video_path: Option<PathBuf>,
    pub output_video_path: Option<PathBuf>,
    pub total_duration: f64,
    pub progress: Option<ProgressCallback>,
}

impl PipelineState {
    fn emit_progress(&self, step: &str, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}

/// 步骤 1: 加载脚本
fn step_load_script(state: &mut PipelineState, script_path: &Path) -> Result<(), PipelineError> {
    let script = crate::script::load_script(script_path)?;
    tracing::info!("## 1. 加载视频脚本 — {} 个片段", script.len());
    state.emit_progress("load_script", 0.0, "加载视频脚本");
    state.script = script;
    Ok(())
}

/// 步骤 2: TTS 语音生成
pub(crate) async fn step_tts(
    state: &mut PipelineState,
    request: &DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
) -> Result<(), PipelineError> {
    let clips_needing_tts: Vec<_> = state
        .script
        .iter()
        .filter(|c| c.ost == OstType::NarrationOnly || c.ost == OstType::Mixed)
        .collect();

    tracing::info!("## 2. TTS 生成 — {} 个片段", clips_needing_tts.len());

    for clip in &clips_needing_tts {
        let output_path = state.task_dir.join(format!("tts_{}.mp3", clip._id));
        let srt_path = state.task_dir.join(format!("subtitle_{}.srt", clip._id));

        let tts_output = crate::tts::synthesize(
            &request.tts_engine,
            &clip.narration,
            &request.voice_name,
            request.voice_rate,
            request.voice_pitch,
            &output_path,
            proxy,
            Some(config),
        )
        .await?;

        // Generate per-clip SRT
        if !tts_output.word_boundaries.is_empty() {
            let srt_content = generate_srt_from_word_boundaries(&tts_output.word_boundaries, 0.0);
            tokio::fs::write(&srt_path, &srt_content)
                .await
                .map_err(PipelineError::from)?;
        }

        state.tts_results.insert(
            clip._id,
            TtsResult {
                clip_id: clip._id,
                audio_path: output_path,
                subtitle_path: srt_path,
                duration: tts_output.duration,
                word_boundaries: tts_output.word_boundaries,
            },
        );
    }

    state.emit_progress("tts", 20.0, "TTS 生成完成");
    Ok(())
}

/// 步骤 3: 视频裁剪
pub(crate) async fn step_clip(
    state: &mut PipelineState,
    video_path: &Path,
) -> Result<(), PipelineError> {
    tracing::info!("## 3. 视频裁剪 — {} 个片段", state.script.len());

    let video_clips =
        clip_all_videos(video_path, &state.script, &state.tts_results, &state.task_dir).await?;

    // Update script clips with computed fields
    let mut cumulative_time = 0.0f64;
    for clip in &mut state.script {
        if let Some(clip_path) = video_clips.get(&clip._id) {
            clip.video = Some(clip_path.clone());
        }

        let clip_duration = calculate_clip_duration(clip, &state.tts_results);
        if clip_duration <= 0.0 {
            return Err(PipelineError::VideoClip {
                details: format!("片段 {} 时长计算为零", clip._id),
            });
        }
        let start_secs = parse_time_to_secs(
            clip.timestamp.split('-').next().unwrap_or(&clip.timestamp),
        )?;
        let end_secs = start_secs + clip_duration;

        clip.source_time_range = Some(format!(
            "{}-{}",
            secs_to_ffmpeg_time(start_secs),
            secs_to_ffmpeg_time(end_secs)
        ));
        clip.duration = Some(clip_duration);
        clip.edited_time_range = Some(format!(
            "{}-{}",
            secs_to_ffmpeg_time(cumulative_time),
            secs_to_ffmpeg_time(cumulative_time + clip_duration)
        ));

        cumulative_time += clip_duration;
    }

    state.video_clips = video_clips;
    state.total_duration = cumulative_time;

    state.emit_progress("clip", 60.0, "视频裁剪完成");
    Ok(())
}

/// 步骤 4: 合并音频和字幕
///
/// Note: `_request` is currently unused but retained for future
/// per-request audio volume control during merge.
pub(crate) async fn step_merge_audio_subtitle(
    state: &mut PipelineState,
    _request: &DocumentaryRequest,
) -> Result<(), PipelineError> {
    tracing::info!("## 4. 合并音频和字幕");

    // Merge audio
    let audio_output = state.task_dir.join("merger_audio.mp3");
    let merged_audio = merge_audio_files(
        &state.script,
        &state.tts_results,
        state.total_duration,
        &audio_output,
    )
    .await?;
    state.merged_audio_path = Some(merged_audio);

    // Merge subtitles
    let merged_sub = merge_subtitle_files(
        &state.script,
        &state.tts_results,
        &state.task_dir,
    )
    .await?;
    state.merged_subtitle_path = merged_sub;

    state.emit_progress("merge_audio", 70.0, "音频字幕合并完成");
    Ok(())
}

/// 步骤 5: 视频拼接
pub(crate) async fn step_concat(state: &mut PipelineState) -> Result<(), PipelineError> {
    tracing::info!("## 5. 合并视频 — {} 个片段", state.script.len());

    let concat_list_path = state.task_dir.join("concat_list.txt");
    let mut concat_content = String::new();
    for clip in &state.script {
        if let Some(ref video_path) = clip.video {
            // FFmpeg concat demuxer requires forward slashes
            let path_str = video_path.to_string_lossy().replace('\\', "/");
            if path_str.contains('\n') || path_str.contains('\r') {
                return Err(PipelineError::Concat {
                    details: format!("视频路径包含非法字符: {}", clip._id),
                });
            }
            let escaped = path_str.replace("'", "'\\''");
            concat_content.push_str(&format!("file '{}'\n", escaped));
        } else {
            return Err(PipelineError::Concat {
                details: format!("片段 {} 缺少视频文件", clip._id),
            });
        }
    }
    tokio::fs::write(&concat_list_path, &concat_content)
        .await
        .map_err(PipelineError::from)?;

    let output_path = state.task_dir.join("merger.mp4");
    let concat_path = concat_list_path.to_string_lossy().to_string();
    let output_str = output_path.to_string_lossy().to_string();

    crate::ffmpeg::command::run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&concat_path)
            .arg("-c")
            .arg("copy")
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

        let mut had_errors = false;
        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::error!("Concat error: {}", e);
                had_errors = true;
            }
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

        if had_errors {
            return Err(crate::error::FFmpegError::ExecutionError(
                "Concat reported FFmpeg errors".into(),
            ));
        }

        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(format!(
                "Concat exited with code {:?}",
                status.code()
            )));
        }

        Ok(())
    })
    .await
    .map_err(PipelineError::from)?;

    state.combined_video_path = Some(output_path);
    state.emit_progress("concat", 80.0, "视频拼接完成");
    Ok(())
}

/// 步骤 6: 最终合成
pub(crate) async fn step_composite(
    state: &mut PipelineState,
    request: &DocumentaryRequest,
) -> Result<(), PipelineError> {
    tracing::info!("## 6. 最终合成");

    let merged_video = state
        .combined_video_path
        .as_ref()
        .ok_or_else(|| PipelineError::Composite {
            details: "缺少拼接视频".to_string(),
        })?;

    let output_path = state.task_dir.join("combined.mp4");

    let output_str = output_path.to_string_lossy().to_string();
    let video_str_clone = merged_video.to_string_lossy().to_string();
    let audio_str_opt = state.merged_audio_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let srt_path_opt = state.merged_subtitle_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let bgm_path_opt = request.bgm_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let tts_vol = request.tts_volume;
    let bgm_vol = request.bgm_volume;
    let total_dur = state.total_duration;
    let orig_vol = request.original_volume;
    let has_original_audio = state.script.iter().any(|c| c.ost != OstType::NarrationOnly);
    let font_size = request.subtitle_font_size;
    let subtitle_enabled = request.subtitle_enabled;
    let subtitle_force_style = {
        let hex = request.subtitle_color.trim_start_matches('#');
        let ass_color = if hex.len() == 6 {
            format!("&H00{}{}{}", &hex[4..6], &hex[2..4], &hex[0..2])
        } else {
            "&H00FFFFFF".to_string()
        };
        let alignment = match request.subtitle_position.as_str() {
            "top" => "8",
            "center" => "5",
            _ => "2",
        };
        let mut style = format!("FontSize={},PrimaryColour={},Alignment={}", font_size, ass_color, alignment);
        if let Some(ref font) = request.subtitle_font {
            let sanitized: String = font
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect();
            if !sanitized.is_empty() {
                style.push_str(&format!(",FontName={}", sanitized));
            }
        }
        style
    };

    crate::ffmpeg::command::run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-i").arg(&video_str_clone);

        let mut filter_complex_parts = Vec::new();
        let mut external_audio_count = 0usize;
        let mut amix_input_count = 0usize;

        // Add original audio from video (volume-adjusted)
        if has_original_audio {
            filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", orig_vol));
            amix_input_count += 1;
        }

        // Add TTS audio input
        if let Some(ref audio_str) = audio_str_opt {
            cmd.arg("-i").arg(audio_str);
            filter_complex_parts.push(format!("[{}:a]volume={:.2}[tts]", 1, tts_vol));
            external_audio_count += 1;
            amix_input_count += 1;
        }

        // Add BGM input
        if let Some(ref bgm_str) = bgm_path_opt {
            cmd.arg("-i").arg(bgm_str);
            let bgm_idx = 1 + external_audio_count;
            let fade_start = if total_dur > 3.0 { total_dur - 3.0 } else { 0.0 };
            filter_complex_parts.push(format!(
                "[{}:a]aloop=loop=-1:size=2e+09,atrim=0:{:.3},asetpts=PTS-STARTPTS,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
                bgm_idx, total_dur, bgm_vol, fade_start
            ));
            amix_input_count += 1;
        }

        // Build audio mix
        if amix_input_count > 0 {
            let mut mix_labels = Vec::new();
            if has_original_audio {
                mix_labels.push("[orig]".to_string());
            }
            if audio_str_opt.is_some() {
                mix_labels.push("[tts]".to_string());
            }
            if bgm_path_opt.is_some() {
                mix_labels.push("[bgm]".to_string());
            }
            let mix_inputs = mix_labels.join("");
            let compensation = if amix_input_count > 1 {
                format!(",volume={}", amix_input_count)
            } else {
                String::new()
            };
            filter_complex_parts.push(format!(
                "{}amix=inputs={}:duration=longest{}[aout]",
                mix_inputs, amix_input_count, compensation
            ));
        }

        // Video filter for subtitles — always use filter_complex, never -vf
        let mut has_video_filter = false;
        if subtitle_enabled {
            if let Some(ref srt_str) = srt_path_opt {
                let escaped_srt = srt_str
                    .replace('\\', "/")
                    .replace(':', "\\:")
                    .replace("'", "\\'")
                    .replace('[', "\\[")
                    .replace(']', "\\]")
                    .replace(';', "\\;")
                    .replace('\n', "")
                    .replace('\r', "");
                filter_complex_parts.push(format!(
                    "[0:v]subtitles='{}':force_style='{}'[vout]",
                    escaped_srt, subtitle_force_style
                ));
                has_video_filter = true;
            }
        }

        // Apply filter_complex
        if !filter_complex_parts.is_empty() {
            cmd.arg("-filter_complex")
                .arg(&filter_complex_parts.join(";"));
        }

        // Map outputs — single video map, single audio map
        if amix_input_count > 0 {
            cmd.arg("-map").arg(if has_video_filter { "[vout]" } else { "0:v" });
            cmd.arg("-map").arg("[aout]");
        } else if has_video_filter {
            cmd.arg("-map").arg("[vout]");
        }

        cmd.codec_video("libx264")
            .arg("-preset")
            .arg("medium")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .codec_audio("aac")
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

        let mut had_errors = false;
        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::error!("Composite error: {}", e);
                had_errors = true;
            }
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

        if had_errors {
            return Err(crate::error::FFmpegError::ExecutionError(
                "Composite reported FFmpeg errors".into(),
            ));
        }

        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(format!(
                "Composite exited with code {:?}",
                status.code()
            )));
        }

        Ok(())
    })
    .await
    .map_err(PipelineError::from)?;

    state.output_video_path = Some(output_path);
    state.emit_progress("composite", 100.0, "最终合成完成");
    Ok(())
}

/// 主入口：执行完整 6 步纪录片流水线
pub async fn run_documentary(
    request: DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
    progress: Option<ProgressCallback>,
) -> Result<PathBuf, PipelineError> {
    request.validate().map_err(|e| PipelineError::Validation { details: e })?;

    // Create task directory
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    tokio::fs::create_dir_all(&task_dir).await?;

    let mut state = PipelineState {
        task_id,
        task_dir,
        script: Vec::new(),
        tts_results: HashMap::new(),
        video_clips: HashMap::new(),
        merged_audio_path: None,
        merged_subtitle_path: None,
        combined_video_path: None,
        output_video_path: None,
        total_duration: 0.0,
        progress,
    };

    // Step 1: Load script
    step_load_script(&mut state, &request.script_path)?;

    // Step 2: TTS generation
    step_tts(&mut state, &request, config, proxy).await?;

    // Step 3: Video clipping
    step_clip(&mut state, &request.video_path).await?;

    // Step 4: Merge audio and subtitles
    step_merge_audio_subtitle(&mut state, &request).await?;

    // Step 5: Concatenate clips
    step_concat(&mut state).await?;

    // Step 6: Final composite
    step_composite(&mut state, &request).await?;

    state
        .output_video_path
        .ok_or_else(|| PipelineError::Composite {
            details: "流水线完成但未生成输出视频".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::{OstType, ScriptClip};

    fn make_test_clip(id: i64, ts: &str, ost: OstType) -> ScriptClip {
        ScriptClip {
            _id: id,
            timestamp: ts.to_string(),
            picture: "测试画面".to_string(),
            narration: "测试解说".to_string(),
            ost,
            duration: None,
            source_time_range: None,
            edited_time_range: None,
            audio: None,
            video: None,
            subtitle: None,
        }
    }

    fn make_tts(id: i64, dur: f64) -> TtsResult {
        TtsResult {
            clip_id: id,
            audio_path: PathBuf::from(format!("tts_{}.mp3", id)),
            subtitle_path: PathBuf::from(format!("subtitle_{}.srt", id)),
            duration: dur,
            word_boundaries: vec![],
        }
    }

    #[test]
    fn test_calculate_clip_duration_ost0_from_tts() {
        let clip = make_test_clip(1, "00:00:00-00:00:10", OstType::NarrationOnly);
        let tts = HashMap::from([(1, make_tts(1, 5.0))]);
        let dur = calculate_clip_duration(&clip, &tts);
        assert!((dur - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_clip_duration_ost1_from_range() {
        let clip = make_test_clip(2, "00:00:05-00:00:15", OstType::OriginalSound);
        let tts = HashMap::new();
        let dur = calculate_clip_duration(&clip, &tts);
        assert!((dur - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_edited_time_range_cumulative() {
        let mut tts = HashMap::new();
        tts.insert(1, make_tts(1, 5.0));
        tts.insert(2, make_tts(2, 8.0));

        let clips = vec![
            make_test_clip(1, "00:00:00-00:00:10", OstType::NarrationOnly),
            make_test_clip(2, "00:00:05-00:00:15", OstType::Mixed),
        ];

        let mut cumulative = 0.0f64;
        for clip in &clips {
            let dur = calculate_clip_duration(clip, &tts);
            assert!(dur > 0.0);
            cumulative += dur;
        }
        assert!((cumulative - 13.0).abs() < f64::EPSILON);
    }
}
