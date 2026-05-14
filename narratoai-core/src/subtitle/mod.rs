pub mod error;
pub mod parser;
pub mod timestamp;
pub mod types;

pub use error::SubtitleError;
pub use parser::{detect_encoding, normalize_subtitle_text, parse_subtitle_file};
pub use timestamp::{find_precise_range, parse_srt_timestamp};
pub use types::SubtitleSegment;
