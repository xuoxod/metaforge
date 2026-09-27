#![allow(clippy::collapsible_if)]
use crate::common::ExifMetadata;
use exif as kamadak_exif;
use metaforge_core::{crc32, MetaForgeError};
use serde::Serialize;
use std::io::Cursor;

/// Details of a discovered PNG chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngChunk {
    pub chunk_type: [u8; 4],
    pub type_name: String,
    pub offset: usize,
    pub length: usize,
    pub crc: u32,
    pub crc_valid: bool,
}

/// Structural details parsed from the IHDR chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

/// Resolution info parsed from the pHYs chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngResolution {
    pub ppu_x: u32,
    pub ppu_y: u32,
    pub unit_specifier: u8, // 1 = meter, 0 = unknown
}

/// Significant bits from sBIT chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngSignificantBits {
    pub bits: Vec<u8>,
}

/// Background color from bKGD chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngBackgroundColor {
    pub gray: Option<u16>,
    pub rgb: Option<(u16, u16, u16)>,
    pub palette_index: Option<u8>,
}

/// Image offset from oFFs chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PngOffset {
    pub offset_x: i32,
    pub offset_y: i32,
    pub unit_specifier: u8, // 0 = pixel, 1 = micrometer
}

/// Physical scale from sCAL chunk.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PngScale {
    pub unit_specifier: u8, // 1 = meter, 2 = radian
    pub width_scale: String,
    pub height_scale: String,
}

/// Comprehensive information extracted from a PNG container.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PngInfo {
    pub chunks: Vec<PngChunk>,
    pub header: Option<PngHeader>,
    pub palette_entries: Option<usize>,
    pub text_metadata: Vec<(String, String)>,
    pub resolution: Option<PngResolution>,
    pub significant_bits: Option<PngSignificantBits>,
    pub background_color: Option<PngBackgroundColor>,
    pub offset: Option<PngOffset>,
    pub scale: Option<PngScale>,
    pub last_modified: Option<String>,
    pub icc_profile_name: Option<String>,
    pub metadata: ExifMetadata,
    pub official_end_offset: usize,
}

pub fn parse_png(bytes: &[u8]) -> Result<PngInfo, MetaForgeError> {
    const SIGNATURE: &[u8; 8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 8 || &bytes[0..8] != SIGNATURE {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-png".to_string(),
            expected: "PNG signature (89 50 4E 47 0D 0A 1A 0A)".to_string(),
        });
    }

    let mut chunks = Vec::new();
    let mut header = None;
    let mut palette_entries = None;
    let mut text_metadata = Vec::new();
    let mut resolution = None;
    let mut significant_bits = None;
    let mut background_color = None;
    let mut offset = None;
    let mut scale = None;
    let mut last_modified = None;
    let mut icc_profile_name = None;
    let mut raw_exif_payload: Option<Vec<u8>> = None;

    let mut pos = 8;
    while pos + 8 <= bytes.len() {
        let chunk_start = pos;
        let length = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
        pos += 4;

        let mut type_bytes = [0u8; 4];
        type_bytes.copy_from_slice(&bytes[pos..pos + 4]);
        let type_name = String::from_utf8_lossy(&type_bytes).to_string();
        pos += 4;

        if pos + length + 4 > bytes.len() {
            return Err(MetaForgeError::TruncatedStructure {
                container: "PNG",
                offset: pos + length + 4,
                length: bytes.len(),
            });
        }

        let payload = &bytes[pos..pos + length];
        pos += length;

        let crc = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
        pos += 4;

        // Verify CRC (covers type + data)
        let mut crc_check_data = Vec::with_capacity(4 + length);
        crc_check_data.extend_from_slice(&type_bytes);
        crc_check_data.extend_from_slice(payload);
        let calculated_crc = crc32(&crc_check_data);
        let crc_valid = calculated_crc == crc;

        chunks.push(PngChunk {
            chunk_type: type_bytes,
            type_name: type_name.clone(),
            offset: chunk_start,
            length: length + 12,
            crc,
            crc_valid,
        });

        match type_name.as_str() {
            "IHDR" => {
                if payload.len() >= 13 {
                    header = Some(PngHeader {
                        width: u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]),
                        height: u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]),
                        bit_depth: payload[8],
                        color_type: payload[9],
                        compression_method: payload[10],
                        filter_method: payload[11],
                        interlace_method: payload[12],
                    });
                }
            }
            "PLTE" => {
                palette_entries = Some(payload.len() / 3);
            }
            "tEXt" => {
                if let Some(null_pos) = payload.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&payload[0..null_pos]).to_string();
                    let val = String::from_utf8_lossy(&payload[null_pos + 1..]).to_string();
                    text_metadata.push((key, val));
                }
            }
            "iTXt" => {
                if let Some(null_pos) = payload.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&payload[0..null_pos]).to_string();
                    let rest = &payload[null_pos + 1..];
                    if rest.len() >= 2 {
                        let comp_flag = rest[0];
                        if comp_flag == 0 {
                            if let Some(lang_null) = rest[2..].iter().position(|&b| b == 0) {
                                let after_lang = &rest[2 + lang_null + 1..];
                                if let Some(trans_null) = after_lang.iter().position(|&b| b == 0) {
                                    let val_bytes = &after_lang[trans_null + 1..];
                                    let val = String::from_utf8_lossy(val_bytes).to_string();
                                    text_metadata.push((key, val));
                                }
                            }
                        }
                    }
                }
            }
            "pHYs" => {
                if payload.len() >= 9 {
                    let ppu_x = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let ppu_y = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    let unit_specifier = payload[8];
                    resolution = Some(PngResolution { ppu_x, ppu_y, unit_specifier });
                }
            }
            "sBIT" => {
                significant_bits = Some(PngSignificantBits { bits: payload.to_vec() });
            }
            "bKGD" => {
                if payload.len() == 1 {
                    background_color = Some(PngBackgroundColor {
                        gray: None,
                        rgb: None,
                        palette_index: Some(payload[0]),
                    });
                } else if payload.len() == 2 {
                    let gray = u16::from_be_bytes([payload[0], payload[1]]);
                    background_color = Some(PngBackgroundColor {
                        gray: Some(gray),
                        rgb: None,
                        palette_index: None,
                    });
                } else if payload.len() == 6 {
                    let r = u16::from_be_bytes([payload[0], payload[1]]);
                    let g = u16::from_be_bytes([payload[2], payload[3]]);
                    let b = u16::from_be_bytes([payload[4], payload[5]]);
                    background_color = Some(PngBackgroundColor {
                        gray: None,
                        rgb: Some((r, g, b)),
                        palette_index: None,
                    });
                }
            }
            "oFFs" => {
                if payload.len() >= 9 {
                    let offset_x = i32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let offset_y = i32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    let unit_specifier = payload[8];
                    offset = Some(PngOffset { offset_x, offset_y, unit_specifier });
                }
            }
            "sCAL" => {
                if payload.len() >= 3 {
                    let unit_specifier = payload[0];
                    let rest = &payload[1..];
                    if let Some(idx) = rest.iter().position(|&b| b == 0) {
                        if let (Ok(x_str), Ok(y_str)) = (
                            std::str::from_utf8(&rest[..idx]),
                            std::str::from_utf8(&rest[idx + 1..]),
                        ) {
                            scale = Some(PngScale {
                                unit_specifier,
                                width_scale: x_str.to_string(),
                                height_scale: y_str.to_string(),
                            });
                        }
                    }
                }
            }
            "tIME" => {
                if payload.len() >= 7 {
                    let year = u16::from_be_bytes([payload[0], payload[1]]);
                    let month = payload[2];
                    let day = payload[3];
                    let hour = payload[4];
                    let minute = payload[5];
                    let second = payload[6];
                    last_modified = Some(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second));
                }
            }
            "iCCP" => {
                if let Some(null_pos) = payload.iter().position(|&b| b == 0) {
                    icc_profile_name = Some(String::from_utf8_lossy(&payload[0..null_pos]).to_string());
                }
            }
            "eXIf" => {
                raw_exif_payload = Some(payload.to_vec());
            }
            "IEND" => {
                break;
            }
            _ => {}
        }
    }

    let mut metadata = ExifMetadata::default();
    if let Some(exif_bytes) = raw_exif_payload {
        let mut cursor = Cursor::new(exif_bytes);
        let exif_reader = kamadak_exif::Reader::new();
        if let Ok(exif_data) = exif_reader.read_from_container(&mut cursor) {
            metadata = crate::common::extract_exif_metadata(&exif_data);
        }
    }

    let official_end_offset = chunks.iter().map(|c| c.offset + c.length).max().unwrap_or(bytes.len());

    Ok(PngInfo {
        chunks,
        header,
        palette_entries,
        text_metadata,
        resolution,
        significant_bits,
        background_color,
        offset,
        scale,
        last_modified,
        icc_profile_name,
        metadata,
        official_end_offset,
    })
}
