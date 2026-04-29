use uuid::Uuid;

use super::error::JianYingError;
use super::segment::{AudioSegment, SegmentOutput, VideoSegment};
use super::types::TrackJson;

// ---------------------------------------------------------------------------
// TrackType 枚举（per RESEARCH Pattern 3）
// ---------------------------------------------------------------------------

/// 轨道类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackType {
    Video,
    Audio,
}

impl TrackType {
    /// 转为剪映 JSON 中的字符串值
    pub fn as_str(&self) -> &str {
        match self {
            TrackType::Video => "video",
            TrackType::Audio => "audio",
        }
    }
}

// ---------------------------------------------------------------------------
// Track builder（per RESEARCH Pattern 3）
// ---------------------------------------------------------------------------

/// 轨道构建器——存储同一类型的多个 segment
pub struct Track {
    track_type: TrackType,
    name: String,
    id: String,
    segments: Vec<SegmentOutput>,
}

impl Track {
    /// 创建新轨道
    pub fn new(track_type: TrackType, name: &str) -> Self {
        Self {
            track_type,
            name: name.to_string(),
            id: Uuid::new_v4().to_string().replace("-", ""),
            segments: Vec::new(),
        }
    }

    /// 添加视频片段
    pub fn add_video_segment(&mut self, seg: VideoSegment) {
        self.segments.push(SegmentOutput::Video(seg.to_json()));
    }

    /// 添加音频片段
    pub fn add_audio_segment(&mut self, seg: AudioSegment) {
        self.segments.push(SegmentOutput::Audio(seg.to_json()));
    }

    /// 导出为 TrackJson
    pub fn to_json(&self) -> Result<TrackJson, JianYingError> {
        let segments: Vec<serde_json::Value> = self
            .segments
            .iter()
            .map(|seg| match seg {
                SegmentOutput::Video(v) => serde_json::to_value(v).map_err(JianYingError::JsonSerialize),
                SegmentOutput::Audio(a) => serde_json::to_value(a).map_err(JianYingError::JsonSerialize),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TrackJson {
            attribute: 0,
            flag: 0,
            id: self.id.clone(),
            is_default_name: self.name.is_empty(),
            name: self.name.clone(),
            segments,
            type_field: self.track_type.as_str().to_string(),
        })
    }

    /// 轨道类型
    pub fn track_type(&self) -> TrackType {
        self.track_type
    }

    /// 轨道名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 计算轨道内所有 segment 的最大结束时间（微秒）
    pub fn max_end_time(&self) -> i64 {
        self.segments.iter().map(|seg| match seg {
            SegmentOutput::Video(v) => v.base.target_timerange.start + v.base.target_timerange.duration,
            SegmentOutput::Audio(a) => a.base.target_timerange.start + a.base.target_timerange.duration,
        }).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Test 8: TrackType::Video.as_str() 返回 "video"，TrackType::Audio.as_str() 返回 "audio"
    #[test]
    fn test_track_type_as_str() {
        assert_eq!(TrackType::Video.as_str(), "video");
        assert_eq!(TrackType::Audio.as_str(), "audio");
    }

    /// Test 9: Track::new 创建空轨道
    #[test]
    fn test_track_new_empty() {
        let track = Track::new(TrackType::Video, "视频轨道");
        assert_eq!(track.track_type(), TrackType::Video);
        assert_eq!(track.name(), "视频轨道");
        let json = track.to_json().expect("空轨道应序列化成功");
        assert!(json.segments.is_empty(), "新轨道应无 segment");
        assert_eq!(json.type_field, "video");
        assert_eq!(json.name, "视频轨道");
    }

    /// Test 10: Track 添加 2 个 segment 后 to_json().segments.len() == 2
    #[test]
    fn test_track_add_segments() {
        let mut track = Track::new(TrackType::Video, "视频轨道");

        let path1 = PathBuf::from("video1.mp4");
        let seg1 = VideoSegment::new(&path1, super::super::time::trange("0s", "5s").expect("应解析时间范围"), 1920, 1080)
            .expect("应成功创建");
        track.add_video_segment(seg1);

        let path2 = PathBuf::from("video2.mp4");
        let seg2 = VideoSegment::new(&path2, super::super::time::trange("5s", "3s").expect("应解析时间范围"), 1920, 1080)
            .expect("应成功创建");
        track.add_video_segment(seg2);

        let json = track.to_json().expect("Track 应序列化成功");
        assert_eq!(json.segments.len(), 2, "应有 2 个 segment");
    }

    /// Test 11: Track to_json 的 type 字段匹配 TrackType
    #[test]
    fn test_track_type_matches_json() {
        let video_track = Track::new(TrackType::Video, "视频轨道");
        assert_eq!(video_track.to_json().expect("应序列化成功").type_field, "video");

        let audio_track = Track::new(TrackType::Audio, "音频轨道");
        assert_eq!(audio_track.to_json().expect("应序列化成功").type_field, "audio");
    }
}
