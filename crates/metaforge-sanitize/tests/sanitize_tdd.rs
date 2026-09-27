use metaforge_sanitize::{read_jpeg_comments, read_png_text, scrub_image, set_jpeg_comment, set_png_text, truncate_overlay};

fn create_minimal_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, // SOI
        0xFF, 0xE0, 0x00, 0x10, // APP0
        b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, // SOS
        0x12, 0x34, 0x56, // Scan data
        0xFF, 0xD9, // EOI
    ]
}

fn create_minimal_png() -> Vec<u8> {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // Header
    // IHDR chunk (13 bytes data)
    let ihdr_data = [
        0x00, 0x00, 0x00, 0x01, // width 1
        0x00, 0x00, 0x00, 0x01, // height 1
        0x08, 0x02, 0x00, 0x00, 0x00, // 8-bit truecolor
    ];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(b"IHDR");
    ihdr.extend_from_slice(&ihdr_data);
    let crc = metaforge_core::crc::crc32(&ihdr);
    png.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
    png.extend_from_slice(&ihdr);
    png.extend_from_slice(&crc.to_be_bytes());

    // IEND chunk
    let mut iend = Vec::new();
    iend.extend_from_slice(b"IEND");
    let crc = metaforge_core::crc::crc32(&iend);
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(&iend);
    png.extend_from_slice(&crc.to_be_bytes());

    png
}

#[test]
fn test_jpeg_comment_lifecycle() {
    let base_jpeg = create_minimal_jpeg();
    assert!(read_jpeg_comments(&base_jpeg).unwrap().is_empty());

    // 1. Set comment
    let with_comment = set_jpeg_comment(&base_jpeg, "Sovereign Rust Engine").unwrap();
    let comments = read_jpeg_comments(&with_comment).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0], "Sovereign Rust Engine");

    // 2. Replace comment
    let updated = set_jpeg_comment(&with_comment, "Updated Sovereign Metadata").unwrap();
    let comments = read_jpeg_comments(&updated).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0], "Updated Sovereign Metadata");

    // 3. Remove comment
    let stripped = set_jpeg_comment(&updated, "").unwrap();
    let comments = read_jpeg_comments(&stripped).unwrap();
    assert!(comments.is_empty());
}

#[test]
fn test_png_text_lifecycle() {
    let base_png = create_minimal_png();
    assert!(read_png_text(&base_png).unwrap().is_empty());

    // 1. Set text
    let with_text = set_png_text(&base_png, "Author", "Sovereign Developer").unwrap();
    let texts = read_png_text(&with_text).unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], ("Author".to_string(), "Sovereign Developer".to_string()));

    // 2. Replace text
    let updated = set_png_text(&with_text, "Author", "Lead Architect").unwrap();
    let texts = read_png_text(&updated).unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], ("Author".to_string(), "Lead Architect".to_string()));

    // 3. Delete text
    let stripped = set_png_text(&updated, "Author", "").unwrap();
    let texts = read_png_text(&stripped).unwrap();
    assert!(texts.is_empty());
}

#[test]
fn test_jpeg_overlay_truncation() {
    let mut jpeg = create_minimal_jpeg();
    jpeg.extend_from_slice(b"MALICIOUS_TRAILING_OVERLAY_BYTES");

    let (truncated, had_overlay) = truncate_overlay(&jpeg).unwrap();
    assert!(had_overlay);
    assert_eq!(truncated.len(), jpeg.len() - 32);
    assert!(truncated.ends_with(&[0xFF, 0xD9]));
}

#[test]
fn test_png_overlay_truncation() {
    let mut png = create_minimal_png();
    png.extend_from_slice(b"MALICIOUS_TRAILING_OVERLAY_BYTES");

    let (truncated, had_overlay) = truncate_overlay(&png).unwrap();
    assert!(had_overlay);
    assert_eq!(truncated.len(), png.len() - 32);
}

#[test]
fn test_full_jpeg_scrub() {
    let mut jpeg = create_minimal_jpeg();
    // Add comment
    jpeg = set_jpeg_comment(&jpeg, "Secret Private Info").unwrap();
    // Add overlay
    jpeg.extend_from_slice(b"TRAILING_SECRET");

    let (scrubbed, report) = scrub_image(&jpeg).unwrap();
    assert!(report.overlay_truncated);
    assert!(report.metadata_stripped);
    assert!(report.bytes_removed > 0);

    // Verify comment is gone
    let comments = read_jpeg_comments(&scrubbed).unwrap();
    assert!(comments.is_empty());
    // Verify overlay is gone
    assert!(scrubbed.ends_with(&[0xFF, 0xD9]));
}
