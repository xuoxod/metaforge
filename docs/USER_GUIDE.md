# 📖 MetaForge User Guide

Welcome to the **MetaForge User Guide**. Whether you are an everyday photographer looking to remove sensitive location tags before posting images online, a digital media creator converting audio and video assets, or a cybersecurity professional auditing files for hidden payloads, this guide walks you through every feature step by step.

---

## 🧭 1. Understanding Digital Media Concepts

Before diving into commands, here is a quick overview of why media inspection and sanitization matter:

* **Metadata (EXIF, XMP, TIFF)**: Digital cameras and smartphones secretly record technical details alongside your pictures—including your exact GPS coordinates, timestamp, camera serial number, exposure settings, and device hardware.
* **Post-EOF Overlays**: The file format specifies where an image should logically end. Attackers or hidden applications can append covert data (such as ZIP archives or executable scripts) past this ending point without distorting the picture on screen.
* **Shannon Entropy**: A mathematical measurement of randomness (from $0.0$ to $8.0$). Highly structured data like plain text has low entropy, while encrypted data, hidden compressed files, or steganographic payloads exhibit very high entropy (close to $8.0$).
* **Steganography**: The practice of concealing a secret message or file inside ordinary-looking carrier data (such as imperceptibly tweaking pixel values).
* **Sanitization**: Cleaning an image by stripping private tracking tags and slicing away hidden trailing data while leaving visual image quality 100% untouched.

---

## ⚡ 2. Quick Start: The One-Second Inspection

To inspect any photo or media file, simply pass its path to `metaforge`:

```bash
metaforge photo.jpg
```

MetaForge analyzes the file instantly and prints a clear, organized table showing:
* **Container Type & Dimensions**: Width, height, aspect ratio, and bit depth.
* **Camera Hardware**: Make, model, lens profile, software version, and device serial numbers.
* **Capture Conditions**: ISO, aperture, shutter speed, focal length, and flash mode.
* **Geolocation**: Precise GPS latitude, longitude, altitude, and a ready-to-click mapping link.

> [!TIP]
> If a photo has no GPS or camera tags, MetaForge displays a clean notification letting you know that no private identifiers were detected.

---

## 🛠️ 3. Real-World Practical Scenarios

### Scenario A: Protecting Your Privacy Before Sharing Photos
When uploading personal photos to social platforms, forums, or classified listings, photos can expose your home location or daily routine. 

To strip all personal identifying data while leaving the picture quality completely intact:

```bash
# Creates a clean copy with all GPS and camera tags removed:
metaforge sanitize vacation.jpg -o vacation_safe.jpg
```

If you want to keep your camera exposure settings (like ISO and shutter speed) but remove suspicious data appended to the end of the file:
```bash
# Only slices away trailing post-EOF overlays:
metaforge sanitize vacation.jpg --overlay-only -o vacation_cleaned.jpg
```

To sanitize the file directly in place without creating a duplicate:
```bash
metaforge sanitize vacation.jpg --inplace
```

---

### Scenario B: Probing Media Properties
Need to quickly check the audio sample rate of a recording or the resolution of an image without opening heavy media editing software?

```bash
# Probe an image:
metaforge probe banner.webp

# Probe an audio track or video recording:
metaforge probe podcast.wav
metaforge probe interview.mp4
```

MetaForge reports container formats, track counts, audio channels (Mono/Stereo), sampling frequencies (e.g., $44.1\text{ kHz}$ or $48\text{ kHz}$), and durations.

---

### Scenario C: Converting Images & Extracting Audio
MetaForge includes a built-in native transcoding engine that requires no external system codecs or complex configurations.

#### 1. Converting Photos for the Web
Transform heavy PNGs or JPEGs into compact, modern WebP images with custom resolution and quality:

```bash
# Convert and resize an image for a website banner:
metaforge convert original.png banner.webp --resize 1280x720 --quality 80

# Convert across standard formats (JPEG, PNG, WebP, GIF, BMP, TIFF):
metaforge convert photo.jpg photo.png
```

#### 2. Extracting High-Fidelity Audio from Video
Extract pristine audio directly from video containers (MP4, MKV, WebM, MOV) into standard uncompressed WAV format:

```bash
# Extract audio from a video recording:
metaforge convert presentation.mp4 presentation_audio.wav
```

#### 3. Voice Transcription Optimization
Prepare spoken recordings for speech-to-text transcription engines by downmixing to single-channel (Mono), resampling to $16\text{ kHz}$, and normalizing volume:

```bash
metaforge convert raw_interview.mp3 voice_clean.wav --channels 1 --rate 16000 --normalize
```

#### 4. Instant UNIX Shell Streaming
Stream media directly through shell pipes without writing intermediate files to disk:

```bash
# Convert streaming video audio directly to WAV:
cat input.mp4 | metaforge convert - - -t wav > output.wav

# Convert an image pipeline stream:
cat raw.png | metaforge convert - - -t webp > web_ready.webp
```

---

### Scenario D: Batch Converting Whole Albums or Folders
Instead of converting files one by one, use the `batch` command to process entire collections concurrently.

```bash
# 1. Preview what will happen first (Dry-Run):
metaforge batch ./my_photos ./web_photos --ext jpg,png --target webp --dry-run

# 2. Execute the batch conversion (mirrors the original folder hierarchy):
metaforge batch ./my_photos ./web_photos --ext jpg,png --target webp

# 3. Extract audio from all video files into a single flat folder:
metaforge batch ./video_archive ./audio_tracks --ext mp4,mkv,webm --target wav --flatten
```

> [!NOTE]
> MetaForge automatically caps concurrent processing to prevent your computer fans from roaring or your desktop from freezing during large batch jobs.

---

### Scenario E: Running a Forensic Security & Steganography Audit
When receiving media files from untrusted sources, you can audit them for hidden threats:

```bash
metaforge audit suspicious_file.jpg
```

MetaForge performs a multi-phase forensic evaluation:
1. **Physical vs Logical Boundary**: Checks whether extra data has been secretly glued onto the end of the file.
2. **Hidden File Signatures**: Scans for polyglot files disguised as pictures (e.g., hidden ZIP archives, PDF documents, Windows/Linux executables, or PHP web scripts).
3. **Shannon Entropy**: Measures the mathematical randomness of the file data.
4. **Chi-Square Goodness-of-Fit**: Checks for subtle mathematical distortions in the compressed stream indicating hidden messages.
5. **Pairs-of-Values (PoVs) Test**: Detects sequential Least Significant Bit (LSB) steganographic injection.

#### 📊 Understanding Entropy Scores:
| Entropy Score (Bits/Byte) | Risk Level | What It Usually Means |
| :---: | :---: | :--- |
| **0.00 – 1.00** | Low | Uniform data, blank backgrounds, or empty padding. |
| **1.00 – 6.00** | Low | Plain text files, uncompressed graphics, or metadata headers. |
| **6.00 – 7.50** | Normal | Standard compressed illustrations or icons. |
| **7.50 – 7.95** | Normal | Standard high-quality photographic JPEG or PNG pixel streams. |
| **7.95 – 8.00** | Elevated / Alert | Strong indicator of encrypted data, compressed hidden archives, or steganography. |

---

### Scenario F: Reading and Writing Embedded Comments
Embed copyright notices, archive metadata, or project identifiers directly into JPEG and PNG files without altering pixel contents:

```bash
# Read an existing comment:
metaforge comment archive_photo.jpg

# Set a new comment on a photo (creates a clean copy):
metaforge comment archive_photo.jpg -s "Property of Archival Trust - 2026" -o tagged.jpg

# Set a specific text tag in a PNG image:
metaforge comment diagram.png -k "Author" -s "Security Research Group" -o diagram_tagged.png

# Strip comments completely:
metaforge comment tagged.jpg -d -o clean.jpg
```

---

### Scenario G: Safe Spreadsheet & Data Pipeline Exports
If you are ingesting metadata into automated scripts, spreadsheets, or SIEM platforms, MetaForge provides safe structured outputs:

```bash
# Export to pretty JSON:
metaforge photo.jpg -f json > metadata.json

# Export to streaming single-line JSONL:
metaforge photo.jpg -f jsonl >> event_stream.jsonl

# Export to spreadsheet-safe CSV:
metaforge photo.jpg -f csv > report.csv
```

> [!IMPORTANT]
> **Built-in Spreadsheet Protection**: Opening CSV files with untrusted input in spreadsheet software can trigger formula execution attacks. MetaForge automatically neutralizes cells starting with `=, +, -, @, \t, \r` by prefixing a safe single-quote (`'`).

---

### Scenario H: Low-Level Structural Dump
For developers and binary analysts wishing to see raw container markers, segment offsets, and byte lengths:

```bash
metaforge dump photo.jpg
```
