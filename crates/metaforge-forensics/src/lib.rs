pub mod entropy;
pub mod overlay;
pub mod signatures;

pub use entropy::{calculate_shannon_entropy, is_suspicious_entropy};
pub use overlay::{detect_overlay, OverlayInfo};
pub use signatures::{scan_embedded_payloads, EmbeddedPayload};
