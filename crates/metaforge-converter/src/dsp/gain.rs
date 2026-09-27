//! Sovereign Audio Gain, Normalization & Limiter DSP

/// Apply linear gain amplification to floating point samples with soft clamping to [-1.0, 1.0].
pub fn apply_gain(samples: &mut [f32], gain: f32) {
    if (gain - 1.0).abs() < 1e-6 {
        return;
    }
    for s in samples.iter_mut() {
        *s = (*s * gain).clamp(-1.0, 1.0);
    }
}

/// Normalize samples such that the highest absolute peak matches `target_peak` (typically 0.95-1.0).
pub fn normalize_peak(samples: &mut [f32], target_peak: f32) {
    if samples.is_empty() {
        return;
    }

    let mut current_peak = 0.0f32;
    for &s in samples.iter() {
        let abs = s.abs();
        if abs > current_peak {
            current_peak = abs;
        }
    }

    if current_peak > 1e-6 {
        let scale = target_peak / current_peak;
        apply_gain(samples, scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peak_normalization() {
        let mut samples = vec![0.5, -0.25, 0.1];
        normalize_peak(&mut samples, 1.0);
        assert!((samples[0] - 1.0).abs() < 1e-4);
        assert!((samples[1] - (-0.5)).abs() < 1e-4);
    }
}
