#![allow(clippy::field_reassign_with_default, clippy::collapsible_if)]
use crate::common::ExifMetadata;
use metaforge_core::MetaForgeError;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GifBlock {
    pub block_type: String,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GifInfo {
    pub blocks: Vec<GifBlock>,
    pub width: u16,
    pub height: u16,
    pub comment: Option<String>,
    pub metadata: ExifMetadata,
    pub official_end_offset: usize,
}

pub fn parse_gif(bytes: &[u8]) -> Result<GifInfo, MetaForgeError> {
    if bytes.len() < 13 || (&bytes[0..6] != b"GIF87a" && &bytes[0..6] != b"GIF89a") {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-gif".to_string(),
            expected: "GIF signature (GIF87a or GIF89a)".to_string(),
        });
    }

    let mut blocks = Vec::new();
    let mut comments = Vec::new();
    let mut raw_xmp_payload = None;

    let width = u16::from_le_bytes([bytes[6], bytes[7]]);
    let height = u16::from_le_bytes([bytes[8], bytes[9]]);

    let packed = bytes[10];
    let has_gct = (packed & 0x80) != 0;
    let gct_size_indicator = packed & 0x07;
    let gct_len = if has_gct { 3 * (1 << (gct_size_indicator + 1)) } else { 0 };

    let mut pos = 13 + gct_len;

    while pos < bytes.len() {
        let block_start = pos;
        let sentinel = bytes[pos];
        pos += 1;

        match sentinel {
            0x2C => {
                // Image Descriptor
                if pos + 9 > bytes.len() {
                    break;
                }
                let packed_local = bytes[pos + 8];
                pos += 9;
                let has_lct = (packed_local & 0x80) != 0;
                let lct_size_indicator = packed_local & 0x07;
                let lct_len = if has_lct { 3 * (1 << (lct_size_indicator + 1)) } else { 0 };
                pos += lct_len;

                // LZW Minimum Code Size
                if pos < bytes.len() {
                    pos += 1;
                }

                // Sub-blocks
                while pos < bytes.len() {
                    let sub_block_len = bytes[pos] as usize;
                    pos += 1;
                    if sub_block_len == 0 {
                        break;
                    }
                    pos += sub_block_len;
                }

                blocks.push(GifBlock {
                    block_type: "Image Descriptor".to_string(),
                    offset: block_start,
                    length: pos - block_start,
                });
            }
            0x21 => {
                // Extension Introducer
                if pos >= bytes.len() {
                    break;
                }
                let ext_label = bytes[pos];
                pos += 1;

                let ext_name = match ext_label {
                    0xF9 => "Graphic Control Extension",
                    0xFE => "Comment Extension",
                    0x01 => "Plain Text Extension",
                    0xFF => "Application Extension",
                    _ => "Extension",
                };

                let mut ext_payload = Vec::new();

                // Read sub-blocks
                while pos < bytes.len() {
                    let sub_block_len = bytes[pos] as usize;
                    pos += 1;
                    if sub_block_len == 0 {
                        break;
                    }
                    if pos + sub_block_len <= bytes.len() {
                        ext_payload.extend_from_slice(&bytes[pos..pos + sub_block_len]);
                    }
                    pos += sub_block_len;
                }

                if ext_label == 0xFE {
                    if let Ok(c) = std::str::from_utf8(&ext_payload) {
                        comments.push(c.to_string());
                    }
                } else if ext_label == 0xFF {
                    // Check for XMP Data in Application Extension: "XMP DataXMP"
                    if ext_payload.starts_with(b"XMP DataXMP") {
                        if let Ok(xmp_str) = std::str::from_utf8(&ext_payload[11..]) {
                            raw_xmp_payload = Some(xmp_str.trim().to_string());
                        }
                    }
                }

                blocks.push(GifBlock {
                    block_type: ext_name.to_string(),
                    offset: block_start,
                    length: pos - block_start,
                });
            }
            0x3B => {
                // Trailer - logical end of GIF
                blocks.push(GifBlock {
                    block_type: "Trailer".to_string(),
                    offset: block_start,
                    length: 1,
                });
                break;
            }
            _ => {
                break;
            }
        }
    }

    let mut metadata = ExifMetadata::default();
    metadata.xmp = raw_xmp_payload;
    metadata.width = Some(width as u32);
    metadata.height = Some(height as u32);

    let comment = if !comments.is_empty() {
        Some(comments.join("; "))
    } else {
        None
    };

    let official_end_offset = blocks.iter().map(|b| b.offset + b.length).max().unwrap_or(bytes.len());

    Ok(GifInfo {
        blocks,
        width,
        height,
        comment,
        metadata,
        official_end_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gif_invalid() {
        assert!(parse_gif(b"not a gif").is_err());
    }

    #[test]
    fn test_parse_gif_valid() {
        let mut gif = b"GIF89a".to_vec();
        gif.extend_from_slice(&100u16.to_le_bytes()); // Width
        gif.extend_from_slice(&200u16.to_le_bytes()); // Height
        gif.push(0x00); // GCT Flag = false
        gif.push(0x00); // Background Color Index
        gif.push(0x00); // Pixel Aspect Ratio

        // Add Comment Extension: 0x21 0xFE 0x05 "hello" 0x00
        gif.extend_from_slice(&[0x21, 0xFE, 0x05]);
        gif.extend_from_slice(b"hello");
        gif.push(0x00);

        // Add Trailer: 0x3B
        gif.push(0x3B);

        let info = parse_gif(&gif).expect("test GIF should parse");
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 200);
        assert_eq!(info.comment, Some("hello".to_string()));
        assert_eq!(info.blocks.len(), 2);
    }
}
