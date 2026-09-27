use metaforge_forensics::{
    analyze_comment_segment, analyze_jpeg_steganography,
    calculate_ascii_ratio, calculate_chi_square_uniformity, calculate_pov_chi_square,
    extract_entropy_continuum, FindingSeverity, StegFindingType, CHI_SQUARE_CRITICAL_DF255_A01,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_chi_square_uniform_distribution() {
    // Generate perfectly uniform 256 * 100 bytes
    let mut data = Vec::with_capacity(25600);
    for _ in 0..100 {
        for b in 0..=255u8 {
            data.push(b);
        }
    }

    let (chi_stat, df, p_val) = calculate_chi_square_uniformity(&data);
    assert_eq!(df, 255);
    assert!(
        chi_stat < 1e-6,
        "Uniform distribution should yield near 0 chi-square stat"
    );
    assert!(
        p_val > 0.99,
        "Uniform distribution should have high p-value"
    );
}

#[test]
fn test_chi_square_skewed_distribution() {
    // Severely skewed distribution (only 0x41 'A' and 0x42 'B')
    let data = vec![0x41u8; 1000];
    let (chi_stat, df, p_val) = calculate_chi_square_uniformity(&data);
    assert_eq!(df, 255);
    assert!(
        chi_stat > CHI_SQUARE_CRITICAL_DF255_A01,
        "Heavily skewed distribution must exceed critical value"
    );
    assert!(
        p_val < 0.001,
        "Heavily skewed distribution should have p-value near 0"
    );
}

#[test]
fn test_pov_chi_square() {
    // Test Pairs of Values: equalized pairs (LSB replacement behavior)
    let mut data = Vec::new();
    for k in 0..128u8 {
        for _ in 0..10 {
            data.push(2 * k);
            data.push(2 * k + 1);
        }
    }
    let (pov_stat, df) = calculate_pov_chi_square(&data);
    assert_eq!(df, 127);
    assert!(
        pov_stat < 1e-6,
        "Equalized pairs should yield zero PoV chi-square"
    );
}

#[test]
fn test_entropy_continuum_unstuffing() {
    // Construct synthetic JPEG stream with SOS, stuffed 0xFF00, RST0 marker (0xFFD0), and EOI
    let mut stream = Vec::new();
    // SOI
    stream.extend_from_slice(&[0xFF, 0xD8]);
    // SOS marker
    stream.extend_from_slice(&[0xFF, 0xDA]);
    // Header len = 4 bytes (len 2 bytes + 2 dummy header bytes)
    stream.extend_from_slice(&[0x00, 0x04, 0x12, 0x34]);
    // Scan payload: 'H', 'E', 0xFF, 0x00 (stuffed), 'L', 0xFF, 0xD0 (RST0), 'P'
    stream.extend_from_slice(&[b'H', b'E', 0xFF, 0x00, b'L', 0xFF, 0xD0, b'P']);
    // EOI marker
    stream.extend_from_slice(&[0xFF, 0xD9]);

    let extracted = extract_entropy_continuum(&stream);
    // Expected: 'H', 'E', 0xFF, 'L', 'P' (0xFF00 unstuffed to 0xFF, RST0 skipped)
    assert_eq!(extracted, vec![b'H', b'E', 0xFF, b'L', b'P']);
}

#[test]
fn test_ascii_ratio_detection() {
    let ascii_text = b"Hello, this is a plain text covert message hidden in scan data!";
    let ratio = calculate_ascii_ratio(ascii_text);
    assert!(ratio > 0.9);

    let random_binary = [0x00, 0x01, 0x02, 0x03, 0xFE, 0xFF, 0x88, 0x99];
    let binary_ratio = calculate_ascii_ratio(&random_binary);
    assert!(binary_ratio < 0.1);
}

#[test]
fn test_suspicious_comment_detection() {
    // 1. Short clean comment
    let clean = b"Author: Sovereign Contributor";
    let findings = analyze_comment_segment(clean);
    assert!(findings.is_empty());

    // 2. High non-printable binary comment
    let mut dirty = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
    dirty.extend_from_slice(b"Some text");
    let findings = analyze_comment_segment(&dirty);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding_type,
        StegFindingType::SuspiciousComment
    );
    assert_eq!(findings[0].severity, FindingSeverity::Medium);
}

#[test]
fn test_real_jpeg_steganalysis() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../metaforge-parsers/tests/fixtures/lynx.jpeg");
    if fixture_path.exists() {
        let bytes = fs::read(&fixture_path).expect("Failed to read fixture");
        let report = analyze_jpeg_steganography(&bytes);
        assert!(
            report.scan_data_bytes > 0,
            "Must successfully extract scan data"
        );
        assert_eq!(report.degrees_of_freedom, 255);
        assert!(report.ascii_ratio < 0.5);
    }
}
