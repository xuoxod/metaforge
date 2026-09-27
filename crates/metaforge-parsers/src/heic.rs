#![allow(clippy::too_many_arguments, clippy::collapsible_match)]
use crate::common::ExifMetadata;
use exif as kamadak_exif;
use metaforge_core::MetaForgeError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HeicBox {
    pub box_type: String,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeicInfo {
    pub boxes: Vec<HeicBox>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub metadata: ExifMetadata,
    pub official_end_offset: usize,
}

pub fn parse_heic(bytes: &[u8]) -> Result<HeicInfo, MetaForgeError> {
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-heic".to_string(),
            expected: "HEIC ftyp box at offset 4".to_string(),
        });
    }

    let mut boxes = Vec::new();
    let mut width = None;
    let mut height = None;
    let mut exif_item_id = None;
    let mut exif_extents = Vec::new();

    // Box scanning helper with recursion guardrails
    fn scan_boxes(
        bytes: &[u8],
        start: usize,
        end: usize,
        depth: usize,
        boxes: &mut Vec<HeicBox>,
        width: &mut Option<u32>,
        height: &mut Option<u32>,
        exif_item_id: &mut Option<u32>,
        exif_extents: &mut Vec<(u64, u64)>,
    ) {
        if depth > 32 {
            return;
        }

        let mut pos = start;
        while pos + 8 <= end {
            let box_start = pos;
            let len = u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
            let mut type_bytes = [0u8; 4];
            type_bytes.copy_from_slice(&bytes[pos + 4..pos + 8]);
            let box_type = String::from_utf8_lossy(&type_bytes).to_string();

            if len == 0 {
                // Extends to end of file
                boxes.push(HeicBox {
                    box_type,
                    offset: box_start,
                    length: end - box_start,
                });
                break;
            }

            if len == 1 {
                // 64-bit large box (large length follows in next 8 bytes)
                if pos + 16 > end {
                    break;
                }
                let large_len = u64::from_be_bytes([
                    bytes[pos + 8], bytes[pos + 9], bytes[pos + 10], bytes[pos + 11],
                    bytes[pos + 12], bytes[pos + 13], bytes[pos + 14], bytes[pos + 15],
                ]) as usize;

                boxes.push(HeicBox {
                    box_type: box_type.clone(),
                    offset: box_start,
                    length: large_len,
                });
                pos += large_len;
                continue;
            }

            if len < 8 || pos + len > end {
                // Malformed box length
                break;
            }

            boxes.push(HeicBox {
                box_type: box_type.clone(),
                offset: box_start,
                length: len,
            });

            let payload_start = pos + 8;
            let payload_end = pos + len;

            match box_type.as_str() {
                "meta" => {
                    // meta box is a FullBox with 4 bytes of version/flags
                    if payload_start + 4 <= payload_end {
                        scan_boxes(bytes, payload_start + 4, payload_end, depth + 1, boxes, width, height, exif_item_id, exif_extents);
                    }
                }
                "iprp" | "ipco" => {
                    scan_boxes(bytes, payload_start, payload_end, depth + 1, boxes, width, height, exif_item_id, exif_extents);
                }
                "ispe" => {
                    // Image Spatial Extents (width and height)
                    // FullBox: 4 bytes version/flags, then 4 bytes width, 4 bytes height
                    if payload_start + 12 <= payload_end && width.is_none() {
                        let w = u32::from_be_bytes([
                            bytes[payload_start + 4], bytes[payload_start + 5],
                            bytes[payload_start + 6], bytes[payload_start + 7],
                        ]);
                        let h = u32::from_be_bytes([
                            bytes[payload_start + 8], bytes[payload_start + 9],
                            bytes[payload_start + 10], bytes[payload_start + 11],
                        ]);
                        *width = Some(w);
                        *height = Some(h);
                    }
                }
                "iinf" => {
                    // Item Info Box
                    if payload_start + 6 <= payload_end {
                        let version = bytes[payload_start];
                        let mut entry_pos = if version == 0 { payload_start + 6 } else { payload_start + 8 };
                        while entry_pos + 8 <= payload_end {
                            let entry_len = u32::from_be_bytes([bytes[entry_pos], bytes[entry_pos + 1], bytes[entry_pos + 2], bytes[entry_pos + 3]]) as usize;
                            if entry_len < 8 || entry_pos + entry_len > payload_end {
                                break;
                            }
                            if &bytes[entry_pos + 4..entry_pos + 8] == b"infe" && entry_pos + 16 <= payload_end {
                                let item_id = u16::from_be_bytes([bytes[entry_pos + 12], bytes[entry_pos + 13]]) as u32;
                                if entry_pos + 20 <= payload_end && &bytes[entry_pos + 16..entry_pos + 20] == b"Exif" {
                                    *exif_item_id = Some(item_id);
                                }
                            }
                            entry_pos += entry_len;
                        }
                    }
                }
                "iloc" => {
                    // Item Location Box
                    if payload_start + 8 <= payload_end {
                        let offset_size = (bytes[payload_start + 4] >> 4) as usize;
                        let length_size = (bytes[payload_start + 4] & 0x0F) as usize;
                        let base_offset_size = (bytes[payload_start + 5] >> 4) as usize;

                        let mut iloc_pos = payload_start + 8;
                        let item_count = u16::from_be_bytes([bytes[payload_start + 6], bytes[payload_start + 7]]) as usize;

                        for _ in 0..item_count {
                            if iloc_pos + 4 > payload_end {
                                break;
                            }
                            let item_id = u16::from_be_bytes([bytes[iloc_pos], bytes[iloc_pos + 1]]) as u32;
                            let data_ref_idx = u16::from_be_bytes([bytes[iloc_pos + 2], bytes[iloc_pos + 3]]);
                            let _ = data_ref_idx;
                            iloc_pos += 4;

                            if iloc_pos + base_offset_size > payload_end {
                                break;
                            }
                            iloc_pos += base_offset_size;

                            if iloc_pos + 2 > payload_end {
                                break;
                            }
                            let extent_count = u16::from_be_bytes([bytes[iloc_pos], bytes[iloc_pos + 1]]) as usize;
                            iloc_pos += 2;

                            for _ in 0..extent_count {
                                if iloc_pos + offset_size + length_size > payload_end {
                                    break;
                                }
                                let mut ext_offset = 0u64;
                                for j in 0..offset_size {
                                    ext_offset = (ext_offset << 8) | (bytes[iloc_pos + j] as u64);
                                }
                                iloc_pos += offset_size;

                                let mut ext_len = 0u64;
                                for j in 0..length_size {
                                    ext_len = (ext_len << 8) | (bytes[iloc_pos + j] as u64);
                                }
                                iloc_pos += length_size;

                                if Some(item_id) == *exif_item_id {
                                    exif_extents.push((ext_offset, ext_len));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            pos += len;
        }
    }

    scan_boxes(bytes, 0, bytes.len(), 0, &mut boxes, &mut width, &mut height, &mut exif_item_id, &mut exif_extents);

    let mut metadata = ExifMetadata {
        width,
        height,
        ..Default::default()
    };

    if let Some(&(ext_offset, ext_len)) = exif_extents.first() {
        let off = ext_offset as usize;
        let l = ext_len as usize;
        if off + l <= bytes.len() && l > 4 {
            let exif_header_offset = u32::from_be_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]) as usize;
            let real_start = off + 4 + exif_header_offset;
            if real_start < off + l {
                let payload = &bytes[real_start..off + l];
                let reader = kamadak_exif::Reader::new();
                if let Ok(exif_data) = reader.read_raw(payload.to_vec()) {
                    metadata = crate::common::extract_exif_metadata(&exif_data);
                    metadata.width = width;
                    metadata.height = height;
                }
            }
        }
    }

    let official_end_offset = boxes.iter().map(|b| b.offset + b.length).max().unwrap_or(bytes.len());

    Ok(HeicInfo {
        boxes,
        width,
        height,
        metadata,
        official_end_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heic_invalid() {
        assert!(parse_heic(b"not a heic file").is_err());
    }

    #[test]
    fn test_parse_heic_valid() {
        let mut heic = vec![
            0x00, 0x00, 0x00, 0x18, // Length 24
            b'f', b't', b'y', b'p',
            b'h', b'e', b'i', b'c', // Major brand
            0x00, 0x00, 0x00, 0x00, // Minor version
            b'm', b'i', b'f', b'1', // Compatible brand 1
            b'h', b'e', b'i', b'c', // Compatible brand 2
        ];

        // Append a meta box
        let meta_start = heic.len();
        heic.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x14, // Length 20
            b'm', b'e', b't', b'a',
            0x00, 0x00, 0x00, 0x00, // Version + Flags
            0x00, 0x00, 0x00, 0x08, // Inner box length 8
            b'h', b'd', b'l', b'r', // Handler box
        ]);
        let _ = meta_start;

        let info = parse_heic(&heic).expect("test HEIC should parse");
        assert!(info.boxes.len() >= 2);
        assert_eq!(info.boxes[0].box_type, "ftyp");
        assert_eq!(info.boxes[1].box_type, "meta");
    }
}
