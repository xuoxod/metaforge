//! Sovereign Audio Digital Signal Processing (DSP) Subsystem
//!
//! Pure mathematical signal processing: sample rate conversion, channel mixing,
//! peak normalization, and amplitude scaling. Zero file system or container dependencies.

pub mod channels;
pub mod gain;
pub mod resample;

pub use channels::{downmix_to_mono, remix_channels, upmix_mono_to_stereo};
pub use gain::{apply_gain, normalize_peak};
pub use resample::resample_pcm_f32;
