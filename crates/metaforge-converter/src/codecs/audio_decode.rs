//! Sovereign Universal Audio & Video Container Demuxer and Decoder
//!
//! Powered by pure-Rust Symphonia (zero C/FFI). Demuxes and decodes audio
//! from MP4, MKV, WebM, MP3, FLAC, OGG/Vorbis, AAC, WAV, AIFF, and CAF.

use metaforge_core::error::{MetaForgeError, Result};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use symphonia::core::codecs::audio::CODEC_ID_NULL_AUDIO;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;

/// Normalized decoded linear PCM audio stream
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
    pub codec_name: String,
    pub container_name: String,
}

/// Decode audio from any file path (audio file or video container)
pub fn decode_audio_file<P: AsRef<Path>>(path: P) -> Result<DecodedAudio> {
    let p = path.as_ref();
    let file = File::open(p).map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", p.display(), e)))?;
    let ext = p.extension().and_then(|s| s.to_str());
    decode_audio_stream(Box::new(file), ext)
}

/// Decode audio from an in-memory byte slice
pub fn decode_audio_bytes(bytes: &[u8], extension_hint: Option<&str>) -> Result<DecodedAudio> {
    if bytes.is_empty() {
        return Err(MetaForgeError::EmptyFile("audio buffer".to_string()));
    }
    let cursor = Cursor::new(bytes.to_vec());
    decode_audio_stream(Box::new(cursor), extension_hint)
}

/// Decode audio from any boxed media source stream
pub fn decode_audio_stream(
    source: Box<dyn MediaSource>,
    extension_hint: Option<&str>,
) -> Result<DecodedAudio> {
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = extension_hint {
        hint.with_extension(ext);
    }

    let fmt_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| MetaForgeError::ConversionError {
            detail: format!("Failed to probe audio/media container: {e}"),
        })?;

    // Find the first valid audio track
    let (track_id, audio_params) = format
        .tracks()
        .iter()
        .find_map(|t| {
            let audio_p = t.codec_params.as_ref()?.audio()?;
            if audio_p.codec != CODEC_ID_NULL_AUDIO && audio_p.sample_rate.is_some() {
                Some((t.id, audio_p.clone()))
            } else {
                None
            }
        })
        .ok_or_else(|| MetaForgeError::ConversionError {
            detail: "No supported audio track found in media container".to_string(),
        })?;

    let sample_rate = audio_params.sample_rate.unwrap_or(44100);
    let channels = audio_params.channels.as_ref().map(|c| c.count()).unwrap_or(2);
    let codec_name = format!("{:?}", audio_params.codec);
    let container_name = "media_container".to_string();

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&audio_params, &Default::default())
        .map_err(|e| MetaForgeError::ConversionError {
            detail: format!("Failed to instantiate decoder for codec {codec_name}: {e}"),
        })?;

    let mut all_samples: Vec<f32> = Vec::new();

    // Max 1 hour of uncompressed 192kHz 8ch audio guardrail (~1.5GB) to prevent DoS bombs
    let max_samples = 192_000 * 8 * 3600;

    while let Ok(Some(packet)) = format.next_packet() {
        if packet.track_id != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf_ref) => {
                audio_buf_ref.copy_to_vec_interleaved(&mut all_samples);

                if all_samples.len() > max_samples {
                    return Err(MetaForgeError::ConversionError {
                        detail: "Decoded audio length exceeds safe resource bounds (max 1 hour)"
                            .to_string(),
                    });
                }
            }
            Err(SymphoniaError::DecodeError(_)) => {
                // Recoverable decode error on packet; proceed with stream
                continue;
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
            }
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => {
                break;
            }
        }
    }

    if all_samples.is_empty() {
        return Err(MetaForgeError::ConversionError {
            detail: "Decoded audio stream contained zero valid PCM samples".to_string(),
        });
    }

    Ok(DecodedAudio {
        samples: all_samples,
        sample_rate,
        channels,
        codec_name,
        container_name,
    })
}
