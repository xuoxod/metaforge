use crate::types::AudioConvertOptions;
use hound::{WavReader, WavSpec, WavWriter};
use metaforge_core::error::{MetaForgeError, Result};
use std::io::Cursor;

/// Pure-Rust audio transcoding for WAV containers with channel mixing and resampling.
pub fn transcode_audio_wav_bytes(input: &[u8], options: &AudioConvertOptions) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(MetaForgeError::EmptyFile("input audio buffer".to_string()));
    }

    let cursor = Cursor::new(input);
    let mut reader = WavReader::new(cursor)
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    let spec = reader.spec();
    let src_channels = spec.channels;
    let src_rate = spec.sample_rate;

    let target_channels = options.target_channels.unwrap_or(src_channels);
    let target_rate = options.target_sample_rate.unwrap_or(src_rate);
    let target_bits = options.target_bits_per_sample.unwrap_or(spec.bits_per_sample);

    if target_channels == 0 || target_channels > 8 {
        return Err(MetaForgeError::ConversionError {
            detail: format!("Unsupported target channel count: {}", target_channels),
        });
    }

    // Read all samples into i32 normalized buffer
    let samples: std::result::Result<Vec<i32>, _> = reader.samples::<i32>().collect();
    let raw_samples = samples.map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    // De-interleave into channels
    let num_frames = raw_samples.len() / (src_channels as usize);
    let mut frames: Vec<Vec<i32>> = Vec::with_capacity(num_frames);

    for chunk in raw_samples.chunks_exact(src_channels as usize) {
        frames.push(chunk.to_vec());
    }

    // Channel conversion (downmix stereo->mono, upmix mono->stereo)
    let processed_frames: Vec<Vec<i32>> = if src_channels == target_channels {
        frames
    } else if src_channels == 2 && target_channels == 1 {
        // Stereo to mono downmix: average
        frames
            .into_iter()
            .map(|frame| {
                let mono = ((frame[0] as i64 + frame[1] as i64) / 2) as i32;
                vec![mono]
            })
            .collect()
    } else if src_channels == 1 && target_channels == 2 {
        // Mono to stereo upmix: duplicate
        frames
            .into_iter()
            .map(|frame| vec![frame[0], frame[0]])
            .collect()
    } else {
        // Fallback: take minimum common channels or pad with zero
        frames
            .into_iter()
            .map(|frame| {
                let mut new_f = vec![0i32; target_channels as usize];
                let copy_count = std::cmp::min(frame.len(), target_channels as usize);
                new_f[..copy_count].copy_from_slice(&frame[..copy_count]);
                new_f
            })
            .collect()
    };

    // Resampling via linear interpolation if target_rate != src_rate
    let resampled_frames: Vec<Vec<i32>> = if target_rate == src_rate {
        processed_frames
    } else {
        let ratio = (src_rate as f64) / (target_rate as f64);
        let target_frame_count = ((processed_frames.len() as f64) / ratio).round() as usize;
        let mut out = Vec::with_capacity(target_frame_count);

        for i in 0..target_frame_count {
            let src_pos = (i as f64) * ratio;
            let idx0 = src_pos.floor() as usize;
            let idx1 = std::cmp::min(idx0 + 1, processed_frames.len().saturating_sub(1));
            let fract = src_pos - (idx0 as f64);

            let frame: Vec<i32> = processed_frames[idx0]
                .iter()
                .zip(&processed_frames[idx1])
                .take(target_channels as usize)
                .map(|(&s0, &s1)| {
                    let interp = (s0 as f64) + ((s1 as f64) - (s0 as f64)) * fract;
                    interp.round() as i32
                })
                .collect();
            out.push(frame);
        }
        out
    };

    // Encode to output WAV
    let out_spec = WavSpec {
        channels: target_channels,
        sample_rate: target_rate,
        bits_per_sample: target_bits,
        sample_format: spec.sample_format,
    };

    let mut out_bytes = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut out_bytes), out_spec)
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    for frame in resampled_frames {
        for sample in frame {
            writer
                .write_sample(sample)
                .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;
        }
    }

    writer
        .finalize()
        .map_err(|e| MetaForgeError::ConversionError { detail: e.to_string() })?;

    Ok(out_bytes)
}
