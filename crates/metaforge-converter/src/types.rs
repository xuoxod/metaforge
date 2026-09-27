use serde::{Deserialize, Serialize};
use std::path::Path;

/// Supported image formats for transcoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageTargetFormat {
    Jpeg,
    Png,
    Webp,
    Gif,
    Bmp,
    Tiff,
}

impl ImageTargetFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageTargetFormat::Jpeg),
            "png" => Some(ImageTargetFormat::Png),
            "webp" => Some(ImageTargetFormat::Webp),
            "gif" => Some(ImageTargetFormat::Gif),
            "bmp" => Some(ImageTargetFormat::Bmp),
            "tif" | "tiff" => Some(ImageTargetFormat::Tiff),
            _ => None,
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            ImageTargetFormat::Jpeg => "jpg",
            ImageTargetFormat::Png => "png",
            ImageTargetFormat::Webp => "webp",
            ImageTargetFormat::Gif => "gif",
            ImageTargetFormat::Bmp => "bmp",
            ImageTargetFormat::Tiff => "tiff",
        }
    }
}

/// Options configuring image conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConvertOptions {
    pub target_format: ImageTargetFormat,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
    pub preserve_metadata: bool,
    pub max_dimension: u32,
}

impl Default for ImageConvertOptions {
    fn default() -> Self {
        Self {
            target_format: ImageTargetFormat::Png,
            quality: Some(85),
            resize: None,
            preserve_metadata: true,
            max_dimension: 16384,
        }
    }
}

/// Options configuring audio conversion
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioConvertOptions {
    pub target_sample_rate: Option<u32>,
    pub target_channels: Option<u16>,
    pub target_bits_per_sample: Option<u16>,
}

/// Outcome report for a completed conversion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertReport {
    pub input_path: String,
    pub output_path: String,
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub input_format: String,
    pub output_format: String,
    pub details: String,
}

/// Detailed probe metadata for any image or media container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProbe {
    pub file_path: String,
    pub media_kind: String,
    pub format: String,
    pub size_bytes: u64,
    pub dimensions: Option<(u32, u32)>,
    pub duration_seconds: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub bits_per_sample: Option<u16>,
}

pub fn detect_target_format_from_path<P: AsRef<Path>>(path: P) -> Option<ImageTargetFormat> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ImageTargetFormat::from_extension)
}
