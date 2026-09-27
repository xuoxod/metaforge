//! Sovereign Video-to-Image / Animated Graphic Subsystem (OJP)
//!
//! Converts video segments into animated GIFs or extracts representative
//! poster frame image thumbnails (JPEG, PNG, WebP) with spatial scaling.

use super::util::{execute_cmd, find_ffmpeg};
use crate::types::{ConvertReport, ImageConvertOptions, ImageTargetFormat};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Convert a video segment to an animated image (e.g. GIF) or extract a poster frame.
pub fn convert_video_to_image<P1: AsRef<Path>, P2: AsRef<Path>>(
    video_path: P1,
    image_output_path: P2,
    options: &ImageConvertOptions,
) -> Result<ConvertReport> {
    let inp = video_path.as_ref();
    let outp = image_output_path.as_ref();

    if !inp.exists() {
        return Err(MetaForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input video file not found: {}", inp.display()),
        )));
    }

    let input_bytes = fs::metadata(inp)?.len() as usize;

    let ffmpeg_bin = find_ffmpeg().ok_or_else(|| MetaForgeError::ConversionError {
        detail: "Host media engine (ffmpeg) is required for video-to-image conversion".to_string(),
    })?;

    let mut cmd = Command::new(ffmpeg_bin);
    cmd.arg("-v").arg("error");
    cmd.arg("-i").arg(inp);

    match options.target_format {
        ImageTargetFormat::Gif => {
            // Animated GIF generation with palette optimization
            let mut filter = "fps=10".to_string();
            if let Some((w, h)) = options.resize {
                filter = format!("fps=10,scale={}:{}:flags=lanczos", w, h);
            }
            cmd.arg("-vf").arg(format!("{},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", filter));
            cmd.arg("-t").arg("5"); // default 5-second sample
        }
        _ => {
            // Still thumbnail extraction (frame at 1.0s)
            cmd.arg("-ss").arg("00:00:01.000");
            cmd.arg("-vframes").arg("1");
            if let Some((w, h)) = options.resize {
                cmd.arg("-vf").arg(format!("scale={}:{}", w, h));
            }
        }
    }

    cmd.arg("-y").arg(outp);

    execute_cmd(cmd, "video to image conversion")
        .map_err(|e| MetaForgeError::ConversionError { detail: e })?;

    let output_bytes = fs::metadata(outp)?.len() as usize;

    Ok(ConvertReport {
        input_path: inp.to_string_lossy().to_string(),
        output_path: outp.to_string_lossy().to_string(),
        input_bytes,
        output_bytes,
        input_format: inp
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("video")
            .to_string(),
        output_format: options.target_format.to_extension().to_string(),
        details: format!(
            "Converted video to {} ({:.2} MB -> {:.2} KB)",
            options.target_format.to_extension(),
            input_bytes as f64 / 1_048_576.0,
            output_bytes as f64 / 1024.0
        ),
    })
}
