use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyteArray, jint, jlong, jstring};
use jni::JNIEnv;
use std::fs;
use std::path::Path;

use metaforge_converter::{
    convert_media_file, AudioConvertOptions, ImageConvertOptions,
};
use metaforge_core::crc32;
use metaforge_forensics::{
    calculate_ascii_ratio, extract_entropy_continuum,
};
use metaforge_parsers::parse_jpeg;

// ============================================================================
// 1. Legacy Bindings for `metaforge-foundry`
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn Java_com_metaforge_MetaForgeApp_getGreetingFromRust<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let output = env
        .new_string("Hello from Sovereign Rust MetaForge!")
        .unwrap();
    output.into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_metaforge_MetaForgeApp_getAllExifDataNative<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    file_path_jstring: JString<'local>,
) -> jstring {
    let path_str: String = match env.get_string(&file_path_jstring) {
        Ok(s) => s.into(),
        Err(e) => {
            return env
                .new_string(format!("Error: Could not convert file path: {}", e))
                .unwrap()
                .into_raw();
        }
    };

    let bytes = match fs::read(&path_str) {
        Ok(b) => b,
        Err(e) => {
            return env
                .new_string(format!("Error: Failed to read file: {}", e))
                .unwrap()
                .into_raw();
        }
    };

    match parse_jpeg(&bytes) {
        Ok(info) => {
            let mut out = String::new();
            if let Some(ref make) = info.metadata.camera_make {
                out.push_str(&format!("Camera Make: {}\n", make));
            }
            if let Some(ref model) = info.metadata.camera_model {
                out.push_str(&format!("Camera Model: {}\n", model));
            }
            if let Some(ref date) = info.metadata.date_time_original {
                out.push_str(&format!("Date/Time: {}\n", date));
            }
            if let Some(ref gps) = info.metadata.gps {
                if let (Some(lat), Some(lon)) = (gps.latitude, gps.longitude) {
                    out.push_str(&format!("GPS: {}, {}\n", lat, lon));
                }
            }
            if out.is_empty() {
                out.push_str("JPEG parsed successfully (no standard EXIF tags found).\n");
            }
            env.new_string(out).unwrap().into_raw()
        }
        Err(e) => env
            .new_string(format!("Error: EXIF parsing failed: {}", e))
            .unwrap()
            .into_raw(),
    }
}

// ============================================================================
// 2. Legacy Bindings for `jpegmeta` & `jpegmeta_rust_bridge`
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_App_processJpegBytesRust<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_jbytes: jbyteArray,
) -> jstring {
    let j_arr = unsafe { JByteArray::from_raw(input_jbytes) };
    let bytes = match env.convert_byte_array(&j_arr) {
        Ok(b) => b,
        Err(e) => {
            return env
                .new_string(format!("Rust Error: Failed to convert byte array: {}", e))
                .unwrap()
                .into_raw();
        }
    };

    let soi_found = bytes.starts_with(&[0xFF, 0xD8]);
    let eoi_found = bytes.ends_with(&[0xFF, 0xD9]);
    let marker_status = match (soi_found, eoi_found) {
        (true, true) => "SOI and EOI markers found.",
        (true, false) => "SOI found, EOI missing.",
        (false, true) => "SOI missing, EOI found.",
        (false, false) => "Neither SOI nor EOI markers found.",
    };

    let crc = crc32(&bytes);
    let continuum = extract_entropy_continuum(&bytes);
    let ascii_ratio = calculate_ascii_ratio(&continuum);

    let exif_summary = match parse_jpeg(&bytes) {
        Ok(info) => {
            let mut desc = Vec::new();
            if let Some(m) = info.metadata.camera_make {
                desc.push(format!("Make: {}", m));
            }
            if let Some(m) = info.metadata.camera_model {
                desc.push(format!("Model: {}", m));
            }
            if desc.is_empty() {
                "Valid JPEG header (minimal EXIF)".to_string()
            } else {
                desc.join(", ")
            }
        }
        Err(e) => format!("EXIF parse error: {}", e),
    };

    let result = format!(
        "--- Rust Bridge Analysis ---\n\
        Markers: {}\n\
        CRC32: 0x{:08X}\n\
        Scan Data ASCII Ratio: {:.2}%\n\
        EXIF: {}\n\
        --- End Rust Bridge Analysis ---",
        marker_status,
        crc,
        ascii_ratio * 100.0,
        exif_summary
    );

    env.new_string(result).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_jpegmeta_bridge_RustBridge_calculateCRC32<
    'local,
>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_data: jbyteArray,
) -> jlong {
    let j_arr = unsafe { JByteArray::from_raw(input_data) };
    match env.convert_byte_array(&j_arr) {
        Ok(bytes) => crc32(&bytes) as jlong,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_jpegmeta_bridge_RustBridge_checkJpegMarkers<
    'local,
>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_data: jbyteArray,
) -> jint {
    let j_arr = unsafe { JByteArray::from_raw(input_data) };
    match env.convert_byte_array(&j_arr) {
        Ok(bytes) => {
            let soi = bytes.starts_with(&[0xFF, 0xD8]);
            let eoi = bytes.ends_with(&[0xFF, 0xD9]);
            match (soi, eoi) {
                (true, true) => 3,
                (true, false) => 1,
                (false, true) => 2,
                (false, false) => 0,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_jpegmeta_bridge_RustBridge_analyzeScanDataAscii<
    'local,
>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    scan_data: jbyteArray,
) -> jstring {
    let j_arr = unsafe { JByteArray::from_raw(scan_data) };
    match env.convert_byte_array(&j_arr) {
        Ok(bytes) => {
            let ratio = calculate_ascii_ratio(&bytes);
            let msg = format!(
                "Rust Scan Analysis: Found printable ASCII ratio {:.2}% across {} bytes",
                ratio * 100.0,
                bytes.len()
            );
            env.new_string(msg).unwrap().into_raw()
        }
        Err(e) => env
            .new_string(format!("Error: {}", e))
            .unwrap()
            .into_raw(),
    }
}

// ============================================================================
// 3. Legacy Bindings for `media-converter-jni`
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_nexusbridge_NativeConverter_getBackendVersion<
    'local,
>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let output = env
        .new_string("metaforge-converter-pure-rust-0.1.0")
        .unwrap();
    output.into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_gmail_xuoxod_nexusbridge_NativeConverter_convertMediaJni<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_path: JString<'local>,
    output_path: JString<'local>,
) -> jint {
    let in_str: String = match env.get_string(&input_path) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };
    let out_str: String = match env.get_string(&output_path) {
        Ok(s) => s.into(),
        Err(_) => return -2,
    };

    let in_p = Path::new(&in_str);
    let out_p = Path::new(&out_str);

    let img_opts = ImageConvertOptions::default();
    let aud_opts = AudioConvertOptions::default();

    match convert_media_file(in_p, out_p, &img_opts, &aud_opts) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}
