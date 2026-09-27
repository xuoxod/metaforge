#!/usr/bin/env bash
# ==============================================================================
# 📦 METAFORGE SOVEREIGN COMPILATION & RELEASE PACKAGER
# File: scripts/metaforge-build.sh
# Purpose: Compiles high-performance native or standalone static musl binaries,
#          verifies zero-dependency linking, and packages release tarballs with SHA-256.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Source UI singleton
# shellcheck source=/dev/null
. "${SCRIPT_DIR}/lib/ui.sh"

show_help() {
    ui_header "METAFORGE BUILD & PACKAGING CLI"
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [TARGET]

TARGETS:
  musl          Build 100% static musl binary (x86_64-unknown-linux-musl) [default]
  native        Build native release binary for the host platform
  all           Build both native and static musl binaries

OPTIONS:
  -p, --package Package release tarball with SHA-256 checksums in dist/
  -j, --jobs <N> Number of parallel cargo build jobs [default: 4]
  --strip       Strip symbols from final binaries for minimal footprint
  -h, --help    Display this help menu and exit

EXAMPLES:
  $(basename "$0") musl --package
  $(basename "$0") native
  $(basename "$0") all --package --strip
EOF
    exit 0
}

TARGET="musl"
PACKAGE=0
STRIP_BIN=0
JOBS=4

while [ $# -gt 0 ]; do
    case "$1" in
        musl|native|all)
            TARGET="$1"
            shift
            ;;
        -p|--package)
            PACKAGE=1
            shift
            ;;
        --strip)
            STRIP_BIN=1
            shift
            ;;
        -j|--jobs)
            JOBS="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            ;;
        *)
            ui_error "Unknown option or target: $1"
            show_help
            ;;
    esac
done

cd "${ROOT_DIR}"

ui_header "METAFORGE SOVEREIGN COMPILATION PIPELINE"
ui_section "Build Parameters"
ui_kv "Target Architecture" "$TARGET"
ui_kv "Cargo Concurrency" "jobs = $JOBS"
ui_kv "Release Packaging" "$([ "$PACKAGE" -eq 1 ] && echo 'Enabled (dist/)' || echo 'Disabled')"
ui_kv "Binary Stripping" "$([ "$STRIP_BIN" -eq 1 ] && echo 'Enabled' || echo 'Disabled')"

build_native() {
    ui_section "Compiling Native Release Binary"
    cargo build --release -j "$JOBS"
    local bin_path="target/release/metaforge"
    [ ! -f "$bin_path" ] && bin_path="target/release/jpeg_meta_rs"

    if [ -f "$bin_path" ]; then
        local sz
        sz=$(du -h "$bin_path" | cut -f1)
        ui_success "Native binary compiled: $bin_path ($sz)"
        if [ "$STRIP_BIN" -eq 1 ]; then
            strip "$bin_path" 2>/dev/null || true
            ui_info "Stripped native binary: $(du -h "$bin_path" | cut -f1)"
        fi
        return 0
    else
        ui_error "Native compilation failed to produce binary"
        return 1
    fi
}

build_musl() {
    ui_section "Compiling Standalone Static Musl Binary"
    ui_info "Target: x86_64-unknown-linux-musl"
    cargo build --release --target x86_64-unknown-linux-musl -j "$JOBS"
    
    local bin_path="target/x86_64-unknown-linux-musl/release/metaforge"
    [ ! -f "$bin_path" ] && bin_path="target/x86_64-unknown-linux-musl/release/jpeg_meta_rs"

    if [ -f "$bin_path" ]; then
        local sz
        sz=$(du -h "$bin_path" | cut -f1)
        ui_success "Musl binary compiled: $bin_path ($sz)"

        # Inspect linkage
        ui_info "Verifying zero dynamic dependencies..."
        if file "$bin_path" | grep -qiE "statically linked|static-pie linked"; then
            ui_pass "100% Statically Linked PIE (Zero Glibc / Shared Object Dependencies)"
        else
            ui_warn "Binary is not fully static: $(file "$bin_path")"
        fi

        if [ "$STRIP_BIN" -eq 1 ]; then
            strip "$bin_path" 2>/dev/null || true
            ui_info "Stripped musl binary: $(du -h "$bin_path" | cut -f1)"
        fi
        return 0
    else
        ui_error "Musl compilation failed to produce binary"
        return 1
    fi
}

package_dist() {
    ui_section "Packaging Sovereign Release Artifacts"
    mkdir -p dist
    local version="v$(grep -m1 '^version =' "${ROOT_DIR}/crates/metaforge-cli/Cargo.toml" | cut -d'"' -f2)"
    
    local musl_bin="target/x86_64-unknown-linux-musl/release/metaforge"
    [ ! -f "$musl_bin" ] && musl_bin="target/x86_64-unknown-linux-musl/release/jpeg_meta_rs"

    if [ -f "$musl_bin" ]; then
        local tarball="dist/metaforge-${version}-x86_64-unknown-linux-musl.tar.gz"
        local files=("$(basename "$musl_bin")")
        local bin_dir
        bin_dir="$(dirname "$musl_bin")"
        [ -f "${bin_dir}/libmetaforge_ffi.so" ] && files+=("libmetaforge_ffi.so")
        [ -f "${bin_dir}/libmetaforge_ffi.a" ] && files+=("libmetaforge_ffi.a")
        tar -czf "$tarball" -C "$bin_dir" "${files[@]}"
        (cd dist && sha256sum "$(basename "$tarball")" > "$(basename "$tarball").sha256")
        ui_success "Created: $tarball ($(du -h "$tarball" | cut -f1))"
        ui_kv "SHA-256" "$(cat "${tarball}.sha256" | cut -d' ' -f1)"
    fi
}

case "$TARGET" in
    native) build_native ;;
    musl) build_musl ;;
    all) build_native && build_musl ;;
esac

if [ "$PACKAGE" -eq 1 ]; then
    package_dist
fi

ui_section "Build Pipeline Complete"
ui_success "MetaForge build process finished successfully."
