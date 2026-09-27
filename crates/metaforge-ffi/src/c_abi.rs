use libc::c_char;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;

use metaforge_converter::{
    convert_media_file, AudioConvertOptions, ImageConvertOptions, ImageTargetFormat,
};
use metaforge_core::{ContainerType, MetaForgeError};
use metaforge_forensics::{
    analyze_jpeg_steganography, calculate_shannon_entropy, detect_overlay, scan_embedded_payloads,
};
use metaforge_parsers::{
    detect_container_type, parse_gif, parse_heic, parse_jpeg, parse_png, parse_webp,
};
use metaforge_sanitize::scrub_image;

/// Safely convert a C string pointer to a Rust &str.
unsafe fn c_char_to_str<'a>(ptr: *const c_char) -> Result<&'a str, MetaForgeError> {
    if ptr.is_null() {
        return Err(MetaForgeError::EmptyFile(
            "C-ABI input pointer is NULL".to_string(),
        ));
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|e| MetaForgeError::CorruptedChunk {
            container: "C-ABI",
            chunk_type: "UTF-8".to_string(),
            detail: e.to_string(),
        })
}

/// Helper to convert a Rust string into a newly allocated C string pointer.
fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a C string allocated by MetaForge FFI.
#[no_mangle]
pub unsafe extern "C" fn metaforge_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Returns the MetaForge library version string.
#[no_mangle]
pub extern "C" fn metaforge_version() -> *mut c_char {
    to_c_string(env!("CARGO_PKG_VERSION").to_string())
}

/// Scans a media file and returns extracted metadata as a JSON string.
/// Caller must free the returned pointer using `metaforge_free_string`.
#[no_mangle]
pub unsafe extern "C" fn metaforge_scan_json(path_ptr: *const c_char) -> *mut c_char {
    let path_str = match c_char_to_str(path_ptr) {
        Ok(s) => s,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let path = Path::new(path_str);
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let container = match detect_container_type(&bytes, path) {
        Ok(c) => c,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let metadata = match container {
        ContainerType::Jpeg => parse_jpeg(&bytes).map(|j| j.metadata),
        ContainerType::Png => parse_png(&bytes).map(|p| p.metadata),
        ContainerType::Webp => parse_webp(&bytes).map(|w| w.metadata),
        ContainerType::Gif => parse_gif(&bytes).map(|g| g.metadata),
        ContainerType::Heic => parse_heic(&bytes).map(|h| h.metadata),
        ContainerType::Unknown => Ok(Default::default()),
    };

    match metadata {
        Ok(meta) => {
            let json = serde_json::json!({
                "container": container.display_name(),
                "file_size": bytes.len(),
                "metadata": meta,
            });
            to_c_string(json.to_string())
        }
        Err(e) => to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    }
}

/// Audits a media file for security threats (entropy, overlay, polyglot payloads, and steganalysis).
/// Returns a JSON string. Caller must free using `metaforge_free_string`.
#[no_mangle]
pub unsafe extern "C" fn metaforge_audit_json(path_ptr: *const c_char) -> *mut c_char {
    let path_str = match c_char_to_str(path_ptr) {
        Ok(s) => s,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let path = Path::new(path_str);
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let container = match detect_container_type(&bytes, path) {
        Ok(c) => c,
        Err(e) => return to_c_string(serde_json::json!({ "error": e.to_string() }).to_string()),
    };

    let official_end_offset = match container {
        ContainerType::Jpeg => parse_jpeg(&bytes).map(|j| j.official_end_offset).unwrap_or(bytes.len()),
        ContainerType::Png => parse_png(&bytes).map(|p| p.official_end_offset).unwrap_or(bytes.len()),
        ContainerType::Webp => parse_webp(&bytes).map(|w| w.official_end_offset).unwrap_or(bytes.len()),
        ContainerType::Gif => parse_gif(&bytes).map(|g| g.official_end_offset).unwrap_or(bytes.len()),
        ContainerType::Heic => parse_heic(&bytes).map(|h| h.official_end_offset).unwrap_or(bytes.len()),
        ContainerType::Unknown => bytes.len(),
    };

    let entropy = calculate_shannon_entropy(&bytes);
    let overlay = detect_overlay(&bytes, official_end_offset);
    let payloads = scan_embedded_payloads(&bytes, official_end_offset);

    let stego = if container == ContainerType::Jpeg {
        Some(analyze_jpeg_steganography(&bytes))
    } else {
        None
    };

    let audit_report = serde_json::json!({
        "container": container.display_name(),
        "entropy": entropy,
        "overlay": overlay,
        "payloads": payloads,
        "steganalysis": stego,
    });

    to_c_string(audit_report.to_string())
}

/// Losslessly scrubs metadata and truncates overlays from an input image to output path.
/// Returns 0 on success, negative value on error.
#[no_mangle]
pub unsafe extern "C" fn metaforge_sanitize_file(
    input_ptr: *const c_char,
    output_ptr: *const c_char,
) -> i32 {
    let input_str = match c_char_to_str(input_ptr) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let output_str = match c_char_to_str(output_ptr) {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let in_bytes = match fs::read(input_str) {
        Ok(b) => b,
        Err(_) => return -3,
    };

    let (cleaned_bytes, _) = match scrub_image(&in_bytes) {
        Ok(res) => res,
        Err(_) => return -4,
    };

    if fs::write(output_str, cleaned_bytes).is_err() {
        return -5;
    }

    0
}

/// Transcodes an input media file to an output file in the specified target format.
/// Supports image formats (jpeg, png, webp, bmp) and audio formats (wav).
/// Returns 0 on success, negative on error.
#[no_mangle]
pub unsafe extern "C" fn metaforge_convert_file(
    input_ptr: *const c_char,
    output_ptr: *const c_char,
    format_ptr: *const c_char,
) -> i32 {
    let input_str = match c_char_to_str(input_ptr) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let output_str = match c_char_to_str(output_ptr) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let format_str = match c_char_to_str(format_ptr) {
        Ok(s) => s.to_ascii_lowercase(),
        Err(_) => return -3,
    };

    let in_path = Path::new(input_str);
    let out_path = Path::new(output_str);

    let mut img_opts = ImageConvertOptions::default();
    let aud_opts = AudioConvertOptions::default();

    let res = match format_str.as_str() {
        "jpeg" | "jpg" => {
            img_opts.target_format = ImageTargetFormat::Jpeg;
            convert_media_file(in_path, out_path, &img_opts, &aud_opts)
        }
        "png" => {
            img_opts.target_format = ImageTargetFormat::Png;
            convert_media_file(in_path, out_path, &img_opts, &aud_opts)
        }
        "webp" => {
            img_opts.target_format = ImageTargetFormat::Webp;
            convert_media_file(in_path, out_path, &img_opts, &aud_opts)
        }
        "bmp" => {
            img_opts.target_format = ImageTargetFormat::Bmp;
            convert_media_file(in_path, out_path, &img_opts, &aud_opts)
        }
        "wav" => convert_media_file(in_path, out_path, &img_opts, &aud_opts),
        _ => return -4,
    };

    if res.is_ok() {
        0
    } else {
        -5
    }
}
