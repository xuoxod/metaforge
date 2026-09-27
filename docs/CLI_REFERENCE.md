# 🛠️ MetaForge CLI Reference Manual

The **`metaforge`** command-line interface provides a fast, unified, zero-dependency suite for inspecting, converting, sanitizing, and forensically auditing media containers across JPEG, PNG, WebP, GIF, HEIC, BMP, TIFF, WAV, and video formats.

---

## 📡 1. Global Syntax & Calling Conventions

```bash
metaforge [OPTIONS] [FILE] [COMMAND]
```

When invoked with just a file path (e.g. `metaforge photo.jpg`), MetaForge defaults to the `scan` command.

### Global Options
| Option | Short | Description | Default |
| :--- | :---: | :--- | :---: |
| `--format <FORMAT>` | `-f` | Output serialization format: `table`, `json`, `jsonl`, `csv` | `table` |
| `--verbose` | `-v` | Enable detailed diagnostic output, byte offsets, and hex dumps | Off |
| `--help` | `-h` | Display help information and usage syntax | — |
| `--version` | `-V` | Display program version information | — |

---

## ⚙️ 2. Subcommand Inventory

| Subcommand | Core Purpose | Common Flags |
| :--- | :--- | :--- |
| **`scan`** | Extract container format, dimensions, EXIF tags, and GPS coordinates | `[FILE]`, `-f, --format` |
| **`probe`** | Inspect container tracks, audio channels, sample rates, and bit depths | `<FILE>`, `-f, --format` |
| **`convert`** | Transcode image and audio formats, resize images, extract audio from video | `<INPUT> <OUTPUT>`, `-q`, `-r`, `--rate`, `--channels` |
| **`batch`** | Concurrently convert entire folders or recursive directory hierarchies | `<INPUT_DIR> <OUTPUT_DIR>`, `--ext`, `--target`, `--dry-run` |
| **`audit`** | Run forensic evaluation: Shannon entropy, Chi-Square stego, overlays, polyglots | `[FILE]`, `-f, --format` |
| **`sanitize`** | Strip privacy-compromising metadata and truncate post-EOF overlays | `[FILE]`, `-o, --out`, `--inplace`, `--overlay-only` |
| **`comment`** | View, insert, edit, or delete JPEG comments and PNG textual keywords | `[FILE]`, `-s, --set`, `-d, --delete`, `-k, --key` |
| **`dump`** | Inspect low-level binary segments, chunk markers, and byte offsets | `[FILE]`, `-v, --verbose` |

---

## 🚀 3. Detailed Command Reference

### `scan` — Metadata Inspection (Default)
Extracts container headers, dimensions, camera specifications, and GPS coordinates.

```bash
metaforge scan [OPTIONS] <FILE>
```
* **Examples**:
  ```bash
  # Standard inspection with formatted table:
  metaforge scan photo.jpg

  # Quick shorthand (equivalent to scan):
  metaforge photo.jpg

  # Export to JSON:
  metaforge scan photo.jpg -f json > metadata.json
  ```

---

### `probe` — Media & Stream Telemetry Probe
Inspects audio and video container properties without decoding full audio/video data.

```bash
metaforge probe [OPTIONS] <FILE>
```
* **Reported Fields**: Container format, visual dimensions, stream track counts, audio sample rates, channel configurations, and durations.
* **Examples**:
  ```bash
  metaforge probe recording.wav
  metaforge probe interview.mp4
  metaforge probe artwork.webp -f json
  ```

---

### `convert` — Media Transcoding & Audio Extraction
Converts images between JPEG, PNG, WebP, GIF, BMP, and TIFF, extracts audio from video containers, and applies linear audio DSP filters.

```bash
metaforge convert [OPTIONS] <INPUT> <OUTPUT>
```
* **Options & Flags**:
  * `-q, --quality <0-100>`: Set output compression quality for lossy formats (JPEG, WebP). Default: `80`.
  * `-r, --resize <WxH>`: Resize visual dimensions (e.g. `1280x720`, `800x600`).
  * `--rate <HZ>`: Resample audio frequency in Hertz (e.g. `16000`, `44100`, `48000`).
  * `--channels <N>`: Adjust audio channel layout (`1` for Mono, `2` for Stereo).
  * `--gain <FACTOR>`: Multiply linear audio gain (e.g. `1.5` for $+50\%$ amplitude).
  * `--normalize`: Apply peak audio normalization to maximize volume without clipping.
  * `-t, --target <FMT>`: Explicit output format (required when piping via stdio).
  * `--dry-run`: Estimate conversion output size without writing to disk.
* **Examples**:
  ```bash
  # Convert PNG to WebP with custom dimensions and quality:
  metaforge convert image.png banner.webp --resize 1280x720 --quality 85

  # Extract audio from MP4 video to pristine WAV:
  metaforge convert presentation.mp4 voice.wav

  # Prepare voice recording: downmix to Mono, resample to 16 kHz with peak normalization:
  metaforge convert audio.flac speech.wav --channels 1 --rate 16000 --normalize

  # Stream media through UNIX pipes (using '-' for stdin/stdout):
  cat source.mp4 | metaforge convert - - -t wav > stream.wav
  cat input.png | metaforge convert - - -t webp > output.webp
  ```

---

### `batch` — Concurrent Recursive Directory Processing
Recursively converts whole collections of files while adhering to workstation concurrency guardrails.

```bash
metaforge batch [OPTIONS] <INPUT_DIR> <OUTPUT_DIR>
```
* **Options & Flags**:
  * `--ext <EXTENSIONS>`: Comma-separated list of input extensions to include (e.g. `jpg,png` or `mp4,mkv`). Default: all supported files.
  * `--target <FORMAT>`: Desired target format (e.g. `webp`, `png`, `jpg`, `wav`).
  * `--quality <0-100>`: Quality setting for image conversions. Default: `80`.
  * `--resize <WxH>`: Resize dimensions applied to all candidate images.
  * `--flatten`: Output all converted files into a flat directory instead of mirroring nested folders.
  * `--dry-run`: Perform a pre-flight scan calculating file counts and disk footprint without writing files.
* **Examples**:
  ```bash
  # Pre-flight preview of batch job:
  metaforge batch ./raw_photos ./web_photos --ext jpg,png --target webp --dry-run

  # Execute conversion preserving folder tree:
  metaforge batch ./raw_photos ./web_photos --ext jpg,png --target webp

  # Extract audio from all videos into a flat destination folder:
  metaforge batch ./video_library ./audio_exports --ext mp4,mkv,webm --target wav --flatten
  ```

---

### `audit` — Forensics & Steganography Evaluation
Audits file boundaries, calculates byte randomness, tests for statistical steganography, and flags embedded polyglot payloads.

```bash
metaforge audit [OPTIONS] <FILE>
```
* **Evaluated Threat Vectors**:
  * **Trailing Overlays**: Detects bytes appended after the container's logical end marker.
  * **Shannon Entropy ($0.0 - 8.0$)**: Flags encrypted data or hidden archives.
  * **Chi-Square Goodness-of-Fit**: Identifies subtle mathematical distortions in compressed scan streams.
  * **Pairs-of-Values (PoVs) Attack**: Detects sequential LSB steganographic embedding.
  * **Polyglot Signatures**: Identifies embedded ZIP, PDF, PE, ELF, or PHP web scripts.
* **Examples**:
  ```bash
  metaforge audit photo.jpg
  metaforge audit suspicious_document.png -f json
  ```

---

### `sanitize` — Privacy Scrubbing & Payload Removal
Removes privacy-sensitive tags and slices away hidden data appended beyond the end of the file.

```bash
metaforge sanitize [OPTIONS] <FILE>
```
* **Options & Flags**:
  * `-o, --out <PATH>`: Write the sanitized result to a new file.
  * `--inplace`: Overwrite the source file directly (destructive).
  * `--overlay-only`: Strip only post-EOF trailing data, leaving camera EXIF tags intact.
* **Examples**:
  ```bash
  # Strip all metadata and overlays into a clean copy:
  metaforge sanitize input.jpg -o safe_share.jpg

  # Remove only trailing overlays while preserving EXIF:
  metaforge sanitize input.jpg --overlay-only -o no_overlays.jpg

  # Sanitize directly in place:
  metaforge sanitize sensitive_image.png --inplace
  ```

---

### `comment` — Comment & Text Tag Management
Inspects, inserts, updates, or deletes embedded JPEG comments and PNG text tags without recompressing image data.

```bash
metaforge comment [OPTIONS] <FILE>
```
* **Options & Flags**:
  * `-s, --set <TEXT>`: Set comment or tag string.
  * `-d, --delete`: Delete the comment or tag.
  * `-k, --key <KEY>`: Specific keyword for PNG textual chunks (e.g. `Author`, `Description`). Default: `Comment`.
  * `-o, --out <PATH>`: Output file path. If omitted, writes in place when setting/deleting.
* **Examples**:
  ```bash
  # Read comment:
  metaforge comment photo.jpg

  # Set comment on JPEG (creates copy):
  metaforge comment photo.jpg -s "Authorized Archive Asset" -o archived.jpg

  # Set PNG text tag:
  metaforge comment icon.png -k "Author" -s "Design Guild" -o icon_tagged.png

  # Delete comment:
  metaforge comment archived.jpg -d -o stripped.jpg
  ```

---

### `dump` — Low-Level Structural Dump
Displays a diagnostic map of raw container markers, segment offsets, and byte lengths.

```bash
metaforge dump [OPTIONS] <FILE>
```
* **Examples**:
  ```bash
  metaforge dump sample.jpg
  metaforge dump asset.png -v
  ```
