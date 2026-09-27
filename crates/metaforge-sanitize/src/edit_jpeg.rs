use metaforge_core::error::{MetaForgeError, Result};

/// Inserts or replaces the JPEG COM (0xFF 0xFE) comment marker in a JPEG byte stream.
///
/// If `comment` is empty, any existing COM markers are removed.
/// Returns the updated JPEG byte stream.
pub fn set_jpeg_comment(bytes: &[u8], comment: &str) -> Result<Vec<u8>> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "0xFFD8 (JPEG SOI)".to_string(),
        });
    }

    let comment_bytes = comment.as_bytes();
    if comment_bytes.len() > 65533 {
        return Err(MetaForgeError::SegmentOverflow {
            size: comment_bytes.len(),
        });
    }

    let mut output = Vec::with_capacity(bytes.len() + comment_bytes.len() + 4);
    output.extend_from_slice(&[0xFF, 0xD8]); // SOI

    let mut i = 2;
    let mut comment_inserted = false;

    // Check if the next segment is APP0 (JFIF). If so, we place COM after APP0.
    // If not APP0, we can insert COM right after SOI.
    let mut has_app0 = false;
    if i + 4 <= bytes.len() && bytes[i] == 0xFF && bytes[i + 1] == 0xE0 {
        has_app0 = true;
    }

    while i < bytes.len() {
        if bytes[i] != 0xFF {
            // Reached scan data or unaligned byte, copy remainder
            output.extend_from_slice(&bytes[i..]);
            break;
        }

        // Skip consecutive 0xFF padding
        while i < bytes.len() && bytes[i] == 0xFF {
            i += 1;
        }
        if i >= bytes.len() {
            output.push(0xFF);
            break;
        }

        let marker = bytes[i];
        i += 1;

        // Standalone markers without length: SOI (0xD8), RST0-7 (0xD0..=0xD7), EOI (0xD9)
        if marker == 0xD8 {
            continue;
        }
        if (0xD0..=0xD7).contains(&marker) {
            output.extend_from_slice(&[0xFF, marker]);
            continue;
        }
        if marker == 0xD9 {
            // EOI
            if !comment_inserted && !comment.is_empty() {
                write_com_segment(&mut output, comment_bytes);
                comment_inserted = true;
            }
            output.extend_from_slice(&[0xFF, 0xD9]);
            // Preserve any trailing overlay
            if i < bytes.len() {
                output.extend_from_slice(&bytes[i..]);
            }
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

        let _seg_payload = &bytes[i + 2..i + seg_len];

        // If marker is COM (0xFE), omit it (we replace or strip it)
        if marker == 0xFE {
            i += seg_len;
            continue;
        }

        // If we hit SOS (0xDA) and haven't inserted yet, insert comment before SOS
        if marker == 0xDA && !comment_inserted && !comment.is_empty() {
            write_com_segment(&mut output, comment_bytes);
            comment_inserted = true;
        }

        // Copy this segment to output
        output.extend_from_slice(&[0xFF, marker]);
        output.extend_from_slice(&bytes[i..i + seg_len]);
        i += seg_len;

        // If this was APP0 and we need to insert comment right after it
        if has_app0 && marker == 0xE0 && !comment_inserted && !comment.is_empty() {
            write_com_segment(&mut output, comment_bytes);
            comment_inserted = true;
        }
    }

    if !comment_inserted && !comment.is_empty() {
        // Fallback: If not inserted (e.g. Minimal JPEG), insert before EOI
        let mut final_out = Vec::with_capacity(output.len() + comment_bytes.len() + 4);
        if output.ends_with(&[0xFF, 0xD9]) {
            let eoi_pos = output.len() - 2;
            final_out.extend_from_slice(&output[..eoi_pos]);
            write_com_segment(&mut final_out, comment_bytes);
            final_out.extend_from_slice(&[0xFF, 0xD9]);
            output = final_out;
        }
    }

    Ok(output)
}

fn write_com_segment(output: &mut Vec<u8>, comment_bytes: &[u8]) {
    let seg_len = (comment_bytes.len() + 2) as u16;
    output.extend_from_slice(&[0xFF, 0xFE]);
    output.extend_from_slice(&seg_len.to_be_bytes());
    output.extend_from_slice(comment_bytes);
}

/// Reads all COM markers from a JPEG byte stream.
pub fn read_jpeg_comments(bytes: &[u8]) -> Result<Vec<String>> {
    let mut comments = Vec::new();
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(MetaForgeError::InvalidSignature {
            path: "in-memory-stream".to_string(),
            expected: "0xFFD8".to_string(),
        });
    }

    let mut i = 2;
    while i < bytes.len() {
        if bytes[i] != 0xFF {
            break;
        }
        while i < bytes.len() && bytes[i] == 0xFF {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let marker = bytes[i];
        i += 1;

        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            break;
        }

        if i + 2 > bytes.len() {
            break;
        }
        let seg_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
        if seg_len < 2 || i + seg_len > bytes.len() {
            break;
        }

        if marker == 0xFE {
            let text_bytes = &bytes[i + 2..i + seg_len];
            let text = String::from_utf8_lossy(text_bytes).trim().to_string();
            comments.push(text);
        }

        i += seg_len;
    }

    Ok(comments)
}
