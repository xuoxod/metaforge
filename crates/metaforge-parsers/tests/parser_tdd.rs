use metaforge_parsers::{detect_container_type, parse_jpeg, parse_png};
use metaforge_core::ContainerType;
use std::path::Path;

#[test]
fn test_detect_all_container_types() {
    assert_eq!(detect_container_type(&[0xFF, 0xD8, 0xFF, 0xE0], Path::new("img.jpg")).unwrap(), ContainerType::Jpeg);
    assert_eq!(detect_container_type(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], Path::new("img.png")).unwrap(), ContainerType::Png);
    assert_eq!(detect_container_type(b"RIFF\0\0\0\0WEBP", Path::new("img.webp")).unwrap(), ContainerType::Webp);
    assert_eq!(detect_container_type(b"GIF89a\0\0", Path::new("img.gif")).unwrap(), ContainerType::Gif);
    assert_eq!(detect_container_type(b"\0\0\0\x18ftypheic", Path::new("img.heic")).unwrap(), ContainerType::Heic);
}

#[test]
fn test_jpeg_basic_parsing() {
    let mut jpeg = vec![0xFF, 0xD8]; // SOI
    jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x07, b'J', b'F', b'I', b'F', 0]); // APP0
    jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI

    let info = parse_jpeg(&jpeg).unwrap();
    assert_eq!(info.segments.len(), 2);
    assert_eq!(info.segments[0].marker, 0xE0);
    assert_eq!(info.segments[1].marker, 0xD9);
}

#[test]
fn test_png_basic_parsing() {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // Header
    
    // IHDR chunk (13 bytes)
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&800u32.to_be_bytes()); // Width
    png.extend_from_slice(&600u32.to_be_bytes()); // Height
    png.extend_from_slice(&[8, 2, 0, 0, 0]); // bit_depth, color_type, comp, filter, interlace
    png.extend_from_slice(&[0, 0, 0, 0]); // CRC placeholder (doesn't fail parse)

    // IEND chunk
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&[0, 0, 0, 0]);

    let info = parse_png(&png).unwrap();
    assert_eq!(info.chunks.len(), 2);
    assert!(info.header.is_some());
    let hdr = info.header.unwrap();
    assert_eq!(hdr.width, 800);
    assert_eq!(hdr.height, 600);
}
