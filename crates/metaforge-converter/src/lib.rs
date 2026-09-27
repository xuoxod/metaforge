pub mod audio_transcode;
pub mod image_transcode;
pub mod probe;
pub mod types;

pub use audio_transcode::{transcode_audio_wav_bytes, transcode_audio_wav_bytes as convert_audio_wav_bytes};
pub use image_transcode::{transcode_image_bytes, transcode_image_bytes as convert_image_bytes};
pub use probe::probe_media_file;
pub use types::{
    detect_target_format_from_path, AudioConvertOptions, ConvertReport, ImageConvertOptions,
    ImageTargetFormat, MediaProbe,
};

use metaforge_core::error::{MetaForgeError, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// High-level function to transcode an image file to a new target container format on disk.
pub fn convert_image_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    options: &ImageConvertOptions,
) -> Result<ConvertReport> {
    let inp = input_path.as_ref();
    let outp = output_path.as_ref();

    let mut in_file = File::open(inp)
        .map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", inp.display(), e)))?;
    let mut in_bytes = Vec::new();
    in_file.read_to_end(&mut in_bytes)?;

    let out_bytes = transcode_image_bytes(&in_bytes, options)?;

    if let Some(parent) = outp.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut out_file = File::create(outp)?;
    out_file.write_all(&out_bytes)?;

    Ok(ConvertReport {
        input_path: inp.display().to_string(),
        output_path: outp.display().to_string(),
        input_bytes: in_bytes.len(),
        output_bytes: out_bytes.len(),
        input_format: "Auto-Detected Image".to_string(),
        output_format: format!("{:?}", options.target_format),
        details: format!(
            "Transcoded to {:?} (quality: {:?}, resize: {:?})",
            options.target_format, options.quality, options.resize
        ),
    })
}

/// High-level function to transcode an audio file (WAV) to disk.
pub fn convert_audio_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    options: &AudioConvertOptions,
) -> Result<ConvertReport> {
    let inp = input_path.as_ref();
    let outp = output_path.as_ref();

    let mut in_file = File::open(inp)
        .map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", inp.display(), e)))?;
    let mut in_bytes = Vec::new();
    in_file.read_to_end(&mut in_bytes)?;

    let out_bytes = transcode_audio_wav_bytes(&in_bytes, options)?;

    if let Some(parent) = outp.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut out_file = File::create(outp)?;
    out_file.write_all(&out_bytes)?;

    Ok(ConvertReport {
        input_path: inp.display().to_string(),
        output_path: outp.display().to_string(),
        input_bytes: in_bytes.len(),
        output_bytes: out_bytes.len(),
        input_format: "WAV Audio".to_string(),
        output_format: "WAV Audio (Transcoded)".to_string(),
        details: format!(
            "Transcoded audio (sample rate: {:?}, channels: {:?})",
            options.target_sample_rate, options.target_channels
        ),
    })
}
