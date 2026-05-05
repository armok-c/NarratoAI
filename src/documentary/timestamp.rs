use crate::script::types::OstType;

use super::error::PipelineError;

/// 解析时间戳范围 "HH:MM:SS-HH:MM:SS" 或 "HH:MM:SS,sss-HH:MM:SS,sss"
pub fn parse_timestamp_range(input: &str) -> Result<(f64, f64), PipelineError> {
    let parts: Vec<&str> = input.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(PipelineError::Timestamp(format!(
            "时间戳范围格式无效，应为 HH:MM:SS-HH:MM:SS: {}",
            input
        )));
    }
    let start = parse_time_to_secs(parts[0])?;
    let end = parse_time_to_secs(parts[1])?;
    if end < start {
        return Err(PipelineError::Timestamp(format!(
            "结束时间 ({}) 早于开始时间 ({}): {}",
            parts[1], parts[0], input
        )));
    }
    Ok((start, end))
}

/// 解析单个时间戳 "HH:MM:SS" 或 "HH:MM:SS,sss" 为秒数
pub fn parse_time_to_secs(input: &str) -> Result<f64, PipelineError> {
    let input = input.trim();
    let (time_part, millis) = match input.find(',') {
        Some(pos) => (&input[..pos], &input[pos + 1..]),
        None => (input, ""),
    };

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 3 {
        return Err(PipelineError::Timestamp(format!(
            "时间戳格式无效，应为 HH:MM:SS 或 HH:MM:SS,sss: {}",
            input
        )));
    }

    let h: f64 = parts[0]
        .parse()
        .map_err(|_| PipelineError::Timestamp(format!("小时解析失败: {}", parts[0])))?;
    let m: f64 = parts[1]
        .parse()
        .map_err(|_| PipelineError::Timestamp(format!("分钟解析失败: {}", parts[1])))?;
    let s: f64 = parts[2]
        .parse()
        .map_err(|_| PipelineError::Timestamp(format!("秒解析失败: {}", parts[2])))?;

    if !(0.0..=23.0).contains(&h) {
        return Err(PipelineError::Timestamp(format!("小时超出范围 (0-23): {}", h)));
    }
    if !(0.0..=59.0).contains(&m) {
        return Err(PipelineError::Timestamp(format!("分钟超出范围 (0-59): {}", m)));
    }
    if !(0.0..=59.0).contains(&s) {
        return Err(PipelineError::Timestamp(format!("秒超出范围 (0-59): {}", s)));
    }

    let ms: f64 = if millis.is_empty() {
        0.0
    } else {
        let ms_val: f64 = millis
            .parse()
            .map_err(|_| PipelineError::Timestamp(format!("毫秒解析失败: {}", millis)))?;
        ms_val / 1000.0
    };

    let total = h * 3600.0 + m * 60.0 + s + ms;
    if total < 0.0 || total > 86400.0 {
        return Err(PipelineError::Timestamp(format!(
            "时间戳超出有效范围 (0-86400秒): {}",
            total
        )));
    }

    Ok(total)
}

/// 将秒数转换为 SRT 时间格式 "HH:MM:SS,mmm"
pub fn secs_to_srt_time(secs: f64) -> String {
    if secs < 0.0 {
        return "00:00:00,000".to_string();
    }
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

/// 将秒数转换为 FFmpeg 时间格式 "HH:MM:SS.mmm"
pub fn secs_to_ffmpeg_time(secs: f64) -> String {
    if secs < 0.0 {
        return "00:00:00.000".to_string();
    }
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

/// 根据起始时间和时长计算结束时间
pub fn calculate_end_time(start: &str, duration: f64) -> Result<String, PipelineError> {
    let start_secs = parse_time_to_secs(start)?;
    let end_secs = start_secs + duration;
    Ok(secs_to_ffmpeg_time(end_secs))
}

/// 根据 OST 类型和起止时间生成片段文件名
pub fn format_clip_filename(ost: OstType, start: &str, end: &str) -> String {
    let ost_num = ost as u8;
    let start_fmt = start.replace([':', ','], "-");
    let end_fmt = end.replace([':', ','], "-");
    format!("ost{}_vid_{}@{}.mp4", ost_num, start_fmt, end_fmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_to_secs_hhmmss() {
        assert_eq!(parse_time_to_secs("00:01:30").unwrap(), 90.0);
    }

    #[test]
    fn test_parse_time_to_secs_with_millis() {
        assert_eq!(parse_time_to_secs("00:01:30,500").unwrap(), 90.5);
    }

    #[test]
    fn test_parse_time_to_secs_zero() {
        assert_eq!(parse_time_to_secs("00:00:00").unwrap(), 0.0);
    }

    #[test]
    fn test_parse_time_to_secs_large() {
        assert_eq!(parse_time_to_secs("01:00:00").unwrap(), 3600.0);
    }

    #[test]
    fn test_parse_time_to_secs_invalid_format() {
        assert!(parse_time_to_secs("invalid").is_err());
    }

    #[test]
    fn test_parse_time_to_secs_too_few_parts() {
        assert!(parse_time_to_secs("01:30").is_err());
    }

    #[test]
    fn test_parse_timestamp_range_basic() {
        let (start, end) = parse_timestamp_range("00:00:05-00:00:15").unwrap();
        assert_eq!(start, 5.0);
        assert_eq!(end, 15.0);
    }

    #[test]
    fn test_parse_timestamp_range_with_millis() {
        let (start, end) = parse_timestamp_range("00:00:05,500-00:00:15,750").unwrap();
        assert!((start - 5.5).abs() < f64::EPSILON);
        assert!((end - 15.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_timestamp_range_end_before_start() {
        assert!(parse_timestamp_range("00:00:15-00:00:05").is_err());
    }

    #[test]
    fn test_parse_timestamp_range_no_dash() {
        assert!(parse_timestamp_range("00:00:05").is_err());
    }

    #[test]
    fn test_secs_to_srt_time() {
        assert_eq!(secs_to_srt_time(90.0), "00:01:30,000");
    }

    #[test]
    fn test_secs_to_srt_time_with_millis() {
        assert_eq!(secs_to_srt_time(90.5), "00:01:30,500");
    }

    #[test]
    fn test_secs_to_srt_time_zero() {
        assert_eq!(secs_to_srt_time(0.0), "00:00:00,000");
    }

    #[test]
    fn test_secs_to_ffmpeg_time() {
        assert_eq!(secs_to_ffmpeg_time(90.0), "00:01:30.000");
    }

    #[test]
    fn test_secs_to_ffmpeg_time_with_millis() {
        assert_eq!(secs_to_ffmpeg_time(90.5), "00:01:30.500");
    }

    #[test]
    fn test_calculate_end_time() {
        let end = calculate_end_time("00:00:05", 10.0).unwrap();
        assert_eq!(end, "00:00:15.000");
    }

    #[test]
    fn test_calculate_end_time_with_millis() {
        let end = calculate_end_time("00:00:05,500", 10.25).unwrap();
        assert_eq!(end, "00:00:15.750");
    }

    #[test]
    fn test_format_clip_filename() {
        let name = format_clip_filename(OstType::NarrationOnly, "00:00:00", "00:00:20.250");
        assert_eq!(name, "ost0_vid_00-00-00@00-00-20.250.mp4");
    }

    #[test]
    fn test_format_clip_filename_ost1() {
        let name = format_clip_filename(OstType::OriginalSound, "00:00:20", "00:00:40");
        assert_eq!(name, "ost1_vid_00-00-20@00-00-40.mp4");
    }

    #[test]
    fn test_format_clip_filename_ost2() {
        let name = format_clip_filename(OstType::Mixed, "00:00:00,500", "00:00:20,250");
        assert_eq!(name, "ost2_vid_00-00-00-500@00-00-20-250.mp4");
    }

    #[test]
    fn test_roundtrip_secs_conversion() {
        let original = 3661.5; // 01:01:01,500
        let srt = secs_to_srt_time(original);
        let parsed = parse_time_to_secs(&srt).unwrap();
        assert!((parsed - original).abs() < 0.01);
    }
}
