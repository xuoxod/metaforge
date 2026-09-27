#[path = "../src/table.rs"]
mod table;

#[path = "../src/export.rs"]
mod export;

use metaforge_core::{MetadataEntry, TagCategory};
use table::scrub_terminal_poisoning;

#[test]
fn test_adversarial_terminal_ansi_csi_scrubbing() {
    let malicious = "Normal Image\x1B[2J\x1B[H\x1B[31;1mSYSTEM COMPROMISED\x1B[0m Photo";
    let cleaned = scrub_terminal_poisoning(malicious);

    // Escape sequences must be stripped
    assert!(!cleaned.contains("\x1B["));
    assert!(!cleaned.contains("2J"));
    assert!(!cleaned.contains("31;1m"));
    assert_eq!(cleaned, "Normal ImageSYSTEM COMPROMISED Photo");
}

#[test]
fn test_adversarial_terminal_osc_hyperlink_scrubbing() {
    // OSC 8 hyperlink injection attack
    let malicious = "Click here: \x1B]8;;http://malicious.payload.com/exploit\x07Phishing Link\x1B]8;;\x07";
    let cleaned = scrub_terminal_poisoning(malicious);

    assert!(!cleaned.contains("http://malicious.payload.com"));
    assert_eq!(cleaned, "Click here: Phishing Link");
}

#[test]
fn test_adversarial_csv_formula_injection_mitigation() {
    let entries = vec![
        MetadataEntry::new(TagCategory::Text, None, "Title", "Photo Title", "=cmd|'/C calc'!A0"),
        MetadataEntry::new(TagCategory::Tiff, None, "Model", "Camera Model", "+@SUM(1,2)"),
        MetadataEntry::new(TagCategory::Exif, None, "UserComment", "Comment", "-5+10"),
        MetadataEntry::new(TagCategory::Text, None, "Author", "Author", "@EVIL_MACRO"),
        MetadataEntry::new(TagCategory::Tiff, None, "SafeTag", "Safe Tag", "Normal Clean Tag"),
    ];

    let mut buffer = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut buffer);
        writer.write_record(["file", "category", "tag_id", "key", "name", "value"]).unwrap();

        for entry in &entries {
            let safe_value = metaforge_sanitize::escape_formula_injection(&entry.value);
            let safe_name = metaforge_sanitize::escape_formula_injection(&entry.name);
            let safe_key = metaforge_sanitize::escape_formula_injection(&entry.key);
            writer.write_record(["test.jpg", entry.category.as_str(), "", &safe_key, &safe_name, &safe_value]).unwrap();
        }
        writer.flush().unwrap();
    }

    let csv_str = String::from_utf8(buffer).unwrap();

    // Assert that dangerous cells are neutralized with leading single quote
    assert!(csv_str.contains("'=cmd|'/C calc'!A0"));
    assert!(csv_str.contains("'+@SUM(1,2)"));
    assert!(csv_str.contains("'-5+10"));
    assert!(csv_str.contains("'@EVIL_MACRO"));
    // Assert normal strings do not have single quote prefixed
    assert!(csv_str.contains("Normal Clean Tag"));
    assert!(!csv_str.contains("'Normal Clean Tag"));
}
