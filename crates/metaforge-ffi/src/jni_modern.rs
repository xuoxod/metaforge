use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;
use std::fs;
use std::path::Path;

use metaforge_converter::{
    convert_media_file, AudioConvertOptions, ImageConvertOptions, ImageTargetFormat,
};
use metaforge_core::ContainerType;
use metaforge_forensics::{
    analyze_jpeg_steganography, calculate_shannon_entropy, detect_overlay, scan_embedded_payloads,
};
use metaforge_parsers::{
    detect_container_type, parse_gif, parse_heic, parse_jpeg, parse_png, parse_webp,
};
use metaforge_sanitize::scrub_image;

#[no_mangle]
pub unsafe extern "system" fn Java_com_rmediatech_metaforge_MetaForge_getVersion<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let version = env!("CARGO_PKG_VERSION");
    let output = env
        .new_string(version)
        .unwrap_or_else(|_| env.new_string("0.1.0").unwrap());
    output.into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rmediatech_metaforge_MetaForge_scanJson<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    path_jstring: JString<'local>,
) -> jstring {
    let path_str: String = match env.get_string(&path_jstring) {
        Ok(s) => s.into(),
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "JNI string conversion failed: {}"}}"#, e))
                .unwrap()
                .into_raw();
        }
    };

    let path = Path::new(&path_str);
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "Failed to read file: {}"}}"#, e))
                .unwrap()
                .into_raw();
        }
    };

    let container = match detect_container_type(&bytes, path) {
        Ok(c) => c,
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "{}"}}"#, e))
                .unwrap()
                .into_raw();
        }
    };

    let metadata = match container {
        ContainerType::Jpeg => parse_jpeg(&bytes).map(|j| j.metadata),
        ContainerType::Png => parse_png(&bytes).map(|p| p.metadata),
        ContainerType::Webp => parse_webp(&bytes).map(|w| w.metadata),
        ContainerType::Gif => parse_gif(&bytes).map(|g| g.metadata),
        ContainerType::Heic => parse_heic(&bytes).map(|h| h.metadata),
        ContainerType::Unknown => Ok(Default::default()),
    };

    let json_str = match metadata {
        Ok(meta) => serde_json::json!({
            "container": container.display_name(),
            "file_size": bytes.len(),
            "metadata": meta,
        })
        .to_string(),
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    };

    env.new_string(json_str)
        .unwrap_or_else(|_| env.new_string(r#"{"error": "Out of memory"}"#).unwrap())
        .into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rmediatech_metaforge_MetaForge_auditJson<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    path_jstring: JString<'local>,
) -> jstring {
    let path_str: String = match env.get_string(&path_jstring) {
        Ok(s) => s.into(),
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "JNI string conversion failed: {}"}}"#, e))
                .unwrap()
                .into_raw();
        }
    };

    let path = Path::new(&path_str);
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "Failed to read file: {}"}}"#, e))
                .unwrap()
                .into_raw();
        }
    };

    let container = match detect_container_type(&bytes, path) {
        Ok(c) => c,
        Err(e) => {
            return env
                .new_string(format!(r#"{{"error": "{}"}}"#, e))
                .unwrap()
                .into_raw();
        }
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

    let report_json = serde_json::json!({
        "container": container.display_name(),
        "entropy": entropy,
        "overlay": overlay,
        "payloads": payloads,
        "steganalysis": stego,
    })
    .to_string();

    env.new_string(report_json)
        .unwrap_or_else(|_| env.new_string(r#"{"error": "Out of memory"}"#).unwrap())
        .into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rmediatech_metaforge_MetaForge_sanitizeFile<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_jstring: JString<'local>,
    output_jstring: JString<'local>,
) -> jboolean {
    let in_str: String = match env.get_string(&input_jstring) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };
    let out_str: String = match env.get_string(&output_jstring) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    let bytes = match fs::read(in_str) {
        Ok(b) => b,
        Err(_) => return 0,
    };

    let (cleaned, _) = match scrub_image(&bytes) {
        Ok(res) => res,
        Err(_) => return 0,
    };

    if fs::write(out_str, cleaned).is_ok() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rmediatech_metaforge_MetaForge_convertFile<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input_jstring: JString<'local>,
    output_jstring: JString<'local>,
    format_jstring: JString<'local>,
) -> jint {
    let in_str: String = match env.get_string(&input_jstring) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };
    let out_str: String = match env.get_string(&output_jstring) {
        Ok(s) => s.into(),
        Err(_) => return -2,
    };
    let fmt_str: String = match env.get_string(&format_jstring) {
        Ok(s) => {
            let rust_s: String = s.into();
            rust_s.to_ascii_lowercase()
        }
        Err(_) => return -3,
    };

    let in_path = Path::new(&in_str);
    let out_path = Path::new(&out_str);

    let mut img_opts = ImageConvertOptions::default();
    let aud_opts = AudioConvertOptions::default();

    let res = match fmt_str.as_str() {
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
