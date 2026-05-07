use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use encoding_rs;
use regex::Regex;

use crate::subtitle::error::SubtitleError;
use crate::subtitle::timestamp::parse_srt_timestamp;
use crate::subtitle::types::SubtitleSegment;

/// SRT 时间戳正则模式（HH:MM:SS,mmm 或 HH:MM:SS.mmm）
static SRT_TIMECODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{2}:\d{2}:\d{2}[,.]\d{3}").unwrap());

/// 毫秒分隔符标准化正则（HH:MM:SS.mmm → HH:MM:SS,mmm）
static MILLIS_SEP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d{2}:\d{2}:\d{2})[.](\d{3})").unwrap());

/// 判定解码内容"有意义"的最少非空白字符数。
/// 保守设定以排除随机字节噪音。
const MIN_MEANINGFUL_CONTENT_CHARS: usize = 20;

/// 检查文本中是否包含 SRT 时间戳模式（HH:MM:SS,mmm 或 HH:MM:SS.mmm）
fn has_srt_timecodes(text: &str) -> bool {
    SRT_TIMECODE_RE.is_match(text)
}

/// 检查文本是否包含有意义的内容（至少有一些可打印字符）
fn has_meaningful_content(text: &str) -> bool {
    let non_whitespace: usize = text.chars().filter(|c| !c.is_whitespace()).count();
    non_whitespace > MIN_MEANINGFUL_CONTENT_CHARS
}

/// 将 u8 切片转换为 UTF-16LE 字符串
fn decode_utf16le(bytes: &[u8]) -> Result<String, SubtitleError> {
    let aligned = if bytes.len() % 2 == 0 {
        bytes
    } else {
        &bytes[..bytes.len() - 1]
    };
    if aligned.is_empty() {
        return Err(SubtitleError::ParseSubtitle {
            details: "UTF-16LE 数据为空".into(),
        });
    }
    let u16_data: Vec<u16> = aligned
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&u16_data).map_err(|e| SubtitleError::ParseSubtitle {
        details: format!("UTF-16LE 解码失败: {}", e),
    })
}

/// 将 u8 切片转换为 UTF-16BE 字符串
fn decode_utf16be(bytes: &[u8]) -> Result<String, SubtitleError> {
    let aligned = if bytes.len() % 2 == 0 {
        bytes
    } else {
        &bytes[..bytes.len() - 1]
    };
    if aligned.is_empty() {
        return Err(SubtitleError::ParseSubtitle {
            details: "UTF-16BE 数据为空".into(),
        });
    }
    let u16_data: Vec<u16> = aligned
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&u16_data).map_err(|e| SubtitleError::ParseSubtitle {
        details: format!("UTF-16BE 解码失败: {}", e),
    })
}

/// 检查数据是否为 UTF-16-LE（每两个字节中有一个 NUL）
fn looks_like_utf16le(data: &[u8]) -> bool {
    if data.len() < 4 || data.len() % 2 != 0 {
        return false;
    }
    let check_len = data.len().min(64);
    let chunks = data[..check_len - (check_len % 2)].chunks_exact(2);
    let mut nul_count = 0usize;
    let total = chunks.len().min(16);
    for (i, chunk) in chunks.enumerate() {
        if i >= 16 {
            break;
        }
        if chunk[1] == 0x00 && chunk[0] != 0x00 && chunk[0].is_ascii_graphic() {
            nul_count += 1;
        }
    }
    total > 0 && nul_count > total / 2
}

/// 编码检测链
///
/// 1. BOM 前缀检测（UTF-8-SIG / UTF-16-LE / UTF-16-BE）——优先于纯编码检测
/// 2. UTF-16-LE 无 BOM（通过 NUL 字节模式启发式）
/// 3. UTF-8（最快路径）
/// 4. GBK（encoding_rs）
/// 5. GB18030（encoding_rs，GB2312 超集）
///
/// 每个步骤成功后，检查文本是否包含 SRT 时间戳或有意义内容 → fast path 返回。
/// 全部失败 → SubtitleError::ParseSubtitle。
pub fn detect_encoding(data: &[u8]) -> Result<String, SubtitleError> {
    // 0. Check BOM-based detection first (BOM is unambiguous)
    if data.len() >= 2 {
        if data[0] == 0xFF && data[1] == 0xFE {
            if let Ok(text) = decode_utf16le(&data[2..]) {
                if has_srt_timecodes(&text) || has_meaningful_content(&text) {
                    return Ok("utf-16-le".to_string());
                }
            }
        } else if data[0] == 0xFE && data[1] == 0xFF {
            if let Ok(text) = decode_utf16be(&data[2..]) {
                if has_srt_timecodes(&text) || has_meaningful_content(&text) {
                    return Ok("utf-16-be".to_string());
                }
            }
        } else if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
            if let Ok(text) = std::str::from_utf8(&data[3..]) {
                if has_srt_timecodes(text) || has_meaningful_content(text) {
                    return Ok("utf-8-sig".to_string());
                }
            }
        }
    }

    // 1. Try UTF-16-LE without BOM (heuristic: NUL byte pattern)
    if data.contains(&0x00) && looks_like_utf16le(data) {
        if let Ok(text) = decode_utf16le(data) {
            if has_srt_timecodes(&text) || has_meaningful_content(&text) {
                return Ok("utf-16-le".to_string());
            }
        }
    }

    // 2. Try UTF-8 (pure, no BOM)
    if let Ok(text) = std::str::from_utf8(data) {
        if has_srt_timecodes(text) || has_meaningful_content(text) {
            return Ok("utf-8".to_string());
        }
    }

    // 3. Try GBK
    {
        let (text, _encoding_used, had_errors) = encoding_rs::GBK.decode(data);
        if !had_errors && (has_srt_timecodes(&text) || has_meaningful_content(&text)) {
            return Ok("gbk".to_string());
        }
    }

    // 4. Try GB18030 (superset of GB2312)
    {
        let (text, _encoding_used, had_errors) = encoding_rs::GB18030.decode(data);
        if !had_errors && (has_srt_timecodes(&text) || has_meaningful_content(&text)) {
            return Ok("gb18030".to_string());
        }
    }

    Err(SubtitleError::ParseSubtitle {
        details: "无法检测字幕文件编码（尝试了 BOM/UTF-8/UTF-16/GBK/GB18030）".into(),
    })
}

/// 标准化字幕文本
///
/// - 统一换行符为 \n（\r\n → \n，\r → \n）
/// - 移除 BOM（\u{feff}）
/// - 移除 NUL 字节（\x00）
/// - 标准化毫秒分隔符：点号 → 逗号
/// - 去除首尾空白
pub fn normalize_subtitle_text(text: &str) -> String {
    let result = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\u{feff}', "")
        .replace('\x00', "");

    // 标准化毫秒分隔符：点号 → 逗号（时间戳模式中的点号）
    let result = MILLIS_SEP_RE.replace_all(&result, "$1,$2").to_string();

    result.trim().to_string()
}

/// 解析 SRT 格式的字幕块
fn parse_srt_blocks(text: &str) -> Vec<SubtitleSegment> {
    let mut segments = Vec::new();
    let blocks: Vec<&str> = text.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.split('\n').collect();
        if lines.len() < 2 {
            continue;
        }

        let ts_line_idx = if lines[0].contains("-->") { 0 } else { 1 };

        let index: usize = if ts_line_idx > 0 {
            lines[0].trim().parse().unwrap_or(0)
        } else {
            0
        };

        if ts_line_idx >= lines.len() {
            continue;
        }

        let ts_line = lines[ts_line_idx].trim();
        if !ts_line.contains("-->") {
            continue;
        }

        let ts_parts: Vec<&str> = ts_line.splitn(2, "-->").collect();
        if ts_parts.len() != 2 {
            continue;
        }

        let start_secs = match parse_srt_timestamp(ts_parts[0].trim()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let end_secs = match parse_srt_timestamp(ts_parts[1].trim()) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let text_lines = &lines[ts_line_idx + 1..];
        let text = text_lines.join("\n").trim().to_string();

        segments.push(SubtitleSegment {
            index,
            start_secs,
            end_secs,
            text,
        });
    }

    segments
}

/// 拆分 ASS 的字段（逗号分隔，但文本中的逗号需要保持）
fn split_ass_fields(input: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut remaining = input.trim();

    for _ in 0..9 {
        match remaining.find(',') {
            Some(pos) => {
                result.push(remaining[..pos].trim());
                remaining = remaining[pos + 1..].trim();
            }
            None => {
                result.push(remaining);
                return result;
            }
        }
    }

    result.push(remaining);
    result
}

/// 规范化 ASS 时间戳以便 parse_srt_timestamp 解析
fn normalize_ass_timestamp(ts: &str) -> String {
    let ts = ts.trim().replace(',', ".");
    if let Some((time_part, frac)) = ts.split_once('.') {
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() == 3 {
            let h = parts[0];
            let m = parts[1];
            let s = parts[2];
            let padded_h = if h.len() == 1 {
                format!("0{}", h)
            } else {
                h.to_string()
            };
            let padded_frac = if frac.is_empty() {
                "000".to_string()
            } else {
                format!("{:0<3}", frac)
            };
            let millis = &padded_frac[..3.min(padded_frac.len())];
            format!("{}:{}:{},{}", padded_h, m, s, millis)
        } else {
            ts
        }
    } else {
        ts
    }
}

/// 解析 ASS 格式的对话行
fn parse_ass_dialogues(text: &str) -> Vec<SubtitleSegment> {
    let mut segments = Vec::new();
    let mut in_events = false;
    let mut index_counter = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("[events]") {
            in_events = true;
            continue;
        }

        if in_events {
            if trimmed.starts_with('[') {
                break;
            }

            if trimmed.to_ascii_lowercase().starts_with("dialogue:") {
                index_counter += 1;

                let content = &trimmed["dialogue:".len()..];
                let fields: Vec<&str> = split_ass_fields(content);

                if fields.len() < 10 {
                    continue;
                }

                let normalized_start = normalize_ass_timestamp(fields[1]);
                let normalized_end = normalize_ass_timestamp(fields[2]);

                let start_secs = match parse_srt_timestamp(&normalized_start) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let end_secs = match parse_srt_timestamp(&normalized_end) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let text = fields[9..].join(",").trim().to_string();

                segments.push(SubtitleSegment {
                    index: index_counter,
                    start_secs,
                    end_secs,
                    text,
                });
            }
        }
    }

    segments
}

/// 从 SRT 文本中提取纯对话内容（移除时间戳和序号行）
fn extract_text_from_srt(text: &str) -> String {
    let mut dialogue_lines = Vec::new();
    let blocks: Vec<&str> = text.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.split('\n').collect();
        if lines.len() < 2 {
            continue;
        }

        let ts_idx = if lines[0].contains("-->") { 0 } else { 1 };
        if ts_idx + 1 < lines.len() {
            let dialogue = lines[ts_idx + 1..].join("\n").trim().to_string();
            if !dialogue.is_empty() {
                dialogue_lines.push(dialogue);
            }
        }
    }

    dialogue_lines.join("\n")
}

/// 从 ASS 文本中提取纯对话内容（移除格式信息）
fn extract_text_from_ass(text: &str) -> String {
    let mut dialogue_lines = Vec::new();
    let mut in_events = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("[events]") {
            in_events = true;
            continue;
        }

        if in_events {
            if trimmed.starts_with('[') {
                break;
            }

            if trimmed.to_ascii_lowercase().starts_with("dialogue:") {
                let content = &trimmed["dialogue:".len()..];
                let fields: Vec<&str> = split_ass_fields(content);

                if fields.len() >= 10 {
                    let dialogue = fields[9..].join(",").trim().to_string();
                    if !dialogue.is_empty() {
                        dialogue_lines.push(dialogue);
                    }
                }
            }
        }
    }

    dialogue_lines.join("\n")
}

/// 根据编码名解码字节为字符串
fn decode_bytes(data: &[u8], encoding: &str) -> Result<String, SubtitleError> {
    match encoding {
        "utf-8" => String::from_utf8(data.to_vec()).map_err(|e| SubtitleError::ParseSubtitle {
            details: format!("UTF-8 解码失败: {}", e),
        }),
        "utf-8-sig" => {
            let stripped = if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF
            {
                &data[3..]
            } else {
                data
            };
            String::from_utf8(stripped.to_vec()).map_err(|e| SubtitleError::ParseSubtitle {
                details: format!("UTF-8-SIG 解码失败: {}", e),
            })
        }
        "utf-16-le" => decode_utf16le(data),
        "utf-16-be" => decode_utf16be(data),
        "gbk" => {
            let (text, _encoding_used, _had_errors) = encoding_rs::GBK.decode(data);
            Ok(text.to_string())
        }
        "gb18030" => {
            let (text, _encoding_used, _had_errors) = encoding_rs::GB18030.decode(data);
            Ok(text.to_string())
        }
        other => Err(SubtitleError::ParseSubtitle {
            details: format!("不支持的编码: {}", other),
        }),
    }
}

/// 高级函数：读取字幕文件 → 检测编码 → 解码 → 标准化 → 解析段落
///
/// 返回 (段落列表, 标准化文本, 编码名) 三元组。
///
/// 注意：此函数是 CPU 密集型的（编码检测 + 正则匹配），
/// 在调用时应使用 tokio::task::spawn_blocking 包装。
pub fn parse_subtitle_file(
    path: &Path,
) -> Result<(Vec<SubtitleSegment>, String, String), SubtitleError> {
    let data = fs::read(path)?;
    let encoding = detect_encoding(&data)?;
    let decoded = decode_bytes(&data, &encoding)?;
    let normalized = normalize_subtitle_text(&decoded);

    let segments = if normalized.contains("-->") {
        parse_srt_blocks(&normalized)
    } else if normalized.to_ascii_lowercase().contains("[events]") {
        parse_ass_dialogues(&normalized)
    } else {
        Vec::new()
    };

    Ok((segments, normalized, encoding))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_encoding_utf8() {
        let data = "00:00:01,000 --> 00:00:05,000\nHello World\n".as_bytes();
        let result = detect_encoding(data).unwrap();
        assert_eq!(result, "utf-8");
    }

    #[test]
    fn test_detect_encoding_utf8_sig() {
        let text = "00:00:01,000 --> 00:00:05,000\nHello World\n";
        let mut data = vec![0xEF, 0xBB, 0xBF];
        data.extend_from_slice(text.as_bytes());
        let result = detect_encoding(&data).unwrap();
        assert_eq!(result, "utf-8-sig");
    }

    #[test]
    fn test_detect_encoding_utf16() {
        let text = "00:00:01,000 --> 00:00:05,000\nHello\n";
        let u16_data: Vec<u16> = text.encode_utf16().collect();
        let mut data = vec![0xFF, 0xFE];
        for &code_unit in &u16_data {
            data.extend_from_slice(&code_unit.to_le_bytes());
        }
        let result = detect_encoding(&data).unwrap();
        assert_eq!(result, "utf-16-le");
    }

    #[test]
    fn test_detect_encoding_utf16_no_bom() {
        let text = "00:00:01,000 --> 00:00:05,000\nHello\n";
        let mut data = Vec::new();
        for code_unit in text.encode_utf16() {
            data.extend_from_slice(&code_unit.to_le_bytes());
        }
        let result = detect_encoding(&data);
        assert!(result.is_ok(), "UTF-16-LE without BOM: {:?}", result.err());
        assert_eq!(result.unwrap(), "utf-16-le");
    }

    #[test]
    fn test_detect_encoding_gbk() {
        let text = "00:00:01,000 --> 00:00:05,000\nHello World\n";
        let (data, _encoding_used, _had_errors) = encoding_rs::GBK.encode(text);
        let result = detect_encoding(&data);
        assert!(result.is_ok(), "GBK should be detected: {:?}", result.err());
    }

    #[test]
    fn test_detect_encoding_all_fail() {
        let data = &[0xFF, 0xFE, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04];
        let result = detect_encoding(data);
        assert!(result.is_err(), "Random bytes should fail");
        match result {
            Err(SubtitleError::ParseSubtitle { .. }) => {}
            _ => panic!("Expected SubtitleError::ParseSubtitle"),
        }
    }

    #[test]
    fn test_normalize_text_line_endings() {
        let input = "line1\r\nline2\rline3";
        let result = normalize_subtitle_text(input);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_normalize_text_bom() {
        let input = "\u{feff}Hello";
        let result = normalize_subtitle_text(input);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_normalize_text_nul() {
        let input = "He\x00llo\x00";
        let result = normalize_subtitle_text(input);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_normalize_text_millis_sep() {
        let input = "00:00:01.000 --> 00:00:05.500";
        let result = normalize_subtitle_text(input);
        assert_eq!(result, "00:00:01,000 --> 00:00:05,500");
    }

    #[test]
    fn test_normalize_text_mixed() {
        let input = "\u{feff}00:00:01.000\r\nline1\r\n00:00:02.000\x00\rline2";
        let result = normalize_subtitle_text(input);
        assert!(result.contains("00:00:01,000"), "should normalize dot to comma: {}", result);
        assert!(result.contains("00:00:02,000"), "should normalize second timestamp: {}", result);
        assert!(result.contains("line1"), "should preserve line1: {}", result);
        assert!(result.contains("line2"), "should preserve line2: {}", result);
        assert!(!result.contains('\u{feff}'), "should remove BOM: {:?}", result);
        assert!(!result.contains('\x00'), "should remove NUL: {:?}", result);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 3, "should have multiple lines: {:?}", lines);
    }

    #[test]
    fn test_parse_srt_valid() {
        let input = "1\n00:00:01,000 --> 00:00:05,000\nHello world\n\n2\n00:00:05,500 --> 00:00:10,000\nSecond line";
        let segments = parse_srt_blocks(input);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].index, 1);
        assert!((segments[0].start_secs - 1.0).abs() < 0.01);
        assert!((segments[0].end_secs - 5.0).abs() < 0.01);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[1].index, 2);
        assert_eq!(segments[1].text, "Second line");
    }

    #[test]
    fn test_parse_srt_multiline_text() {
        let input = "1\n00:00:01,000 --> 00:00:05,000\nLine one\nLine two\n\n2\n00:00:06,000 --> 00:00:10,000\nMore text";
        let segments = parse_srt_blocks(input);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Line one\nLine two");
    }

    #[test]
    fn test_parse_srt_dot_separator() {
        let input = "1\n00:00:01.000 --> 00:00:05.000\nHello";
        let normalized = normalize_subtitle_text(input);
        let segments = parse_srt_blocks(&normalized);
        assert_eq!(segments.len(), 1);
        assert!((segments[0].start_secs - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_srt_empty() {
        let segments = parse_srt_blocks("");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_parse_srt_no_index() {
        let input = "00:00:01,000 --> 00:00:05,000\nHello";
        let segments = parse_srt_blocks(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].index, 0);
    }

    #[test]
    fn test_parse_ass_valid() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello world\nDialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Second line";
        let segments = parse_ass_dialogues(input);
        assert_eq!(segments.len(), 2);
        assert!((segments[0].start_secs - 1.0).abs() < 0.01);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[1].text, "Second line");
    }

    #[test]
    fn test_parse_ass_text_with_commas() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello, world!";
        let segments = parse_ass_dialogues(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello, world!");
    }

    #[test]
    fn test_parse_ass_empty_events() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";
        let segments = parse_ass_dialogues(input);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_parse_subtitle_file_nonexistent() {
        let path = Path::new("/tmp/nonexistent_subtitle_test_file_12345.srt");
        let result = parse_subtitle_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_subtitle_file_srt_content() {
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let path = dir.path().join("test.srt");
        let content = "1\n00:00:01,000 --> 00:00:05,000\nHello World\n\n2\n00:00:06,000 --> 00:00:10,000\nSecond line";
        let mut file = std::fs::File::create(&path).expect("create file");
        file.write_all(content.as_bytes()).expect("write file");
        drop(file);

        let (segments, text, encoding) = parse_subtitle_file(&path).expect("parse");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Hello World");
        assert!(text.contains("Hello World"));
        assert_eq!(encoding, "utf-8");
    }

    #[test]
    fn test_extract_text_from_srt() {
        let input = "1\n00:00:01,000 --> 00:00:05,000\nHello world\n\n2\n00:00:06,000 --> 00:00:10,000\nSecond line";
        let text = extract_text_from_srt(input);
        assert_eq!(text, "Hello world\nSecond line");
    }

    #[test]
    fn test_extract_text_from_ass() {
        let input = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello world\nDialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Second line";
        let text = extract_text_from_ass(input);
        assert_eq!(text, "Hello world\nSecond line");
    }

    #[test]
    fn test_srt_timecode_regex() {
        assert!(has_srt_timecodes("00:00:01,000"));
        assert!(has_srt_timecodes("00:00:01.000"));
        assert!(!has_srt_timecodes("simple text"));
        assert!(!has_srt_timecodes(""));
        assert!(has_srt_timecodes("prefix 00:00:05,200 suffix"));
    }

    #[test]
    fn test_split_ass_fields_basic() {
        let input = "0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello world";
        let fields = split_ass_fields(input);
        assert_eq!(fields.len(), 10);
        assert_eq!(fields[9], "Hello world");
    }

    #[test]
    fn test_split_ass_fields_extra_commas_in_text() {
        let input = "0,0:00:01.00,0:00:05.00,Default,Name,1,2,3,Effect,Text,with,extra,commas";
        let fields = split_ass_fields(input);
        assert_eq!(fields.len(), 10);
        assert_eq!(fields[9], "Text,with,extra,commas");
    }

    #[test]
    fn test_normalize_ass_timestamp_single_hour() {
        let result = normalize_ass_timestamp("0:00:01.00");
        assert_eq!(result, "00:00:01,000");
    }

    #[test]
    fn test_normalize_ass_timestamp_double_hour() {
        let result = normalize_ass_timestamp("00:00:05.50");
        assert_eq!(result, "00:00:05,500");
    }

    #[test]
    fn test_normalize_ass_timestamp_with_comma() {
        let result = normalize_ass_timestamp("0:00:01,00");
        assert_eq!(result, "00:00:01,000");
    }

    #[test]
    fn test_parse_subtitle_file_gbk() {
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let path = dir.path().join("test_gbk.srt");
        let text = "1\n00:00:01,000 --> 00:00:05,000\nHello World\n\n2\n00:00:06,000 --> 00:00:10,000\nSecond line\n";
        let (gbk_data, _encoding_used, _had_errors) = encoding_rs::GBK.encode(text);
        let mut file = std::fs::File::create(&path).expect("create file");
        file.write_all(&gbk_data).expect("write file");
        drop(file);

        let (_segments, _text, encoding) = parse_subtitle_file(&path).expect("parse");
        assert!(encoding == "gbk" || encoding == "utf-8",
            "Expected gbk or utf-8, got {}", encoding);
    }
}
