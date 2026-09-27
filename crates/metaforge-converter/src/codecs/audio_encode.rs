//! Sovereign Pure-Rust Audio WAV Encoder
//!
//! Encodes floating point linear PCM audio samples into WAV files via Hound.

use hound::{WavSpec, WavWriter};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

/// Encode floating point PCM samples into an in-memory WAV buffer.
pub fn encode_wav_bytes(
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    encode_wav_writer(&mut cursor, samples, channels, sample_rate, bits_per_sample)?;
    Ok(cursor.into_inner())
}

/// Encode floating point PCM samples directly to a WAV file on disk.
pub fn encode_wav_file<P: AsRef<Path>>(
    path: P,
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
) -> Result<()> {
    let p = path.as_ref();
    let file = File::create(p)
        .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", p.display(), e))))?;
    encode_wav_writer(file, samples, channels, sample_rate, bits_per_sample)?;
    Ok(())
}

fn encode_wav_writer<W: std::io::Write + std::io::Seek>(
    writer: W,
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
) -> Result<()> {
    let sample_format = if bits_per_sample == 32 {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    };

    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: if bits_per_sample == 32 { 32 } else { 16 },
        sample_format,
    };

    let mut wav_writer = WavWriter::new(writer, spec)
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    if spec.bits_per_sample == 32 {
        for &s in samples {
            wav_writer
                .write_sample(s)
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
    } else {
        for &s in samples {
            // Quantize normalized float [-1.0, 1.0] to signed 16-bit integer
            let clamped = s.clamp(-1.0, 1.0);
            let int_val = (clamped * 32767.0).round() as i16;
            wav_writer
                .write_sample(int_val)
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
    }

    wav_writer
        .finalize()
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    Ok(())
}
