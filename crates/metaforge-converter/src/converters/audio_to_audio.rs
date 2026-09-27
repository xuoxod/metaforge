//! Sovereign Audio-to-Audio Transcoding Subsystem (OJP)
//!
//! Handles cross-format audio conversions (WAV, MP3, FLAC, OGG, AAC)
//! with DSP resampling, channel matrixing, linear gain, and peak volume normalization.

use super::util::{execute_cmd, find_ffmpeg};
use crate::pipeline::AudioPipeline;
use crate::types::{AudioConvertOptions, AudioTargetFormat, ConvertReport};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Convert an audio file to any target audio format.
pub fn convert_audio_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    options: &AudioConvertOptions,
) -> Result<ConvertReport> {
    let inp = input_path.as_ref();
    let outp = output_path.as_ref();

    if !inp.exists() {
        return Err(MetaForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input audio file not found: {}", inp.display()),
        )));
    }

    let input_bytes = fs::metadata(inp)?.len() as usize;

    // Check if target is WAV - we can use pure-Rust pipeline
    if options.target_format == AudioTargetFormat::Wav {
        let pipeline = AudioPipeline::from_options(options);
        return pipeline.process_file(inp, outp);
    }

    // Target is MP3, FLAC, OGG, or AAC: use host media transcode engine if available
    if let Some(ffmpeg_bin) = find_ffmpeg() {
        let mut cmd = Command::new(ffmpeg_bin);
        cmd.arg("-v").arg("error");
        cmd.arg("-i").arg(inp);
        cmd.arg("-vn"); // audio only

        match options.target_format {
            AudioTargetFormat::Wav => {
                cmd.arg("-c:a").arg("pcm_s16le");
            }
            AudioTargetFormat::Mp3 => {
                cmd.arg("-c:a").arg("libmp3lame");
                cmd.arg("-b:a").arg("192k");
            }
            AudioTargetFormat::Flac => {
                cmd.arg("-c:a").arg("flac");
            }
            AudioTargetFormat::Ogg => {
                cmd.arg("-c:a").arg("libvorbis");
                cmd.arg("-q:a").arg("5");
            }
            AudioTargetFormat::Aac => {
                cmd.arg("-c:a").arg("aac");
                cmd.arg("-b:a").arg("192k");
            }
        }

        if let Some(rate) = options.target_sample_rate {
            cmd.arg("-ar").arg(rate.to_string());
        }

        if let Some(channels) = options.target_channels {
            cmd.arg("-ac").arg(channels.to_string());
        }

        let mut filters = Vec::new();
        if let Some(g) = options.gain {
            filters.push(format!("volume={:.2}", g));
        }
        if options.normalize {
            filters.push("loudnorm=I=-16:TP=-1.5:LRA=11".to_string());
        }
        if !filters.is_empty() {
            cmd.arg("-filter:a").arg(filters.join(","));
        }

        cmd.arg("-y").arg(outp);

        execute_cmd(cmd, "ffmpeg audio transcode")
            .map_err(|e| MetaForgeError::ConversionError { detail: e })?;

        let output_bytes = fs::metadata(outp)?.len() as usize;

        return Ok(ConvertReport {
            input_path: inp.to_string_lossy().to_string(),
            output_path: outp.to_string_lossy().to_string(),
            input_bytes,
            output_bytes,
            input_format: inp
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("audio")
                .to_string(),
            output_format: options.target_format.to_extension().to_string(),
            details: format!(
                "Audio transcode to {} ({:.2} KB -> {:.2} KB)",
                options.target_format.to_extension(),
                input_bytes as f64 / 1024.0,
                output_bytes as f64 / 1024.0
            ),
        });
    }

    // Host transcode engine not found; fallback to pure-Rust WAV conversion
    Err(MetaForgeError::ConversionError {
        detail: format!(
            "Encoding to '{}' requires host media encoder (ffmpeg), but it was not found on PATH. Target 'wav' is supported pure-Rust.",
            options.target_format.to_extension()
        ),
    })
}
