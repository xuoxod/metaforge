//! Sovereign Media Codecs & Container Transcoders
//!
//! Pure-Rust image encoders/decoders (JPEG, PNG, WebP, GIF, BMP, TIFF) and
//! audio decoders (MP3, FLAC, OGG, AAC, MP4, MKV, WAV) with WAV output.

pub mod audio_decode;
pub mod audio_encode;
pub mod image;

pub use audio_decode::{decode_audio_bytes, decode_audio_file, decode_audio_stream, DecodedAudio};
pub use audio_encode::{encode_wav_bytes, encode_wav_file};
pub use image::transcode_image_bytes;
