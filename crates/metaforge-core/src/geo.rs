/// Converts Degrees, Minutes, Seconds (DMS) coordinates to standard Decimal degrees.
pub fn dms_to_decimal(degrees: f64, minutes: f64, seconds: f64, is_negative: bool) -> f64 {
    if degrees.is_nan() || minutes.is_nan() || seconds.is_nan() {
        return 0.0;
    }
    let dec = degrees.abs() + (minutes.abs() / 60.0) + (seconds.abs() / 3600.0);
    if is_negative { -dec } else { dec }
}

/// Converts DMS coordinates with cardinal direction reference (N, S, E, W).
pub fn dms_with_ref_to_decimal(degrees: f64, minutes: f64, seconds: f64, ref_char: &str) -> f64 {
    let is_negative = matches!(ref_char.trim().to_uppercase().as_str(), "S" | "W");
    dms_to_decimal(degrees, minutes, seconds, is_negative)
}

/// Converts pixels per unit (ppu) to dots per inch (DPI).
pub fn ppu_to_dpi(ppu: u32) -> f64 {
    (ppu as f64 * 0.0254).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dms_to_decimal_basic() {
        let val = dms_to_decimal(43.0, 28.0, 2.118, false);
        assert!((val - 43.467255).abs() < 1e-5);

        let val_neg = dms_to_decimal(43.0, 28.0, 2.118, true);
        assert!((val_neg - -43.467255).abs() < 1e-5);
    }

    #[test]
    fn test_dms_with_ref() {
        let lat = dms_with_ref_to_decimal(37.0, 48.0, 36.12, "N");
        assert!((lat - 37.810033).abs() < 1e-5);

        let lon = dms_with_ref_to_decimal(122.0, 24.0, 15.0, "W");
        assert!((lon - -122.404166).abs() < 1e-4);
    }

    #[test]
    fn test_ppu_to_dpi() {
        assert_eq!(ppu_to_dpi(2835), 72.0);
    }
}
