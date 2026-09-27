use metaforge_forensics::{calculate_shannon_entropy, is_suspicious_entropy, scan_embedded_payloads};

#[test]
fn test_adversarial_php_web_shell_injection() {
    let mut image_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    image_bytes.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    // Attacker injects PHP shell into trailing bytes or comment
    image_bytes.extend_from_slice(b"<?php system($_GET['cmd']); ?>");

    let payloads = scan_embedded_payloads(&image_bytes, 20);
    assert!(payloads.iter().any(|p| p.name == "PHP Script Block" && p.category == "Script"));
    assert!(payloads.iter().any(|p| p.name == "Suspicious System Execution Call"));
}

#[test]
fn test_adversarial_pe_false_positive_immunity() {
    // Normal JPEG data might contain 'M', 'Z' by coincidence without being a valid PE binary
    let mut image_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    image_bytes.extend_from_slice(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    image_bytes.extend_from_slice(b"Random byte stream containing MZ but not a valid PE header");

    let payloads = scan_embedded_payloads(&image_bytes, image_bytes.len());
    // Must NOT flag as Windows PE because e_lfanew check fails
    assert!(!payloads.iter().any(|p| p.name == "Windows Portable Executable (PE)"));
}

#[test]
fn test_adversarial_high_entropy_steganography_threshold() {
    // Plain text has low entropy (~4.0 - 4.5)
    let plain = b"This is a normal photographic description tag with repeated characters.";
    let plain_ent = calculate_shannon_entropy(plain);
    assert!(!is_suspicious_entropy(plain_ent));

    // High entropy cryptographic stream (> 7.95)
    let mut crypto_stream = Vec::new();
    for i in 0..256 {
        crypto_stream.push(i as u8);
    }
    let crypto_ent = calculate_shannon_entropy(&crypto_stream);
    assert!(is_suspicious_entropy(crypto_ent));
}

#[test]
fn test_adversarial_steganalysis_truncated_streams_and_fuzzing() {
    use metaforge_forensics::analyze_jpeg_steganography;

    // 1. Truncated stream directly at SOS marker
    let truncated = [0xFF, 0xD8, 0xFF, 0xDA];
    let report = analyze_jpeg_steganography(&truncated);
    assert_eq!(report.scan_data_bytes, 0);

    // 2. SOS marker with overflow header length
    let corrupt_header = [0xFF, 0xD8, 0xFF, 0xDA, 0xFF, 0xFF, 0x01, 0x02];
    let report = analyze_jpeg_steganography(&corrupt_header);
    assert_eq!(report.scan_data_bytes, 0);

    // 3. Endless 0xFF padding fuzzing
    let mut flood = vec![0xFF; 5000];
    flood[0] = 0xFF;
    flood[1] = 0xD8;
    flood[2] = 0xFF;
    flood[3] = 0xDA;
    flood[4] = 0x00;
    flood[5] = 0x02; // header length 2
    let report = analyze_jpeg_steganography(&flood);
    // Should safely terminate without panic or hanging
    assert!(report.scan_data_bytes <= flood.len());
}
