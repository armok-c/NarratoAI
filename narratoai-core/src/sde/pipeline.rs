use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::types::{AppConfig, ProxySection};
use crate::documentary::audio::{calculate_clip_duration, merge_audio_files, merge_subtitle_files};
use crate::documentary::clip::clip_all_videos;
use crate::documentary::error::PipelineError;
use crate::documentary::subtitle::generate_srt_from_word_boundaries;
use crate::documentary::timestamp::{parse_time_to_secs, secs_to_ffmpeg_time, secs_to_srt_time};
use crate::documentary::types::TtsResult;
use crate::ffmpeg::command::run_ffmpeg;
use crate::llm::provider::LlmProvider;
use crate::llm::registry::Registry;
use crate::prompt::manager::PromptManager;
use crate::script::types::{OstType, ScriptClip};
use crate::sde::error::SdeError;
use crate::sde::script_gen::{parse_script, step_analyze_plot, step_generate_script};
use crate::subtitle::parser::parse_subtitle_file;
use crate::sde::timestamp::find_precise_range;
use crate::sde::types::{SdePipelineState, SdeProgressCallback, SdeRequest};

// ============================================================
//  主编排器：run_sde()
//   接收完整参数集，顺序执行 9 步 SDE 流水线，返回输出视频路径
// ============================================================

/// SDE 流水线主入口——9 步顺序编排
///
/// 步骤：ParseSubtitle(0%) → PlotAnalysis(15%) → ScriptGenerate(30%)
///       → LoadScript(40%) → Tts(50%) → Clip(70%)
///       → MergeAudio(80%) → Concat(90%) → Composite(100%)
///
/// 返回最终输出视频的 PathBuf。中间产物保存在 task_dir 中。
pub async fn run_sde(
    request: SdeRequest,
    config: &AppConfig,
    proxy: Option<&ProxySection>,
    registry: &Registry,
    prompt_manager: &PromptManager,
    progress: Option<SdeProgressCallback>,
) -> Result<PathBuf, SdeError> {
    // 0. 参数校验（D-04, T-7-10）
    request
        .validate()
        .map_err(|e| SdeError::Validation { details: e })?;

    // 1. 创建任务目录
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    tokio::fs::create_dir_all(&task_dir).await?;

    // 2. 初始化流水线状态
    let mut state = SdePipelineState::new(task_id, task_dir);
    state.progress = progress;

    // 3. 获取 text LLM provider
    let text_provider = registry
        .get(&config.app.text_llm_provider)
        .map_err(|e| SdeError::Validation {
            details: format!("获取 LLM provider 失败: {}", e),
        })?;

    // ============================================================
    //  步骤 0: 字幕解析 (ParseSubtitle)  —  0% → 10%
    // ============================================================
    state.emit_progress("parse_subtitle", 0.0, "解析字幕文件");

    let subtitle_path = request.subtitle_path.clone();
    let (segments, text, encoding) = tokio::task::spawn_blocking(move || {
        parse_subtitle_file(&subtitle_path)
    })
    .await
    .map_err(|e| SdeError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e),
    })??;

    state.subtitle_text = text;
    state.subtitle_segments = segments;
    state.detected_encoding = encoding;

    // 保存中间产物
    tokio::fs::write(state.task_dir.join("subtitle_source.txt"), &state.subtitle_text)
        .await?;
    state.emit_progress("parse_subtitle", 10.0, "字幕解析完成");

    // ============================================================
    //  步骤 1: 剧情分析 (PlotAnalysis)  —  15% → 30%
    // ============================================================
    state.emit_progress("plot_analysis", 15.0, "正在分析剧情...");
    step_analyze_plot(
        &mut state,
        text_provider.as_ref(),
        prompt_manager,
        request.temperature,
    )
    .await?;
    state.emit_progress("plot_analysis", 30.0, "剧情分析完成");

    // ============================================================
    //  步骤 2: 解说脚本生成 (ScriptGenerate)  —  30% → 40%
    // ============================================================
    state.emit_progress("script_generate", 30.0, "正在生成解说脚本...");
    let subtitle_text_for_script = state.subtitle_text.clone();
    step_generate_script(
        &mut state,
        text_provider.as_ref(),
        prompt_manager,
        request.temperature,
        &request.drama_name,
        &subtitle_text_for_script,
    )
    .await?;
    state.emit_progress("script_generate", 40.0, "解说脚本生成完成");

    // ============================================================
    //  步骤 3: 脚本解析 + 校验 (LoadScript)  —  40% → 45%
    //  G1/G5: JSON Parse Guard + Empty Script Guard
    //  G2: Timestamp Non-Overlap Guard
    //  G3: OST Ratio Warning
    // ============================================================
    state.emit_progress("load_script", 40.0, "正在解析脚本...");

    // G1: JSON Parse Guard — parse_script 自动执行 JSON 修复（repair_json）+ 反序列化
    // G5: Empty Script Guard — parse_script 检查 clips.is_empty()
    state.script = parse_script(&state.narration_raw)?;

    // 保存最终脚本（异步 I/O，避免阻塞 tokio runtime）
    {
        let script_json = serde_json::to_string_pretty(&state.script)
            .map_err(|e| SdeError::JsonRepair {
                details: format!("序列化脚本失败: {}", e),
            })?;
        tokio::fs::write(state.task_dir.join("script_final.json"), &script_json)
            .await
            .map_err(|e| SdeError::Io { source: e })?;
    }

    // G2: Timestamp Non-Overlap Guard
    detect_and_abort_overlaps(&state.script)?;

    // G3: OST Ratio Warning
    warn_ost_ratio(&state.script);

    state.emit_progress("load_script", 45.0, "脚本解析完成");

    // ============================================================
    //  步骤 4: TTS 生成 (Tts)  —  50% → 60%
    //  D-23: "播放原片" 前缀检测跳过 TTS
    //  OST=1 片段无需 TTS（原始音轨）
    // ============================================================
    state.emit_progress("tts", 50.0, "正在生成配音...");

    for clip in &state.script {
        // D-23: "播放原片" 前缀仅对 OST=1 片段跳过 TTS；OST=0/2 仍需 TTS 结果
        if clip.narration.starts_with("播放原片") && clip.ost == OstType::OriginalSound {
            continue;
        }
        // OST=1 片段使用原始音轨，无需 TTS
        if clip.ost == OstType::OriginalSound {
            continue;
        }

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
        .await
        .map_err(|e| SdeError::VideoProcess(PipelineError::TtsGeneration { source: e }))?;

        // 生成单片段 SRT 字幕
        if !tts_output.word_boundaries.is_empty() {
            let srt_content =
                generate_srt_from_word_boundaries(&tts_output.word_boundaries, 0.0);
            tokio::fs::write(&srt_path, &srt_content)
                .await
                .map_err(|e| SdeError::Io { source: e })?;
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

    state.emit_progress("tts", 60.0, "配音生成完成");

    // ============================================================
    //  步骤 5: 视频裁剪 (Clip)  —  70% → 75%
    //  D-29: OST=1 精确时间戳校正（find_precise_range）
    // ============================================================
    state.emit_progress("clip", 70.0, "正在裁剪视频...");

    // D-29: OST=1 精确时间戳校正
    for clip in &mut state.script {
        if clip.ost == OstType::OriginalSound {
            if let Some((precise_start, precise_end)) =
                find_precise_range(&clip.timestamp, &state.subtitle_segments)
            {
                let start_str = secs_to_srt_time(precise_start);
                let end_str = secs_to_srt_time(precise_end);
                clip.timestamp = format!("{}-{}", start_str, end_str);
            }
            // 找不到精确匹配时保留 LLM 原始时间戳（降级不崩溃）
        }
    }

    // 调用纪录片裁剪
    let video_clips = clip_all_videos(
        &request.video_path,
        &state.script,
        &state.tts_results,
        &state.task_dir,
    )
    .await?;
    state.video_clips = video_clips;

    // 计算 cumulative_time 和每个片段的 source_time_range/edited_time_range
    let mut cumulative_time = 0.0f64;
    for clip in &mut state.script {
        if let Some(clip_path) = state.video_clips.get(&clip._id) {
            clip.video = Some(clip_path.clone());
        }

        let clip_duration = calculate_clip_duration(clip, &state.tts_results);
        if clip_duration <= 0.0 {
            return Err(SdeError::VideoProcess(PipelineError::VideoClip {
                details: format!("片段 {} 时长计算为零", clip._id),
            }));
        }

        let start_secs = parse_time_to_secs(
            clip.timestamp.split('-').next().unwrap_or(&clip.timestamp),
        )
        .map_err(|e| SdeError::VideoProcess(PipelineError::Timestamp(e.to_string())))?;
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

    state.total_duration = cumulative_time;
    state.emit_progress("clip", 75.0, "视频裁剪完成");

    // ============================================================
    //  步骤 6: 音频字幕合并 (MergeAudio)  —  80% → 85%
    // ============================================================
    state.emit_progress("merge_audio", 80.0, "正在合并音频字幕...");

    let audio_out = state.task_dir.join("merger_audio.mp3");
    let merged_audio = merge_audio_files(
        &state.script,
        &state.tts_results,
        state.total_duration,
        &audio_out,
    )
    .await?;
    state.merged_audio_path = Some(merged_audio);

    let merged_sub = merge_subtitle_files(
        &state.script,
        &state.tts_results,
        &state.task_dir,
    )
    .await?;
    state.merged_subtitle_path = merged_sub;

    state.emit_progress("merge_audio", 85.0, "音频字幕合并完成");

    // ============================================================
    //  步骤 7: 视频拼接 (Concat)  —  90% → 95%
    // ============================================================
    state.emit_progress("concat", 90.0, "正在拼接视频...");

    let concat_list_path = state.task_dir.join("concat_list.txt");
    let mut concat_content = String::new();
    for clip in &state.script {
        if let Some(ref video_path) = clip.video {
            let path_str = video_path.to_string_lossy().replace('\\', "/");
            if path_str.contains('\n') || path_str.contains('\r') {
                return Err(SdeError::Validation {
                    details: format!("视频路径包含非法字符: {}", clip._id),
                });
            }
            let escaped = path_str.replace("'", "'\\''");
            concat_content.push_str(&format!("file '{}'\n", escaped));
        } else {
            return Err(SdeError::VideoProcess(PipelineError::Concat {
                details: format!("片段 {} 缺少视频文件", clip._id),
            }));
        }
    }
    tokio::fs::write(&concat_list_path, &concat_content)
        .await
        .map_err(|e| SdeError::Io { source: e })?;

    let combined_path = state.task_dir.join("merger.mp4");
    let concat_path_str = concat_list_path.to_string_lossy().to_string();
    let combined_path_str = combined_path.to_string_lossy().to_string();

    run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&concat_path_str)
            .arg("-c")
            .arg("copy")
            .arg("-y")
            .output(&combined_path_str);

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
    .map_err(|e: crate::error::FFmpegError| {
        SdeError::VideoProcess(PipelineError::FFmpeg { source: e })
    })?;

    state.combined_video_path = Some(combined_path);
    state.emit_progress("concat", 95.0, "视频拼接完成");

    // ============================================================
    //  步骤 8: 最终合成 (Composite)  —  95% → 100%
    //  参照 src/documentary/pipeline.rs step_composite 实现
    // ============================================================
    state.emit_progress("composite", 95.0, "正在最终合成...");

    let merged_video = state
        .combined_video_path
        .as_ref()
        .ok_or_else(|| SdeError::VideoProcess(PipelineError::Composite {
            details: "缺少拼接视频".to_string(),
        }))?;

    let final_path = state.task_dir.join("combined_sde.mp4");

    let output_str = final_path.to_string_lossy().to_string();
    let video_str_clone = merged_video.to_string_lossy().to_string();
    let audio_str_opt = state
        .merged_audio_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let srt_path_opt = state
        .merged_subtitle_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let bgm_path_opt = request.bgm_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let tts_vol = request.tts_volume;
    let bgm_vol = request.bgm_volume;
    let total_dur = state.total_duration;
    let orig_vol = request.original_volume;
    let has_original_audio = state
        .script
        .iter()
        .any(|c| c.ost != OstType::NarrationOnly);
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
        let mut style =
            format!(
                "FontSize={},PrimaryColour={},Alignment={}",
                font_size, ass_color, alignment
            );
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

    run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-i").arg(&video_str_clone);

        let mut filter_complex_parts = Vec::new();
        let mut external_audio_count = 0usize;
        let mut amix_input_count = 0usize;

        // 原始音频（音量调整）
        if has_original_audio {
            filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", orig_vol));
            amix_input_count += 1;
        }

        // TTS 音频输入
        if let Some(ref audio_str) = audio_str_opt {
            cmd.arg("-i").arg(audio_str);
            filter_complex_parts.push(format!("[{}:a]volume={:.2}[tts]", 1, tts_vol));
            external_audio_count += 1;
            amix_input_count += 1;
        }

        // BGM 输入
        if let Some(ref bgm_str) = bgm_path_opt {
            cmd.arg("-i").arg(bgm_str);
            let bgm_idx = 1 + external_audio_count;
            let fade_start = if total_dur > 3.0 {
                total_dur - 3.0
            } else {
                0.0
            };
            filter_complex_parts.push(format!(
                "[{}:a]aloop=loop=-1:size=2e+09,atrim=0:{:.3},asetpts=PTS-STARTPTS,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
                bgm_idx, total_dur, bgm_vol, fade_start
            ));
            amix_input_count += 1;
        }

        // 构建音频混音
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
            filter_complex_parts.push(format!(
                "{}amix=inputs={}:duration=longest[aout]",
                mix_inputs, amix_input_count
            ));
        }

        // 视频滤镜——字幕烧录
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

        // 应用 filter_complex
        if !filter_complex_parts.is_empty() {
            cmd.arg("-filter_complex")
                .arg(&filter_complex_parts.join(";"));
        }

        // 映射输出
        if amix_input_count > 0 {
            cmd.arg("-map")
                .arg(if has_video_filter {
                    "[vout]"
                } else {
                    "0:v"
                });
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
    .map_err(|e: crate::error::FFmpegError| {
        SdeError::VideoProcess(PipelineError::FFmpeg { source: e })
    })?;

    state.output_video_path = Some(final_path);
    state.emit_progress("composite", 100.0, "最终合成完成");

    // 返回结果
    state.output_video_path.ok_or_else(|| SdeError::Validation {
        details: "流水线完成但未生成输出视频".into(),
    })
}

// ============================================================
//  私有辅助函数
// ============================================================

/// G2 Guard: 时间戳非重叠校验
///
/// 线性扫描相邻片段时间戳，若结束时间 >= 下一段开始时间则报错。
fn detect_and_abort_overlaps(script: &[ScriptClip]) -> Result<(), SdeError> {
    for i in 0..script.len().saturating_sub(1) {
        let current = &script[i];
        let next = &script[i + 1];

        let cur_parts: Vec<&str> = current.timestamp.splitn(2, '-').collect();
        let next_parts: Vec<&str> = next.timestamp.splitn(2, '-').collect();

        if cur_parts.len() != 2 || next_parts.len() != 2 {
            continue;
        }

        let cur_end = match parse_time_to_secs(cur_parts[1]) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let next_start = match parse_time_to_secs(next_parts[0]) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if cur_end > next_start {
            return Err(SdeError::Validation {
                details: format!(
                    "片段 {} 和 {} 时间戳重叠: {} 结束于 {:.3}s, {} 开始于 {:.3}s",
                    current._id,
                    next._id,
                    current.timestamp,
                    cur_end,
                    next.timestamp,
                    next_start
                ),
            });
        }
    }
    Ok(())
}

/// G3 Guard: OST 比例警告
///
/// 计算 OST=1（OriginalSound）片段占比，低于 10% 或高于 50% 时记录警告。
fn warn_ost_ratio(script: &[ScriptClip]) {
    if script.is_empty() {
        return;
    }

    let ost1_count = script
        .iter()
        .filter(|c| c.ost == OstType::OriginalSound)
        .count();
    let total = script.len();
    let ratio = ost1_count as f64 / total as f64;

    if ratio < 0.10 {
        tracing::warn!(
            "OST=1 片段比例偏低 ({:.1}%)，可能缺少纯原声片段",
            ratio * 100.0
        );
    } else if ratio > 0.50 {
        tracing::warn!(
            "OST=1 片段比例偏高 ({:.1}%)，解说片段可能不足",
            ratio * 100.0
        );
    }
}

// ============================================================
//  独立公开 API（D-07）
// ============================================================

/// 独立分析字幕剧情（步骤 1，不生成脚本）
///
/// 从字幕文件直接分析剧情，返回剧情分析文本。
pub async fn analyze_subtitle_plot(
    subtitle_path: &Path,
    provider: Arc<dyn LlmProvider>,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<String, SdeError> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = std::env::temp_dir().join("narratoai").join(&task_id);
    tokio::fs::create_dir_all(&task_dir).await?;

    let subtitle_path_clone = subtitle_path.to_path_buf();
    let (segments, text, _encoding) = tokio::task::spawn_blocking(move || {
        parse_subtitle_file(&subtitle_path_clone)
    })
    .await
    .map_err(|e| SdeError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e),
    })??;

    let mut state = SdePipelineState::new(task_id, task_dir);
    state.subtitle_text = text;
    state.subtitle_segments = segments;

    step_analyze_plot(&mut state, provider.as_ref(), prompt_manager, temperature).await?;

    Ok(state.plot_analysis)
}

/// 独立生成解说脚本（步骤 2，需先有剧情分析）
///
/// 传入剧情分析文本和字幕文本，返回 JSON 格式的解说脚本。
pub async fn generate_sde_script(
    plot_analysis: &str,
    subtitle_text: &str,
    drama_name: &str,
    provider: Arc<dyn LlmProvider>,
    prompt_manager: &PromptManager,
    temperature: f64,
) -> Result<String, SdeError> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = std::env::temp_dir().join("narratoai").join(&task_id);
    tokio::fs::create_dir_all(&task_dir).await?;

    let mut state = SdePipelineState::new(task_id, task_dir);
    state.plot_analysis = plot_analysis.to_string();
    state.subtitle_text = subtitle_text.to_string();

    step_generate_script(
        &mut state,
        provider.as_ref(),
        prompt_manager,
        temperature,
        drama_name,
        subtitle_text,
    )
    .await?;

    Ok(state.narration_raw)
}

// ============================================================
//  单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::ScriptClip;

    fn make_clip(id: i64, ts: &str, ost: OstType) -> ScriptClip {
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

    // ---- G2: detect_and_abort_overlaps ----

    #[test]
    fn test_detect_overlaps_no_overlap() {
        let clips = vec![
            make_clip(1, "00:00:00,000-00:00:05,000", OstType::NarrationOnly),
            make_clip(2, "00:00:05,000-00:00:10,000", OstType::OriginalSound),
            make_clip(3, "00:00:10,000-00:00:15,000", OstType::Mixed),
        ];
        assert!(detect_and_abort_overlaps(&clips).is_ok());
    }

    #[test]
    fn test_detect_overlaps_with_overlap() {
        let clips = vec![
            make_clip(1, "00:00:00,000-00:00:06,000", OstType::NarrationOnly),
            make_clip(2, "00:00:05,000-00:00:10,000", OstType::OriginalSound),
        ];
        let result = detect_and_abort_overlaps(&clips);
        assert!(result.is_err(), "重叠应返回错误");
    }

    #[test]
    fn test_detect_overlaps_single_clip() {
        let clips = vec![make_clip(1, "00:00:00,000-00:00:05,000", OstType::NarrationOnly)];
        assert!(detect_and_abort_overlaps(&clips).is_ok());
    }

    #[test]
    fn test_detect_overlaps_empty() {
        let clips: Vec<ScriptClip> = vec![];
        assert!(detect_and_abort_overlaps(&clips).is_ok());
    }

    // ---- G3: warn_ost_ratio ----

    #[test]
    fn test_warn_ost_ratio_normal() {
        let clips = vec![
            make_clip(1, "00:00:00-00:00:05", OstType::NarrationOnly),
            make_clip(2, "00:00:05-00:00:10", OstType::OriginalSound),
            make_clip(3, "00:00:10-00:00:15", OstType::NarrationOnly),
            make_clip(4, "00:00:15-00:00:20", OstType::Mixed),
        ];
        warn_ost_ratio(&clips);
    }

    #[test]
    fn test_warn_ost_ratio_empty() {
        let clips: Vec<ScriptClip> = vec![];
        warn_ost_ratio(&clips);
    }

    // ---- secs_to_srt_time ----

    #[test]
    fn test_sde_secs_to_srt_time() {
        assert_eq!(secs_to_srt_time(0.0), "00:00:00,000");
        assert_eq!(secs_to_srt_time(3661.5), "01:01:01,500");
        assert_eq!(secs_to_srt_time(90.0), "00:01:30,000");
    }
}
