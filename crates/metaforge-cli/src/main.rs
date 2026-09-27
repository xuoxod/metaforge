mod args;
mod export;
mod table;

use args::{
    AuditArgs, BatchArgs, Cli, Commands, CommentArgs, ConvertArgs, DumpArgs, OutputFormat,
    ProbeArgs, SanitizeArgs, ScanArgs,
};
use clap::Parser;
use metaforge_converter::{
    convert_media_file, convert_stream, detect_target_format_from_path, execute_batch_convert,
    probe_media_file, AudioConvertOptions, BatchConvertOptions, ConvertReport, ImageConvertOptions,
    ImageTargetFormat,
};
use metaforge_core::{ContainerType, MetadataEntry, MetaForgeError, TagCategory};
use metaforge_forensics::{
    analyze_jpeg_steganography, calculate_shannon_entropy, detect_overlay, scan_embedded_payloads,
};
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
        Some(Commands::Convert(args)) => execute_convert(&args, cli.format),
        Some(Commands::Batch(args)) => execute_batch(&args, cli.format),
        Some(Commands::Probe(args)) => execute_probe(&args, cli.format),
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

    let stego = if container == ContainerType::Jpeg {
        Some(analyze_jpeg_steganography(&bytes))
    } else {
        None
    };

    let output = table::render_audit_table(
        &file_str,
        entropy,
        is_suspicious_entropy,
        &overlay,
        &payloads,
        stego.as_ref(),
    );
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

fn print_convert_report(report: &ConvertReport, format: OutputFormat) -> Result<(), MetaForgeError> {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json = serde_json::to_string_pretty(report)
                .map_err(|e| MetaForgeError::Io(std::io::Error::other(e.to_string())))?;
            println!("{}", json);
        }
        _ => {
            println!("\n🔄 Transcode Complete: {} ➔ {}", report.input_path, report.output_path);
            let mut t = comfy_table::Table::new();
            t.load_preset(comfy_table::presets::UTF8_FULL);
            t.set_header(vec!["Property", "Value"]);
            t.add_row(vec![
                "Input Size",
                &format!("{} bytes ({:.2} KB)", report.input_bytes, report.input_bytes as f64 / 1024.0),
            ]);
            t.add_row(vec![
                "Output Size",
                &format!("{} bytes ({:.2} KB)", report.output_bytes, report.output_bytes as f64 / 1024.0),
            ]);
            t.add_row(vec!["Target Format", &report.output_format]);
            t.add_row(vec!["Details", &report.details]);
            println!("{t}\n");
        }
    }
    Ok(())
}

fn execute_convert(args: &ConvertArgs, format: OutputFormat) -> Result<(), MetaForgeError> {
    let is_stdin = args.input.as_os_str() == "-";
    let is_stdout = args.output.as_os_str() == "-";

    let audio_opts = AudioConvertOptions {
        target_sample_rate: args.rate,
        target_channels: args.channels,
        target_bits_per_sample: None,
        gain: args.gain,
        normalize: args.normalize,
    };

    let target_img_fmt = if let Some(ref f) = args.target {
        ImageTargetFormat::from_extension(f).unwrap_or(ImageTargetFormat::Png)
    } else {
        detect_target_format_from_path(&args.output).unwrap_or(ImageTargetFormat::Png)
    };

    let resize = if let Some(ref r) = args.resize {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            let w: u32 = parts[0].trim().parse().map_err(|_| MetaForgeError::ConversionError {
                detail: format!("Invalid resize width: '{}'", parts[0]),
            })?;
            let h: u32 = parts[1].trim().parse().map_err(|_| MetaForgeError::ConversionError {
                detail: format!("Invalid resize height: '{}'", parts[1]),
            })?;
            Some((w, h))
        } else {
            return Err(MetaForgeError::ConversionError {
                detail: format!("Invalid resize format '{}'. Expected WxH (e.g. 800x600)", r),
            });
        }
    } else {
        None
    };

    let img_opts = ImageConvertOptions {
        target_format: target_img_fmt,
        quality: Some(args.quality),
        resize,
        preserve_metadata: true,
        max_dimension: 16384,
    };

    // Stdio streaming pipe support
    if is_stdin || is_stdout {
        let target_fmt_str = if let Some(ref f) = args.target {
            f.clone()
        } else if !is_stdout {
            args.output
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("wav")
                .to_string()
        } else {
            return Err(MetaForgeError::ConversionError {
                detail: "Explicit --target (e.g. -t wav, -t webp) is required when streaming to stdout '-'".to_string(),
            });
        };

        let in_hint = if !is_stdin {
            args.input.extension().and_then(|s| s.to_str())
        } else {
            None
        };

        if args.dry_run {
            println!(
                "🔍 Dry-run: Streaming transcode from {} to {} planned (format: {})",
                if is_stdin { "<stdin>" } else { args.input.to_str().unwrap_or("input") },
                if is_stdout { "<stdout>" } else { args.output.to_str().unwrap_or("output") },
                target_fmt_str
            );
            return Ok(());
        }

        let report = if is_stdin && is_stdout {
            convert_stream(
                std::io::stdin().lock(),
                std::io::stdout().lock(),
                in_hint,
                &target_fmt_str,
                &img_opts,
                &audio_opts,
            )?
        } else if is_stdin {
            let out_file = std::fs::File::create(&args.output)?;
            convert_stream(
                std::io::stdin().lock(),
                out_file,
                in_hint,
                &target_fmt_str,
                &img_opts,
                &audio_opts,
            )?
        } else {
            let in_file = std::fs::File::open(&args.input)?;
            convert_stream(
                in_file,
                std::io::stdout().lock(),
                in_hint,
                &target_fmt_str,
                &img_opts,
                &audio_opts,
            )?
        };

        if !is_stdout {
            print_convert_report(&report, format)?;
        }
        return Ok(());
    }

    if args.dry_run {
        let in_meta = fs::metadata(&args.input)?;
        println!("\n🔍 Dry-run: Conversion Pre-Flight Estimation");
        println!("  Input Path:   {}", args.input.display());
        println!("  Output Path:  {}", args.output.display());
        println!(
            "  Input Size:   {} bytes ({:.2} KB)",
            in_meta.len(),
            in_meta.len() as f64 / 1024.0
        );
        println!("  Target Fmt:   {}", target_img_fmt.to_extension());
        println!("  Operation:    Transcode to target container format\n");
        return Ok(());
    }

    let report = convert_media_file(&args.input, &args.output, &img_opts, &audio_opts)?;
    print_convert_report(&report, format)?;
    Ok(())
}

fn execute_batch(args: &BatchArgs, format: OutputFormat) -> Result<(), MetaForgeError> {
    let extensions: Vec<String> = if let Some(ref e) = args.ext {
        e.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    let resize = if let Some(ref r) = args.resize {
        let parts: Vec<&str> = r.split('x').collect();
        if parts.len() == 2 {
            let w: u32 = parts[0].trim().parse().map_err(|_| MetaForgeError::ConversionError {
                detail: format!("Invalid resize width: '{}'", parts[0]),
            })?;
            let h: u32 = parts[1].trim().parse().map_err(|_| MetaForgeError::ConversionError {
                detail: format!("Invalid resize height: '{}'", parts[1]),
            })?;
            Some((w, h))
        } else {
            return Err(MetaForgeError::ConversionError {
                detail: format!("Invalid resize format '{}'. Expected WxH", r),
            });
        }
    } else {
        None
    };

    let target_img_fmt = args
        .target
        .as_deref()
        .and_then(ImageTargetFormat::from_extension)
        .unwrap_or(ImageTargetFormat::Png);

    let batch_opts = BatchConvertOptions {
        input_dir: args.input.clone(),
        output_dir: args.output.clone(),
        extensions,
        target_format: args.target.clone().unwrap_or_default(),
        max_workers: args.workers,
        dry_run: args.dry_run,
        flatten: args.flatten,
        image_options: ImageConvertOptions {
            target_format: target_img_fmt,
            quality: Some(args.quality),
            resize,
            preserve_metadata: true,
            max_dimension: 16384,
        },
        audio_options: AudioConvertOptions {
            target_sample_rate: args.rate,
            target_channels: args.channels,
            target_bits_per_sample: None,
            gain: args.gain,
            normalize: args.normalize,
        },
    };

    let report = execute_batch_convert(&batch_opts)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json = serde_json::to_string_pretty(&report)
                .map_err(|e| MetaForgeError::Io(std::io::Error::other(e.to_string())))?;
            println!("{}", json);
        }
        _ => {
            let title = if report.dry_run {
                "🔍 Batch Conversion Pre-Flight Plan (Dry Run)"
            } else {
                "🚀 Batch Conversion Completed"
            };
            println!("\n{}", title);
            let mut t = comfy_table::Table::new();
            t.load_preset(comfy_table::presets::UTF8_FULL);
            t.set_header(vec!["Batch Metric", "Value"]);
            t.add_row(vec!["Total Scanned", &report.total_scanned.to_string()]);
            t.add_row(vec!["Converted", &report.converted.to_string()]);
            t.add_row(vec!["Skipped", &report.skipped.to_string()]);
            t.add_row(vec!["Failed", &report.failed.to_string()]);
            t.add_row(vec![
                "Total Input Data",
                &format!(
                    "{} bytes ({:.2} MB)",
                    report.total_input_bytes,
                    report.total_input_bytes as f64 / 1_048_576.0
                ),
            ]);
            if !report.dry_run {
                t.add_row(vec![
                    "Total Output Data",
                    &format!(
                        "{} bytes ({:.2} MB)",
                        report.total_output_bytes,
                        report.total_output_bytes as f64 / 1_048_576.0
                    ),
                ]);
            }
            t.add_row(vec!["Elapsed Time", &format!("{} ms", report.elapsed_ms)]);
            println!("{t}\n");

            if report.failed > 0 {
                eprintln!("⚠️ Warning: {} items failed to convert.", report.failed);
            }
        }
    }

    Ok(())
}

fn execute_probe(args: &ProbeArgs, format: OutputFormat) -> Result<(), MetaForgeError> {
    let probe = probe_media_file(&args.file)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json = serde_json::to_string_pretty(&probe)
                .map_err(|e| MetaForgeError::Io(std::io::Error::other(e.to_string())))?;
            println!("{}", json);
        }
        _ => {
            println!("\n🔍 Media Container Probe: {}", probe.file_path);
            let mut t = comfy_table::Table::new();
            t.load_preset(comfy_table::presets::UTF8_FULL);
            t.set_header(vec!["Stream Property", "Value"]);
            t.add_row(vec!["Media Kind", &probe.media_kind]);
            t.add_row(vec!["Format / Container", &probe.format]);
            t.add_row(vec![
                "File Size",
                &format!(
                    "{} bytes ({:.2} KB)",
                    probe.size_bytes,
                    probe.size_bytes as f64 / 1024.0
                ),
            ]);
            if let Some((w, h)) = probe.dimensions {
                t.add_row(vec!["Dimensions", &format!("{} x {} px", w, h)]);
            }
            if let Some(dur) = probe.duration_seconds {
                t.add_row(vec!["Duration", &format!("{:.3} s", dur)]);
            }
            if probe.audio_tracks > 0 {
                t.add_row(vec!["Audio Tracks", &probe.audio_tracks.to_string()]);
            }
            if probe.video_tracks > 0 {
                t.add_row(vec!["Video Tracks", &probe.video_tracks.to_string()]);
            }
            if let Some(codec) = probe.codec_name {
                t.add_row(vec!["Codec Name", &codec]);
            }
            if let Some(rate) = probe.sample_rate {
                t.add_row(vec!["Sample Rate", &format!("{} Hz", rate)]);
            }
            if let Some(ch) = probe.channels {
                let desc = if ch == 1 {
                    "Mono (1 channel)"
                } else if ch == 2 {
                    "Stereo (2 channels)"
                } else {
                    "Multi-Channel"
                };
                t.add_row(vec!["Channels", desc]);
            }
            if let Some(bits) = probe.bits_per_sample {
                t.add_row(vec!["Bits per Sample", &format!("{}-bit", bits)]);
            }
            println!("{t}\n");
        }
    }
    Ok(())
}

