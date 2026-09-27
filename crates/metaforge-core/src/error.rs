use thiserror::Error;

pub type Result<T> = std::result::Result<T, MetaForgeError>;

/// Sovereign error domain for MetaForge operations
#[derive(Debug, Error)]
pub enum MetaForgeError {
    #[error("File not found or unreadable: {0}")]
    FileNotFound(String),

    #[error("File is empty: {0}")]
    EmptyFile(String),

    #[error("File size {size} bytes exceeds maximum supported allocation limit of {max_size} bytes")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("Invalid container signature for {path}: expected {expected}")]
    InvalidSignature { path: String, expected: String },

    #[error("Truncated {container} structure: offset {offset} exceeds available slice length {length}")]
    TruncatedStructure {
        container: &'static str,
        offset: usize,
        length: usize,
    },

    #[error("Corrupted {container} chunk '{chunk_type}': {detail}")]
    CorruptedChunk {
        container: &'static str,
        chunk_type: String,
        detail: String,
    },

    #[error("Cyclical or deeply nested ISOBMFF box hierarchy detected (depth {depth} exceeds limit {limit})")]
    CyclicalBoxHierarchy { depth: usize, limit: usize },

    #[error("Segment overflow: payload size {size} bytes exceeds maximum 16-bit segment capacity of 65,533 bytes")]
    SegmentOverflow { size: usize },

    #[error("Metadata sanitization failed for {path}: {detail}")]
    SanitizationError { path: String, detail: String },

    #[error("Security invariant violation: {detail}")]
    SecurityViolation { detail: String },

    #[error("Conversion error: {detail}")]
    ConversionError { detail: String },

    #[error("Unsupported media format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Allocation bomb detected: dimensions {width}x{height} exceed safety limits")]
    DimensionAllocationBomb { width: u32, height: u32 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
