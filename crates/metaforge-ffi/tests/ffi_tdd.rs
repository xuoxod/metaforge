use metaforge_ffi::*;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_c_abi_version() {
    unsafe {
        let ptr = metaforge_version();
        assert!(!ptr.is_null());
        let version_str = CStr::from_ptr(ptr).to_str().unwrap();
        assert_eq!(version_str, env!("CARGO_PKG_VERSION"));
        metaforge_free_string(ptr);
    }
}

#[test]
fn test_c_abi_scan_json() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../metaforge-parsers/tests/fixtures/lynx.jpeg");
    if fixture_path.exists() {
        let c_path = CString::new(fixture_path.to_str().unwrap()).unwrap();
        unsafe {
            let json_ptr = metaforge_scan_json(c_path.as_ptr());
            assert!(!json_ptr.is_null());
            let json_str = CStr::from_ptr(json_ptr).to_str().unwrap();
            let parsed: serde_json::Value =
                serde_json::from_str(json_str).expect("Scan JSON must be valid");
            assert_eq!(parsed["container"], "JPEG Image Container");
            metaforge_free_string(json_ptr);
        }
    }
}

#[test]
fn test_c_abi_audit_json_with_steganalysis() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../metaforge-parsers/tests/fixtures/lynx.jpeg");
    if fixture_path.exists() {
        let c_path = CString::new(fixture_path.to_str().unwrap()).unwrap();
        unsafe {
            let json_ptr = metaforge_audit_json(c_path.as_ptr());
            assert!(!json_ptr.is_null());
            let json_str = CStr::from_ptr(json_ptr).to_str().unwrap();
            let parsed: serde_json::Value =
                serde_json::from_str(json_str).expect("Audit JSON must be valid");
            assert!(parsed["entropy"].as_f64().unwrap() > 7.0);
            assert!(parsed["steganalysis"].is_object());
            metaforge_free_string(json_ptr);
        }
    }
}

#[test]
fn test_c_abi_image_conversion() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../metaforge-parsers/tests/fixtures/lynx.jpeg");
    if fixture_path.exists() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let out_path = tmp.path().with_extension("png");

        let c_in = CString::new(fixture_path.to_str().unwrap()).unwrap();
        let c_out = CString::new(out_path.to_str().unwrap()).unwrap();
        let c_fmt = CString::new("png").unwrap();

        unsafe {
            let res = metaforge_convert_file(c_in.as_ptr(), c_out.as_ptr(), c_fmt.as_ptr());
            assert_eq!(res, 0);
            assert!(out_path.exists());
            assert!(fs::metadata(&out_path).unwrap().len() > 0);
            let _ = fs::remove_file(out_path);
        }
    }
}

#[test]
fn test_c_abi_sanitize() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../metaforge-parsers/tests/fixtures/lynx.jpeg");
    if fixture_path.exists() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let out_path = tmp.path().with_extension("jpg");

        let c_in = CString::new(fixture_path.to_str().unwrap()).unwrap();
        let c_out = CString::new(out_path.to_str().unwrap()).unwrap();

        unsafe {
            let res = metaforge_sanitize_file(c_in.as_ptr(), c_out.as_ptr());
            assert_eq!(res, 0);
            assert!(out_path.exists());
            let _ = fs::remove_file(out_path);
        }
    }
}

#[test]
fn test_adversarial_null_pointer_safety() {
    unsafe {
        // Free null pointer must not panic or segfault
        metaforge_free_string(std::ptr::null_mut());

        // Null path in scan must return error JSON
        let ptr1 = metaforge_scan_json(std::ptr::null());
        assert!(!ptr1.is_null());
        let str1 = CStr::from_ptr(ptr1).to_str().unwrap();
        assert!(str1.contains("error"));
        metaforge_free_string(ptr1);

        // Null path in audit must return error JSON
        let ptr2 = metaforge_audit_json(std::ptr::null());
        assert!(!ptr2.is_null());
        let str2 = CStr::from_ptr(ptr2).to_str().unwrap();
        assert!(str2.contains("error"));
        metaforge_free_string(ptr2);

        // Null inputs in convert must return error code
        let res1 = metaforge_convert_file(std::ptr::null(), std::ptr::null(), std::ptr::null());
        assert!(res1 < 0);

        // Null inputs in sanitize must return error code
        let res2 = metaforge_sanitize_file(std::ptr::null(), std::ptr::null());
        assert!(res2 < 0);
    }
}
