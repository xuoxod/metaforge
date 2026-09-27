use crate::types::MediaProbe;
use hound::WavReader;
use metaforge_core::error::{MetaForgeError, Result};
use metaforge_parsers::detector::detect_container_type;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

/// Probes an input media or image file and returns detailed container and stream telemetry.
pub fn probe_media_file<P: AsRef<Path>>(path: P) -> Result<MediaProbe> {
    let p = path.as_ref();
    let mut file = File::open(p).map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", p.display(), e)))?;
    let metadata = file.metadata()?;
    let size_bytes = metadata.len();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.is_empty() {
        return Err(MetaForgeError::EmptyFile(p.display().to_string()));
    }

    // 1. Attempt audio WAV probe
    if buffer.len() >= 12 && &buffer[0..4] == b"RIFF" && &buffer[8..12] == b"WAVE" {
        if let Ok(reader) = WavReader::new(Cursor::new(&buffer)) {
            let spec = reader.spec();
            let duration = reader.duration() as f64 / spec.sample_rate as f64;
            return Ok(MediaProbe {
                file_path: p.display().to_string(),
                media_kind: "Audio".to_string(),
                format: "WAV (Linear PCM Audio)".to_string(),
                size_bytes,
                dimensions: None,
                duration_seconds: Some(duration),
                sample_rate: Some(spec.sample_rate),
                channels: Some(spec.channels),
                bits_per_sample: Some(spec.bits_per_sample),
            });
        }
    }

    // 2. Attempt image container probe
    if let Ok(container_type) = detect_container_type(&buffer, p) {
        let (width, height) = match image::load_from_memory(&buffer) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => (None, None),
        };

        let dimensions = match (width, height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        };

        return Ok(MediaProbe {
            file_path: p.display().to_string(),
            media_kind: "Image".to_string(),
            format: container_type.display_name().to_string(),
            size_bytes,
            dimensions,
            duration_seconds: None,
            sample_rate: None,
            channels: None,
            bits_per_sample: None,
        });
    }

    // 3. Fallback generic image detection
    if let Ok(format) = image::guess_format(&buffer) {
        let (w, h) = match image::load_from_memory(&buffer) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => (None, None),
        };
        let dimensions = match (w, h) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        };

        return Ok(MediaProbe {
            file_path: p.display().to_string(),
            media_kind: "Image".to_string(),
            format: format!("{:?}", format),
            size_bytes,
            dimensions,
            duration_seconds: None,
            sample_rate: None,
            channels: None,
            bits_per_sample: None,
        });
    }

    Err(MetaForgeError::UnsupportedFormat {
        format: p.display().to_string(),
    })
}
