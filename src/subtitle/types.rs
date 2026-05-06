/// 字幕段落——解析后的 SRT/ASS 条目
#[derive(Debug, Clone)]
pub struct SubtitleSegment {
    pub index: usize,
    pub start_secs: f64,
    pub end_secs: f64,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_segment_creation() {
        let seg = SubtitleSegment {
            index: 1,
            start_secs: 0.0,
            end_secs: 5.5,
            text: "你好".into(),
        };
        assert_eq!(seg.index, 1);
        assert!((seg.start_secs - 0.0).abs() < f64::EPSILON);
        assert!((seg.end_secs - 5.5).abs() < f64::EPSILON);
        assert_eq!(seg.text, "你好");
    }

    #[test]
    fn test_subtitle_segment_default_values() {
        let seg = SubtitleSegment {
            index: 0,
            start_secs: 0.0,
            end_secs: 0.0,
            text: String::new(),
        };
        assert_eq!(seg.index, 0);
        assert!(seg.text.is_empty());
    }
}
