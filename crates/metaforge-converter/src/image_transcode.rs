use crate::types::{ImageConvertOptions, ImageTargetFormat};
use image::{
    codecs::{
        bmp::BmpEncoder, gif::GifEncoder, jpeg::JpegEncoder, png::PngEncoder, tiff::TiffEncoder,
        webp::WebPEncoder,
    },
    ColorType, ImageEncoder,
};
use metaforge_core::error::{MetaForgeError, Result};
use std::io::Cursor;

/// Transcodes an input image from arbitrary container format to a specified target container.
pub fn transcode_image_bytes(input: &[u8], options: &ImageConvertOptions) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(MetaForgeError::EmptyFile("input buffer".to_string()));
    }

    // 1. Decode image into memory
    let img = image::load_from_memory(input)
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    let (width, height) = (img.width(), img.height());

    // 2. Allocation bomb guardrail
    if width > options.max_dimension || height > options.max_dimension {
        return Err(MetaForgeError::DimensionAllocationBomb { width, height });
    }
    let total_pixels = (width as u64) * (height as u64);
    if total_pixels > 268_435_456 {
        // Exceeds 256 megapixels
        return Err(MetaForgeError::DimensionAllocationBomb { width, height });
    }

    // 3. Optional resize
    let processed = if let Some((target_w, target_h)) = options.resize {
        if target_w == 0 || target_h == 0 {
            return Err(MetaForgeError::ConversionError {
                detail: "Target resize dimensions must be non-zero".to_string(),
            });
        }
        if target_w > options.max_dimension || target_h > options.max_dimension {
            return Err(MetaForgeError::DimensionAllocationBomb {
                width: target_w,
                height: target_h,
            });
        }
        img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // 4. Encode to target format
    let mut output_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut output_bytes);
    let color = processed.color();
    let raw_bytes = processed.as_bytes();
    let (out_w, out_h) = (processed.width(), processed.height());

    match options.target_format {
        ImageTargetFormat::Jpeg => {
            let quality = options.quality.unwrap_or(85).clamp(1, 100);
            let encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
            // JPEG cannot encode RGBA directly; convert to RGB8 if needed
            let rgb_img = processed.to_rgb8();
            encoder
                .write_image(rgb_img.as_raw(), out_w, out_h, ColorType::Rgb8.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
        ImageTargetFormat::Png => {
            let encoder = PngEncoder::new(&mut cursor);
            encoder
                .write_image(raw_bytes, out_w, out_h, color.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
        ImageTargetFormat::Webp => {
            let encoder = WebPEncoder::new_lossless(&mut cursor);
            encoder
                .write_image(raw_bytes, out_w, out_h, color.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
        ImageTargetFormat::Bmp => {
            let mut encoder = BmpEncoder::new(&mut cursor);
            encoder
                .encode(raw_bytes, out_w, out_h, color.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
        ImageTargetFormat::Tiff => {
            let encoder = TiffEncoder::new(&mut cursor);
            encoder
                .write_image(raw_bytes, out_w, out_h, color.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
        ImageTargetFormat::Gif => {
            let mut encoder = GifEncoder::new(&mut cursor);
            // GIF expects Rgba8
            let rgba_img = processed.to_rgba8();
            encoder
                .encode(rgba_img.as_raw(), out_w, out_h, ColorType::Rgba8.into())
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
    }

    Ok(output_bytes)
}
