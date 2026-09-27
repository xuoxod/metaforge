//! Sovereign Video-to-Video Transcoding Subsystem (OJP)
//!
//! Handles cross-format video container remuxing and transcoding across
//! MP4, MKV, WebM, MOV, and AVI with fast-path stream copy and spatial scaling.

use super::util::{execute_cmd, find_ffmpeg};
use crate::types::{ConvertReport, VideoConvertOptions, VideoTargetFormat};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Convert a video file to another video container format.
pub fn convert_video_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    options: &VideoConvertOptions,
) -> Result<ConvertReport> {
    let inp = input_path.as_ref();
    let outp = output_path.as_ref();

    if !inp.exists() {
        return Err(MetaForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input video file not found: {}", inp.display()),
        )));
    }

    let input_bytes = fs::metadata(inp)?.len() as usize;

    let ffmpeg_bin = find_ffmpeg().ok_or_else(|| MetaForgeError::ConversionError {
        detail: "Host media engine (ffmpeg) is required for video-to-video conversion".to_string(),
    })?;

    let mut cmd = Command::new(ffmpeg_bin);
    cmd.arg("-v").arg("error");
    cmd.arg("-i").arg(inp);

    if options.copy_streams {
        cmd.arg("-c").arg("copy");
    } else {
        // Configure video codec per target format
        match options.target_format {
            VideoTargetFormat::Mp4 | VideoTargetFormat::Mov => {
                cmd.arg("-c:v").arg("libx264");
                cmd.arg("-preset").arg("fast");
                cmd.arg("-crf").arg(options.crf.unwrap_or(23).to_string());
                cmd.arg("-c:a").arg("aac");
                cmd.arg("-b:a").arg("160k");
                if options.faststart {
                    cmd.arg("-movflags").arg("+faststart");
                }
            }
            VideoTargetFormat::Webm => {
                cmd.arg("-c:v").arg("libvpx-vp9");
                cmd.arg("-crf").arg(options.crf.unwrap_or(30).to_string());
                cmd.arg("-b:v").arg("0");
                cmd.arg("-c:a").arg("libopus");
                cmd.arg("-b:a").arg("128k");
            }
            VideoTargetFormat::Mkv => {
                cmd.arg("-c:v").arg("libx264");
                cmd.arg("-preset").arg("fast");
                cmd.arg("-crf").arg(options.crf.unwrap_or(23).to_string());
                cmd.arg("-c:a").arg("aac");
            }
            VideoTargetFormat::Avi => {
                cmd.arg("-c:v").arg("mpeg4");
                cmd.arg("-q:v").arg("4");
                cmd.arg("-c:a").arg("mp3");
            }
        }

        // Apply video resize if specified
        if let Some((w, h)) = options.resize {
            cmd.arg("-vf").arg(format!("scale={}:{}", w, h));
        }

        // Apply frame rate if specified
        if let Some(fps) = options.fps {
            cmd.arg("-r").arg(fps.to_string());
        }
    }

    cmd.arg("-y").arg(outp);

    execute_cmd(cmd, "ffmpeg video transcode")
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
            "Video conversion to {} ({:.2} MB -> {:.2} MB)",
            options.target_format.to_extension(),
            input_bytes as f64 / 1_048_576.0,
            output_bytes as f64 / 1_048_576.0
        ),
    })
}
