use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// MetaForge: Sovereign High-Precision Image & Media Transcoder, Forensics & Metadata Engine
#[derive(Parser, Debug)]
#[command(
    name = "metaforge",
    author = "RMediaTech Contributors <contact@rmediatech.com>",
    version,
    about = "Zero-dependency pure-Rust multi-format image & media transcoder, forensics auditor and metadata scrubber",
    long_about = "MetaForge delivers mathematically sound, microsecond binary parsing, lossless sanitization, \
                  steganography detection, multi-format media transcoding (MP4, MKV, MP3, FLAC, OGG, WAV, WebP, PNG, JPEG), \
                  and batch directory conversion with zero C/FFI runtime dependencies."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Target file to scan when no subcommand is given
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Output report format (table, json, jsonl, csv)
    #[arg(short, long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Suppress verbose visual headers and process quietly
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Jsonl,
    Csv,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect metadata segments and EXIF/GPS dictionary tags
    Scan(ScanArgs),

    /// Perform steganography, Shannon entropy, and malicious polyglot forensics audit
    Audit(AuditArgs),

    /// Sanitize image by stripping private metadata and truncating trailing overlays
    Sanitize(SanitizeArgs),

    /// Read or edit comments/text tags in JPEG and PNG containers
    Comment(CommentArgs),

    /// Dump low-level container segments and chunk hierarchies
    Dump(DumpArgs),

    /// Transcode and convert media containers and images with resize, DSP audio and streaming pipe controls
    Convert(ConvertArgs),

    /// Batch transcode entire directory recursively with concurrency guardrails
    Batch(BatchArgs),

    /// Probe media container, audio/video streams, and image telemetry
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
    /// Input media or image file to convert (use '-' for stdin)
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Target output file path (use '-' for stdout)
    #[arg(value_name = "OUTPUT")]
    pub output: PathBuf,

    /// Explicit target format extension (e.g. --target webp, -t wav; required when output is stdout '-')
    #[arg(short, long)]
    pub target: Option<String>,

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

    /// Linear audio gain multiplier (e.g., 1.5)
    #[arg(long)]
    pub gain: Option<f32>,

    /// Normalize audio peak to 0.98
    #[arg(long)]
    pub normalize: bool,

    /// Perform a dry-run estimation without writing output files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct BatchArgs {
    /// Source directory containing input media files
    #[arg(value_name = "INPUT_DIR")]
    pub input: PathBuf,

    /// Target destination directory
    #[arg(value_name = "OUTPUT_DIR")]
    pub output: PathBuf,

    /// Filter by input file extensions (comma-separated, e.g. "mp4,mp3,flac" or "png,jpg")
    #[arg(short, long)]
    pub ext: Option<String>,

    /// Target output format (e.g. "wav", "webp", "png") [default: auto]
    #[arg(short, long)]
    pub target: Option<String>,

    /// Max concurrent worker threads (capped at 4 per system rules) [default: 4]
    #[arg(short, long, default_value_t = 4)]
    pub workers: usize,

    /// Flatten destination directory rather than mirroring source directory tree
    #[arg(long)]
    pub flatten: bool,

    /// Perform dry-run pre-flight check without modifying disk
    #[arg(long)]
    pub dry_run: bool,

    /// Target image quality (1-100) [default: 85]
    #[arg(short, long, default_value_t = 85)]
    pub quality: u8,

    /// Resize image to WxH (e.g., "800x600")
    #[arg(short, long)]
    pub resize: Option<String>,

    /// Target audio sample rate in Hz
    #[arg(long)]
    pub rate: Option<u32>,

    /// Target audio channels (1=mono, 2=stereo)
    #[arg(long)]
    pub channels: Option<u16>,

    /// Linear audio gain multiplier
    #[arg(long)]
    pub gain: Option<f32>,

    /// Normalize audio peak
    #[arg(long)]
    pub normalize: bool,
}

#[derive(Args, Debug)]
pub struct ProbeArgs {
    /// Target image or media file to probe
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
}
