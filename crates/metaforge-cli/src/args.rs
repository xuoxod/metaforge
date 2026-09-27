use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "metaforge",
    author = "RMediaTech Contributors <contact@rmediatech.com>",
    version = "0.1.0",
    about = "Sovereign High-Performance Image Metadata Extraction, Forensic Steganography & Privacy Sanitization Engine",
    long_about = "A unified, zero-dependency sovereign toolkit for extracting, forensically auditing, editing, and scrubbing metadata across JPEG, PNG, WebP, GIF, and HEIC containers."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Target image file to scan (default command if no subcommand provided)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub format: OutputFormat,

    /// Verbose output showing hex offsets and raw byte previews
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Jsonl,
    Csv,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan and extract structured metadata (EXIF, GPS, TIFF, XMP, Dimensions)
    Scan(ScanArgs),

    /// Perform steganography, Shannon entropy, and malicious polyglot forensics audit
    Audit(AuditArgs),

    /// Sanitize image by stripping private metadata and truncating trailing overlays
    Sanitize(SanitizeArgs),

    /// Read or edit comments/text tags in JPEG and PNG containers
    Comment(CommentArgs),

    /// Dump low-level container segments and chunk hierarchies
    Dump(DumpArgs),

    /// Transcode and convert image and audio containers with resize and quality controls
    Convert(ConvertArgs),

    /// Probe media container, audio streams, and image telemetry
    Probe(ProbeArgs),
}

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Target image file to scan
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Filter to only EXIF tags
    #[arg(long)]
    pub exif_only: bool,

    /// Filter to only GPS coordinate tags
    #[arg(long)]
    pub gps_only: bool,
}

#[derive(Args, Debug)]
pub struct AuditArgs {
    /// Target image file to audit
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Custom Shannon entropy threshold for suspicious payload alerts (default: 7.92)
    #[arg(long, default_value_t = 7.92)]
    pub entropy_threshold: f64,
}

#[derive(Args, Debug)]
pub struct SanitizeArgs {
    /// Target image file to sanitize
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Output file path (if omitted and --inplace not specified, writes to <name>.clean.<ext>)
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// Modify the file in place (overwrites original file)
    #[arg(long)]
    pub inplace: bool,

    /// Only truncate trailing overlay data, preserving image metadata
    #[arg(long)]
    pub overlay_only: bool,
}

#[derive(Args, Debug)]
pub struct CommentArgs {
    /// Target image file
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Set or replace comment / text tag value
    #[arg(short, long)]
    pub set: Option<String>,

    /// Delete existing comment / text tag
    #[arg(short, long)]
    pub delete: bool,

    /// PNG tag keyword (default: "Description" for PNG)
    #[arg(short, long, default_value = "Description")]
    pub key: String,

    /// Output file path (if omitted, modifies in-place or prints current comment)
    #[arg(short, long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct DumpArgs {
    /// Target image file to inspect
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
}

#[derive(Args, Debug)]
pub struct ConvertArgs {
    /// Input media or image file to convert
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Target output file path
    #[arg(value_name = "OUTPUT")]
    pub output: PathBuf,

    /// Target image quality (1-100, JPEG and WebP only) [default: 85]
    #[arg(short, long, default_value_t = 85)]
    pub quality: u8,

    /// Resize image to WxH (e.g., "800x600")
    #[arg(short, long)]
    pub resize: Option<String>,

    /// Target audio sample rate in Hz (e.g., 44100, 48000 for WAV)
    #[arg(long)]
    pub rate: Option<u32>,

    /// Target audio channels (1=mono, 2=stereo)
    #[arg(long)]
    pub channels: Option<u16>,
}

#[derive(Args, Debug)]
pub struct ProbeArgs {
    /// Target image or media file to probe
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
}

