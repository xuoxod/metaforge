use serde::{Deserialize, Serialize};

/// High-level container type recognized by MetaForge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerType {
    Jpeg,
    Png,
    Webp,
    Gif,
    Heic,
    Unknown,
}

impl ContainerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerType::Jpeg => "jpeg",
            ContainerType::Png => "png",
            ContainerType::Webp => "webp",
            ContainerType::Gif => "gif",
            ContainerType::Heic => "heic",
            ContainerType::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ContainerType::Jpeg => "JPEG Image Container",
            ContainerType::Png => "PNG Image Container",
            ContainerType::Webp => "WebP RIFF Container",
            ContainerType::Gif => "GIF Image Container",
            ContainerType::Heic => "HEIC/HEIF ISOBMFF Container",
            ContainerType::Unknown => "Unknown Binary Container",
        }
    }
}

/// Metadata domain categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagCategory {
    Exif,
    Gps,
    Tiff,
    Xmp,
    Text,
    Forensics,
}

impl TagCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagCategory::Exif => "EXIF",
            TagCategory::Gps => "GPS",
            TagCategory::Tiff => "TIFF",
            TagCategory::Xmp => "XMP",
            TagCategory::Text => "TEXT",
            TagCategory::Forensics => "FORENSICS",
        }
    }
}

/// A normalized, human-readable metadata entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub category: TagCategory,
    pub tag_id: Option<u16>,
    pub key: String,
    pub name: String,
    pub value: String,
}

impl MetadataEntry {
    pub fn new(category: TagCategory, tag_id: Option<u16>, key: impl Into<String>, name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            category,
            tag_id,
            key: key.into(),
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Basic dimensions and color geometry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
    pub bit_depth: Option<u8>,
    pub color_components: Option<u8>,
}
