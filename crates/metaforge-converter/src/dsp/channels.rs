//! Sovereign Channel Remixing & Matrix Downmix Engine

/// Downmix multi-channel interleaved audio to single-channel mono.
pub fn downmix_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let frames = samples.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    let inv_ch = 1.0 / channels as f32;

    for frame in 0..frames {
        let mut sum = 0.0f32;
        let base = frame * channels;
        for ch in 0..channels {
            sum += samples[base + ch];
        }
        mono.push(sum * inv_ch);
    }
    mono
}

/// Upmix single-channel mono to stereo (dual mono).
pub fn upmix_mono_to_stereo(samples: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        stereo.push(s);
        stereo.push(s);
    }
    stereo
}

/// Remix interleaved samples from `in_channels` to `out_channels`.
pub fn remix_channels(samples: &[f32], in_channels: usize, out_channels: usize) -> Vec<f32> {
    if in_channels == out_channels || in_channels == 0 || out_channels == 0 || samples.is_empty() {
        return samples.to_vec();
    }

    if out_channels == 1 {
        return downmix_to_mono(samples, in_channels);
    }

    if in_channels == 1 && out_channels == 2 {
        return upmix_mono_to_stereo(samples);
    }

    // Generic remixing: truncate extra channels or duplicate/pad with zero
    let in_frames = samples.len() / in_channels;
    let mut out = Vec::with_capacity(in_frames * out_channels);

    for frame in 0..in_frames {
        let base = frame * in_channels;
        for ch in 0..out_channels {
            if ch < in_channels {
                out.push(samples[base + ch]);
            } else {
                // Pad with mono/first channel
                out.push(samples[base]);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downmix_to_mono() {
        let stereo = vec![1.0, -1.0, 0.5, 0.5];
        let mono = downmix_to_mono(&stereo, 2);
        assert_eq!(mono, vec![0.0, 0.5]);
    }

    #[test]
    fn test_upmix_to_stereo() {
        let mono = vec![0.5, -0.5];
        let stereo = upmix_mono_to_stereo(&mono);
        assert_eq!(stereo, vec![0.5, 0.5, -0.5, -0.5]);
    }
}
