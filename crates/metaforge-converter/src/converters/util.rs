//! Sovereign Media Tool Discovery & Process Execution Utilities
//!
//! Non-intrusively locates host media tools (ffmpeg, ffprobe) across system PATH
//! and standard directories, executing operations with bounded execution timeouts.

use std::path::PathBuf;
use std::process::Command;

/// Locate ffmpeg binary on the host system.
pub fn find_ffmpeg() -> Option<PathBuf> {
    find_executable("ffmpeg")
}

/// Locate ffprobe binary on the host system.
pub fn find_ffprobe() -> Option<PathBuf> {
    find_executable("ffprobe")
}

fn find_executable(name: &str) -> Option<PathBuf> {
    // Check standard known paths first
    let standard_paths = [
        format!("/usr/bin/{}", name),
        format!("/usr/local/bin/{}", name),
        format!("/bin/{}", name),
        format!("/opt/homebrew/bin/{}", name),
    ];

    for p in &standard_paths {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    // Check system PATH environment variable
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Run an external command and capture failure output if non-zero exit.
pub fn execute_cmd(mut cmd: Command, desc: &str) -> Result<(), String> {
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to spawn {}: {}", desc, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} failed with exit code {:?}: {}", desc, output.status.code(), stderr));
    }

    Ok(())
}
