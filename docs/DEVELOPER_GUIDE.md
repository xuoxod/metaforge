# 💻 Developer & Contribution Guide

This guide outlines compilation pipelines, test harnesses, coding standards, and architectural invariants for developers working on the `metaforge` sovereign workspace.

---

## 🛠️ 1. Compilation & Build Workflows

All builds are orchestrated via `scripts/metaforge-build.sh`:

```bash
# Compile native release binary (target/release/metaforge)
./scripts/metaforge-build.sh native

# Compile standalone static musl binary (target/x86_64-unknown-linux-musl/release/metaforge)
./scripts/metaforge-build.sh musl

# Compile, strip symbols, and package distribution tarball with SHA-256
./scripts/metaforge-build.sh musl --strip --package
```

---

## 🔬 2. 6-Tier TDD & Adversarial Test Harness

We enforce a comprehensive multi-tier testing framework, including Tier 6 adversarial self-attack suites:

```bash
# Execute complete workspace test matrix
./scripts/metaforge-test.sh --all

# Execute Tier 6 Red-Team Adversarial Self-Attack suites only
./scripts/metaforge-test.sh --adversarial

# Execute tests for a specific crate
./scripts/metaforge-test.sh --parsers
./scripts/metaforge-test.sh --forensics
./scripts/metaforge-test.sh --sanitize
./scripts/metaforge-test.sh --core
./scripts/metaforge-test.sh --cli
```

### Adversarial Test Invariants (`POC TDD+++++`)
*   **`tests/adversarial_cli_tests.rs`**: Asserts formula-injection protection in CSV exports and ANSI/OSC terminal escape sequence scrubbing.
*   **`tests/adversarial_core_tests.rs`**: Asserts resilience against NaN/Infinity GPS coordinates, invalid cardinal references, and fuzzing inputs.
*   **`tests/adversarial_forensics_tests.rs`**: Tests PE false-positive immunity, PHP web shell injection, and high-entropy stego thresholds.
*   **`tests/adversarial_parser_bombs_tests.rs`**: Tests resistance against billion-byte PNG allocation bombs, truncated JPEG segments, nested HEIC box bombs, and corrupted RIFF sizes.
*   **`tests/adversarial_sanitize_tests.rs`**: Tests truncated stream safety, JPEG comment overflows, and formula mitigation.

---

## 🏛️ 3. Sovereign Architecture & OJP Audit

Inspect crate boundaries, LOC metrics, and security invariants:

```bash
# View workspace topology and LOC summary
./scripts/metaforge-arch.sh summary

# Inspect individual crate OJP boundaries
./scripts/metaforge-arch.sh crates

# Run automated invariant audit (identity leaks, musl targets, panic discipline)
./scripts/metaforge-arch.sh audit
```

---

## 📂 4. Crate Modification Rules

When extending `metaforge`:
1.  **Zero Panic Policy**: Production library crates (`metaforge-core`, `metaforge-parsers`, `metaforge-forensics`, `metaforge-sanitize`) must not use `.unwrap()` or `.expect()` in non-test code. Return typed `MetaForgeError` results instead.
2.  **Zero-Copy Ingestion**: Ingest binary containers as immutable byte slices `&[u8]`. Avoid allocating unneeded intermediate buffers.
3.  **Strict OJP**: Keep duties strictly decoupled across crates. Do not import UI/formatting crates into parser/forensic engines.
