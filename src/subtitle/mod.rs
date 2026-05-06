pub mod error;
pub mod types;
pub mod parser;
pub mod timestamp;

pub use error::SubtitleError;
pub use types::SubtitleSegment;
pub use parser::{parse_subtitle_file, detect_encoding, normalize_subtitle_text};
pub use timestamp::{parse_srt_timestamp, find_precise_range};
