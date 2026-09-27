#!/usr/bin/env bash
# ==============================================================================
# 🧪 METAFORGE 6-TIER TDD++++++ TEST RUNNER & VERIFICATION HARNESS
# File: scripts/metaforge-test.sh
# Purpose: Orchestrates unit tests, integration tests, golden file verifications,
#          and Tier 6 adversarial self-attack suites with modern console reporting.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Source UI singleton
# shellcheck source=/dev/null
. "${SCRIPT_DIR}/lib/ui.sh"

show_help() {
    ui_header "METAFORGE 6-TIER TDD++++++ TEST HARNESS CLI"
    cat << EOF
Usage: $(basename "$0") [OPTIONS] [FILTER]

TEST SUITE SELECTION:
  --all             Run complete workspace test suite (default)
  --adversarial     Run Tier 6 Red-Team Adversarial Self-Attack suites only
  --unit            Run Tier 1 & 2 unit and real-world tests only
  --core            Test metaforge-core crate only
  --parsers         Test metaforge-parsers crate only
  --forensics       Test metaforge-forensics crate only
  --sanitize        Test metaforge-sanitize crate only
  --cli             Test metaforge-cli crate only

OPTIONS:
  -j, --jobs <N>    Number of parallel cargo test jobs [default: 4]
  -q, --quiet       Quiet mode: suppress detailed compiler messages
  -v, --verbose     Verbose mode: display full test output
  --nocapture       Pass --nocapture to cargo test to see internal stdout/stderr
  -h, --help        Display this help menu and exit

EXAMPLES:
  $(basename "$0") --all
  $(basename "$0") --adversarial
  $(basename "$0") --parsers --nocapture
  $(basename "$0") --adversarial -j 2
EOF
    exit 0
}

MODE="all"
JOBS=4
EXTRA_ARGS=()
VERBOSE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --all|--adversarial|--unit|--core|--parsers|--forensics|--sanitize|--cli)
            MODE="${1#--}"
            shift
            ;;
        -j|--jobs)
            JOBS="$2"
            shift 2
            ;;
        -q|--quiet)
            EXTRA_ARGS+=("-q")
            shift
            ;;
        -v|--verbose)
            VERBOSE=1
            shift
            ;;
        --nocapture)
            EXTRA_ARGS+=("--" "--nocapture")
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            EXTRA_ARGS+=("$1")
            shift
            ;;
    esac
done

cd "${ROOT_DIR}"

ui_header "METAFORGE 6-TIER TDD++++++ TEST HARNESS"
ui_section "Execution Configuration"
ui_kv "Target Mode" "$MODE"
ui_kv "Cargo Concurrency" "jobs = $JOBS"
ui_kv "Working Root" "$ROOT_DIR"

run_suite() {
    local title="$1"
    shift
    local cmd=("$@")
    
    ui_section "$title"
    local start_time
    start_time=$(date +%s%N 2>/dev/null || date +%s)
    
    if "${cmd[@]}"; then
        local end_time
        end_time=$(date +%s%N 2>/dev/null || date +%s)
        local dur_ms=0
        if [ "$end_time" -gt "$start_time" ]; then
            dur_ms=$(( (end_time - start_time) / 1000000 ))
        fi
        ui_pass "$title (${dur_ms}ms)"
        return 0
    else
        ui_fail "$title"
        return 1
    fi
}

FAILED=0
PASSED=0

case "$MODE" in
    all)
        ui_info "Executing complete workspace test suite..."
        if run_suite "Full Workspace Test Suite" cargo test --workspace -j "$JOBS" "${EXTRA_ARGS[@]}"; then
            PASSED=$(( PASSED + 1 ))
        else
            FAILED=$(( FAILED + 1 ))
        fi
        ;;
    adversarial)
        ui_info "Executing Tier 6 Red-Team Adversarial Self-Attack Suites..."
        for test_target in $(find crates/*/tests -name "adversarial_*.rs" -exec basename {} .rs \; | sort); do
            if run_suite "Adversarial Suite: $test_target" cargo test --workspace --test "$test_target" -j "$JOBS" "${EXTRA_ARGS[@]}"; then
                PASSED=$(( PASSED + 1 ))
            else
                FAILED=$(( FAILED + 1 ))
            fi
        done
        if [ "$PASSED" -eq 0 ] && [ "$FAILED" -eq 0 ]; then
            ui_warn "No adversarial test targets discovered yet. Will execute full workspace."
            cargo test --workspace -j "$JOBS" "${EXTRA_ARGS[@]}"
        fi
        ;;
    unit)
        ui_info "Executing Unit & Real-World Test Suites..."
        if run_suite "Unit Tests" cargo test --workspace --lib -j "$JOBS" "${EXTRA_ARGS[@]}"; then
            PASSED=$(( PASSED + 1 ))
        else
            FAILED=$(( FAILED + 1 ))
        fi
        ;;
    core|parsers|forensics|sanitize|cli)
        pkg="metaforge-$MODE"
        [ "$MODE" = "cli" ] && pkg="metaforge"
        ui_info "Executing crate test suite: $pkg..."
        if run_suite "Crate: $pkg" cargo test -p "$pkg" -j "$JOBS" "${EXTRA_ARGS[@]}"; then
            PASSED=$(( PASSED + 1 ))
        else
            FAILED=$(( FAILED + 1 ))
        fi
        ;;
esac

ui_section "Harness Summary"
ui_kv "Passed Suites" "$PASSED"
ui_kv "Failed Suites" "$FAILED"

if [ "$FAILED" -eq 0 ]; then
    ui_success "All executed test suites verified 100% Green."
    exit 0
else
    ui_error "One or more test suites failed. Remediate before deployment."
    exit 1
fi
