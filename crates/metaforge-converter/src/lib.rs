//! Sovereign Zero-Dependency Multi-Format Media Transcoder & Converter Engine
//!
//! Decoupled into strict One-Job-Principle (OJP) subsystems:
//! - `dsp`: Pure mathematical audio digital signal processing (resample, remix, normalize).
//! - `codecs`: Pure-Rust decoders (MP3, FLAC, OGG, AAC, MP4, MKV, WAV) & encoders (WAV, JPEG, PNG, WebP, GIF, BMP, TIFF).
//! - `pipeline`: Chainable, fluent builders (`ImagePipeline`, `AudioPipeline`).
//! - `batch`: Recursive directory batch converter with concurrency guardrails ($\le 4$ workers).
//! - `probe`: Universal media container and stream introspection.

pub mod batch;
pub mod codecs;
pub mod converters;
pub mod detect;
pub mod dsp;
pub mod pipeline;
pub mod probe;
pub mod types;

pub use batch::execute_batch_convert;
pub use codecs::{
    decode_audio_bytes, decode_audio_file, decode_audio_stream, encode_wav_bytes, encode_wav_file,
    transcode_image_bytes, DecodedAudio,
};
pub use converters::convert_media;
pub use detect::{detect_file_format, detect_media_format, detect_media_kind, sniff_magic_bytes};
pub use dsp::{
    apply_gain, downmix_to_mono, normalize_peak, remix_channels, resample_pcm_f32,
    upmix_mono_to_stereo,
};
pub use pipeline::{AudioPipeline, ImagePipeline};
pub use probe::{probe_media_bytes, probe_media_file};
pub use types::*;

use metaforge_core::error::{MetaForgeError, Result};
use std::io::{Read, Write};
use std::path::Path;

/// Convenience function to transcode image bytes in-memory.
pub fn convert_image_bytes(input: &[u8], options: &ImageConvertOptions) -> Result<Vec<u8>> {
    transcode_image_bytes(input, options)
}

/// Convenience function to transcode audio bytes in-memory.
pub fn convert_audio_wav_bytes(input: &[u8], options: &AudioConvertOptions) -> Result<Vec<u8>> {
    let pipeline = AudioPipeline::from_options(options);
    pipeline.process_bytes(input, Some("wav"))
}

/// High-level function to transcode an image file on disk.
pub fn convert_image_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    options: &ImageConvertOptions,
) -> Result<ConvertReport> {
    let pipeline = ImagePipeline::new()
        .target(options.target_format)
        .quality(options.quality.unwrap_or(85))
        .max_dimension(options.max_dimension);

    let pipeline = if let Some((w, h)) = options.resize {
        pipeline.resize(w, h)
    } else {
        pipeline
    };

    pipeline.process_file(input_path, output_path)
}

/// High-level function to transcode any audio or video file on disk to WAV.
pub fn convert_audio_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    options: &AudioConvertOptions,
) -> Result<ConvertReport> {
    let pipeline = AudioPipeline::from_options(options);
    pipeline.process_file(input_path, output_path)
}

/// Unified dispatcher to convert an arbitrary media file (image, audio, or video container).
pub fn convert_media_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    image_options: &ImageConvertOptions,
    audio_options: &AudioConvertOptions,
) -> Result<ConvertReport> {
    let unified = UnifiedConvertOptions {
        image: image_options.clone(),
        audio: audio_options.clone(),
        ..Default::default()
    };
    convert_media(input_path, output_path, &unified)
}

/// Transcode from an arbitrary `Read` stream into a `Write` stream (UNIX stdio pipe support).
pub fn convert_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    input_format_hint: Option<&str>,
    target_format: &str,
    image_options: &ImageConvertOptions,
    audio_options: &AudioConvertOptions,
) -> Result<ConvertReport> {
    let mut in_bytes = Vec::new();
    reader
        .read_to_end(&mut in_bytes)
        .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("Stream read error: {e}"))))?;

    if in_bytes.is_empty() {
        return Err(MetaForgeError::EmptyFile("stream stdin".to_string()));
    }

    let input_bytes_len = in_bytes.len();
    let target_ext = target_format.trim_start_matches('.').to_ascii_lowercase();

    let (out_bytes, details) = if is_audio_or_video_extension(&target_ext) {
        let pipeline = AudioPipeline::from_options(audio_options);
        let bytes = pipeline.process_bytes(&in_bytes, input_format_hint)?;
        (bytes, format!("Streamed audio transcode to WAV ({} bytes)", input_bytes_len))
    } else {
        let img_fmt = ImageTargetFormat::from_extension(&target_ext)
            .ok_or_else(|| MetaForgeError::UnsupportedFormat { format: target_format.to_string() })?;
        let pipeline = ImagePipeline::new()
            .target(img_fmt)
            .quality(image_options.quality.unwrap_or(85));
        let bytes = pipeline.process_bytes(&in_bytes)?;
        (bytes, format!("Streamed image transcode to {} ({} bytes)", target_format.to_ascii_uppercase(), input_bytes_len))
    };

    let output_bytes_len = out_bytes.len();
    writer
        .write_all(&out_bytes)
        .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("Stream write error: {e}"))))?;

    Ok(ConvertReport {
        input_path: "<stdin>".to_string(),
        output_path: "<stdout>".to_string(),
        input_bytes: input_bytes_len,
        output_bytes: output_bytes_len,
        input_format: input_format_hint.unwrap_or("stream").to_string(),
        output_format: target_format.to_string(),
        details,
    })
}
