use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::documentary::audio::{merge_audio_files, merge_subtitle_files};
use crate::documentary::clip::clip_all_videos;
use crate::documentary::error::PipelineError;
use crate::documentary::subtitle::generate_srt_from_word_boundaries;
use crate::documentary::timestamp::{parse_time_to_secs, secs_to_ffmpeg_time};
use crate::documentary::types::{DocumentaryRequest, ProgressCallback, ProgressStep, TtsResult};
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
    fn emit_progress(&self, step: ProgressStep, pct: f32, msg: &str) {
        if let Some(ref cb) = self.progress {
            cb(step, pct, msg);
        }
    }
}

/// 步骤 1: 加载脚本
fn step_load_script(state: &mut PipelineState, script_path: &Path) -> Result<(), PipelineError> {
    let script = crate::script::load_script(script_path)?;
    tracing::info!("## 1. 加载视频脚本 — {} 个片段", script.len());
    state.emit_progress(ProgressStep::LoadScript, 0.0, "加载视频脚本");
    state.script = script;
    Ok(())
}

/// 步骤 2: TTS 语音生成
async fn step_tts(
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
            crate::documentary::subtitle::write_srt_file(&srt_content, &srt_path)?;
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

    state.emit_progress(ProgressStep::Tts, 20.0, "TTS 生成完成");
    Ok(())
}

/// 步骤 3: 视频裁剪
async fn step_clip(
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

        let clip_duration = compute_clip_duration(clip, &state.tts_results);
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

    state.emit_progress(ProgressStep::Clip, 60.0, "视频裁剪完成");
    Ok(())
}

/// 步骤 4: 合并音频和字幕
async fn step_merge_audio_subtitle(
    state: &mut PipelineState,
    request: &DocumentaryRequest,
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

    state.emit_progress(ProgressStep::MergeAudio, 60.0, "音频字幕合并完成");
    Ok(())
}

/// 步骤 5: 视频拼接
async fn step_concat(state: &mut PipelineState) -> Result<(), PipelineError> {
    tracing::info!("## 5. 合并视频 — {} 个片段", state.script.len());

    let concat_list_path = state.task_dir.join("concat_list.txt");
    let mut concat_content = String::new();
    for clip in &state.script {
        if let Some(ref video_path) = clip.video {
            // FFmpeg concat demuxer requires forward slashes
            let path_str = video_path.to_string_lossy().replace('\\', "/");
            concat_content.push_str(&format!("file '{}'\n", path_str));
        }
    }
    std::fs::write(&concat_list_path, &concat_content)?;

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

        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::error!("Concat error: {}", e);
            }
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

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
    state.emit_progress(ProgressStep::Concat, 80.0, "视频拼接完成");
    Ok(())
}

/// 步骤 6: 最终合成
async fn step_composite(
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

    let mut input_idx = 0usize;

    // Input 0: concatenated video
    let video_str = merged_video.to_string_lossy().to_string();
    let _ = &input_idx; // used later in closure

    // Build filter chains
    let mut filter_parts = Vec::new();
    let mut audio_mix_inputs = Vec::new();

    // Add merged TTS audio
    if let Some(ref audio_path) = state.merged_audio_path {
        let audio_str = audio_path.to_string_lossy().to_string();
        let vol = request.tts_volume;
        filter_parts.push(format!(
            "[{}:a]volume={:.2}[tts]",
            input_idx, vol
        ));
        audio_mix_inputs.push("[tts]".to_string());
        input_idx += 1;

        // We need to add the audio as another input
        // This will be handled by building the full command
    }

    let has_bgm = request.bgm_path.as_ref().is_some();
    let _needs_subtitle = request.subtitle_enabled && state.merged_subtitle_path.is_some();
    let _has_audio_mix = !audio_mix_inputs.is_empty() || has_bgm;

    // For simplicity, build the FFmpeg command directly
    let output_str = output_path.to_string_lossy().to_string();
    let video_str_clone = video_str.clone();
    let audio_str_opt = state.merged_audio_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let srt_path_opt = state.merged_subtitle_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let bgm_path_opt = request.bgm_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let tts_vol = request.tts_volume;
    let orig_vol = request.original_volume;
    let bgm_vol = request.bgm_volume;
    let total_dur = state.total_duration;
    let font_size = request.subtitle_font_size;
    let _subtitle_color = request.subtitle_color.clone();
    let subtitle_enabled = request.subtitle_enabled;

    crate::ffmpeg::command::run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-i").arg(&video_str_clone);

        let mut filter_complex_parts = Vec::new();
        let mut audio_inputs_count = 0usize;

        // Add TTS audio input
        if let Some(ref audio_str) = audio_str_opt {
            cmd.arg("-i").arg(audio_str);
            filter_complex_parts.push(format!("[{}:a]volume={:.2}[tts]", 1, tts_vol));
            audio_inputs_count += 1;
        }

        // Add BGM input
        if let Some(ref bgm_str) = bgm_path_opt {
            cmd.arg("-i").arg(bgm_str);
            let bgm_idx = 1 + audio_inputs_count;
            let fade_start = if total_dur > 3.0 { total_dur - 3.0 } else { 0.0 };
            filter_complex_parts.push(format!(
                "[{}:a]aloop=loop=-1:size=2e+09,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
                bgm_idx, bgm_vol, fade_start
            ));
            audio_inputs_count += 1;
        }

        // Build audio mix
        if audio_inputs_count > 0 {
            let mut mix_labels = Vec::new();
            if audio_str_opt.is_some() {
                mix_labels.push("[tts]".to_string());
            }
            if bgm_path_opt.is_some() {
                mix_labels.push("[bgm]".to_string());
            }
            let mix_inputs = mix_labels.join("");
            filter_complex_parts.push(format!(
                "{}amix=inputs={}:duration=longest[aout]",
                mix_inputs, audio_inputs_count
            ));
        }

        // Video filter for subtitles
        let video_filter = if subtitle_enabled {
            if let Some(ref srt_str) = srt_path_opt {
                // Escape special chars in path for FFmpeg subtitles filter
                let escaped_srt = srt_str.replace('\\', "/").replace(':', "\\:").replace("'", "\\'");
                Some(format!(
                    "subtitles='{}':force_style='FontSize={}'",
                    escaped_srt, font_size
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Build complete filter_complex
        let mut full_filter = String::new();
        if !filter_complex_parts.is_empty() {
            full_filter = filter_complex_parts.join(";");
        }

        if audio_inputs_count > 0 {
            cmd.arg("-map").arg("0:v");
            cmd.arg("-map").arg("[aout]");
        }

        if let Some(ref vf) = video_filter {
            if full_filter.is_empty() {
                cmd.arg("-vf").arg(vf);
            } else {
                full_filter = format!("{};[0:v]{}[vout]", full_filter, vf);
                cmd.arg("-map").arg("[vout]");
                // Remove the earlier -map 0:v if we added it
            }
        } else if audio_inputs_count == 0 {
            // No filters at all, just copy
        }

        if !full_filter.is_empty() {
            cmd.arg("-filter_complex").arg(&full_filter);
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

        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::error!("Composite error: {}", e);
            }
        }

        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;

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
    state.emit_progress(ProgressStep::Composite, 100.0, "最终合成完成");
    Ok(())
}

/// 计算片段时长
fn compute_clip_duration(
    clip: &crate::script::types::ScriptClip,
    tts_results: &HashMap<i64, TtsResult>,
) -> f64 {
    // TTS duration for OST=0/2
    if clip.ost != OstType::OriginalSound {
        if let Some(tts) = tts_results.get(&clip._id) {
            if tts.duration > 0.0 {
                return tts.duration;
            }
        }
    }

    // Timestamp range for OST=1
    if let Ok((start, end)) = crate::documentary::timestamp::parse_timestamp_range(&clip.timestamp)
    {
        let dur = end - start;
        if dur > 0.0 {
            return dur;
        }
    }

    0.0
}

/// 主入口：执行完整 6 步纪录片流水线
pub async fn run_documentary(
    request: DocumentaryRequest,
    config: &crate::config::types::AppConfig,
    proxy: Option<&crate::config::types::ProxySection>,
) -> Result<PathBuf, PipelineError> {
    // Create task directory
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    std::fs::create_dir_all(&task_dir)?;

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
        progress: None,
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
    use crate::tts::WordBoundary;

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
    fn test_compute_clip_duration_ost0_from_tts() {
        let clip = make_test_clip(1, "00:00:00-00:00:10", OstType::NarrationOnly);
        let tts = HashMap::from([(1, make_tts(1, 5.0))]);
        let dur = compute_clip_duration(&clip, &tts);
        assert!((dur - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_clip_duration_ost1_from_range() {
        let clip = make_test_clip(2, "00:00:05-00:00:15", OstType::OriginalSound);
        let tts = HashMap::new();
        let dur = compute_clip_duration(&clip, &tts);
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
            let dur = compute_clip_duration(clip, &tts);
            assert!(dur > 0.0);
            cumulative += dur;
        }
        assert!((cumulative - 13.0).abs() < f64::EPSILON);
    }
}
