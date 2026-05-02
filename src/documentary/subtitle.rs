use std::path::Path;

use crate::tts::WordBoundary;

use super::error::PipelineError;
use super::timestamp::secs_to_srt_time;

/// 字幕片段——用于合并多个 SRT 文件时携带时间偏移
#[derive(Debug, Clone)]
pub struct SubtitleSegment {
    pub srt_content: String,
    pub offset_secs: f64,
}

/// 从词边界生成 SRT 内容
///
/// 每个词边界对应一个 SRT 块，offset_secs 为该片段在最终视频中的起始偏移。
/// 词边界的 start_offset/end_offset 单位为 100 纳秒。
pub fn generate_srt_from_word_boundaries(
    word_boundaries: &[WordBoundary],
    offset_secs: f64,
) -> String {
    let mut blocks = Vec::new();
    for (i, wb) in word_boundaries.iter().enumerate() {
        let start_secs = offset_secs + wb.start_offset as f64 / 10_000_000.0;
        let end_secs = offset_secs + wb.end_offset as f64 / 10_000_000.0;
        let start_str = secs_to_srt_time(start_secs);
        let end_str = secs_to_srt_time(end_secs);
        blocks.push(format!(
            "{}\n{} --> {}\n{}\n",
            i + 1,
            start_str,
            end_str,
            wb.text
        ));
    }
    blocks.join("\n")
}

/// 合并多个字幕片段，应用时间偏移并重新编号
///
/// 解析每个片段中的 SRT 块，将时间戳加上各自的 offset_secs，然后按顺序重新编号。
pub fn merge_srt_files(segments: &[SubtitleSegment]) -> Result<String, PipelineError> {
    let mut merged_blocks = Vec::new();
    let mut index = 1u32;

    for seg in segments {
        let blocks = parse_srt_blocks(&seg.srt_content);
        for block in blocks {
            let adjusted = apply_offset_to_block(&block, seg.offset_secs, index)?;
            merged_blocks.push(adjusted);
            index += 1;
        }
    }

    Ok(merged_blocks.join("\n"))
}

/// 将 SRT 内容写入文件
pub fn write_srt_file(content: &str, path: &Path) -> Result<(), PipelineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

/// 解析 SRT 文本为块列表: [(start_secs, end_secs, text)]
fn parse_srt_blocks(content: &str) -> Vec<(f64, f64, String)> {
    let mut blocks = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 跳过序号行
        if line.chars().all(|c| c.is_ascii_digit()) {
            // 下一行应该是时间戳行
            if let Some(ts_line) = lines.next() {
                let ts_line = ts_line.trim();
                if let Some((start, end)) = parse_srt_timestamp_line(ts_line) {
                    // 收集文本行直到空行
                    let mut text_lines = Vec::new();
                    while let Some(text_line) = lines.next() {
                        let text_line = text_line.trim();
                        if text_line.is_empty() {
                            break;
                        }
                        text_lines.push(text_line.to_string());
                    }
                    if !text_lines.is_empty() {
                        blocks.push((start, end, text_lines.join("\n")));
                    }
                }
            }
        }
    }

    blocks
}

/// 解析 SRT 时间戳行 "HH:MM:SS,mmm --> HH:MM:SS,mmm"
fn parse_srt_timestamp_line(line: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = line.split("-->").collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_srt_time(parts[0].trim())?;
    let end = parse_srt_time(parts[1].trim())?;
    Some((start, end))
}

/// 解析 SRT 时间 "HH:MM:SS,mmm" 为秒数
fn parse_srt_time(time: &str) -> Option<f64> {
    let (time_part, millis_str) = match time.find(',') {
        Some(pos) => (&time[..pos], &time[pos + 1..]),
        None => (time, ""),
    };
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    let ms: f64 = if millis_str.is_empty() {
        0.0
    } else {
        millis_str.parse::<f64>().ok()? / 1000.0
    };
    Some(h * 3600.0 + m * 60.0 + s + ms)
}

/// 对单个 SRT 块应用偏移并格式化
fn apply_offset_to_block(
    block: &(f64, f64, String),
    offset_secs: f64,
    index: u32,
) -> Result<String, PipelineError> {
    let (start, end, text) = block;
    let new_start = start + offset_secs;
    let new_end = end + offset_secs;
    Ok(format!(
        "{}\n{} --> {}\n{}\n",
        index,
        secs_to_srt_time(new_start),
        secs_to_srt_time(new_end),
        text
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_word_boundaries() -> Vec<WordBoundary> {
        vec![
            WordBoundary {
                start_offset: 0,
                end_offset: 500_000_0, // 0.5s in 100ns units
                text: "你好".to_string(),
            },
            WordBoundary {
                start_offset: 500_000_0,
                end_offset: 1_200_000_0, // 1.2s
                text: "世界".to_string(),
            },
            WordBoundary {
                start_offset: 1_200_000_0,
                end_offset: 2_000_000_0, // 2.0s
                text: "测试".to_string(),
            },
        ]
    }

    #[test]
    fn test_generate_srt_three_blocks() {
        let wbs = make_word_boundaries();
        let srt = generate_srt_from_word_boundaries(&wbs, 0.0);
        let blocks: Vec<&str> = srt.split("\n\n").filter(|b| !b.is_empty()).collect();
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn test_generate_srt_with_offset() {
        let wbs = make_word_boundaries();
        let srt = generate_srt_from_word_boundaries(&wbs, 10.0);
        // 第一块的开始时间应该是 10.0 秒
        assert!(
            srt.contains("00:00:10,000"),
            "SRT should contain offset start time: {}",
            srt
        );
    }

    #[test]
    fn test_generate_srt_format() {
        let wbs = make_word_boundaries();
        let srt = generate_srt_from_word_boundaries(&wbs, 0.0);
        // 验证 SRT 格式: 序号, 时间戳行, 文本
        assert!(srt.contains("1\n"));
        assert!(srt.contains("-->"));
        assert!(srt.contains("你好"));
    }

    #[test]
    fn test_generate_srt_empty_boundaries() {
        let srt = generate_srt_from_word_boundaries(&[], 0.0);
        assert!(srt.is_empty());
    }

    #[test]
    fn test_merge_srt_two_segments() {
        let seg1 = SubtitleSegment {
            srt_content: "1\n00:00:00,000 --> 00:00:05,000\n第一段\n\n".to_string(),
            offset_secs: 0.0,
        };
        let seg2 = SubtitleSegment {
            srt_content: "1\n00:00:00,000 --> 00:00:03,000\n第二段\n\n".to_string(),
            offset_secs: 5.0,
        };
        let merged = merge_srt_files(&[seg1, seg2]).unwrap();
        // 第二段应被偏移到 5 秒开始
        assert!(merged.contains("00:00:05,000"), "merged: {}", merged);
        // 序号应重新编号
        assert!(merged.contains("2\n00:00:05,000"), "merged: {}", merged);
    }

    #[test]
    fn test_write_srt_file_creates_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.srt");
        write_srt_file("test content", &path).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "test content");
    }
}
