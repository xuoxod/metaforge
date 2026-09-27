#![allow(clippy::collapsible_if)]
use crate::common::{extract_exif_metadata, ExifMetadata};
use exif as kamadak_exif;
use metaforge_core::MetaForgeError;
use serde::Serialize;
use std::io::Cursor;

/// Details of a discovered JPEG segment marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JpegSegment {
    pub marker: u8,
    pub name: String,
    pub offset: usize,
    pub length: usize,
}

/// Fully extracted structure and metadata of a JPEG file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JpegInfo {
    pub segments: Vec<JpegSegment>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub precision: Option<u8>,
    pub channels: Option<u8>,
    pub comment: Option<String>,
    pub metadata: ExifMetadata,
    pub official_end_offset: usize,
}

/// Parses raw JPEG bytes to extract segment lists, dimensions, comments, and EXIF tags.
pub fn parse_jpeg(bytes: &[u8]) -> Result<JpegInfo, MetaForgeError> {
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-jpeg".to_string(),
            expected: "JPEG SOI marker (0xFFD8)".to_string(),
        });
    }

    let mut segments = Vec::new();
    let mut width = None;
    let mut height = None;
    let mut precision = None;
    let mut channels = None;
    let mut comment = None;
    let mut raw_exif_payload: Option<Vec<u8>> = None;
    let mut raw_xmp_payload: Option<String> = None;

    let mut pos = 2;

    while pos + 2 <= bytes.len() {
        if bytes[pos] != 0xFF {
            break;
        }

        // Skip any padding 0xFF bytes
        while pos < bytes.len() && bytes[pos] == 0xFF {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        let marker = bytes[pos];
        pos += 1;

        // Standalone markers without length: SOI (0xD8), EOI (0xD9), RST0-RST7 (0xD0-0xD7)
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            let name = match marker {
                0xD8 => "SOI (Start of Image)",
                0xD9 => "EOI (End of Image)",
                0xD0..=0xD7 => "RST (Restart Marker)",
                _ => "Standalone Marker",
            };
            segments.push(JpegSegment {
                marker,
                name: name.to_string(),
                offset: pos - 2,
                length: 2,
            });
            if marker == 0xD9 {
                // EOI reached - logical end of JPEG
                break;
            }
            continue;
        }

        // Variable length markers must have 2-byte length
        if pos + 2 > bytes.len() {
            break;
        }

        let length = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        if length < 2 {
            return Err(MetaForgeError::CorruptedChunk {
                container: "JPEG",
                chunk_type: format!("0xFF{:02X}", marker),
                detail: format!("Segment length {} is less than minimum 2 bytes", length),
            });
        }

        let segment_start = pos - 2;
        let segment_end = pos + length;

        if segment_end > bytes.len() {
            return Err(MetaForgeError::TruncatedStructure {
                container: "JPEG",
                offset: segment_end,
                length: bytes.len(),
            });
        }

        let payload = &bytes[pos + 2..segment_end];

        let name = match marker {
            0xC0 => "SOF0 (Baseline DCT)",
            0xC1 => "SOF1 (Extended Sequential)",
            0xC2 => "SOF2 (Progressive DCT)",
            0xC3 => "SOF3 (Lossless)",
            0xC4 => "DHT (Define Huffman Table)",
            0xDB => "DQT (Define Quantization Table)",
            0xDD => "DRI (Define Restart Interval)",
            0xDA => "SOS (Start of Scan)",
            0xFE => "COM (Comment)",
            0xE0 => "APP0 (JFIF / JFXX)",
            0xE1 => "APP1 (EXIF / XMP)",
            0xE2..=0xEF => "APP (Application Specific)",
            _ => "Unknown Segment",
        };

        segments.push(JpegSegment {
            marker,
            name: name.to_string(),
            offset: segment_start,
            length: length + 2,
        });

        // Parse Start of Frame (SOF0, SOF1, SOF2) for image dimensions
        if (0xC0..=0xC3).contains(&marker) && payload.len() >= 6 {
            precision = Some(payload[0]);
            let h = u16::from_be_bytes([payload[1], payload[2]]) as u32;
            let w = u16::from_be_bytes([payload[3], payload[4]]) as u32;
            channels = Some(payload[5]);
            height = Some(h);
            width = Some(w);
        }

        // Parse Comment (COM)
        if marker == 0xFE {
            comment = Some(String::from_utf8_lossy(payload).trim().to_string());
        }

        // Parse APP1 (EXIF or XMP)
        if marker == 0xE1 {
            if payload.starts_with(b"Exif\0\0") {
                raw_exif_payload = Some(payload[6..].to_vec());
            } else if payload.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
                raw_xmp_payload = Some(String::from_utf8_lossy(&payload[29..]).trim().to_string());
            }
        }

        // Start of Scan (SOS): Following bytes are compressed image scan data until EOI
        if marker == 0xDA {
            pos = segment_end;
            // Scan for EOI (0xFFD9)
            let mut scan_pos = pos;
            while scan_pos + 1 < bytes.len() {
                if bytes[scan_pos] == 0xFF && bytes[scan_pos + 1] == 0xD9 {
                    segments.push(JpegSegment {
                        marker: 0xD9,
                        name: "EOI (End of Image)".to_string(),
                        offset: scan_pos,
                        length: 2,
                    });
                    break;
                }
                scan_pos += 1;
            }
            break;
        }

        pos = segment_end;
    }

    let mut metadata = ExifMetadata::default();
    if let Some(exif_bytes) = raw_exif_payload {
        let mut cursor = Cursor::new(exif_bytes);
        let exif_reader = kamadak_exif::Reader::new();
        if let Ok(exif_data) = exif_reader.read_from_container(&mut cursor) {
            metadata = extract_exif_metadata(&exif_data);
        }
    }
    metadata.xmp = raw_xmp_payload;

    let official_end_offset = segments.iter().map(|s| s.offset + s.length).max().unwrap_or(bytes.len());

    Ok(JpegInfo {
        segments,
        width,
        height,
        precision,
        channels,
        comment,
        metadata,
        official_end_offset,
    })
}
