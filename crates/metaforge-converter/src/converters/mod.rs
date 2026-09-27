//! Sovereign Media Converters Architecture (OJP Decoupled)
//!
//! Organizes media conversion into dedicated, single-responsibility subsystems:
//! - `image_to_image`: Image cross-format transcoding & resizing.
//! - `audio_to_audio`: Audio cross-format transcoding & DSP.
//! - `video_to_video`: Video container transcoding & stream remuxing.
//! - `video_to_audio`: Audio extraction from video containers.
//! - `video_to_image`: Video to animated graphic (GIF) or thumbnail poster.

pub mod audio_to_audio;
pub mod image_to_image;
pub mod util;
pub mod video_to_audio;
pub mod video_to_image;
pub mod video_to_video;

pub use audio_to_audio::convert_audio_file;
pub use image_to_image::{convert_image_bytes, convert_image_file};
pub use video_to_audio::extract_audio_from_video;
pub use video_to_image::convert_video_to_image;
pub use video_to_video::convert_video_file;

use crate::detect::detect_file_format;
use crate::types::{ConvertReport, MediaFormat, MediaKind, UnifiedConvertOptions};
use metaforge_core::error::{MetaForgeError, Result};
use std::path::Path;

/// Route conversion of any media file to the designated target format automatically.
pub fn convert_media<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    options: &UnifiedConvertOptions,
) -> Result<ConvertReport> {
    let inp = input_path.as_ref();
    let outp = output_path.as_ref();

    if !inp.exists() {
        return Err(MetaForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input media file not found: {}", inp.display()),
        )));
    }

    // Detect source format & kind
    let source_format = detect_file_format(inp).ok_or_else(|| MetaForgeError::ConversionError {
        detail: format!(
            "Could not determine source media format for '{}'",
            inp.display()
        ),
    })?;

    // Detect target format & kind from destination extension
    let target_ext = outp
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| MetaForgeError::ConversionError {
            detail: format!(
                "Output path '{}' must have a valid file extension to determine target format",
                outp.display()
            ),
        })?;

    let target_format = MediaFormat::from_extension(target_ext).ok_or_else(|| {
        MetaForgeError::ConversionError {
            detail: format!("Unsupported target extension: '.{}'", target_ext),
        }
    })?;

    let source_kind = source_format.kind();
    let target_kind = target_format.kind();

    match (source_kind, target_kind) {
        // Image to Image
        (MediaKind::Image, MediaKind::Image) => {
            if let MediaFormat::Image(fmt) = target_format {
                let mut opts = options.image.clone();
                opts.target_format = fmt;
                convert_image_file(inp, outp, &opts)
            } else {
                unreachable!()
            }
        }

        // Audio to Audio
        (MediaKind::Audio, MediaKind::Audio) => {
            if let MediaFormat::Audio(fmt) = target_format {
                let mut opts = options.audio.clone();
                opts.target_format = fmt;
                convert_audio_file(inp, outp, &opts)
            } else {
                unreachable!()
            }
        }

        // Video to Video
        (MediaKind::Video, MediaKind::Video) => {
            if let MediaFormat::Video(fmt) = target_format {
                let mut opts = options.video.clone();
                opts.target_format = fmt;
                convert_video_file(inp, outp, &opts)
            } else {
                unreachable!()
            }
        }

        // Video to Audio (Audio Extraction)
        (MediaKind::Video, MediaKind::Audio) => {
            if let MediaFormat::Audio(fmt) = target_format {
                let mut opts = options.audio.clone();
                opts.target_format = fmt;
                extract_audio_from_video(inp, outp, &opts)
            } else {
                unreachable!()
            }
        }

        // Video to Image (Animated GIF or Thumbnail Poster)
        (MediaKind::Video, MediaKind::Image) => {
            if let MediaFormat::Image(fmt) = target_format {
                let mut opts = options.image.clone();
                opts.target_format = fmt;
                convert_video_to_image(inp, outp, &opts)
            } else {
                unreachable!()
            }
        }

        // Image to Video / Audio (Unsupported cross-kind)
        (MediaKind::Image, MediaKind::Video) => Err(MetaForgeError::ConversionError {
            detail: format!(
                "Converting static image '{}' to video is not supported directly. Target format: {}",
                inp.display(),
                target_ext
            ),
        }),
        (MediaKind::Image, MediaKind::Audio) => Err(MetaForgeError::ConversionError {
            detail: format!(
                "Cannot extract audio from static image '{}'",
                inp.display()
            ),
        }),
        (MediaKind::Audio, MediaKind::Image) => Err(MetaForgeError::ConversionError {
            detail: format!(
                "Cannot convert audio file '{}' directly to image",
                inp.display()
            ),
        }),
        (MediaKind::Audio, MediaKind::Video) => Err(MetaForgeError::ConversionError {
            detail: format!(
                "Converting audio file '{}' to video requires a video stream input",
                inp.display()
            ),
        }),
    }
}
