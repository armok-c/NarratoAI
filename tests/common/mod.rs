use std::path::Path;
use std::process::Command;

/// Check if FFmpeg is available on the system
pub fn ffmpeg_available() -> bool {
    let bin = ffmpeg_sidecar::paths::ffmpeg_path();
    Command::new(&bin)
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
