use metaforge_forensics::{calculate_shannon_entropy, detect_overlay, scan_embedded_payloads};

#[test]
fn test_shannon_entropy_bounds() {
    assert_eq!(calculate_shannon_entropy(&[]), 0.0);
    assert_eq!(calculate_shannon_entropy(&[0u8; 100]), 0.0);

    let mut random_bytes = Vec::new();
    for i in 0..256 {
        random_bytes.push(i as u8);
    }
    let ent = calculate_shannon_entropy(&random_bytes);
    assert!((ent - 8.0).abs() < 1e-9);
}

#[test]
fn test_overlay_detection() {
    let bytes = b"VALID_IMAGE_DATA_EXACTLY_20_BYTES_WITH_OVERLAY_ATTACHED";
    let overlay = detect_overlay(bytes, 20);
    assert!(overlay.has_overlay);
    assert_eq!(overlay.logical_size, 20);
    assert_eq!(overlay.physical_size, bytes.len());
    assert_eq!(overlay.overlay_size, bytes.len() - 20);
}

#[test]
fn test_embedded_zip_detection() {
    let mut data = vec![0u8; 32];
    data.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // ZIP magic
    data.extend_from_slice(b"secret_payload.txt");

    let payloads = scan_embedded_payloads(&data, data.len());
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].name, "ZIP Archive");
    assert_eq!(payloads[0].category, "Archive");
}
