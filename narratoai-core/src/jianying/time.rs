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
///
/// 如果解析失败或 duration 为负数或零，返回 None。
pub fn trange(start: &str, duration: &str) -> Option<Timerange> {
    let start_us = parse_seconds(start)?;
    let duration_us = parse_seconds(duration)?;
    if start_us < 0 || duration_us <= 0 {
        return None;
    }
    Some(Timerange {
        start: start_us,
        duration: duration_us,
    })
}

/// 从 f64 秒数构造 Timerange（Rust API 友好版本）
///
/// 如果任一参数为负数、零（duration）、NaN 或 Inf，返回 None。
pub fn trange_from_secs(start_secs: f64, duration_secs: f64) -> Option<Timerange> {
    if !start_secs.is_finite()
        || start_secs < 0.0
        || !duration_secs.is_finite()
        || duration_secs <= 0.0
    {
        return None;
    }
    let start_us = (start_secs * SEC as f64).round() as i64;
    let duration_us = (duration_secs * SEC as f64).round() as i64;
    if start_us < 0 || duration_us <= 0 {
        return None;
    }
    Some(Timerange {
        start: start_us,
        duration: duration_us,
    })
}

/// 解析时间字符串为微秒——支持 "1.5s" 和纯数字
///
/// 如果输入格式无效（如 "abc" 或 "5x"）或结果为负数，返回 None 而非静默转为 0。
fn parse_seconds(input: &str) -> Option<i64> {
    let input = input.trim().to_lowercase();
    let secs: f64 = if input.ends_with('s') {
        input.trim_end_matches('s').parse().ok()?
    } else {
        input.parse().ok()?
    };
    let us = (secs * SEC as f64).round() as i64;
    if us < 0 {
        return None;
    }
    Some(us)
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
        let tr = trange("1.5s", "7.441s").expect("应解析成功");
        assert_eq!(tr.start, 1_500_000);
        assert_eq!(tr.duration, 7_441_000);
    }

    #[test]
    fn test_trange_small_duration() {
        let tr = trange("0s", "0.1s").expect("应解析成功");
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
        let tr = trange_from_secs(0.0, 6.959).expect("正数 duration 应返回 Some");
        assert_eq!(tr.start, 0);
        assert_eq!(tr.duration, 6_959_000);
    }

    #[test]
    fn test_trange_from_secs_negative_duration_returns_none() {
        assert!(
            trange_from_secs(0.0, -3.0).is_none(),
            "负 duration 应返回 None"
        );
    }

    #[test]
    fn test_trange_from_secs_zero_duration_returns_none() {
        assert!(
            trange_from_secs(0.0, 0.0).is_none(),
            "零 duration 应返回 None"
        );
    }

    #[test]
    fn test_trange_pure_number_string() {
        let tr = trange("2.0", "3.5").expect("应解析成功");
        assert_eq!(tr.start, 2_000_000);
        assert_eq!(tr.duration, 3_500_000);
    }

    #[test]
    fn test_trange_zero_seconds() {
        assert!(trange("0s", "0s").is_none(), "零 duration 应返回 None");
    }

    #[test]
    fn test_sec_constant() {
        assert_eq!(SEC, 1_000_000);
    }

    #[test]
    fn test_parse_seconds_invalid_returns_none() {
        assert!(parse_seconds("abc").is_none(), "无效输入应返回 None");
        assert!(parse_seconds("5x").is_none(), "非数字后缀应返回 None");
        assert!(parse_seconds("").is_none(), "空字符串应返回 None");
    }

    #[test]
    fn test_parse_seconds_negative_returns_none() {
        assert!(parse_seconds("-5s").is_none(), "负数输入应返回 None");
        assert!(parse_seconds("-3.0").is_none(), "负数纯数字应返回 None");
    }

    #[test]
    fn test_trange_negative_duration_returns_none() {
        assert!(trange("5s", "-3s").is_none(), "负 duration 应返回 None");
    }

    #[test]
    fn test_trange_negative_start_returns_none() {
        assert!(trange("-2s", "5s").is_none(), "负 start 应返回 None");
    }

    #[test]
    fn test_trange_from_secs_negative_start_returns_none() {
        assert!(
            trange_from_secs(-1.0, 5.0).is_none(),
            "负 start 应返回 None"
        );
    }

    #[test]
    fn test_trange_from_secs_nan_returns_none() {
        assert!(
            trange_from_secs(f64::NAN, 5.0).is_none(),
            "NaN start 应返回 None"
        );
        assert!(
            trange_from_secs(0.0, f64::NAN).is_none(),
            "NaN duration 应返回 None"
        );
    }

    #[test]
    fn test_trange_from_secs_inf_returns_none() {
        assert!(
            trange_from_secs(f64::INFINITY, 5.0).is_none(),
            "Inf start 应返回 None"
        );
        assert!(
            trange_from_secs(0.0, f64::INFINITY).is_none(),
            "Inf duration 应返回 None"
        );
    }
}
