use exif as kamadak_exif;
use metaforge_core::geo::dms_to_decimal;
use serde::Serialize;

/// Geographic coordinates and tracking metadata extracted from EXIF GPS tags.
#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct GpsInfo {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f32>,
    pub speed: Option<f64>,
    pub speed_ref: Option<String>,
    pub track: Option<f64>,
    pub track_ref: Option<String>,
    pub img_direction: Option<f64>,
    pub img_direction_ref: Option<String>,
    pub date_stamp: Option<String>,
    pub time_stamp: Option<String>,
}

/// Standardized EXIF and generic metadata tags extracted from JPEG or PNG containers.
#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct ExifMetadata {
    // Basic Camera Tags
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps: Option<GpsInfo>,
    pub f_number: Option<f32>,
    pub iso: Option<u16>,
    pub exposure_time: Option<String>,
    pub focal_length_mm: Option<f32>,
    pub focal_length_35mm: Option<u16>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub date_time_original: Option<String>,
    pub orientation: Option<u16>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub software: Option<String>,

    // Advanced & Forensic Tags
    pub body_serial_number: Option<String>,
    pub lens_serial_number: Option<String>,
    pub flash: Option<u16>,
    pub exposure_program: Option<u16>,
    pub metering_mode: Option<u16>,
    pub white_balance: Option<u16>,
    pub light_source: Option<u16>,
    pub user_comment: Option<String>,

    // Extensible XML Metadata Block
    pub xmp: Option<String>,
}

/// Extracts full EXIF data properties from a kamadak-exif payload structure.
#[allow(clippy::collapsible_if)]
pub fn extract_exif_metadata(exif_data: &kamadak_exif::Exif) -> ExifMetadata {
    let mut metadata = ExifMetadata::default();
    
    let get_ascii_string = |tag: kamadak_exif::Tag| -> Option<String> {
        exif_data
            .get_field(tag, kamadak_exif::In::PRIMARY)
            .map(|f| f.display_value().to_string().trim_matches('"').to_string())
    };

    metadata.camera_make = get_ascii_string(kamadak_exif::Tag::Make);
    metadata.camera_model = get_ascii_string(kamadak_exif::Tag::Model);
    metadata.software = get_ascii_string(kamadak_exif::Tag::Software);
    metadata.lens_make = get_ascii_string(kamadak_exif::Tag::LensMake);
    metadata.lens_model = get_ascii_string(kamadak_exif::Tag::LensModel);
    metadata.body_serial_number = get_ascii_string(kamadak_exif::Tag::BodySerialNumber);
    metadata.lens_serial_number = get_ascii_string(kamadak_exif::Tag::LensSerialNumber);
    metadata.date_time_original = get_ascii_string(kamadak_exif::Tag::DateTimeOriginal);
    metadata.user_comment = get_ascii_string(kamadak_exif::Tag::UserComment);

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::FNumber, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                metadata.f_number = Some(rat.to_f64() as f32);
            }
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::PhotographicSensitivity, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.iso = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::ExposureTime, kamadak_exif::In::PRIMARY) {
        metadata.exposure_time = Some(field.display_value().to_string());
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::FocalLength, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                metadata.focal_length_mm = Some(rat.to_f64() as f32);
            }
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::FocalLengthIn35mmFilm, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.focal_length_35mm = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::Orientation, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.orientation = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::PixelXDimension, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.width = Some(val);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::PixelYDimension, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.height = Some(val);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::Flash, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.flash = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::ExposureProgram, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.exposure_program = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::MeteringMode, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.metering_mode = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::WhiteBalance, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.white_balance = Some(val as u16);
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::LightSource, kamadak_exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            metadata.light_source = Some(val as u16);
        }
    }

    // Extract GPS Tags
    let mut gps_info = GpsInfo::default();
    let mut has_gps = false;

    let parse_dms = |tag: kamadak_exif::Tag, ref_tag: kamadak_exif::Tag, is_neg_ref: fn(&str) -> bool| -> Option<f64> {
        let field = exif_data.get_field(tag, kamadak_exif::In::PRIMARY)?;
        let ref_field = exif_data.get_field(ref_tag, kamadak_exif::In::PRIMARY);
        let ref_val = ref_field.map(|f| f.display_value().to_string().trim_matches('"').to_string()).unwrap_or_default();
        
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if rats.len() >= 3 {
                let deg = rats[0].to_f64();
                let min = rats[1].to_f64();
                let sec = rats[2].to_f64();
                let is_neg = is_neg_ref(&ref_val);
                return Some(dms_to_decimal(deg, min, sec, is_neg));
            }
        }
        None
    };

    if let Some(lat) = parse_dms(kamadak_exif::Tag::GPSLatitude, kamadak_exif::Tag::GPSLatitudeRef, |r| r == "S") {
        gps_info.latitude = Some(lat);
        has_gps = true;
    }

    if let Some(lon) = parse_dms(kamadak_exif::Tag::GPSLongitude, kamadak_exif::Tag::GPSLongitudeRef, |r| r == "W") {
        gps_info.longitude = Some(lon);
        has_gps = true;
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::GPSAltitude, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                let alt = rat.to_f64() as f32;
                let is_below_sea = exif_data
                    .get_field(kamadak_exif::Tag::GPSAltitudeRef, kamadak_exif::In::PRIMARY)
                    .and_then(|f| f.value.get_uint(0))
                    .map(|v| v == 1)
                    .unwrap_or(false);
                gps_info.altitude = Some(if is_below_sea { -alt } else { alt });
                has_gps = true;
            }
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::GPSSpeed, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                gps_info.speed = Some(rat.to_f64());
                gps_info.speed_ref = get_ascii_string(kamadak_exif::Tag::GPSSpeedRef);
                has_gps = true;
            }
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::GPSTrack, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                gps_info.track = Some(rat.to_f64());
                gps_info.track_ref = get_ascii_string(kamadak_exif::Tag::GPSTrackRef);
                has_gps = true;
            }
        }
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::GPSImgDirection, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if let Some(rat) = rats.first() {
                gps_info.img_direction = Some(rat.to_f64());
                gps_info.img_direction_ref = get_ascii_string(kamadak_exif::Tag::GPSImgDirectionRef);
                has_gps = true;
            }
        }
    }

    if let Some(date) = get_ascii_string(kamadak_exif::Tag::GPSDateStamp) {
        gps_info.date_stamp = Some(date);
        has_gps = true;
    }

    if let Some(field) = exif_data.get_field(kamadak_exif::Tag::GPSTimeStamp, kamadak_exif::In::PRIMARY) {
        if let kamadak_exif::Value::Rational(ref rats) = field.value {
            if rats.len() >= 3 {
                let h = rats[0].to_f64() as u32;
                let m = rats[1].to_f64() as u32;
                let s = rats[2].to_f64();
                gps_info.time_stamp = Some(format!("{:02}:{:02}:{:05.2}Z", h, m, s));
                has_gps = true;
            }
        }
    }

    if has_gps {
        metadata.gps = Some(gps_info);
    }

    metadata
}
