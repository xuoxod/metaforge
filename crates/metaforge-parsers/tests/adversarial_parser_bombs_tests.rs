use metaforge_parsers::{parse_heic, parse_jpeg, parse_png, parse_webp};

#[test]
fn test_adversarial_jpeg_truncated_segment_bomb() {
    // SOI + APP1 marker claiming 65000 bytes with only 10 bytes provided
    let mut bomb = vec![0xFF, 0xD8, 0xFF, 0xE1, 0xFD, 0xE8]; // length 65000
    bomb.extend_from_slice(b"12345678");

    let result = parse_jpeg(&bomb);
    assert!(result.is_err(), "Parser must reject truncated segment bomb");
}

#[test]
fn test_adversarial_png_billion_byte_allocation_bomb() {
    // Valid PNG signature + chunk claiming 4,000,000,000 bytes on small file
    let mut bomb = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bomb.extend_from_slice(&0xEE000000u32.to_be_bytes()); // ~4GB length
    bomb.extend_from_slice(b"iCCP");
    bomb.extend_from_slice(b"small-data");

    let result = parse_png(&bomb);
    assert!(result.is_err(), "Parser must reject billion-byte chunk length without allocating RAM");
}

#[test]
fn test_adversarial_heic_cyclical_nested_box_bomb() {
    // Attempt to blow the call stack with deeply nested or recursive meta boxes
    let mut bomb = vec![
        0x00, 0x00, 0x00, 0x18,
        b'f', b't', b'y', b'p',
        b'h', b'e', b'i', b'c',
        0x00, 0x00, 0x00, 0x00,
        b'm', b'i', b'f', b'1',
        b'h', b'e', b'i', b'c',
    ];

    // Nest 100 meta boxes inside each other (exceeding limit of 32)
    for _ in 0..100 {
        bomb.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x20, // Length 32
            b'm', b'e', b't', b'a',
            0x00, 0x00, 0x00, 0x00, // Version + Flags
        ]);
    }

    // Must execute cleanly without stack overflow panic
    let result = parse_heic(&bomb);
    assert!(result.is_ok(), "Parser recursion guard must prevent stack overflow");
}

#[test]
fn test_adversarial_webp_riff_size_bomb() {
    // RIFF header claiming 4GB file with only 20 bytes provided
    let mut bomb = b"RIFF".to_vec();
    bomb.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // 4GB size
    bomb.extend_from_slice(b"WEBPVP8X\0\0\0\0");

    let result = parse_webp(&bomb);
    // Must parse safely up to available bytes without allocating 4GB
    assert!(result.is_ok());
}

#[test]
fn test_adversarial_empty_and_single_byte_files() {
    assert!(parse_jpeg(&[]).is_err());
    assert!(parse_png(&[0x89]).is_err());
    assert!(parse_webp(b"R").is_err());
}
