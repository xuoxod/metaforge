use metaforge_core::{ContainerType, MetaForgeError};
use std::path::Path;

pub const PNG_SIGNATURE: &[u8; 8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub const JPEG_SIGNATURE: &[u8; 2] = &[0xFF, 0xD8];

/// Sniffs the container type from magic bytes only without filesystem access.
pub fn detect_container_bytes(bytes: &[u8]) -> Option<ContainerType> {
    if bytes.len() >= 8 && &bytes[0..8] == PNG_SIGNATURE {
        return Some(ContainerType::Png);
    }
    if bytes.len() >= 2 && &bytes[0..2] == JPEG_SIGNATURE {
        return Some(ContainerType::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ContainerType::Webp);
    }
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return Some(ContainerType::Gif);
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        let brand = &bytes[8..12];
        if brand == b"heic" || brand == b"heix" || brand == b"hevc" || brand == b"mif1" || brand == b"msf1" {
            return Some(ContainerType::Heic);
        }
    }
    None
}

/// Detects the container type using magic bytes, falling back to file extension.
pub fn detect_container_type(bytes: &[u8], path: &Path) -> Result<ContainerType, MetaForgeError> {
    if let Some(container) = detect_container_bytes(bytes) {
        return Ok(container);
    }

    // Fallback to extension check
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("png") => Ok(ContainerType::Png),
        Some("jpg") | Some("jpeg") => Ok(ContainerType::Jpeg),
        Some("webp") => Ok(ContainerType::Webp),
        Some("gif") => Ok(ContainerType::Gif),
        Some("heic") | Some("heif") => Ok(ContainerType::Heic),
        _ => Err(MetaForgeError::InvalidSignature {
            path: path.to_string_lossy().into_owned(),
            expected: "JPEG, PNG, WebP, GIF, or HEIC magic bytes/extension".to_string(),
        }),
    }
}
