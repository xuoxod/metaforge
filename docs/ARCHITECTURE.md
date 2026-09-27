# 🧬 MetaForge Architecture & Design Manual

This document details the modular layout, parsing flowcharts, data structures, and security invariants of `metaforge`.

---

## 🎨 1. Modular Core (Decoupled 6-Crate Workspace)

The codebase strictly enforces the **One-Job-Principle (OJP)** across six independent crates:

1.  **`metaforge-core`**: Foundational domain types (`MetadataEntry`, `ContainerType`), comprehensive 200+ EXIF/TIFF/GPS tag registry, WGS84 GPS coordinate calculations, and strongly typed `MetaForgeError` definitions. Zero parsing logic.
2.  **`metaforge-parsers`**: Zero-copy binary container scanners for JPEG (SOF/SOS/APP markers), PNG (chunk walkers), WebP (RIFF/VP8X), GIF (descriptor blocks), and HEIC (ISOBMFF box hierarchies). Zero terminal or formatting logic.
3.  **`metaforge-forensics`**: Forensics analysis engine calculating Shannon entropy ($0.0$ to $8.0$ bits/byte), post-EOF overlay boundary detection, and signature matching for embedded polyglots (ZIP, PDF, PE, ELF, PHP shells). Zero pixel rendering.
4.  **`metaforge-sanitize`**: Lossless metadata scrubbing, post-EOF truncation, comment injection/removal, and CSV formula injection neutralization (`=`, `+`, `-`, `@`, `\t`, `\r`). Zero bitmap re-encoding.
5.  **`metaforge-converter`**: Pure-Rust, zero-dependency transcoding engine for image containers (JPEG, PNG, WebP, GIF, BMP, TIFF) and audio containers (WAV), channel mixing, sample rate resampling, and stream telemetry probing. Zero C/FFI runtime bindings.
6.  **`metaforge-cli`**: Unified terminal cockpit, monospace table visualizer, structured exporter (Table, JSON, JSONL, formula-shielded CSV), and command orchestrator. Zero raw parsing.

```mermaid
graph TD
    subgraph Cockpit ["crates/metaforge-cli"]
        Main["main.rs"]
        Args["args.rs"]
        Table["table.rs"]
        Export["export.rs"]
    end

    subgraph Scanners ["crates/metaforge-parsers"]
        Detector["detector.rs"]
        JpegParser["jpeg.rs"]
        PngParser["png.rs"]
        WebpParser["webp.rs"]
        GifParser["gif.rs"]
        HeicParser["heic.rs"]
    end

    subgraph Converter ["crates/metaforge-converter"]
        DspSub["dsp/ (resample, remix, gain, norm)"]
        CodecsSub["codecs/ (symphonia demux, hound, image)"]
        PipelineSub["pipeline/ (AudioPipeline, ImagePipeline)"]
        BatchSub["batch/ (worker pool, recursion, dry-run)"]
        ProbeSub["probe/ (media & stream inspection)"]
    end

    subgraph Forensics ["crates/metaforge-forensics"]
        Entropy["entropy.rs"]
        Overlay["overlay.rs"]
        Signatures["signatures.rs"]
    end

    subgraph Sanitize ["crates/metaforge-sanitize"]
        Scrub["scrub.rs"]
        EditJpeg["edit_jpeg.rs"]
        EditPng["edit_png.rs"]
        Formula["formula.rs"]
    end

    subgraph Core ["crates/metaforge-core"]
        Types["types.rs"]
        Tags["tags.rs"]
        Geo["geo.rs"]
        Crc["crc.rs"]
        Error["error.rs"]
    end

    Main --> Args
    Main --> Detector
    Main --> Forensics
    Main --> Sanitize
    Main --> Converter
    Main --> Table
    Main --> Export

    Converter --> Core
    Converter --> Parsers
    Detector --> JpegParser
    Detector --> PngParser
    Detector --> WebpParser
    Detector --> GifParser
    Detector --> HeicParser

    JpegParser --> Core
    PngParser --> Core
    WebpParser --> Core
    GifParser --> Core
    HeicParser --> Core

    Forensics --> Core
    Sanitize --> Core
    Sanitize --> Parsers
```

---

## 📸 2. Container Parsing & Segment Walking

Each container parser operates directly over immutable byte slices (`&[u8]`) without allocating image buffers:

### JPEG Marker Traversal
```mermaid
flowchart TD
    Start([🚀 Start JPEG Scan]) --> CheckSOI{"SOI Header 0xFFD8?"}
    CheckSOI -- No --> Err[InvalidFormat Error]
    CheckSOI -- Yes --> FindMarker{"Read 0xFFxx Marker"}
    
    FindMarker --> EOI{"0xFFD9 EOI?"}
    EOI -- Yes --> SaveSegment[Save EOI Segment] --> End([🏁 End Scan])
    
    FindMarker --> SOS{"0xFFDA SOS?"}
    SOS -- Yes --> SaveSOS[Save SOS Segment] --> End
    
    SOS -- No --> ReadLen[Read 2-Byte Segment Length]
    ReadLen --> SaveSeg[Save Segment Metadata]
    
    SaveSeg --> IsAPP1{"Is APP1 EXIF?"}
    IsAPP1 -- Yes --> ExtractExif["Extract Raw EXIF Bytes via kamadak-exif"]
    IsAPP1 -- No --> IsXMP{"Is APP1 XMP?"}
    
    IsXMP -- Yes --> ExtractXMP[Extract Raw XML String]
    IsXMP -- No --> IsSOF{"Is SOF0/SOF2?"}
    
    IsSOF -- Yes --> ParseDim[Parse Width, Height & Precision]
    IsSOF -- No --> Skip[Advance pos by segment length]
    
    ExtractExif --> Skip
    ExtractXMP --> Skip
    ParseDim --> Skip
    Skip --> FindMarker
```

### PNG Chunk Walk
1. Validates 8-byte PNG signature: `\x89PNG\r\n\x1a\n`.
2. Loops sequentially through chunks: 4-byte length + 4-byte type + payload + 4-byte CRC-32.
3. Decodes standard chunks:
   - `IHDR`: Width, height, bit depth, color type, compression method.
   - `pHYs`: Pixel dimensions and DPI calculation.
   - `tEXt` / `iTXt`: Textual metadata and comments.
   - `eXIf`: Raw EXIF block embedded in PNG.
4. Ceases traversal upon encountering the `IEND` marker, recording any trailing bytes as post-EOF overlay.

---

## 🔄 3. Media Transcoding & Batch Processing Engine (`metaforge-converter`)

The `metaforge-converter` crate enforces strict One-Job-Principle (OJP) separation across 5 dedicated internal subsystems:

### 1. Pure Digital Signal Processing (`dsp/`)
- **Resampling**: Deterministic linear interpolation across arbitrary sample frequencies (e.g. $48,000\text{ Hz} \leftrightarrow 44,100\text{ Hz} \leftrightarrow 16,000\text{ Hz}$).
- **Channel Matrixing**: Downmixing (multi-channel to stereo/mono with $M_i = \frac{\sum C_{i}}{N}$ normalization to prevent clipping) and upmixing (mono to stereo).
- **Gain & Normalization**: Linear gain scaling and peak normalization with soft clamping to $[-1.0, 1.0]$. Zero I/O dependencies.

### 2. Universal Codecs & Demuxers (`codecs/`)
- **Universal Container Demuxing (Symphonia)**: Pure-Rust decoding of audio from video containers (MP4, MKV, WebM, MOV) and audio containers (MP3, FLAC, OGG/Vorbis, AAC, WAV, AIFF, CAF). Zero C/FFI runtime dependencies.
- **Audio Encoding (Hound)**: High-precision 16-bit signed integer or 32-bit float linear PCM WAV output.
- **Image Codecs (`image`)**: Cross-format conversion (JPEG, PNG, WebP, GIF, BMP, TIFF) with Lanczos3 resampling and strict allocation bomb defenses ($\le 16,384 \times 16,384\text{ px}$, $\le 256\text{ MP}$).

### 3. Chainable Pipelines (`pipeline/`)
- `AudioPipeline`: Fluent builder for audio processing: `.sample_rate(hz).channels(ch).gain(g).normalize(bool).process_file(...)`.
- `ImagePipeline`: Fluent builder for image transcoding: `.target(fmt).quality(q).resize(w, h).process_file(...)`.
- Full UNIX pipe and stdio streaming (`metaforge convert - - -t webp`, `convert_stream`).

### 4. Concurrent Batch Engine (`batch/`)
- **Recursive Tree Mirroring**: Traverses directory hierarchies, preserving relative paths or optionally flattening.
- **Workstation Concurrency Guardrail**: Thread pool strictly clamped to $\le 4$ workers per ecosystem invariant (`jobs = 4`).
- **Pre-Flight Dry Run (`--dry-run`)**: Scans candidate files, calculates input sizes, and plans output operations without touching the filesystem.

### 5. Media Stream Inspector (`probe/`)
- Container and stream introspection reporting dimensions, audio/video track counts, codecs, duration, channels, and sample rates.

---

## 🔬 4. Steganography & Forensics Engine

### Shannon Entropy Computation
Entropy quantifies the uncertainty and randomness in the byte distribution ($0.0$ to $8.0$ bits per byte):
$$H(X) = -\sum_{i=0}^{255} P(x_i) \log_2 P(x_i)$$
- **$0.0 - 1.0$**: Homogeneous, zeroed, or uncompressed sparse data.
- **$6.0 - 7.5$**: Standard textual, structural, or low-density bitmap streams.
- **$7.5 - 7.95$**: Normal compressed JPEG/PNG bitstreams.
- **$> 7.95$**: Encrypted payload, high-entropy steganography, or compressed archive injection.

### Post-EOF Overlay Detection
Images often store malicious scripts or payload archives appended directly behind their logical end marker (`0xFFD9` for JPEG, `IEND` chunk for PNG). MetaForge locates the exact logical container boundary, compares it with physical file size, and flags any trailing bytes as post-EOF overlay candidates.

### Polyglot Signature Harvester
Walks data starting past header offsets to discover hidden file signatures without false positives:
- **ZIP / JAR / APK**: `PK\x03\x04`
- **PDF Documents**: `%PDF-`
- **PE Executables**: `MZ` header verified via DOS stub `e_lfanew` pointer to `PE\0\0`.
- **ELF Executables**: `\x7fELF`
- **PHP Web Shells**: `<?php`, `<?=`, `<script language="php">`

---

## 🛡️ 5. Tier 6 Adversarial Invariants (`POC TDD+++++`)

1. **Formula Injection Neutralization**:
   Any cell beginning with `=, +, -, @, \t, \r` in CSV exports is automatically escaped with a leading single-quote `'` to prevent remote code execution in spreadsheet software.
2. **Terminal Escapes Neutralization**:
   All user-supplied EXIF text, comments, and camera model strings are scrubbed of raw ANSI CSI (`\x1b[...]`) and OSC (`\x1b]...;...`) control codes before table rendering.
3. **Allocation Bomb Resistance**:
   Segment lengths, box bounds, and image resize dimensions exceeding safety capacity are rejected immediately to prevent memory exhaustion DoS attacks.
