#![allow(dead_code)]
use metaforge_core::MetadataEntry;
use metaforge_sanitize::escape_formula_injection;
use serde::Serialize;
use std::io;

#[derive(Serialize)]
pub struct MetadataExport<'a> {
    pub file: &'a str,
    pub count: usize,
    pub entries: &'a [MetadataEntry],
}

/// Emits metadata entries as standard pretty-printed JSON to stdout.
pub fn export_json(file_path: &str, entries: &[MetadataEntry]) -> Result<(), std::io::Error> {
    let export = MetadataExport {
        file: file_path,
        count: entries.len(),
        entries,
    };
    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| io::Error::other(e.to_string()))?;
    println!("{}", json);
    Ok(())
}

/// Emits metadata entries as single-line JSONL to stdout.
pub fn export_jsonl(file_path: &str, entries: &[MetadataEntry]) -> Result<(), std::io::Error> {
    let export = MetadataExport {
        file: file_path,
        count: entries.len(),
        entries,
    };
    let json = serde_json::to_string(&export)
        .map_err(|e| io::Error::other(e.to_string()))?;
    println!("{}", json);
    Ok(())
}

/// Emits metadata entries as formula-injection protected CSV to stdout.
pub fn export_csv(file_path: &str, entries: &[MetadataEntry]) -> Result<(), std::io::Error> {
    let stdout = io::stdout();
    let mut writer = csv::Writer::from_writer(stdout.lock());

    // Header
    writer.write_record(["file", "category", "tag_id", "key", "name", "value"])?;

    for entry in entries {
        // Formula injection mitigation: prefix formula trigger characters with '
        let safe_value = escape_formula_injection(&entry.value);
        let safe_name = escape_formula_injection(&entry.name);
        let safe_key = escape_formula_injection(&entry.key);
        let tag_id_str = entry.tag_id.map(|id| format!("0x{:04X}", id)).unwrap_or_default();

        writer.write_record([
            file_path,
            entry.category.as_str(),
            &tag_id_str,
            &safe_key,
            &safe_name,
            &safe_value,
        ])?;
    }

    writer.flush()?;
    Ok(())
}
