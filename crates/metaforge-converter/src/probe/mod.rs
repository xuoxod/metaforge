//! Sovereign Media Probing & Stream Introspection Subsystem

pub mod inspector;

pub use inspector::{probe_media_bytes, probe_media_file};
