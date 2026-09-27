//! Sovereign Media Format & Container Sniffing Subsystem
//!
//! Provides zero-copy header inspection and extension classification to detect
//! media kind (Image, Audio, Video) and concrete container format.

use crate::types::{AudioTargetFormat, ImageTargetFormat, MediaFormat, MediaKind, VideoTargetFormat};
use std::path::Path;

/// Sniff format from memory buffer, falling back to path extension if available.
pub fn detect_media_format(bytes: Option<&[u8]>, path: Option<&Path>) -> Option<MediaFormat> {
    if let Some(buf) = bytes {
        if let Some(fmt) = sniff_magic_bytes(buf) {
            return Some(fmt);
        }
    }

    if let Some(p) = path {
        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            return MediaFormat::from_extension(ext);
        }
    }

    None
}

/// Detect format from a file path by reading header bytes and checking extension.
pub fn detect_file_format<P: AsRef<Path>>(path: P) -> Option<MediaFormat> {
    let p = path.as_ref();
    if let Ok(mut file) = std::fs::File::open(p) {
        use std::io::Read;
        let mut header = [0u8; 1024];
        if let Ok(n) = file.read(&mut header) {
            if let Some(fmt) = sniff_magic_bytes(&header[..n]) {
                return Some(fmt);
            }
        }
    }

    p.extension()
        .and_then(|s| s.to_str())
        .and_then(MediaFormat::from_extension)
}

/// Sniff magic signature bytes to identify concrete image, audio, or video formats.
pub fn sniff_magic_bytes(bytes: &[u8]) -> Option<MediaFormat> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(MediaFormat::Image(ImageTargetFormat::Jpeg));
    }

    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(MediaFormat::Image(ImageTargetFormat::Png));
    }

    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(MediaFormat::Image(ImageTargetFormat::Gif));
    }

    if bytes.starts_with(b"BM") {
        return Some(MediaFormat::Image(ImageTargetFormat::Bmp));
    }

    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Some(MediaFormat::Image(ImageTargetFormat::Tiff));
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" {
        let fourcc = &bytes[8..12];
        if fourcc == b"WEBP" {
            return Some(MediaFormat::Image(ImageTargetFormat::Webp));
        }
        if fourcc == b"WAVE" {
            return Some(MediaFormat::Audio(AudioTargetFormat::Wav));
        }
        if fourcc == b"AVI " {
            return Some(MediaFormat::Video(VideoTargetFormat::Avi));
        }
    }

    if bytes.starts_with(b"fLaC") {
        return Some(MediaFormat::Audio(AudioTargetFormat::Flac));
    }

    if bytes.starts_with(b"OggS") {
        return Some(MediaFormat::Audio(AudioTargetFormat::Ogg));
    }

    if bytes.starts_with(b"ID3")
        || (bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0)
    {
        return Some(MediaFormat::Audio(AudioTargetFormat::Mp3));
    }

    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        if bytes.len() >= 12 && &bytes[8..12] == b"M4A " {
            return Some(MediaFormat::Audio(AudioTargetFormat::Aac));
        }
        return Some(MediaFormat::Video(VideoTargetFormat::Mp4));
    }

    if bytes.len() >= 8
        && (&bytes[4..8] == b"moov" || &bytes[4..8] == b"mdat" || &bytes[4..8] == b"wide")
    {
        return Some(MediaFormat::Video(VideoTargetFormat::Mov));
    }

    if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        // Matroska / WebM EBML header
        let scan_limit = bytes.len().min(512);
        let slice = &bytes[..scan_limit];
        if slice.windows(4).any(|w| w == b"webm") {
            return Some(MediaFormat::Video(VideoTargetFormat::Webm));
        }
        return Some(MediaFormat::Video(VideoTargetFormat::Mkv));
    }

    None
}

/// Detect the general media category (Image, Audio, Video).
pub fn detect_media_kind(bytes: Option<&[u8]>, path: Option<&Path>) -> Option<MediaKind> {
    detect_media_format(bytes, path).map(|f| f.kind())
}
