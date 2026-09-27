use metaforge_core::error::{MetaForgeError, Result};
use metaforge_core::types::ContainerType;
use metaforge_parsers::detector::detect_container_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubReport {
    pub original_size: usize,
    pub scrubbed_size: usize,
    pub bytes_removed: usize,
    pub overlay_truncated: bool,
    pub metadata_stripped: bool,
}

/// Truncates any overlay (trailing data after logical EOF) without modifying metadata.
pub fn truncate_overlay(bytes: &[u8]) -> Result<(Vec<u8>, bool)> {
    let format = detect_container_bytes(bytes);
    match format {
        Some(ContainerType::Jpeg) => truncate_jpeg_overlay(bytes),
        Some(ContainerType::Png) => truncate_png_overlay(bytes),
        _ => {
            // For other formats or unknown, return as-is
            Ok((bytes.to_vec(), false))
        }
    }
}

fn truncate_jpeg_overlay(bytes: &[u8]) -> Result<(Vec<u8>, bool)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "0xFFD8".to_string(),
        });
    }

    // Find the last occurrence of EOI marker (0xFF, 0xD9)
    let mut eoi_idx = None;
    for i in (2..bytes.len() - 1).rev() {
        if bytes[i] == 0xFF && bytes[i + 1] == 0xD9 {
            eoi_idx = Some(i + 2);
            break;
        }
    }

    if let Some(logical_end) = eoi_idx {
        if logical_end < bytes.len() {
            return Ok((bytes[..logical_end].to_vec(), true));
        }
    }

    Ok((bytes.to_vec(), false))
}

fn truncate_png_overlay(bytes: &[u8]) -> Result<(Vec<u8>, bool)> {
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 8 || &bytes[0..8] != PNG_MAGIC {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "PNG Header".to_string(),
        });
    }

    let mut offset = 8;
    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;

        let chunk_type = &bytes[offset + 4..offset + 8];
        let total_chunk_len = 12 + length;

        if chunk_type == b"IEND" {
            let logical_end = offset + total_chunk_len;
            if logical_end < bytes.len() {
                return Ok((bytes[..logical_end].to_vec(), true));
            } else {
                return Ok((bytes.to_vec(), false));
            }
        }

        offset += total_chunk_len;
    }

    Ok((bytes.to_vec(), false))
}

/// Fully scrubs an image: strips private metadata (EXIF, XMP, IPTC, comments) and truncates overlays.
pub fn scrub_image(bytes: &[u8]) -> Result<(Vec<u8>, ScrubReport)> {
    let original_size = bytes.len();
    let format = detect_container_bytes(bytes);

    let (scrubbed, overlay_truncated, metadata_stripped) = match format {
        Some(ContainerType::Jpeg) => scrub_jpeg(bytes)?,
        Some(ContainerType::Png) => scrub_png(bytes)?,
        _ => (bytes.to_vec(), false, false),
    };

    let scrubbed_size = scrubbed.len();
    let bytes_removed = original_size.saturating_sub(scrubbed_size);

    let report = ScrubReport {
        original_size,
        scrubbed_size,
        bytes_removed,
        overlay_truncated,
        metadata_stripped,
    };

    Ok((scrubbed, report))
}

fn scrub_jpeg(bytes: &[u8]) -> Result<(Vec<u8>, bool, bool)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "0xFFD8".to_string(),
        });
    }

    let (truncated_bytes, overlay_truncated) = truncate_jpeg_overlay(bytes)?;
    let bytes = &truncated_bytes[..];

    let mut output = Vec::with_capacity(bytes.len());
    output.extend_from_slice(&[0xFF, 0xD8]);

    let mut i = 2;
    let mut stripped_meta = false;

    while i < bytes.len() {
        if bytes[i] != 0xFF {
            output.extend_from_slice(&bytes[i..]);
            break;
        }

        while i < bytes.len() && bytes[i] == 0xFF {
            i += 1;
        }
        if i >= bytes.len() {
            output.push(0xFF);
            break;
        }

        let marker = bytes[i];
        i += 1;

        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) {
            output.extend_from_slice(&[0xFF, marker]);
            continue;
        }

        if marker == 0xD9 {
            // EOI
            output.extend_from_slice(&[0xFF, 0xD9]);
            break;
        }

        if i + 2 > bytes.len() {
            return Err(MetaForgeError::TruncatedStructure {
                container: "JPEG",
                offset: i,
                length: bytes.len(),
            });
        }

        let seg_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
        if seg_len < 2 || i + seg_len > bytes.len() {
            return Err(MetaForgeError::CorruptedChunk {
                container: "JPEG",
                chunk_type: format!("0xFF{:02X}", marker),
                detail: format!("Invalid segment length {}", seg_len),
            });
        }

        // Check if marker is metadata:
        // APP1 (0xE1 = EXIF/XMP), APP2 (0xE2), APP13 (0xED = Photoshop), COM (0xFE)
        // Keep APP0 (JFIF) if standard, but strip others
        let is_meta = (marker == 0xE1) || (marker == 0xE2) || (marker == 0xED) || (marker == 0xFE);

        if is_meta {
            stripped_meta = true;
            i += seg_len;
            continue;
        }

        // If marker is SOS (Start of Scan), copy SOS segment and entire remaining scan data
        if marker == 0xDA {
            output.extend_from_slice(&[0xFF, 0xDA]);
            output.extend_from_slice(&bytes[i..]);
            break;
        }

        // Copy non-metadata segment
        output.extend_from_slice(&[0xFF, marker]);
        output.extend_from_slice(&bytes[i..i + seg_len]);
        i += seg_len;
    }

    Ok((output, overlay_truncated, stripped_meta))
}

fn scrub_png(bytes: &[u8]) -> Result<(Vec<u8>, bool, bool)> {
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 8 || &bytes[0..8] != PNG_MAGIC {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "PNG Header".to_string(),
        });
    }

    let (truncated_bytes, overlay_truncated) = truncate_png_overlay(bytes)?;
    let bytes = &truncated_bytes[..];

    let mut output = Vec::with_capacity(bytes.len());
    output.extend_from_slice(PNG_MAGIC);

    let mut offset = 8;
    let mut stripped_meta = false;

    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;

        let chunk_type = &bytes[offset + 4..offset + 8];
        let total_chunk_len = 12 + length;

        if offset + total_chunk_len > bytes.len() {
            return Err(MetaForgeError::TruncatedStructure {
                container: "PNG",
                offset,
                length: bytes.len(),
            });
        }

        // Critical PNG chunks that must be preserved: IHDR, PLTE, IDAT, IEND
        // Strip metadata chunks: tEXt, zTXt, iTXt, eXIf, tIME
        let is_meta_chunk = matches!(chunk_type, b"tEXt" | b"zTXt" | b"iTXt" | b"eXIf" | b"tIME");

        if is_meta_chunk {
            stripped_meta = true;
            offset += total_chunk_len;
            continue;
        }

        // Copy chunk
        output.extend_from_slice(&bytes[offset..offset + total_chunk_len]);
        offset += total_chunk_len;

        if chunk_type == b"IEND" {
            break;
        }
    }

    Ok((output, overlay_truncated, stripped_meta))
}
