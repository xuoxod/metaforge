mod args;
mod export;
mod table;

use args::{AuditArgs, Cli, Commands, CommentArgs, DumpArgs, OutputFormat, SanitizeArgs, ScanArgs};
use clap::Parser;
use metaforge_core::{ContainerType, MetadataEntry, MetaForgeError, TagCategory};
use metaforge_forensics::{calculate_shannon_entropy, detect_overlay, scan_embedded_payloads};
use metaforge_parsers::{
    detect_container_type, parse_gif, parse_heic, parse_jpeg, parse_png, parse_webp,
};
use metaforge_sanitize::{read_jpeg_comments, read_png_text, scrub_image, set_jpeg_comment, set_png_text, truncate_overlay};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const MAX_FILE_SIZE_BYTES: u64 = 256 * 1024 * 1024; // 256 MB guardrail

fn read_guarded_file(path: &Path) -> Result<Vec<u8>, MetaForgeError> {
    let metadata = fs::metadata(path).map_err(|_| {
        MetaForgeError::FileNotFound(path.to_string_lossy().to_string())
    })?;

    let size = metadata.len();
    if size == 0 {
        return Err(MetaForgeError::EmptyFile(path.to_string_lossy().to_string()));
    }
    if size > MAX_FILE_SIZE_BYTES {
        return Err(MetaForgeError::FileTooLarge {
            size,
            max_size: MAX_FILE_SIZE_BYTES,
        });
    }

    fs::read(path).map_err(MetaForgeError::Io)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Scan(args)) => execute_scan(&args, cli.format),
        Some(Commands::Audit(args)) => execute_audit(&args, cli.format),
        Some(Commands::Sanitize(args)) => execute_sanitize(&args),
        Some(Commands::Comment(args)) => execute_comment(&args),
        Some(Commands::Dump(args)) => execute_dump(&args),
        None => {
            if let Some(file) = cli.file {
                let scan_args = ScanArgs {
                    file,
                    exif_only: false,
                    gps_only: false,
                };
                execute_scan(&scan_args, cli.format)
            } else {
                eprintln!("Error: No input file or subcommand specified.");
                eprintln!("Run 'metaforge --help' for usage instructions.");
                return ExitCode::FAILURE;
            }
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("\n❌ Error: {}\n", err);
            ExitCode::FAILURE
        }
    }
}

fn execute_scan(args: &ScanArgs, format: OutputFormat) -> Result<(), MetaForgeError> {
    let path = &args.file;
    let bytes = read_guarded_file(path)?;
    let container = detect_container_type(&bytes, path)?;
    let file_str = path.to_string_lossy().to_string();

    let mut entries = Vec::new();

    // 1. Container Basic Metadata
    entries.push(MetadataEntry::new(
        TagCategory::Forensics,
        None,
        "Container",
        "Container Format",
        container.display_name(),
    ));
    entries.push(MetadataEntry::new(
        TagCategory::Forensics,
        None,
        "FileSize",
        "File Size",
        format!("{} bytes ({:.2} KB)", bytes.len(), bytes.len() as f64 / 1024.0),
    ));

    // 2. Parse by format
    match container {
        ContainerType::Jpeg => {
            let info = parse_jpeg(&bytes)?;
            if let (Some(w), Some(h)) = (info.width, info.height) {
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0100),
                    "ImageWidth",
                    "Image Width",
                    format!("{} px", w),
                ));
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0101),
                    "ImageHeight",
                    "Image Height",
                    format!("{} px", h),
                ));
            }
            if let Some(c) = info.comment {
                entries.push(MetadataEntry::new(
                    TagCategory::Text,
                    None,
                    "JPEGComment",
                    "JPEG Comment",
                    c,
                ));
            }
            append_exif_metadata(&mut entries, &info.metadata);
        }
        ContainerType::Png => {
            let info = parse_png(&bytes)?;
            if let Some(h) = info.header {
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0100),
                    "ImageWidth",
                    "Image Width",
                    format!("{} px", h.width),
                ));
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0101),
                    "ImageHeight",
                    "Image Height",
                    format!("{} px", h.height),
                ));
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    None,
                    "BitDepth",
                    "Bit Depth",
                    format!("{} bits", h.bit_depth),
                ));
            }
            for (key, val) in info.text_metadata {
                entries.push(MetadataEntry::new(
                    TagCategory::Text,
                    None,
                    format!("PNG:{}", key),
                    format!("PNG Text ({})", key),
                    val,
                ));
            }
            append_exif_metadata(&mut entries, &info.metadata);
        }
        ContainerType::Webp => {
            let info = parse_webp(&bytes)?;
            if let (Some(w), Some(h)) = (info.width, info.height) {
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0100),
                    "ImageWidth",
                    "Image Width",
                    format!("{} px", w),
                ));
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0101),
                    "ImageHeight",
                    "Image Height",
                    format!("{} px", h),
                ));
            }
            append_exif_metadata(&mut entries, &info.metadata);
        }
        ContainerType::Gif => {
            let info = parse_gif(&bytes)?;
            entries.push(MetadataEntry::new(
                TagCategory::Tiff,
                Some(0x0100),
                "ImageWidth",
                "Image Width",
                format!("{} px", info.width),
            ));
            entries.push(MetadataEntry::new(
                TagCategory::Tiff,
                Some(0x0101),
                "ImageHeight",
                "Image Height",
                format!("{} px", info.height),
            ));
            if let Some(c) = info.comment {
                entries.push(MetadataEntry::new(
                    TagCategory::Text,
                    None,
                    "GIFComment",
                    "GIF Comment",
                    c,
                ));
            }
            append_exif_metadata(&mut entries, &info.metadata);
        }
        ContainerType::Heic => {
            let info = parse_heic(&bytes)?;
            if let (Some(w), Some(h)) = (info.width, info.height) {
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0100),
                    "ImageWidth",
                    "Image Width",
                    format!("{} px", w),
                ));
                entries.push(MetadataEntry::new(
                    TagCategory::Tiff,
                    Some(0x0101),
                    "ImageHeight",
                    "Image Height",
                    format!("{} px", h),
                ));
            }
            append_exif_metadata(&mut entries, &info.metadata);
        }
        ContainerType::Unknown => {}
    }

    // Apply filtering
    if args.exif_only {
        entries.retain(|e| e.category == TagCategory::Exif);
    } else if args.gps_only {
        entries.retain(|e| e.category == TagCategory::Gps);
    }

    match format {
        OutputFormat::Table => {
            let rendered = table::render_metadata_table(&file_str, &entries);
            println!("{}", rendered);
        }
        OutputFormat::Json => {
            export::export_json(&file_str, &entries).map_err(MetaForgeError::Io)?;
        }
        OutputFormat::Jsonl => {
            export::export_jsonl(&file_str, &entries).map_err(MetaForgeError::Io)?;
        }
        OutputFormat::Csv => {
            export::export_csv(&file_str, &entries).map_err(MetaForgeError::Io)?;
        }
    }

    Ok(())
}

fn append_exif_metadata(entries: &mut Vec<MetadataEntry>, meta: &metaforge_parsers::ExifMetadata) {
    if let Some(ref make) = meta.camera_make {
        entries.push(MetadataEntry::new(TagCategory::Tiff, Some(0x010F), "Make", "Camera Make", make));
    }
    if let Some(ref model) = meta.camera_model {
        entries.push(MetadataEntry::new(TagCategory::Tiff, Some(0x0110), "Model", "Camera Model", model));
    }
    if let Some(ref software) = meta.software {
        entries.push(MetadataEntry::new(TagCategory::Tiff, Some(0x0131), "Software", "Software", software));
    }
    if let Some(ref dto) = meta.date_time_original {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0x9003), "DateTimeOriginal", "Date/Time Original", dto));
    }
    if let Some(iso) = meta.iso {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0x8827), "ISOSpeedRatings", "ISO Speed", iso.to_string()));
    }
    if let Some(f_num) = meta.f_number {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0x829D), "FNumber", "F-Number", format!("f/{:.1}", f_num)));
    }
    if let Some(ref exp) = meta.exposure_time {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0x829A), "ExposureTime", "Exposure Time", format!("{} s", exp)));
    }
    if let Some(focal) = meta.focal_length_mm {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0x920A), "FocalLength", "Focal Length", format!("{:.1} mm", focal)));
    }
    if let Some(ref lmake) = meta.lens_make {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0xA433), "LensMake", "Lens Make", lmake));
    }
    if let Some(ref lmodel) = meta.lens_model {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0xA434), "LensModel", "Lens Model", lmodel));
    }
    if let Some(ref bserial) = meta.body_serial_number {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0xA431), "BodySerialNumber", "Camera Serial", bserial));
    }
    if let Some(ref lserial) = meta.lens_serial_number {
        entries.push(MetadataEntry::new(TagCategory::Exif, Some(0xA435), "LensSerialNumber", "Lens Serial", lserial));
    }

    // GPS Tags
    if let Some(ref gps) = meta.gps {
        if let Some(lat) = gps.latitude {
            entries.push(MetadataEntry::new(TagCategory::Gps, Some(0x0002), "GPSLatitude", "GPS Latitude", format!("{:.6}°", lat)));
        }
        if let Some(lon) = gps.longitude {
            entries.push(MetadataEntry::new(TagCategory::Gps, Some(0x0004), "GPSLongitude", "GPS Longitude", format!("{:.6}°", lon)));
        }
        if let Some(alt) = gps.altitude {
            entries.push(MetadataEntry::new(TagCategory::Gps, Some(0x0006), "GPSAltitude", "GPS Altitude", format!("{:.2} m", alt)));
        }
        if let Some(speed) = gps.speed {
            entries.push(MetadataEntry::new(TagCategory::Gps, Some(0x000D), "GPSSpeed", "GPS Speed", format!("{:.2}", speed)));
        }
        if let Some(track) = gps.track {
            entries.push(MetadataEntry::new(TagCategory::Gps, Some(0x000F), "GPSTrack", "GPS Track", format!("{:.2}°", track)));
        }
    }
}

fn execute_audit(args: &AuditArgs, _format: OutputFormat) -> Result<(), MetaForgeError> {
    let path = &args.file;
    let bytes = read_guarded_file(path)?;
    let container = detect_container_type(&bytes, path)?;
    let file_str = path.to_string_lossy().to_string();

    let official_end_offset = match container {
        ContainerType::Jpeg => parse_jpeg(&bytes)?.official_end_offset,
        ContainerType::Png => parse_png(&bytes)?.official_end_offset,
        ContainerType::Webp => parse_webp(&bytes)?.official_end_offset,
        ContainerType::Gif => parse_gif(&bytes)?.official_end_offset,
        ContainerType::Heic => parse_heic(&bytes)?.official_end_offset,
        ContainerType::Unknown => bytes.len(),
    };

    let entropy = calculate_shannon_entropy(&bytes);
    let is_suspicious_entropy = entropy >= args.entropy_threshold;
    let overlay = detect_overlay(&bytes, official_end_offset);
    let payloads = scan_embedded_payloads(&bytes, official_end_offset);

    let output = table::render_audit_table(&file_str, entropy, is_suspicious_entropy, &overlay, &payloads);
    println!("{}", output);

    Ok(())
}

fn execute_sanitize(args: &SanitizeArgs) -> Result<(), MetaForgeError> {
    let path = &args.file;
    let bytes = read_guarded_file(path)?;
    let original_size = bytes.len();

    let (cleaned_bytes, overlay_truncated, meta_stripped, bytes_removed) = if args.overlay_only {
        let (cleaned, truncated) = truncate_overlay(&bytes)?;
        let removed = original_size.saturating_sub(cleaned.len());
        (cleaned, truncated, false, removed)
    } else {
        let (cleaned, report) = scrub_image(&bytes)?;
        (cleaned, report.overlay_truncated, report.metadata_stripped, report.bytes_removed)
    };

    let target_path = if args.inplace {
        path.clone()
    } else if let Some(ref out) = args.out {
        out.clone()
    } else {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("clean");
        let new_name = format!("{}.clean.{}", stem, ext);
        path.with_file_name(new_name)
    };

    fs::write(&target_path, &cleaned_bytes).map_err(MetaForgeError::Io)?;

    println!("\n✨ Sanitization Complete: {}", target_path.display());
    println!(" • Original Size:     {} bytes", original_size);
    println!(" • Scrubbed Size:     {} bytes", cleaned_bytes.len());
    println!(" • Space Saved:       {} bytes", bytes_removed);
    println!(" • Overlay Removed:   {}", if overlay_truncated { "YES" } else { "NO" });
    println!(" • Metadata Stripped: {}\n", if meta_stripped { "YES" } else { "NO" });

    Ok(())
}

fn execute_comment(args: &CommentArgs) -> Result<(), MetaForgeError> {
    let path = &args.file;
    let bytes = read_guarded_file(path)?;
    let container = detect_container_type(&bytes, path)?;

    // Deletion
    if args.delete {
        let cleaned = match container {
            ContainerType::Jpeg => set_jpeg_comment(&bytes, "")?,
            ContainerType::Png => set_png_text(&bytes, &args.key, "")?,
            _ => {
                return Err(MetaForgeError::SanitizationError {
                    path: path.to_string_lossy().to_string(),
                    detail: "Comment editing is only supported for JPEG and PNG containers".to_string(),
                });
            }
        };
        let out_path = args.out.as_ref().unwrap_or(path);
        fs::write(out_path, cleaned).map_err(MetaForgeError::Io)?;
        println!("🗑️  Deleted comment from {}", out_path.display());
        return Ok(());
    }

    // Setting
    if let Some(ref text) = args.set {
        let updated = match container {
            ContainerType::Jpeg => set_jpeg_comment(&bytes, text)?,
            ContainerType::Png => set_png_text(&bytes, &args.key, text)?,
            _ => {
                return Err(MetaForgeError::SanitizationError {
                    path: path.to_string_lossy().to_string(),
                    detail: "Comment editing is only supported for JPEG and PNG containers".to_string(),
                });
            }
        };
        let out_path = args.out.as_ref().unwrap_or(path);
        fs::write(out_path, updated).map_err(MetaForgeError::Io)?;
        println!("✍️  Updated comment in {}", out_path.display());
        return Ok(());
    }

    // Reading
    match container {
        ContainerType::Jpeg => {
            let comments = read_jpeg_comments(&bytes)?;
            if comments.is_empty() {
                println!("No JPEG comments found in {}", path.display());
            } else {
                println!("\n📝 JPEG Comments in {}:", path.display());
                for (idx, c) in comments.iter().enumerate() {
                    println!("  [{}] {}", idx + 1, c);
                }
                println!();
            }
        }
        ContainerType::Png => {
            let texts = read_png_text(&bytes)?;
            if texts.is_empty() {
                println!("No PNG text chunks found in {}", path.display());
            } else {
                println!("\n📝 PNG Text Chunks in {}:", path.display());
                for (k, v) in texts {
                    println!("  • {}: {}", k, v);
                }
                println!();
            }
        }
        _ => {
            println!("Comment reading not supported for container: {:?}", container);
        }
    }

    Ok(())
}

fn execute_dump(args: &DumpArgs) -> Result<(), MetaForgeError> {
    let path = &args.file;
    let bytes = read_guarded_file(path)?;
    let container = detect_container_type(&bytes, path)?;

    println!("\n🔍 Container Structure Dump: {}", path.display());
    println!("Container Type: {}\n", container.display_name());

    match container {
        ContainerType::Jpeg => {
            let info = parse_jpeg(&bytes)?;
            println!("Found {} segments:", info.segments.len());
            for seg in info.segments {
                println!(
                    "  [0x{:08X}] 0xFF{:02X} {:<24} (length: {} bytes)",
                    seg.offset, seg.marker, seg.name, seg.length
                );
            }
        }
        ContainerType::Png => {
            let info = parse_png(&bytes)?;
            println!("Found {} chunks:", info.chunks.len());
            for chunk in info.chunks {
                let crc_status = if chunk.crc_valid { "CRC OK" } else { "CRC INVALID" };
                println!(
                    "  [0x{:08X}] {:<6} length: {:<8} CRC: 0x{:08X} ({})",
                    chunk.offset, chunk.type_name, chunk.length, chunk.crc, crc_status
                );
            }
        }
        ContainerType::Webp => {
            let info = parse_webp(&bytes)?;
            println!("Found {} RIFF chunks:", info.chunks.len());
            for c in info.chunks {
                println!("  [0x{:08X}] {:<6} length: {} bytes", c.offset, c.tag, c.length);
            }
        }
        ContainerType::Gif => {
            let info = parse_gif(&bytes)?;
            println!("Found {} GIF blocks:", info.blocks.len());
            for b in info.blocks {
                println!("  [0x{:08X}] {:<24} length: {} bytes", b.offset, b.block_type, b.length);
            }
        }
        ContainerType::Heic => {
            let info = parse_heic(&bytes)?;
            println!("Found {} ISOBMFF boxes:", info.boxes.len());
            for b in info.boxes {
                println!("  [0x{:08X}] {:<8} length: {} bytes", b.offset, b.box_type, b.length);
            }
        }
        ContainerType::Unknown => {
            println!("Unknown container type.");
        }
    }
    println!();
    Ok(())
}
