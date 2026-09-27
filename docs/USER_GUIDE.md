# 📖 MetaForge User Guide

Welcome to the user guide for `metaforge`—an enterprise-grade binary forensics, metadata inspection, media transcoding, and privacy sanitization engine for digital image and audio containers. This manual details real-world usage instructions, forensic evaluation tables, and threat mitigation workflows.

---

## 📡 1. Quick Start

Run `metaforge` directly on any image to perform an instant metadata and container inspection:

```bash
metaforge test_images/img6-gps.jpg
```

---

## 🚀 2. Operational Scenarios

### Scenario A: Media Stream Probing
Inspect audio/image properties without decoding the entire bitstream:

```bash
metaforge probe test_images/img6-gps.jpg
metaforge probe recording.wav
```

---

### Scenario B: Media & Image Transcoding
Convert images between JPEG, PNG, WebP, GIF, BMP, and TIFF, or transcode WAV audio:

```bash
# Convert photo to WebP with custom dimensions and quality:
metaforge convert original.png web_ready.webp --resize 1280x720 --quality 85

# Convert stereo audio recording to mono at 44.1 kHz:
metaforge convert voice_stereo.wav voice_mono.wav --channels 1 --rate 44100
```

---

### Scenario C: Forensics & Steganography Audit
Audit an image for post-EOF overlays, hidden ZIP/executable payloads, or high-entropy encrypted steganography:

```bash
metaforge audit test_images/img6-gps.jpg
```

**Interpretation of Shannon Entropy:**
| Entropy Range (Bits/Byte) | Interpretation | Typical Occurrence |
| :--- | :--- | :--- |
| **0.00 – 1.00** | Ultra-Low Randomness | Zeroed padding, uniform fill patterns |
| **1.00 – 6.00** | Low Randomness | Text, metadata headers, uncompressed bitmaps |
| **6.00 – 7.50** | Moderate Randomness | Standard image payloads, compressed icons |
| **7.50 – 7.95** | High Randomness | Normal JPEG / PNG compressed pixel streams |
| **7.95 – 8.00** | Extreme Randomness | Encrypted containers, steganography, packed payloads |

---

### Scenario D: Deep Privacy Sanitization
Strip sensitive geolocation tags, camera serials, timestamps, and private metadata blocks prior to publishing an asset:

```bash
metaforge sanitize test_images/img6-gps.jpg -o clean_photo.jpg
```

To strip only post-EOF trailing overlay payloads while keeping camera EXIF tags intact:
```bash
metaforge sanitize test_images/img6-gps.jpg --overlay-only -o no_overlay.jpg
```

---

### Scenario E: Reading & Editing Image Comments
Read or modify embedded JPEG comments or PNG textual tags without recompressing the image:

```bash
# Read existing comment
metaforge comment test_images/img1.jpg

# Set new comment
metaforge comment test_images/img1.jpg -s "Authorized Archive Asset" -o archived.jpg

# Delete comment
metaforge comment archived.jpg -d -o stripped.jpg
```

---

### Scenario F: Exporting for Data Pipelines & SIEM
Export structured metadata in JSON, JSONL, or formula-injection protected CSV:

```bash
# Export to pretty JSON:
metaforge -f json test_images/img6-gps.jpg > report.json

# Export to single-line JSONL:
metaforge -f jsonl test_images/img6-gps.jpg

# Export to spreadsheet-safe CSV:
metaforge -f csv test_images/img6-gps.jpg > report.csv
```

---

### Scenario G: Low-Level Structure Dump
Dump raw container markers, segment offsets, and lengths for binary inspection:

```bash
metaforge dump test_images/img6-gps.jpg
```
