# 💻 Developer & Contribution Guide

Welcome to the **MetaForge Developer Guide**. This manual outlines compilation workflows, multi-tier test execution, architectural invariants, and contribution standards for developers working on the MetaForge sovereign workspace.

---

## 🛠️ 1. Compilation & Build Workflows

All builds are managed via the sovereign builder script [`scripts/metaforge-build.sh`](file:///home/emhcet/private/projects/desktop/rust/metaforge/scripts/metaforge-build.sh):

```bash
# 1. Compile native optimized binary (target/release/metaforge):
./scripts/metaforge-build.sh native

# 2. Compile standalone static musl PIE binary (target/x86_64-unknown-linux-musl/release/metaforge):
./scripts/metaforge-build.sh musl

# 3. Compile, strip debug symbols, and generate a release tarball with SHA-256 checksum:
./scripts/metaforge-build.sh musl --strip --package
```

The resulting static binary is 100% self-contained and runs on any modern x86_64 Linux system with zero external runtime dependencies.

---

## 🔬 2. Multi-Tier TDD & Adversarial Verification

We enforce a comprehensive multi-tier testing framework across all 7 crates, orchestrated by [`scripts/metaforge-test.sh`](file:///home/emhcet/private/projects/desktop/rust/metaforge/scripts/metaforge-test.sh):

```bash
# Execute complete workspace test matrix across all 7 crates:
./scripts/metaforge-test.sh --all

# Execute Tier 6 Red-Team Adversarial Self-Attack suites only:
./scripts/metaforge-test.sh --adversarial

# Execute tests for an individual crate:
./scripts/metaforge-test.sh --converter
./scripts/metaforge-test.sh --forensics
./scripts/metaforge-test.sh --parsers
./scripts/metaforge-test.sh --sanitize
./scripts/metaforge-test.sh --core
./scripts/metaforge-test.sh --ffi
./scripts/metaforge-test.sh --cli
```

### Adversarial Test Invariants (`POC TDD+++++`)
The workspace includes 22 dedicated adversarial self-attack tests asserting immunity against malicious inputs:
* **`tests/adversarial_converter_tests.rs`**: Asserts immunity against dimension allocation bombs ($>16,384 \times 16,384\text{ px}$), corrupted stream payloads, zero-byte buffers, and audio channel overflow attacks.
* **`tests/adversarial_forensics_tests.rs`**: Asserts immunity against truncated scan streams, 0-byte SOS segments, marker padding fuzzing, high-entropy steganography thresholds, PE false-positive immunity, and PHP webshell detection.
* **`tests/adversarial_parser_bombs_tests.rs`**: Asserts resistance against billion-byte PNG chunk allocation bombs, cyclical nested ISOBMFF HEIC box traps, corrupted RIFF sizes, and truncated JPEG segments.
* **`tests/adversarial_sanitize_tests.rs`**: Asserts truncated stream safety, JPEG comment overflows, and formula-injection escaping.
* **`tests/adversarial_cli_tests.rs`**: Asserts spreadsheet formula neutralization in CSV exports and ANSI CSI / OSC 8 terminal sequence scrubbing.
* **`tests/adversarial_core_tests.rs`**: Asserts resilience against NaN/Infinity GPS coordinates, invalid cardinal references, and fuzzing inputs.
* **`tests/ffi_tdd.rs`**: Asserts null-pointer safety across all C-ABI entry points.

---

## 🏛️ 3. Architecture & OJP Boundary Inspector

Verify crate boundaries, LOC metrics, and security invariants using [`scripts/metaforge-arch.sh`](file:///home/emhcet/private/projects/desktop/rust/metaforge/scripts/metaforge-arch.sh):

```bash
# View workspace topology and LOC summary:
./scripts/metaforge-arch.sh summary

# Inspect individual crate OJP boundaries and module layouts:
./scripts/metaforge-arch.sh crates

# Run automated invariant audit (identity leaks, musl targets, panic discipline):
./scripts/metaforge-arch.sh audit
```

---

## 📂 4. Architectural Rules & Coding Standards

When contributing code to `metaforge`:

1. **Zero Panic Policy in Library Crates**:
   Production library code (`metaforge-core`, `metaforge-parsers`, `metaforge-forensics`, `metaforge-sanitize`, `metaforge-converter`, `metaforge-ffi`) must never call `.unwrap()` or `.expect()` in non-test paths. All potential error states must return a typed `Result<T, MetaForgeError>`.
2. **Zero-Copy Ingestion**:
   Container parsers must operate directly over immutable byte slices (`&[u8]`). Avoid allocating intermediate buffers or decoding pixel bitmaps into memory during metadata or structural inspections.
3. **Strict One-Job-Principle (OJP)**:
   Keep crate duties strictly isolated:
   * Do not import terminal formatting or UI crates into parser, forensics, or conversion crates.
   * Expose clean domain data models from `metaforge-core` and let the presentation layer (`metaforge-cli`) handle rendering.
4. **Sanitized Stack Discipline**:
   Avoid leaking internal third-party dependency names or private development paths in user-facing manuals and documentation. Present the system as a unified native engine.
