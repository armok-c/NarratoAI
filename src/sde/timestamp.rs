use super::error::SdeError;
use super::types::SubtitleSegment;

/// 解析 SRT 时间戳 "HH:MM:SS,mmm" 为 f64 秒数
///
/// 也支持 "HH:MM:SS.mmm"（点号分隔符）和 "HH:MM:SS"（无毫秒）格式。
pub fn parse_srt_timestamp(input: &str) -> Result<f64, SdeError> {
    let input = input.trim();

    // 检测实际分隔符（逗号或点号），不做全局替换以避免混淆时间部分中的点号
    let (time_part, millis_str) = if let Some(pos) = input.find(',') {
        (input[..pos].to_string(), input[pos + 1..].to_string())
    } else if let Some(pos) = input.rfind('.') {
        // 仅当点号位于最后一个冒号之后时才视为亚秒分隔符
        let last_colon = input.rfind(':').unwrap_or(0);
        if pos > last_colon {
            (input[..pos].to_string(), input[pos + 1..].to_string())
        } else {
            (input.to_string(), String::new())
        }
    } else {
        (input.to_string(), String::new())
    };

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 3 {
        return Err(SdeError::ParseSubtitle {
            details: format!("时间戳格式无效，应为 HH:MM:SS,mmm: {}", input),
        });
    }

    let h: u32 = parts[0]
        .parse()
        .map_err(|_| SdeError::ParseSubtitle {
            details: format!("小时解析失败: {}", parts[0]),
        })?;
    let m: u32 = parts[1]
        .parse()
        .map_err(|_| SdeError::ParseSubtitle {
            details: format!("分钟解析失败: {}", parts[1]),
        })?;
    let s: u32 = parts[2]
        .parse()
        .map_err(|_| SdeError::ParseSubtitle {
            details: format!("秒解析失败: {}", parts[2]),
        })?;

    // 范围校验
    if h > 23 {
        return Err(SdeError::ParseSubtitle {
            details: format!("小时超出范围 (0-23): {}", h),
        });
    }
    if m > 59 {
        return Err(SdeError::ParseSubtitle {
            details: format!("分钟超出范围 (0-59): {}", m),
        });
    }
    if s > 59 {
        return Err(SdeError::ParseSubtitle {
            details: format!("秒超出范围 (0-59): {}", s),
        });
    }

    let ms: f64 = if millis_str.is_empty() {
        0.0
    } else {
        // 填充到 3 位："2" -> "200", "20" -> "200", "200" -> "200"
        let padded = format!("{:0<3}", millis_str);
        let ms_val: u32 = padded
            .parse()
            .map_err(|_| SdeError::ParseSubtitle {
                details: format!("毫秒解析失败: {}", millis_str),
            })?;
        ms_val as f64 / 1000.0
    };

    let total = h as f64 * 3600.0 + m as f64 * 60.0 + s as f64 + ms;
    if !(0.0..=86400.0).contains(&total) {
        return Err(SdeError::ParseSubtitle {
            details: format!("时间戳超出有效范围 (0-86400秒): {}", total),
        });
    }

    Ok(total)
}

/// 将 LLM 时间范围与字幕段落精确对齐
///
/// `llm_timestamp` 格式: "HH:MM:SS,mmm-HH:MM:SS,mmm"
/// 查找与 LLM 范围重叠的字幕段落（±1 秒容差）。
/// 返回匹配段落中第一个的 start_secs 和最后一个的 end_secs。
pub fn find_precise_range(
    llm_timestamp: &str,
    segments: &[SubtitleSegment],
) -> Option<(f64, f64)> {
    let parts: Vec<&str> = llm_timestamp.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }

    let llm_start = parse_srt_timestamp(parts[0]).ok()?;
    let llm_end = parse_srt_timestamp(parts[1]).ok()?;

    if llm_end < llm_start {
        return None;
    }

    // 查找与 LLM 范围重叠的字幕段落（±1 秒容差）
    let matched: Vec<&SubtitleSegment> = segments
        .iter()
        .filter(|seg| seg.start_secs <= llm_end + 1.0 && seg.end_secs >= llm_start - 1.0)
        .collect();

    if matched.is_empty() {
        return None;
    }

    let start = matched.first().expect("checked non-empty above").start_secs;
    let end = matched.last().expect("checked non-empty above").end_secs;
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segments() -> Vec<SubtitleSegment> {
        // 段间有 2 秒间隔，便于测试 +/-1 秒容差
        vec![
            SubtitleSegment {
                index: 0,
                start_secs: 0.0,
                end_secs: 5.0,
                text: "第一段".into(),
            },
            SubtitleSegment {
                index: 1,
                start_secs: 7.0,
                end_secs: 12.0,
                text: "第二段".into(),
            },
            SubtitleSegment {
                index: 2,
                start_secs: 14.0,
                end_secs: 19.0,
                text: "第三段".into(),
            },
        ]
    }

    #[test]
    fn test_parse_srt_valid() {
        let result = parse_srt_timestamp("00:00:05,200").unwrap();
        assert!((result - 5.2).abs() < 0.001);
    }

    #[test]
    fn test_parse_srt_dot_separator() {
        let result = parse_srt_timestamp("00:00:05.200").unwrap();
        assert!((result - 5.2).abs() < 0.001);
    }

    #[test]
    fn test_parse_srt_no_millis() {
        let result = parse_srt_timestamp("00:01:00").unwrap();
        assert!((result - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_srt_single_digit_millis() {
        // "05.2" means 5 seconds and 200ms, not 5.020s
        let result = parse_srt_timestamp("00:00:05.2").unwrap();
        assert!((result - 5.2).abs() < 0.001, "expected 5.2, got {}", result);
    }

    #[test]
    fn test_parse_srt_two_digit_millis() {
        // "05.50" means 5 seconds and 500ms
        let result = parse_srt_timestamp("00:00:05.50").unwrap();
        assert!((result - 5.5).abs() < 0.001, "expected 5.5, got {}", result);
    }

    #[test]
    fn test_parse_srt_invalid_format() {
        let err = parse_srt_timestamp("bad").unwrap_err();
        assert!(matches!(err, SdeError::ParseSubtitle { .. }));
    }

    #[test]
    fn test_parse_srt_invalid_range() {
        let err = parse_srt_timestamp("99:99:99,999");
        assert!(err.is_err());
    }

    #[test]
    fn test_find_precise_range_exact_match() {
        let segs = make_segments();
        // seg1 range 7.0-12.0, exact match
        let result = find_precise_range("00:00:07,000-00:00:12,000", &segs);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert!((start - 7.0).abs() < 0.001);
        assert!((end - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_find_precise_range_with_tolerance() {
        let segs = make_segments();
        // LLM time 4.5-6.5, tolerance 1s, should match seg0 (0-5) and seg1 (7-12)
        let result = find_precise_range("00:00:04,500-00:00:06,500", &segs);
        assert!(result.is_some());
        let (start, end) = result.unwrap();
        assert!((start - 0.0).abs() < 0.001);
        assert!((end - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_find_precise_range_no_match() {
        let segs = make_segments();
        // LLM 时间远超出字幕范围
        let result = find_precise_range("00:10:00,000-00:10:05,000", &segs);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_precise_range_invalid_timestamp() {
        let segs = make_segments();
        let result = find_precise_range("invalid-timestamp", &segs);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_precise_range_empty_segments() {
        let result = find_precise_range("00:00:05,000-00:00:10,000", &[]);
        assert!(result.is_none());
    }
}
