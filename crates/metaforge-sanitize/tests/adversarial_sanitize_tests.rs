use metaforge_core::error::MetaForgeError;
use metaforge_sanitize::{escape_formula_injection, is_formula_injection, set_jpeg_comment};

#[test]
fn test_adversarial_jpeg_comment_overflow() {
    let base_jpeg = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10,
        b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0xFF, 0xD9,
    ];

    // Attacker tries to inject comment > 65,533 bytes (segment length overflow attack)
    let huge_comment = "A".repeat(70_000);
    let result = set_jpeg_comment(&base_jpeg, &huge_comment);

    assert!(result.is_err());
    match result.unwrap_err() {
        MetaForgeError::SegmentOverflow { size } => {
            assert_eq!(size, 70_000);
        }
        other => panic!("Expected SegmentOverflow error, got {:?}", other),
    }
}

#[test]
fn test_adversarial_formula_injection_mitigation() {
    let payloads = [
        "=cmd|'/C calc'!A0",
        "+@SUM(1,2)",
        "-2+3*cmd|' /C calc'!A0",
        "@SUM(A1:A10)",
        "\t=1+1",
        "\r+HYPERLINK(\"http://evil.com\",\"Click\")",
    ];

    for payload in payloads {
        assert!(is_formula_injection(payload), "Should detect formula: {}", payload);
        let escaped = escape_formula_injection(payload);
        assert!(escaped.starts_with('\''), "Escaped must begin with single quote: {}", escaped);
    }

    // Benign strings must NOT be modified
    let benign = "Standard photographic tag";
    assert!(!is_formula_injection(benign));
    assert_eq!(escape_formula_injection(benign), benign);
}

#[test]
fn test_adversarial_truncated_stream_safety() {
    let truncated_jpeg = vec![0xFF, 0xD8, 0xFF, 0xFE, 0x00, 0x50]; // Claims 80 bytes length but ends here
    let result = set_jpeg_comment(&truncated_jpeg, "Test");
    assert!(result.is_err());
}
