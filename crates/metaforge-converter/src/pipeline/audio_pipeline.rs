//! Sovereign Chainable Audio Processing Pipeline

use crate::codecs::audio_decode::{decode_audio_bytes, decode_audio_file, decode_audio_stream};
use crate::codecs::audio_encode::encode_wav_bytes;
use crate::dsp::{apply_gain, normalize_peak, remix_channels, resample_pcm_f32};
use crate::types::{AudioConvertOptions, AudioTargetFormat, ConvertReport};
use metaforge_core::error::Result;
use std::path::Path;
use symphonia::core::io::MediaSource;

/// Fluent builder for chainable audio transcoding and DSP pipelines.
#[derive(Debug, Clone, Default)]
pub struct AudioPipeline {
    target_format: Option<AudioTargetFormat>,
    target_sample_rate: Option<u32>,
    target_channels: Option<u16>,
    target_bits_per_sample: Option<u16>,
    gain: Option<f32>,
    normalize: bool,
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            target_format: Some(AudioTargetFormat::Wav),
            target_sample_rate: None,
            target_channels: None,
            target_bits_per_sample: Some(16),
            gain: None,
            normalize: false,
        }
    }

    pub fn from_options(options: &AudioConvertOptions) -> Self {
        Self {
            target_format: Some(AudioTargetFormat::Wav),
            target_sample_rate: options.target_sample_rate,
            target_channels: options.target_channels,
            target_bits_per_sample: options.target_bits_per_sample.or(Some(16)),
            gain: options.gain,
            normalize: options.normalize,
        }
    }

    pub fn target_format(mut self, format: AudioTargetFormat) -> Self {
        self.target_format = Some(format);
        self
    }

    pub fn get_target_format(&self) -> Option<AudioTargetFormat> {
        self.target_format
    }

    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.target_sample_rate = Some(rate);
        self
    }

    pub fn channels(mut self, channels: u16) -> Self {
        self.target_channels = Some(channels);
        self
    }

    pub fn bits_per_sample(mut self, bits: u16) -> Self {
        self.target_bits_per_sample = Some(bits);
        self
    }

    pub fn gain(mut self, gain: f32) -> Self {
        self.gain = Some(gain);
        self
    }

    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    /// Process in-memory audio/video container bytes to WAV bytes
    pub fn process_bytes(&self, bytes: &[u8], extension_hint: Option<&str>) -> Result<Vec<u8>> {
        let decoded = decode_audio_bytes(bytes, extension_hint)?;
        self.process_decoded_audio(decoded)
    }

    /// Process media source stream to WAV bytes
    pub fn process_stream(
        &self,
        source: Box<dyn MediaSource>,
        extension_hint: Option<&str>,
    ) -> Result<Vec<u8>> {
        let decoded = decode_audio_stream(source, extension_hint)?;
        self.process_decoded_audio(decoded)
    }

    /// Process a media file on disk to a target WAV file
    pub fn process_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_path: P1,
        output_path: P2,
    ) -> Result<ConvertReport> {
        let inp = input_path.as_ref();
        let outp = output_path.as_ref();

        let input_meta = std::fs::metadata(inp)?;
        let in_bytes = input_meta.len() as usize;

        let decoded = decode_audio_file(inp)?;
        let in_rate = decoded.sample_rate;
        let in_channels = decoded.channels;

        let wav_bytes = self.process_decoded_audio(decoded)?;
        let out_bytes = wav_bytes.len();

        let final_rate = self.target_sample_rate.unwrap_or(in_rate);
        let final_ch = self.target_channels.unwrap_or(in_channels as u16);
        let final_bits = self.target_bits_per_sample.unwrap_or(16);

        std::fs::write(outp, &wav_bytes)?;

        let in_fmt = inp.extension().and_then(|s| s.to_str()).unwrap_or("unknown");
        let details = format!(
            "Decoded from {} ({}), exported as WAV ({} Hz, {} ch, {}-bit)",
            in_fmt.to_ascii_uppercase(),
            if in_channels == 1 { "mono" } else { "stereo" },
            final_rate,
            final_ch,
            final_bits
        );

        Ok(ConvertReport {
            input_path: inp.to_string_lossy().into_owned(),
            output_path: outp.to_string_lossy().into_owned(),
            input_bytes: in_bytes,
            output_bytes: out_bytes,
            input_format: in_fmt.to_string(),
            output_format: "wav".to_string(),
            details,
        })
    }

    fn process_decoded_audio(
        &self,
        decoded: crate::codecs::audio_decode::DecodedAudio,
    ) -> Result<Vec<u8>> {
        let mut samples = decoded.samples;
        let mut current_channels = decoded.channels;
        let mut current_rate = decoded.sample_rate;

        // 1. Channel remixing with safety bounds
        if let Some(target_ch) = self.target_channels {
            if target_ch == 0 || target_ch > 8 {
                return Err(metaforge_core::error::MetaForgeError::ConversionError {
                    detail: format!("Unsupported target channel count: {target_ch} (allowed: 1..=8)"),
                });
            }
            let target_ch_usize = target_ch as usize;
            if target_ch_usize != current_channels {
                samples = remix_channels(&samples, current_channels, target_ch_usize);
                current_channels = target_ch_usize;
            }
        }

        // 2. Resampling
        if let Some(target_rate) = self.target_sample_rate {
            if target_rate != current_rate {
                samples = resample_pcm_f32(&samples, current_channels, current_rate, target_rate);
                current_rate = target_rate;
            }
        }

        // 3. DSP Gain & Normalization
        if let Some(g) = self.gain {
            apply_gain(&mut samples, g);
        }
        if self.normalize {
            normalize_peak(&mut samples, 0.98);
        }

        // 4. Encode to WAV
        let bits = self.target_bits_per_sample.unwrap_or(16);
        encode_wav_bytes(&samples, current_channels as u16, current_rate, bits)
    }
}
