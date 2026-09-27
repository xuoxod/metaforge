use metaforge_core::{dms_with_ref_to_decimal, lookup_tag_name, ContainerType, MetadataEntry, TagCategory};

#[test]
fn test_exif_tag_lookup() {
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0x829A), Some("Exposure Time"));
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0x829D), Some("F-Number (Aperture)"));
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0x8827), Some("ISO Speed Ratings"));
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0x9003), Some("Date and Time (Original)"));
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0xA434), Some("Lens Model"));
}

#[test]
fn test_gps_tag_lookup() {
    assert_eq!(lookup_tag_name(TagCategory::Gps, 0x0002), Some("GPS Latitude"));
    assert_eq!(lookup_tag_name(TagCategory::Gps, 0x0004), Some("GPS Longitude"));
    assert_eq!(lookup_tag_name(TagCategory::Gps, 0x0006), Some("GPS Altitude"));
}

#[test]
fn test_tiff_tag_lookup() {
    assert_eq!(lookup_tag_name(TagCategory::Tiff, 0x010F), Some("Make (Camera Manufacturer)"));
    assert_eq!(lookup_tag_name(TagCategory::Tiff, 0x0110), Some("Model (Camera Model)"));
    assert_eq!(lookup_tag_name(TagCategory::Tiff, 0x0131), Some("Software"));
}

#[test]
fn test_unknown_tag_returns_none() {
    assert_eq!(lookup_tag_name(TagCategory::Exif, 0xFFFF), None);
    assert_eq!(lookup_tag_name(TagCategory::Gps, 0xFFFF), None);
}

#[test]
fn test_dms_conversion() {
    let lat = dms_with_ref_to_decimal(40.0, 26.0, 46.0, "N");
    assert!((lat - 40.446111).abs() < 1e-4);

    let lon = dms_with_ref_to_decimal(79.0, 58.0, 56.0, "W");
    assert!((lon - -79.982222).abs() < 1e-4);
}

#[test]
fn test_container_type_properties() {
    assert_eq!(ContainerType::Jpeg.as_str(), "jpeg");
    assert_eq!(ContainerType::Png.display_name(), "PNG Image Container");
}

#[test]
fn test_metadata_entry_serialization() {
    let entry = MetadataEntry::new(TagCategory::Exif, Some(0x829A), "exif:33434", "Exposure Time", "1/250s");
    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("Exposure Time"));
    assert!(serialized.contains("1/250s"));
}
