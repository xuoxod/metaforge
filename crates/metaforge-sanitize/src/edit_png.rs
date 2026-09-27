use metaforge_core::crc::crc32;
use metaforge_core::error::{MetaForgeError, Result};

const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// Inserts or replaces a `tEXt` metadata chunk in a PNG image.
///
/// If `text` is empty, any existing `tEXt` chunk matching `keyword` is removed.
pub fn set_png_text(bytes: &[u8], keyword: &str, text: &str) -> Result<Vec<u8>> {
    if bytes.len() < 8 || &bytes[0..8] != PNG_MAGIC {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "PNG Header (89 50 4E 47 ...)".to_string(),
        });
    }

    if keyword.is_empty() || keyword.len() > 79 {
        return Err(MetaForgeError::CorruptedChunk {
            container: "PNG",
            chunk_type: "tEXt".to_string(),
            detail: "Keyword must be between 1 and 79 characters".to_string(),
        });
    }

    let kw_bytes = keyword.as_bytes();
    let text_bytes = text.as_bytes();

    let mut output = Vec::with_capacity(bytes.len() + kw_bytes.len() + text_bytes.len() + 32);
    output.extend_from_slice(PNG_MAGIC);

    let mut offset = 8;
    let mut inserted = false;

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

        let chunk_data = &bytes[offset + 8..offset + 8 + length];

        // Check if this chunk is tEXt with the matching keyword
        if chunk_type == b"tEXt" {
            if let Some(null_pos) = chunk_data.iter().position(|&b| b == 0) {
                let existing_kw = &chunk_data[..null_pos];
                if existing_kw == kw_bytes {
                    // Skip existing chunk
                    offset += total_chunk_len;
                    continue;
                }
            }
        }

        // If we hit IEND, insert our new tEXt chunk right before IEND
        if chunk_type == b"IEND" && !inserted && !text.is_empty() {
            write_text_chunk(&mut output, kw_bytes, text_bytes);
            inserted = true;
        }

        // Copy chunk
        output.extend_from_slice(&bytes[offset..offset + total_chunk_len]);
        offset += total_chunk_len;

        if chunk_type == b"IEND" {
            // Append any trailing overlay after IEND
            if offset < bytes.len() {
                output.extend_from_slice(&bytes[offset..]);
            }
            break;
        }
    }

    if !inserted && !text.is_empty() {
        // Fallback if IEND wasn't encountered
        write_text_chunk(&mut output, kw_bytes, text_bytes);
    }

    Ok(output)
}

fn write_text_chunk(output: &mut Vec<u8>, kw_bytes: &[u8], text_bytes: &[u8]) {
    let mut chunk_content = Vec::with_capacity(4 + kw_bytes.len() + 1 + text_bytes.len());
    chunk_content.extend_from_slice(b"tEXt");
    chunk_content.extend_from_slice(kw_bytes);
    chunk_content.push(0x00); // null separator
    chunk_content.extend_from_slice(text_bytes);

    let data_len = (kw_bytes.len() + 1 + text_bytes.len()) as u32;
    let crc = crc32(&chunk_content);

    output.extend_from_slice(&data_len.to_be_bytes());
    output.extend_from_slice(&chunk_content);
    output.extend_from_slice(&crc.to_be_bytes());
}

/// Reads all `tEXt` chunks from a PNG byte stream into key-value pairs.
pub fn read_png_text(bytes: &[u8]) -> Result<Vec<(String, String)>> {
    if bytes.len() < 8 || &bytes[0..8] != PNG_MAGIC {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "PNG Header".to_string(),
        });
    }

    let mut results = Vec::new();
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

        if offset + total_chunk_len > bytes.len() {
            break;
        }

        let chunk_data = &bytes[offset + 8..offset + 8 + length];
        if chunk_type == b"tEXt" {
            if let Some(null_pos) = chunk_data.iter().position(|&b| b == 0) {
                let kw = String::from_utf8_lossy(&chunk_data[..null_pos]).to_string();
                let txt = String::from_utf8_lossy(&chunk_data[null_pos + 1..]).to_string();
                results.push((kw, txt));
            }
        }

        if chunk_type == b"IEND" {
            break;
        }

        offset += total_chunk_len;
    }

    Ok(results)
}
