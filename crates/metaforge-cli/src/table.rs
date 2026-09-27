#![allow(dead_code)]
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use metaforge_core::MetadataEntry;
use metaforge_forensics::{EmbeddedPayload, OverlayInfo, SteganalysisReport};

/// Scrubs raw ANSI CSI escape codes, OSC sequences, and unprintable controls
/// to prevent terminal display hijacking and log poisoning attacks.
pub fn scrub_terminal_poisoning(input: &str) -> String {
    let mut clean = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1B' {
            // Escape code: check for CSI '[' or OSC ']'
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next(); // consume '['
                    // Consume characters until terminator (0x40 - 0x7E)
                    for term in chars.by_ref() {
                        if (0x40..=0x7E).contains(&(term as u32)) {
                            break;
                        }
                    }
                    continue;
                } else if next == ']' {
                    chars.next(); // consume ']'
                    // Consume until BEL (\x07) or ST (\x1B\)
                    while let Some(osc_c) = chars.next() {
                        if osc_c == '\x07' {
                            break;
                        }
                        if osc_c == '\x1B' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
            }
            continue;
        }

        // Filter unprintable control characters, but allow tab and newline if needed
        if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
            continue;
        }

        clean.push(c);
    }

    clean
}

/// Formats a list of metadata entries into an elegant Unicode terminal table.
pub fn render_metadata_table(file_path: &str, entries: &[MetadataEntry]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Category").fg(Color::Cyan),
            Cell::new("Tag / Property").fg(Color::Green),
            Cell::new("Value").fg(Color::White),
        ]);

    for entry in entries {
        let clean_key = scrub_terminal_poisoning(&entry.name);
        let clean_val = scrub_terminal_poisoning(&entry.value);
        let cat_str = entry.category.as_str();

        table.add_row(Row::from(vec![
            Cell::new(cat_str).fg(Color::Yellow),
            Cell::new(clean_key),
            Cell::new(clean_val),
        ]));
    }

    format!("\n📊 Metadata Report: {}\n{}\n", file_path, table)
}

/// Formats a forensics audit report into an elegant Unicode terminal table.
pub fn render_audit_table(
    file_path: &str,
    entropy: f64,
    is_suspicious_entropy: bool,
    overlay: &OverlayInfo,
    payloads: &[EmbeddedPayload],
    stego: Option<&SteganalysisReport>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("\n🛡️  Sovereign Forensics & Steganography Audit: {}\n", file_path));

    let mut summary_table = Table::new();
    summary_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Diagnostic Check").fg(Color::Cyan),
            Cell::new("Result").fg(Color::White),
            Cell::new("Verdict").fg(Color::Yellow),
        ]);

    let entropy_verdict = if is_suspicious_entropy {
        Cell::new("SUSPICIOUS (Possible Encryption/Stego)").fg(Color::Red)
    } else {
        Cell::new("NORMAL").fg(Color::Green)
    };

    summary_table.add_row(Row::from(vec![
        Cell::new("Shannon Entropy"),
        Cell::new(format!("{:.4} / 8.0000", entropy)),
        entropy_verdict,
    ]));

    let overlay_verdict = if overlay.has_overlay {
        Cell::new(format!("DETECTED (+{} bytes)", overlay.overlay_size)).fg(Color::Red)
    } else {
        Cell::new("CLEAN (0 bytes)").fg(Color::Green)
    };

    summary_table.add_row(Row::from(vec![
        Cell::new("Trailing Overlay (Post-EOF)"),
        Cell::new(format!(
            "Logical: {} B | Physical: {} B",
            overlay.logical_size, overlay.physical_size
        )),
        overlay_verdict,
    ]));

    let payload_verdict = if !payloads.is_empty() {
        Cell::new(format!("ALERT: {} Found", payloads.len())).fg(Color::Red)
    } else {
        Cell::new("CLEAN (None Detected)").fg(Color::Green)
    };

    summary_table.add_row(Row::from(vec![
        Cell::new("Embedded Polyglot Signatures"),
        Cell::new(format!("{} candidate patterns inspected", payloads.len())),
        payload_verdict,
    ]));

    if let Some(st) = stego {
        let stego_verdict = if st.is_suspicious {
            Cell::new(format!("ALERT ({} Anomalies)", st.findings.len())).fg(Color::Red)
        } else {
            Cell::new("CLEAN (Chi-Square Verified)").fg(Color::Green)
        };

        summary_table.add_row(Row::from(vec![
            Cell::new("Chi-Square Steganalysis"),
            Cell::new(format!(
                "Stat: {:.2} (df={}, p={:.4}) | Scan: {} B",
                st.chi_square_stat, st.degrees_of_freedom, st.p_value_approx, st.scan_data_bytes
            )),
            stego_verdict,
        ]));
    }

    output.push_str(&format!("{}\n", summary_table));

    if let Some(st) = stego {
        if !st.findings.is_empty() {
            output.push_str("\n🔍 Steganalysis Anomalies & Statistical Deviations:\n");
            let mut stable = Table::new();
            stable
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new("Category").fg(Color::Cyan),
                    Cell::new("Severity").fg(Color::Yellow),
                    Cell::new("Observation").fg(Color::White),
                ]);

            for f in &st.findings {
                let sev_cell = match f.severity {
                    metaforge_forensics::FindingSeverity::Info => Cell::new("INFO").fg(Color::Blue),
                    metaforge_forensics::FindingSeverity::Low => Cell::new("LOW").fg(Color::Yellow),
                    metaforge_forensics::FindingSeverity::Medium => Cell::new("MEDIUM").fg(Color::Red),
                    metaforge_forensics::FindingSeverity::High => Cell::new("HIGH").fg(Color::Red),
                    metaforge_forensics::FindingSeverity::Critical => Cell::new("CRITICAL").fg(Color::Red),
                };
                stable.add_row(Row::from(vec![
                    Cell::new(format!("{:?}", f.finding_type)),
                    sev_cell,
                    Cell::new(scrub_terminal_poisoning(&f.description)),
                ]));
            }
            output.push_str(&format!("{}\n", stable));
        }
    }

    if !payloads.is_empty() {
        output.push_str("\n🚨 Detected Signatures & Payloads:\n");
        let mut ptable = Table::new();
        ptable
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("Offset").fg(Color::Cyan),
                Cell::new("Category").fg(Color::Yellow),
                Cell::new("Payload Signature").fg(Color::Red),
                Cell::new("Byte Preview").fg(Color::White),
            ]);

        for p in payloads {
            ptable.add_row(Row::from(vec![
                Cell::new(format!("0x{:08X}", p.offset)),
                Cell::new(&p.category),
                Cell::new(&p.name),
                Cell::new(scrub_terminal_poisoning(&p.preview)),
            ]));
        }
        output.push_str(&format!("{}\n", ptable));
    }

    output
}
