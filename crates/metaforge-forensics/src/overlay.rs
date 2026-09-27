/// Metadata describing an appended overlay payload
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OverlayInfo {
    pub has_overlay: bool,
    pub logical_size: usize,
    pub physical_size: usize,
    pub overlay_size: usize,
}

/// Detects if a container has extra bytes appended past its logical end-of-file.
pub fn detect_overlay(bytes: &[u8], official_end_offset: usize) -> OverlayInfo {
    let physical_size = bytes.len();
    let has_overlay = physical_size > official_end_offset;
    let overlay_size = if has_overlay { physical_size - official_end_offset } else { 0 };

    OverlayInfo {
        has_overlay,
        logical_size: official_end_offset,
        physical_size,
        overlay_size,
    }
}
