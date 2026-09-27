use metaforge_core::{dms_to_decimal, dms_with_ref_to_decimal, lookup_tag_name, TagCategory};

#[test]
fn test_adversarial_nan_infinity_coordinates() {
    // Assert NaN or Inf does not crash or panic
    let nan_lat = dms_to_decimal(f64::NAN, 12.0, 34.0, false);
    assert_eq!(nan_lat, 0.0);

    let inf_lat = dms_with_ref_to_decimal(f64::INFINITY, 0.0, 0.0, "N");
    assert!(inf_lat.is_infinite());
}

#[test]
fn test_adversarial_tag_lookup_fuzzing() {
    // Fuzz all 65536 possible u16 values across all categories
    for tag_id in 0..=u16::MAX {
        let _ = lookup_tag_name(TagCategory::Exif, tag_id);
        let _ = lookup_tag_name(TagCategory::Gps, tag_id);
        let _ = lookup_tag_name(TagCategory::Tiff, tag_id);
        let _ = lookup_tag_name(TagCategory::Xmp, tag_id);
    }
}

#[test]
fn test_adversarial_malformed_cardinal_refs() {
    // Test weird, hostile, or lowercase direction refs
    let res1 = dms_with_ref_to_decimal(10.0, 0.0, 0.0, "s");
    assert_eq!(res1, -10.0);

    let res2 = dms_with_ref_to_decimal(10.0, 0.0, 0.0, " w ");
    assert_eq!(res2, -10.0);

    let res3 = dms_with_ref_to_decimal(10.0, 0.0, 0.0, "\0\x1b[2J");
    assert_eq!(res3, 10.0); // Safe fallback to positive without crash
}
