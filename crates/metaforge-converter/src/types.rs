use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

/// Supported audio target formats for encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioTargetFormat {
    Wav,
    Mp3,
    Flac,
    Ogg,
    Aac,
}

impl AudioTargetFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "wav" => Some(AudioTargetFormat::Wav),
            "mp3" => Some(AudioTargetFormat::Mp3),
            "flac" => Some(AudioTargetFormat::Flac),
            "ogg" | "oga" => Some(AudioTargetFormat::Ogg),
            "aac" | "m4a" => Some(AudioTargetFormat::Aac),
            _ => None,
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            AudioTargetFormat::Wav => "wav",
            AudioTargetFormat::Mp3 => "mp3",
            AudioTargetFormat::Flac => "flac",
            AudioTargetFormat::Ogg => "ogg",
            AudioTargetFormat::Aac => "aac",
        }
    }
}

/// Supported video target formats for transcoding and container remuxing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoTargetFormat {
    Mp4,
    Mkv,
    Webm,
    Mov,
    Avi,
}

impl VideoTargetFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "mp4" => Some(VideoTargetFormat::Mp4),
            "mkv" => Some(VideoTargetFormat::Mkv),
            "webm" => Some(VideoTargetFormat::Webm),
            "mov" => Some(VideoTargetFormat::Mov),
            "avi" => Some(VideoTargetFormat::Avi),
            _ => None,
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            VideoTargetFormat::Mp4 => "mp4",
            VideoTargetFormat::Mkv => "mkv",
            VideoTargetFormat::Webm => "webm",
            VideoTargetFormat::Mov => "mov",
            VideoTargetFormat::Avi => "avi",
        }
    }
}

/// Broad category of media container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Audio,
    Video,
}

/// Strongly typed universal media format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaFormat {
    Image(ImageTargetFormat),
    Audio(AudioTargetFormat),
    Video(VideoTargetFormat),
}

impl MediaFormat {
    pub fn kind(&self) -> MediaKind {
        match self {
            MediaFormat::Image(_) => MediaKind::Image,
            MediaFormat::Audio(_) => MediaKind::Audio,
            MediaFormat::Video(_) => MediaKind::Video,
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            MediaFormat::Image(f) => f.to_extension(),
            MediaFormat::Audio(f) => f.to_extension(),
            MediaFormat::Video(f) => f.to_extension(),
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        if let Some(f) = ImageTargetFormat::from_extension(ext) {
            return Some(MediaFormat::Image(f));
        }
        if let Some(f) = AudioTargetFormat::from_extension(ext) {
            return Some(MediaFormat::Audio(f));
        }
        if let Some(f) = VideoTargetFormat::from_extension(ext) {
            return Some(MediaFormat::Video(f));
        }
        None
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConvertOptions {
    pub target_format: AudioTargetFormat,
    pub target_sample_rate: Option<u32>,
    pub target_channels: Option<u16>,
    pub target_bits_per_sample: Option<u16>,
    pub gain: Option<f32>,
    pub normalize: bool,
}

impl Default for AudioConvertOptions {
    fn default() -> Self {
        Self {
            target_format: AudioTargetFormat::Wav,
            target_sample_rate: None,
            target_channels: None,
            target_bits_per_sample: Some(16),
            gain: None,
            normalize: false,
        }
    }
}

/// Options configuring video conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConvertOptions {
    pub target_format: VideoTargetFormat,
    pub quality: Option<u8>,
    pub resize: Option<(u32, u32)>,
    pub fps: Option<f32>,
    pub crf: Option<u32>,
    pub faststart: bool,
    pub copy_streams: bool,
}

impl Default for VideoConvertOptions {
    fn default() -> Self {
        Self {
            target_format: VideoTargetFormat::Mp4,
            quality: Some(80),
            resize: None,
            fps: None,
            crf: Some(23),
            faststart: true,
            copy_streams: false,
        }
    }
}

/// Unified options covering image, audio, and video conversion
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnifiedConvertOptions {
    pub image: ImageConvertOptions,
    pub audio: AudioConvertOptions,
    pub video: VideoConvertOptions,
    pub dry_run: bool,
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
    pub codec_name: Option<String>,
    pub audio_tracks: usize,
    pub video_tracks: usize,
    pub bit_rate: Option<u32>,
}

/// Execution status of an individual batch item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatchItemStatus {
    Success,
    Skipped,
    Failed(String),
}

/// Individual item report within a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemReport {
    pub input_path: String,
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub status: BatchItemStatus,
    pub duration_ms: u64,
}

/// Options configuring directory-wide batch conversion
#[derive(Debug, Clone)]
pub struct BatchConvertOptions {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub extensions: Vec<String>,
    pub target_format: String,
    pub max_workers: usize,
    pub dry_run: bool,
    pub flatten: bool,
    pub image_options: ImageConvertOptions,
    pub audio_options: AudioConvertOptions,
}

impl Default for BatchConvertOptions {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            output_dir: PathBuf::new(),
            extensions: Vec::new(),
            target_format: String::new(),
            max_workers: 4,
            dry_run: false,
            flatten: false,
            image_options: ImageConvertOptions::default(),
            audio_options: AudioConvertOptions::default(),
        }
    }
}

/// Summary report for a completed batch conversion run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    pub total_scanned: usize,
    pub converted: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total_input_bytes: u64,
    pub total_output_bytes: u64,
    pub items: Vec<BatchItemReport>,
    pub elapsed_ms: u64,
    pub dry_run: bool,
}

pub fn detect_target_format_from_path<P: AsRef<Path>>(path: P) -> Option<ImageTargetFormat> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ImageTargetFormat::from_extension)
}

pub fn is_audio_or_video_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "wav"
            | "mp3"
            | "flac"
            | "ogg"
            | "oga"
            | "opus"
            | "aac"
            | "m4a"
            | "mp4"
            | "mkv"
            | "webm"
            | "mov"
            | "aiff"
            | "aif"
            | "caf"
    )
}

/// Sniff magic header bytes to detect audio or video container formats.
pub fn sniff_audio_or_video_container(bytes: &[u8]) -> (bool, Option<&'static str>) {
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return (true, Some("wav"));
    }
    if bytes.starts_with(b"fLaC") {
        return (true, Some("flac"));
    }
    if bytes.starts_with(b"OggS") {
        return (true, Some("ogg"));
    }
    if bytes.starts_with(b"ID3")
        || (bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0)
    {
        return (true, Some("mp3"));
    }
    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        return (true, Some("mp4"));
    }
    if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return (true, Some("mkv"));
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"FORM" && &bytes[8..12] == b"AIFF" {
        return (true, Some("aiff"));
    }
    if bytes.starts_with(b"caff") {
        return (true, Some("caf"));
    }
    (false, None)
}
