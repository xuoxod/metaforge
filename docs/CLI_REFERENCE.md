# 🛠️ MetaForge CLI Reference Manual

The `metaforge` CLI provides a unified, zero-dependency sovereign toolkit for extracting, forensically auditing, editing, transcoding, and scrubbing media across JPEG, PNG, WebP, GIF, HEIC, BMP, TIFF, and WAV containers.

---

## 📡 1. Global Syntax & Ingress

```bash
metaforge [OPTIONS] [FILE] [COMMAND]
```

### 📥 Arguments & Options
*   `[FILE]`: Target image or media file to scan (default command if no subcommand provided).
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
| **`convert`** | Transcode image/audio containers with resize/quality | `<INPUT> <OUTPUT>`, `-q`, `-r`, `--rate`, `--channels` | `metaforge-converter` |
| **`probe`** | Probe media kind, streams, duration, sample rate | `<FILE>`, `--format` | `metaforge-converter` |

---

## 🚀 3. Detailed Subcommand Usage

### A. Metadata Scan (`scan` or Default)
Extracts container format, dimensions, EXIF tags, GPS coordinates, and camera properties:
```bash
metaforge scan test_images/img6-gps.jpg
# Or simply:
metaforge test_images/img6-gps.jpg
```

---

### B. Media & Stream Telemetry Probe (`probe`)
Inspects container format, image dimensions, audio channels, sample rates, and bit depths:
```bash
metaforge probe test_images/img6-gps.jpg
metaforge probe recording.wav
```

---

### C. Media & Image Transcoding (`convert`)
Converts images, extracts audio from video containers, and transcodes audio formats with zero C/FFI dependencies:

```bash
# Convert JPEG to PNG:
metaforge convert input.jpg output.png

# Convert and resize image to WebP with custom quality:
metaforge convert photo.jpg web_optimized.webp --quality 80 --resize 1280x720

# Extract audio from MP4 video container to WAV:
metaforge convert interview.mp4 audio.wav

# Transcode audio (FLAC/MP3/OGG/WAV): downmix to Mono, resample to 16 kHz with peak normalization:
metaforge convert input.flac speech.wav --channels 1 --rate 16000 --normalize

# Pre-flight dry run estimation:
metaforge convert raw_photo.tiff banner.webp --dry-run

# UNIX Stdio stream piping (stdin to stdout):
cat input.mp4 | metaforge convert - - -t wav > output.wav
cat photo.png | metaforge convert - - -t webp > photo.webp
```

---

### D. Concurrent Batch Transcoding (`batch`)
Recursively converts entire directory trees while strictly adhering to the 4-worker concurrency cap:

```bash
# Batch transcode all JPEGs in a folder to WebP (preserves directory hierarchy):
metaforge batch /path/to/photos /path/to/optimized --ext jpg,jpeg --target webp

# Batch extract audio from all video files to WAV:
metaforge batch /path/to/recordings /path/to/audio --ext mp4,mkv,webm --target wav

# Pre-flight batch dry run (calculates files and bytes without modifying filesystem):
metaforge batch /data/images /data/export --target png --dry-run

# Flatten output directory tree (all files written directly into target dir):
metaforge batch /nested/library /flat/export --ext flac --target wav --flatten
```

---

### D. Forensics & Steganography Audit (`audit`)
Calculates Shannon entropy ($0.0 - 8.0$), compares logical vs physical file boundaries to detect post-EOF overlays, and scans for embedded polyglots:
```bash
metaforge audit test_images/img6-gps.jpg
```

---

### E. Low-Level Segment & Chunk Dump (`dump`)
Inspects raw markers, segment offsets, and byte lengths:
```bash
metaforge dump test_images/img6-gps.jpg
```

---

### F. Privacy Sanitizer & Overlay Scrubber (`sanitize`)
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

### G. Comment & Text Tag Editor (`comment`)
Inspects, sets, or removes user comments:

```bash
# Read existing comment:
metaforge comment test_images/img1.jpg

# Set comment on JPEG (creates copy):
metaforge comment test_images/img1.jpg -s "Authorized Archive" -o modified.jpg

# Set PNG textual keyword:
metaforge comment test_images/flower.png -k "Author" -s "Archive Team" -o flower_tagged.png
```
