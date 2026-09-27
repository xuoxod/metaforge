//! Sovereign Image-to-Image Transcoding Subsystem (OJP)
//!
//! Handles cross-format image conversions (JPEG, PNG, WebP, GIF, BMP, TIFF)
//! with spatial resizing, compression quality tuning, and boundary defense.

use crate::pipeline::ImagePipeline;
use crate::types::{ConvertReport, ImageConvertOptions};
use metaforge_core::error::Result;
use std::path::Path;

/// Convert an image file on disk to another image format.
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

/// Convert image bytes in memory to target image format bytes.
pub fn convert_image_bytes(input: &[u8], options: &ImageConvertOptions) -> Result<Vec<u8>> {
    crate::codecs::transcode_image_bytes(input, options)
}
