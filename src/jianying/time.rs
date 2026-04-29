use serde::{Deserialize, Serialize};

/// 一秒对应的微秒数（per pyJianYingDraft time_util.py SEC 常量）
pub const SEC: i64 = 1_000_000;

/// 时间范围——start 和 duration 均为微秒（per RESEARCH Pitfall 1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timerange {
    pub start: i64,
    pub duration: i64,
}

/// 从秒字符串构造 Timerange（per pyJianYingDraft trange() 函数）
/// 接受 "1.5s" 格式，也接受纯数字字符串（视为秒）
pub fn trange(start: &str, duration: &str) -> Timerange {
    Timerange {
        start: parse_seconds(start),
        duration: parse_seconds(duration),
    }
}

/// 从 f64 秒数构造 Timerange（Rust API 友好版本）
pub fn trange_from_secs(start_secs: f64, duration_secs: f64) -> Timerange {
    Timerange {
        start: (start_secs * SEC as f64).round() as i64,
        duration: (duration_secs * SEC as f64).round() as i64,
    }
}

/// 解析时间字符串为微秒——支持 "1.5s" 和纯数字
fn parse_seconds(input: &str) -> i64 {
    let input = input.trim().to_lowercase();
    let secs: f64 = if input.ends_with('s') {
        input.trim_end_matches('s').parse().unwrap_or(0.0)
    } else {
        input.parse().unwrap_or(0.0)
    };
    (secs * SEC as f64).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timerange_serialize_microseconds() {
        let tr = Timerange {
            start: 0,
            duration: 1_000_000,
        };
        let json = serde_json::to_string(&tr).expect("应序列化成功");
        assert_eq!(
            json, r#"{"start":0,"duration":1000000}"#,
            "Timerange 应序列化为微秒整数 JSON"
        );
    }

    #[test]
    fn test_trange_from_second_strings() {
        let tr = trange("1.5s", "7.441s");
        assert_eq!(tr.start, 1_500_000);
        assert_eq!(tr.duration, 7_441_000);
    }

    #[test]
    fn test_trange_small_duration() {
        let tr = trange("0s", "0.1s");
        assert_eq!(tr.start, 0);
        assert_eq!(tr.duration, 100_000);
    }

    #[test]
    fn test_timerange_serde_roundtrip() {
        let original = Timerange {
            start: 1_500_000,
            duration: 7_441_000,
        };
        let json = serde_json::to_string(&original).expect("应序列化成功");
        let restored: Timerange = serde_json::from_str(&json).expect("应反序列化成功");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_trange_from_secs() {
        let tr = trange_from_secs(0.0, 6.959);
        assert_eq!(tr.start, 0);
        assert_eq!(tr.duration, 6_959_000);
    }

    #[test]
    fn test_trange_pure_number_string() {
        let tr = trange("2.0", "3.5");
        assert_eq!(tr.start, 2_000_000);
        assert_eq!(tr.duration, 3_500_000);
    }

    #[test]
    fn test_trange_zero_seconds() {
        let tr = trange("0s", "0s");
        assert_eq!(tr.start, 0);
        assert_eq!(tr.duration, 0);
    }

    #[test]
    fn test_sec_constant() {
        assert_eq!(SEC, 1_000_000);
    }
}
