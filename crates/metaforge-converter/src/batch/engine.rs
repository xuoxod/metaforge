//! Sovereign Concurrent Batch Conversion Engine
//!
//! Traverses directories recursively, preserves relative directory trees,
//! and executes conversions concurrently under an invariant concurrency cap (max 4 workers).

use crate::pipeline::{AudioPipeline, ImagePipeline};
use crate::types::{
    is_audio_or_video_extension, BatchConvertOptions, BatchItemReport, BatchItemStatus, BatchReport,
    ImageTargetFormat,
};
use metaforge_core::error::{MetaForgeError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

/// Run a batch conversion job across a directory tree.
pub fn execute_batch_convert(options: &BatchConvertOptions) -> Result<BatchReport> {
    let start_time = Instant::now();

    if !options.input_dir.exists() {
        return Err(MetaForgeError::FileNotFound(format!(
            "Input directory '{}' does not exist",
            options.input_dir.display()
        )));
    }

    // 1. Discover all candidate files
    let mut files = Vec::new();
    collect_files_recursive(&options.input_dir, &mut files)?;

    // Filter by requested extensions if provided
    let allowed_exts: Vec<String> = options
        .extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
        .collect();

    let candidates: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| {
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_ascii_lowercase();
                if allowed_exts.is_empty() {
                    // Default: accept any supported audio, video, or image format
                    is_audio_or_video_extension(&ext_lower)
                        || ImageTargetFormat::from_extension(&ext_lower).is_some()
                } else {
                    allowed_exts.contains(&ext_lower)
                }
            } else {
                false
            }
        })
        .collect();

    let total_scanned = candidates.len();

    // 2. Pre-flight Dry-Run planning
    if options.dry_run {
        let mut dry_items = Vec::with_capacity(total_scanned);
        let mut total_in_bytes = 0u64;

        for inp in &candidates {
            let in_bytes = fs::metadata(inp).map(|m| m.len()).unwrap_or(0);
            total_in_bytes += in_bytes;

            let outp = compute_output_path(inp, options);
            dry_items.push(BatchItemReport {
                input_path: inp.to_string_lossy().into_owned(),
                output_path: outp.to_string_lossy().into_owned(),
                input_bytes: in_bytes,
                output_bytes: 0,
                status: BatchItemStatus::Success,
                duration_ms: 0,
            });
        }

        return Ok(BatchReport {
            total_scanned,
            converted: total_scanned,
            skipped: 0,
            failed: 0,
            total_input_bytes: total_in_bytes,
            total_output_bytes: 0,
            items: dry_items,
            elapsed_ms: start_time.elapsed().as_millis() as u64,
            dry_run: true,
        });
    }

    // Ensure output directory root exists
    fs::create_dir_all(&options.output_dir)
        .map_err(|e| MetaForgeError::Io(std::io::Error::other(format!("{}: {}", options.output_dir.display(), e))))?;

    // 3. Worker Concurrency: strictly clamp max workers to [1, 4] per Rule 4
    let num_workers = options.max_workers.clamp(1, 4);
    let (tx, rx) = mpsc::channel();

    // Chunk candidate files across workers
    let items_per_worker = candidates.len().div_ceil(num_workers);
    let chunks: Vec<Vec<PathBuf>> = candidates
        .chunks(if items_per_worker == 0 { 1 } else { items_per_worker })
        .map(|c| c.to_vec())
        .collect();

    std::thread::scope(|s| {
        for chunk in chunks {
            let tx_clone = tx.clone();
            let opts_clone = options.clone();

            s.spawn(move || {
                for inp in chunk {
                    let item_start = Instant::now();
                    let outp = compute_output_path(&inp, &opts_clone);

                    // Ensure target parent directory exists
                    if let Some(parent) = outp.parent() {
                        let _ = fs::create_dir_all(parent);
                    }

                    let in_bytes = fs::metadata(&inp).map(|m| m.len()).unwrap_or(0);
                    let ext = inp
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();

                    let result = if is_audio_or_video_extension(&ext) {
                        let pipeline = AudioPipeline::from_options(&opts_clone.audio_options);
                        pipeline.process_file(&inp, &outp)
                    } else {
                        let target_img_fmt = ImageTargetFormat::from_extension(&opts_clone.target_format)
                            .unwrap_or(ImageTargetFormat::Png);
                        let pipeline = ImagePipeline::new()
                            .target(target_img_fmt)
                            .quality(opts_clone.image_options.quality.unwrap_or(85));
                        pipeline.process_file(&inp, &outp)
                    };

                    let (status, out_bytes) = match result {
                        Ok(rep) => (BatchItemStatus::Success, rep.output_bytes as u64),
                        Err(e) => (BatchItemStatus::Failed(e.to_string()), 0),
                    };

                    let item_report = BatchItemReport {
                        input_path: inp.to_string_lossy().into_owned(),
                        output_path: outp.to_string_lossy().into_owned(),
                        input_bytes: in_bytes,
                        output_bytes: out_bytes,
                        status,
                        duration_ms: item_start.elapsed().as_millis() as u64,
                    };

                    let _ = tx_clone.send(item_report);
                }
            });
        }
    });

    drop(tx); // Close original transmitter

    // 4. Collect results
    let mut items = Vec::with_capacity(total_scanned);
    let mut converted = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut total_input_bytes = 0u64;
    let mut total_output_bytes = 0u64;

    for item in rx {
        total_input_bytes += item.input_bytes;
        total_output_bytes += item.output_bytes;
        match item.status {
            BatchItemStatus::Success => converted += 1,
            BatchItemStatus::Skipped => skipped += 1,
            BatchItemStatus::Failed(_) => failed += 1,
        }
        items.push(item);
    }

    Ok(BatchReport {
        total_scanned,
        converted,
        skipped,
        failed,
        total_input_bytes,
        total_output_bytes,
        items,
        elapsed_ms: start_time.elapsed().as_millis() as u64,
        dry_run: false,
    })
}

fn compute_output_path(input_file: &Path, options: &BatchConvertOptions) -> PathBuf {
    let target_ext = if !options.target_format.is_empty() {
        options.target_format.as_str()
    } else {
        // Fallback default
        let in_ext = input_file
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if is_audio_or_video_extension(&in_ext) {
            "wav"
        } else {
            "png"
        }
    };

    if options.flatten {
        let file_stem = input_file.file_stem().unwrap_or_default();
        options.output_dir.join(file_stem).with_extension(target_ext)
    } else {
        let rel_path = input_file
            .strip_prefix(&options.input_dir)
            .unwrap_or(input_file);
        options.output_dir.join(rel_path).with_extension(target_ext)
    }
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        MetaForgeError::Io(std::io::Error::other(format!("{}: {}", dir.display(), e)))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            MetaForgeError::Io(std::io::Error::other(format!("{}: {}", dir.display(), e)))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }

    Ok(())
}
