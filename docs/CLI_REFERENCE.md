# 🛠️ MetaForge CLI Reference Manual

The `metaforge` CLI provides a unified, zero-dependency sovereign toolkit for extracting, forensically auditing, editing, and scrubbing metadata across JPEG, PNG, WebP, GIF, and HEIC containers.

---

## 📡 1. Global Syntax & Ingress

```bash
metaforge [OPTIONS] [FILE] [COMMAND]
```

### 📥 Arguments & Options
*   `[FILE]`: Target image file to scan (default command if no subcommand provided).
*   `-f, --format <FORMAT>`: Output serialization format. Default: `table`.
    *   `table`: Rich Unicode monospace table with ANSI styling.
    *   `json`: Pretty-printed structured JSON object.
    *   `jsonl`: Single-line newline-delimited JSON.
    *   `csv`: RFC 4180 CSV export with formula injection defense (`'` prefixed).
*   `-v, --verbose`: Verbose output showing byte offsets, hex dumps, and segment maps.
*   `-h, --help`: Print comprehensive help information.
*   `-V, --version`: Print tool version.

---

## ⚙️ 2. Subcommand Inventory

| Subcommand | Purpose | Key Flags | Primary Crate |
| :--- | :--- | :--- | :--- |
| **`scan`** | Extract EXIF, GPS, TIFF, XMP, and dimensions | `[FILE]`, `--format` | `metaforge-parsers` |
| **`audit`** | Perform stego, Shannon entropy, and polyglots audit | `[FILE]`, `--format` | `metaforge-forensics` |
| **`sanitize`** | Strip metadata and truncate post-EOF overlays | `--out`, `--inplace`, `--overlay-only` | `metaforge-sanitize` |
| **`comment`** | Read or edit comments / PNG text tags | `-s, --set`, `-d, --delete`, `-k, --key` | `metaforge-sanitize` |
| **`dump`** | Dump low-level segments and chunk structures | `[FILE]`, `-v` | `metaforge-parsers` |

---

## 🚀 3. Detailed Subcommand Usage

### A. Metadata Scan (`scan` or Default)
Extracts container format, dimensions, EXIF tags, GPS coordinates, and camera properties:
```bash
metaforge scan test_images/img6-gps.jpg
# Or simply:
metaforge test_images/img6-gps.jpg
```

Export to formula-shielded CSV:
```bash
metaforge -f csv test_images/img6-gps.jpg > report.csv
```

Export to newline-delimited JSON stream:
```bash
metaforge -f jsonl test_images/img6-gps.jpg
```

---

### B. Forensics & Steganography Audit (`audit`)
Calculates Shannon entropy ($0.0 - 8.0$), compares logical vs physical file boundaries to detect post-EOF overlays, and scans for embedded polyglots:
```bash
metaforge audit test_images/img6-gps.jpg
```

**Audit Verdicts:**
*   `CLEAN (0 bytes)`: No post-EOF trailing overlay detected.
*   `ALERT: Trailing Overlay`: Physical file size exceeds logical image end.
*   `ALERT: Found`: Embedded ZIP, PDF, PE, ELF, or PHP web shell signature detected.

---

### C. Low-Level Segment & Chunk Dump (`dump`)
Inspects raw markers, segment offsets, and byte lengths:
```bash
metaforge dump test_images/img6-gps.jpg
```

Verbose dump with hex view:
```bash
metaforge dump -v test_images/flower.png
```

---

### D. Privacy Sanitizer & Overlay Scrubber (`sanitize`)
Removes privacy-compromising metadata and strips hidden payloads:

```bash
# Strip all metadata and truncate overlays to new file:
metaforge sanitize test_images/img6-gps.jpg -o clean.jpg

# Truncate post-EOF overlays only (preserve camera EXIF metadata):
metaforge sanitize test_images/img6-gps.jpg --overlay-only -o no_overlay.jpg

# Sanitize file in place (overwrites original):
metaforge sanitize suspicious_image.png --inplace
```

---

### E. Comment & Text Tag Editor (`comment`)
Inspects, sets, or removes user comments:

```bash
# Read existing comment:
metaforge comment test_images/img1.jpg

# Set comment on JPEG (creates copy):
metaforge comment test_images/img1.jpg -s "Authorized Archive" -o modified.jpg

# Set PNG textual keyword:
metaforge comment test_images/flower.png -k "Author" -s "Archive Team" -o flower_tagged.png

# Delete existing comment:
metaforge comment modified.jpg -d -o stripped.jpg
```
