//! Sovereign Chainable Image Processing Pipeline

use crate::codecs::image::transcode_image_bytes;
use crate::types::{ConvertReport, ImageConvertOptions, ImageTargetFormat};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Fluent builder for chainable image transcoding pipelines.
#[derive(Debug, Clone)]
pub struct ImagePipeline {
    options: ImageConvertOptions,
}

impl Default for ImagePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ImagePipeline {
    pub fn new() -> Self {
        Self {
            options: ImageConvertOptions::default(),
        }
    }

    pub fn target(mut self, format: ImageTargetFormat) -> Self {
        self.options.target_format = format;
        self
    }

    pub fn quality(mut self, quality: u8) -> Self {
        self.options.quality = Some(quality);
        self
    }

    pub fn resize(mut self, width: u32, height: u32) -> Self {
        self.options.resize = Some((width, height));
        self
    }

    pub fn max_dimension(mut self, max_dim: u32) -> Self {
        self.options.max_dimension = max_dim;
        self
    }

    pub fn preserve_metadata(mut self, preserve: bool) -> Self {
        self.options.preserve_metadata = preserve;
        self
    }

    /// Process in-memory image bytes
    pub fn process_bytes(&self, input: &[u8]) -> Result<Vec<u8>> {
        transcode_image_bytes(input, &self.options)
    }

    /// Process file on disk
    pub fn process_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        input_path: P1,
        output_path: P2,
    ) -> Result<ConvertReport> {
        let inp = input_path.as_ref();
        let outp = output_path.as_ref();

        let mut in_file = File::open(inp)
            .map_err(|e| MetaForgeError::FileNotFound(format!("{}: {}", inp.display(), e)))?;
        let mut buffer = Vec::new();
        in_file
            .read_to_end(&mut buffer)
            .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", inp.display(), e))))?;

        let in_bytes = buffer.len();
        let converted = self.process_bytes(&buffer)?;
        let out_bytes = converted.len();

        let mut out_file = File::create(outp)
            .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", outp.display(), e))))?;
        out_file
            .write_all(&converted)
            .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", outp.display(), e))))?;

        let in_fmt = inp.extension().and_then(|s| s.to_str()).unwrap_or("unknown");
        let out_fmt = self.options.target_format.to_extension();

        Ok(ConvertReport {
            input_path: inp.to_string_lossy().into_owned(),
            output_path: outp.to_string_lossy().into_owned(),
            input_bytes: in_bytes,
            output_bytes: out_bytes,
            input_format: in_fmt.to_string(),
            output_format: out_fmt.to_string(),
            details: format!(
                "Transcoded to {} (quality: {:?}, resize: {:?})",
                out_fmt.to_ascii_uppercase(),
                self.options.quality,
                self.options.resize
            ),
        })
    }
}
