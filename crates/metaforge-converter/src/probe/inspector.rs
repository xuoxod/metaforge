//! Sovereign Universal Media Prober
//!
//! Introspects image containers, audio files, and video streams to extract
//! format metadata, track counts, sample rates, and dimensional bounds.

use crate::types::{is_audio_or_video_extension, sniff_audio_or_video_container, MediaProbe};
use metaforge_core::error::{MetaForgeError, Result};
use std::io::Cursor;
use std::path::Path;
use symphonia::core::codecs::audio::CODEC_ID_NULL_AUDIO;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

/// Probe a media file on disk.
pub fn probe_media_file<P: AsRef<Path>>(path: P) -> Result<MediaProbe> {
    let p = path.as_ref();
    let meta = std::fs::metadata(p)
        .map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", p.display(), e)))?;
    let size_bytes = meta.len();
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    let bytes = std::fs::read(p)
        .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", p.display(), e))))?;

    let (is_av, detected_ext) = sniff_audio_or_video_container(&bytes);
    let effective_ext = if is_av {
        detected_ext
    } else if !ext.is_empty() {
        Some(ext.as_str())
    } else {
        None
    };

    if is_av || is_audio_or_video_extension(&ext) {
        let cursor = Cursor::new(bytes);
        probe_media_source(Box::new(cursor), size_bytes, p.to_string_lossy().into_owned(), effective_ext)
    } else {
        probe_image_bytes(&bytes, p.to_string_lossy().into_owned(), size_bytes)
    }
}

/// Probe media from an in-memory byte buffer.
pub fn probe_media_bytes(bytes: &[u8], extension_hint: Option<&str>) -> Result<MediaProbe> {
    if bytes.is_empty() {
        return Err(MetaForgeError::EmptyFile("buffer".to_string()));
    }
    let size_bytes = bytes.len() as u64;
    let ext_hint = extension_hint.unwrap_or("").to_lowercase();

    let (is_av, detected_ext) = sniff_audio_or_video_container(bytes);
    let effective_ext = if is_av {
        detected_ext
    } else if !ext_hint.is_empty() {
        Some(ext_hint.as_str())
    } else {
        None
    };

    if is_av || is_audio_or_video_extension(&ext_hint) {
        let cursor = Cursor::new(bytes.to_vec());
        probe_media_source(Box::new(cursor), size_bytes, "memory_buffer".to_string(), effective_ext)
    } else {
        probe_image_bytes(bytes, "memory_buffer".to_string(), size_bytes)
    }
}

fn probe_image_bytes(bytes: &[u8], file_path: String, size_bytes: u64) -> Result<MediaProbe> {
    if bytes.is_empty() {
        return Err(MetaForgeError::EmptyFile("image buffer".to_string()));
    }
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    let format_str = reader
        .format()
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|| "Unknown".to_string());

    let (dimensions, bits_per_sample) = if let Ok(img) = image::load_from_memory(bytes) {
        (Some((img.width(), img.height())), Some(img.color().bits_per_pixel()))
    } else {
        (None, None)
    };

    Ok(MediaProbe {
        file_path,
        media_kind: "Image".to_string(),
        format: format_str,
        size_bytes,
        dimensions,
        duration_seconds: None,
        sample_rate: None,
        channels: None,
        bits_per_sample,
        codec_name: None,
        audio_tracks: 0,
        video_tracks: 0,
        bit_rate: None,
    })
}

fn probe_media_source(
    source: Box<dyn symphonia::core::io::MediaSource>,
    size_bytes: u64,
    file_path: String,
    ext_hint: Option<&str>,
) -> Result<MediaProbe> {
    let mss = MediaSourceStream::new(source, Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = ext_hint {
        hint.with_extension(ext);
    }

    let fmt_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();

    let format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| MetaForgeError::ConversionError {
            detail: format!("Failed to probe media stream: {e}"),
        })?;

    let tracks = format.tracks();

    let mut audio_tracks = 0;
    let mut video_tracks = 0;
    let mut sample_rate = None;
    let mut channels = None;
    let mut bits_per_sample = None;
    let mut codec_name = None;
    let mut duration_seconds = None;

    for t in tracks {
        if let Some(params) = &t.codec_params {
            if let Some(audio_p) = params.audio() {
                if audio_p.codec != CODEC_ID_NULL_AUDIO && audio_p.sample_rate.is_some() {
                    audio_tracks += 1;
                    if sample_rate.is_none() {
                        sample_rate = audio_p.sample_rate;
                        channels = audio_p.channels.as_ref().map(|c| c.count() as u16);
                        bits_per_sample = audio_p.bits_per_sample.map(|b| b as u16);
                        codec_name = Some(format!("{:?}", audio_p.codec));

                        if let Some(dur) = t.duration {
                            if let Some(tb) = t.time_base {
                                duration_seconds =
                                    Some(dur.get() as f64 * tb.numer.get() as f64 / tb.denom.get() as f64);
                            }
                        }
                    }
                }
            } else if params.is_video() {
                video_tracks += 1;
            }
        }
    }

    let media_kind = if video_tracks > 0 {
        "Video Container".to_string()
    } else {
        "Audio".to_string()
    };

    let container_desc = ext_hint.unwrap_or("media").to_ascii_uppercase();

    Ok(MediaProbe {
        file_path,
        media_kind,
        format: container_desc,
        size_bytes,
        dimensions: None,
        duration_seconds,
        sample_rate,
        channels,
        bits_per_sample,
        codec_name,
        audio_tracks,
        video_tracks,
        bit_rate: None,
    })
}
