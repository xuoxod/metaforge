//! Sovereign Pure-Rust Audio Resampling DSP Engine
//!
//! Provides linear interpolation resampling for arbitrary sample rates.

/// Resample interleaved multi-channel 32-bit floating point PCM audio.
pub fn resample_pcm_f32(
    samples: &[f32],
    channels: usize,
    in_rate: u32,
    out_rate: u32,
) -> Vec<f32> {
    if in_rate == out_rate || in_rate == 0 || out_rate == 0 || channels == 0 || samples.is_empty() {
        return samples.to_vec();
    }

    let in_frames = samples.len() / channels;
    if in_frames == 0 {
        return Vec::new();
    }

    let ratio = in_rate as f64 / out_rate as f64;
    let out_frames = ((in_frames as f64) * (out_rate as f64) / (in_rate as f64)).round() as usize;
    let mut out = Vec::with_capacity(out_frames * channels);

    for out_frame in 0..out_frames {
        let src_pos = out_frame as f64 * ratio;
        let idx0 = src_pos.floor() as usize;
        let frac = (src_pos - idx0 as f64) as f32;
        let idx1 = (idx0 + 1).min(in_frames.saturating_sub(1));

        for ch in 0..channels {
            let s0 = samples[idx0 * channels + ch];
            let s1 = samples[idx1 * channels + ch];
            let interpolated = s0 + frac * (s1 - s0);
            out.push(interpolated);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_identity() {
        let input = vec![0.1, 0.2, 0.3, 0.4];
        let out = resample_pcm_f32(&input, 2, 44100, 44100);
        assert_eq!(input, out);
    }

    #[test]
    fn test_resample_downsample() {
        let input = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        let out = resample_pcm_f32(&input, 1, 48000, 24000);
        assert_eq!(out.len(), 3);
        assert!((out[0] - 0.0).abs() < 1e-4);
        assert!((out[1] - 0.4).abs() < 1e-4);
        assert!((out[2] - 0.8).abs() < 1e-4);
    }
}
