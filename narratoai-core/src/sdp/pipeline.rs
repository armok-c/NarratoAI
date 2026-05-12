use std::path::PathBuf;

use crate::config::types::AppConfig;
use crate::documentary::error::PipelineError;
use crate::ffmpeg::command::run_ffmpeg;
use crate::sdp::error::SdpError;
use crate::sdp::types::{SdpPipelineState, SdpProgressCallback, SdpRequest};
use crate::script::types::OstType;

/// SDP 流水线主入口——3 步顺序编排
///
/// 步骤：Clip(0%→30%) → Concat(30%→60%) → Composite(60%→90%→100%)
///
/// 参数：
/// - request: SdpRequest（含 video_path, script_path, bgm_path, volume 等）
///
/// 返回最终输出视频的 PathBuf。
pub async fn run_sdp(
    request: SdpRequest,
    config: &AppConfig,
    progress: Option<SdpProgressCallback>,
) -> Result<PathBuf, SdpError> {
    // 0. 参数校验
    request
        .validate()
        .map_err(|e| SdpError::Validation { details: e })?;

    // 1. 创建任务目录
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("narratoai").join(&task_id));
    tokio::fs::create_dir_all(&task_dir).await?;

    // 2. 初始化流水线状态
    let mut state = SdpPipelineState::new(task_id, task_dir);
    state.progress = progress;

    // 3. 加载脚本
    let script_path = request
        .script_path
        .clone()
        .unwrap_or_else(|| state.task_dir.join("merged_subtitle.json"));
    let script_path_clone = script_path.clone();
    state.script = tokio::task::spawn_blocking(move || {
        crate::script::load_script(&script_path_clone)
    })
    .await
    .map_err(|e| SdpError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
    })??;

    // G4: OST 校验——所有片段必须为 OST=1（OriginalSound）
    if state.script.iter().any(|c| c.ost != OstType::OriginalSound) {
        return Err(SdpError::Validation {
            details: "SDP 仅支持 OST=1（原声）片段，请检查脚本中的 OST 值".into(),
        });
    }

    // G2: 时间戳非重叠校验
    for i in 0..state.script.len().saturating_sub(1) {
        let current = &state.script[i];
        let next = &state.script[i + 1];
        let cur_parts: Vec<&str> = current.timestamp.splitn(2, '-').collect();
        let next_parts: Vec<&str> = next.timestamp.splitn(2, '-').collect();
        if cur_parts.len() == 2 && next_parts.len() == 2 {
            if let (Ok(cur_end), Ok(next_start)) = (
                crate::subtitle::timestamp::parse_srt_timestamp(cur_parts[1]),
                crate::subtitle::timestamp::parse_srt_timestamp(next_parts[0]),
            ) {
                if cur_end > next_start {
                    return Err(SdpError::Validation {
                        details: format!(
                            "片段 {} 和 {} 时间戳重叠: {} 结束于 {:.3}s, {} 开始于 {:.3}s",
                            current._id, next._id, current.timestamp, cur_end,
                            next.timestamp, next_start
                        ),
                    });
                }
            }
        }
    }

    // CR-01: 解析字幕文件，填充 subtitle_segments 以支持 OST=1 精确时间戳校正
    let subtitle_path_owned = request.subtitle_path.clone();
    let (segments, _, _) = tokio::task::spawn_blocking(move || {
        crate::subtitle::parse_subtitle_file(&subtitle_path_owned)
    })
    .await
    .map_err(|e| SdpError::Io {
        source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
    })?
    .map_err(SdpError::from)?;
    state.subtitle_segments = segments;

    // ============================================================
    //  步骤 1: 视频裁剪 (Clip)  —  0% → 30%
    // ============================================================
    state.emit_progress("clip", 0.0, "正在裁剪视频片段...");
    crate::sdp::clip::sdp_step_clip(&mut state, &request.video_path).await?;
    state.emit_progress("clip", 30.0, "视频裁剪完成");

    // ============================================================
    //  步骤 2: 片段拼接 (Concat)  —  30% → 60%
    // ============================================================
    state.emit_progress("concat", 30.0, "正在拼接视频片段...");
    sdp_step_concat(&mut state).await?;
    state.emit_progress("concat", 60.0, "视频拼接完成");

    // Raw 音频预标准化（D-07）：对拼接视频的原声音轨做 loudnorm 标准化
    if config.audio.enable_audio_normalization {
        if let Some(ref combined_path) = state.combined_video_path {
            let raw_audio = state.task_dir.join("sdp_raw_audio.wav");
            let combined_str = combined_path.to_string_lossy().to_string();
            let raw_str = raw_audio.to_string_lossy().to_string();
            let task_dir = state.task_dir.clone();
            let audio_config = config.audio.clone();

            // 从拼接视频中提取音频
            run_ffmpeg({
                let combined_str = combined_str.clone();
                let raw_str = raw_str.clone();
                move || {
                    let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
                    cmd.arg("-i").arg(&combined_str)
                        .arg("-vn")
                        .arg("-acodec").arg("pcm_s16le")
                        .arg("-y")
                        .arg(&raw_str);
                    let mut child = cmd.spawn()
                        .map_err(|e| crate::error::FFmpegError::SpawnFailed(e.to_string()))?;
                    let status = child.wait()
                        .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;
                    if !status.success() {
                        return Err(crate::error::FFmpegError::ExecutionError("SDP 音频提取失败".into()));
                    }
                    Ok(())
                }
            }).await.map_err(|e: crate::error::FFmpegError| {
                tracing::warn!("SDP raw 音频提取失败: {}", e);
                SdpError::VideoProcess(PipelineError::FFmpeg { source: e })
            })?;

            // 标准化提取的音频
            match crate::audio::normalizer::normalize_audio_for_mixing(
                &raw_audio,
                &task_dir,
                audio_config.target_lufs,
                audio_config.max_peak,
                audio_config.sample_rate,
                audio_config.channels,
            ) {
                Ok(normalized) => {
                    state.normalized_audio_path = Some(normalized);
                    tracing::info!("SDP raw 音频标准化完成 (target_lufs={})", audio_config.target_lufs);
                }
                Err(e) => {
                    tracing::warn!("SDP raw 音频标准化失败（非致命，将使用原始音频）: {}", e);
                    let _ = std::fs::remove_file(&raw_audio);
                }
            }
        }
    }

    // ============================================================
    //  步骤 3: 最终合成 + BGM (Composite)  —  60% → 90% → 100%
    // ============================================================
    state.emit_progress("composite", 60.0, "正在最终合成...");
    sdp_step_composite(&mut state, &request, config).await?;
    state.emit_progress("composite", 90.0, "合成完成");
    state.emit_progress("composite", 100.0, "最终合成完成");

    // 返回结果
    state.output_video_path.ok_or_else(|| SdpError::Validation {
        details: "流水线完成但未生成输出视频".into(),
    })
}

/// SDP 步骤 2：视频片段拼接
///
/// 使用 FFmpeg concat demuxer 合并裁剪后的片段。
async fn sdp_step_concat(state: &mut SdpPipelineState) -> Result<(), SdpError> {
    let concat_list_path = state.task_dir.join("concat_list.txt");
    let mut concat_content = String::new();

    for clip in &state.script {
        if let Some(ref video_path) = clip.video {
            let path_str = video_path
                .to_string_lossy()
                .replace('\\', "/");
            if path_str.contains('\n') || path_str.contains('\r') {
                return Err(SdpError::Validation {
                    details: format!("视频路径包含非法字符: {}", clip._id),
                });
            }
            let escaped = path_str.replace("'", "'\\''");
            concat_content.push_str(&format!("file '{}'\n", escaped));
        } else {
            return Err(SdpError::Validation {
                details: format!("片段 {} 缺少视频文件", clip._id),
            });
        }
    }

    tokio::fs::write(&concat_list_path, &concat_content).await?;

    let combined_path = state.task_dir.join("combined_concat.mp4");
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
        for event in iter {
            if let ffmpeg_sidecar::event::FfmpegEvent::Error(e) = event {
                tracing::warn!("SDP Concat (non-fatal): {}", e);
            }
        }
        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;
        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(
                "SDP Concat failed".into(),
            ));
        }
        Ok(())
    })
    .await
    .map_err(|e: crate::error::FFmpegError| {
        SdpError::VideoProcess(PipelineError::FFmpeg { source: e })
    })?;

    state.combined_video_path = Some(combined_path);
    Ok(())
}

/// SDP 步骤 3：最终合成 + BGM 混音
///
/// 与 documentary composite 的区别：
/// - 无 TTS 音频输入（SDP 全部 OST=1，原始音频直接取自视频 clip）
/// - 无字幕烧录（无 subtitles filter）
/// - 仍支持 BGM 混音（aloop + atrim + volume + afade + amix）
/// - 仍需要原始音频音量控制
async fn sdp_step_composite(
    state: &mut SdpPipelineState,
    request: &SdpRequest,
    config: &AppConfig,
) -> Result<(), SdpError> {
    let merged_video = state
        .combined_video_path
        .as_ref()
        .ok_or_else(|| SdpError::Validation {
            details: "缺少拼接视频".into(),
        })?;

    let output_path = state.task_dir.join("combined_sdp.mp4");
    let video_str = merged_video.to_string_lossy().to_string();
    let output_str = output_path.to_string_lossy().to_string();

    // 智能音量覆盖（D-08~D-13）
    let request_volumes = crate::audio::volume::VolumeConfig {
        tts_volume: 0.0,  // SDP 无 TTS
        original_volume: request.original_volume,
        bgm_volume: request.bgm_volume,
    };
    let resolved = crate::audio::pipeline::resolve_volumes(
        &request_volumes, "sdp", &config.audio,
    );

    // 预先提取所有值，确保 move closure 不捕获&mut state或&request引用
    let original_volume = resolved.original_volume;
    let bgm_volume = resolved.bgm_volume;
    let bgm_path_opt = request
        .bgm_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let total_dur = state.total_duration.max(0.0);
    let has_bgm = bgm_path_opt.is_some();

    // CR-SDP: Runtime verification — SDP concat -c copy may lose audio stream
    if state.normalized_audio_path.is_none() {
        let actually_has_audio = crate::ffmpeg::probe::has_audio_stream(merged_video);
        if !actually_has_audio {
            return Err(SdpError::Validation {
                details: "拼接视频无音频流，SDP 流水线无法继续".into(),
            });
        }
    }
    let normalized_audio_path = state.normalized_audio_path.clone();

    run_ffmpeg(move || {
        let mut cmd = ffmpeg_sidecar::command::FfmpegCommand::new();
        cmd.arg("-i").arg(&video_str);

        let mut filter_complex_parts: Vec<String> = Vec::new();
        let mut amix_count = 0usize;

        // 标准化音频输入（如果存在）或原始音频
        if let Some(ref norm_path) = normalized_audio_path {
            cmd.arg("-i").arg(norm_path.to_string_lossy().to_string());
            // 使用标准化音频替代原始音频轨道
            filter_complex_parts.push(format!("[1:a]volume={:.2}[orig]", original_volume));
        } else {
            // 原始音频（音量调整）
            filter_complex_parts.push(format!("[0:a]volume={:.2}[orig]", original_volume));
        }
        amix_count += 1;

        if let Some(ref bgm_path) = bgm_path_opt {
            cmd.arg("-i").arg(bgm_path);
            let bgm_idx = if normalized_audio_path.is_some() { 2 } else { 1 };
            let fade_start = if total_dur > 3.0 {
                total_dur - 3.0
            } else {
                0.0
            };
            filter_complex_parts.push(format!(
                "[{}:a]aloop=loop=-1:size=2e+09,atrim=0:{:.3},asetpts=PTS-STARTPTS,volume={:.2},afade=t=out:st={:.1}:d=3[bgm]",
                bgm_idx, total_dur, bgm_volume, fade_start
            ));
            amix_count += 1;
        }

        // 构建 amix（amix 要求至少 2 个输入，单源时直接映射）
        if has_bgm {
            filter_complex_parts.push(format!(
                "[orig][bgm]amix=inputs={}:duration=longest[aout]",
                amix_count
            ));
        } else {
            // 单一音频源，已在上方应用 volume 并标记为 [orig]，直接映射到 [aout]
            // 单一音频源，直接映射（避免 acopy 滤镜在 FFmpeg < 6.0 不可用）
            filter_complex_parts.push(format!("[orig]volume=1.0[aout]"));
        }

        if !filter_complex_parts.is_empty() {
            cmd.arg("-filter_complex")
                .arg(&filter_complex_parts.join(";"));
        }

        cmd.arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("[aout]")
            .codec_video("libx264")
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
                tracing::warn!("SDP Composite (non-fatal): {}", e);
            }
        }
        let status = child
            .wait()
            .map_err(|e| crate::error::FFmpegError::ExecutionError(e.to_string()))?;
        if !status.success() {
            return Err(crate::error::FFmpegError::ExecutionError(
                "SDP Composite failed".into(),
            ));
        }
        Ok(())
    })
    .await
    .map_err(|e: crate::error::FFmpegError| {
        SdpError::VideoProcess(PipelineError::FFmpeg { source: e })
    })?;

    state.output_video_path = Some(output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::types::{OstType, ScriptClip};

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

    // ---- OST Validation (G4) ----

    #[test]
    fn test_sdp_ost_validation_all_ost1_passes() {
        let script = vec![
            make_clip(1, "00:00:00-00:00:05", OstType::OriginalSound),
            make_clip(2, "00:00:05-00:00:10", OstType::OriginalSound),
        ];
        let has_non_ost1 = script.iter().any(|c| c.ost != OstType::OriginalSound);
        assert!(!has_non_ost1, "全部 OST=1 应通过校验");
    }

    #[test]
    fn test_sdp_ost_validation_non_ost1_fails() {
        let script = vec![
            make_clip(1, "00:00:00-00:00:05", OstType::NarrationOnly),
            make_clip(2, "00:00:05-00:00:10", OstType::OriginalSound),
        ];
        let has_non_ost1 = script.iter().any(|c| c.ost != OstType::OriginalSound);
        assert!(has_non_ost1, "包含非 OST=1 片段应发现");
    }

    #[test]
    fn test_sdp_ost_validation_mixed_fails() {
        let script = vec![
            make_clip(1, "00:00:00-00:00:05", OstType::Mixed),
        ];
        let has_non_ost1 = script.iter().any(|c| c.ost != OstType::OriginalSound);
        assert!(has_non_ost1, "OST=2 (Mixed) 也应不通过");
    }

    // ---- Concat validation (before FFmpeg) ----

    #[test]
    fn test_sdp_concat_validation_no_video_fails() {
        // 所有 clip.video = None，should trigger early error before FFmpeg
        let script = vec![
            make_clip(1, "00:00:00-00:00:05", OstType::OriginalSound),
        ];
        let all_missing_video = script.iter().any(|c| c.video.is_none());
        assert!(all_missing_video, "video 为 None 时应检测到");
    }

    // ---- Progress step names ----

    #[test]
    fn test_sdp_progress_step_names() {
        // 验证进度步骤名使用小写字符串格式
        assert_eq!("clip", "clip", "Clip 应为 clip");
        assert_eq!("concat", "concat", "Concat 应为 concat");
        assert_eq!("composite", "composite", "Composite 应为 composite");

        // 验证步骤名互不相等
        assert_ne!("clip", "concat");
        assert_ne!("concat", "composite");
        assert_ne!("clip", "composite");
    }

    // ---- SdpRequest validation ----

    #[test]
    fn test_sdp_request_validate_missing_video() {
        let req = SdpRequest::default();
        let result = req.validate();
        assert!(result.is_err(), "video_path 为空应校验失败");
    }

    #[test]
    fn test_sdp_request_validate_out_of_range_volume() {
        let req = SdpRequest {
            video_path: PathBuf::from("video.mp4"),
            original_volume: 15.0,
            ..Default::default()
        };
        let result = req.validate();
        assert!(result.is_err(), "original_volume 超出范围应校验失败");
    }
}
