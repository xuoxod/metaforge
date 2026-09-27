pub mod entropy;
pub mod overlay;
pub mod signatures;
pub mod steganalysis;

pub use entropy::{calculate_shannon_entropy, is_suspicious_entropy};
pub use overlay::{detect_overlay, OverlayInfo};
pub use signatures::{scan_embedded_payloads, EmbeddedPayload};
pub use steganalysis::{
    analyze_comment_segment, analyze_jpeg_steganography, analyze_scan_data,
    calculate_ascii_ratio, calculate_chi_square_uniformity, calculate_pov_chi_square,
    extract_entropy_continuum, FindingSeverity, StegFindingType, SteganalysisFinding,
    SteganalysisReport, CHI_SQUARE_CRITICAL_DF255_A01, CHI_SQUARE_CRITICAL_DF255_A05,
};
