#!/usr/bin/env bash
# ==============================================================================
# 🖥️ METAFORGE ENTERPRISE CONSOLE UI & LAYOUT SINGLETON
# File: scripts/lib/ui.sh
# Purpose: High-elegance terminal UI primitives, boxed banners, status reporting,
#          and aligned table layouts for sovereign CLI tools.
# ==============================================================================

# Guard against duplicate inclusion
if [ -n "${_METAFORGE_UI_LOADED:-}" ]; then
    return 0 2>/dev/null || exit 0
fi
_METAFORGE_UI_LOADED=1

# Resolve script directory and source colors singleton
_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "${_LIB_DIR}/colors.sh" ]; then
    # shellcheck source=/dev/null
    . "${_LIB_DIR}/colors.sh"
fi

# Terminal width calculation with fallback
ui_term_width() {
    local cols
    cols=$(tput cols 2>/dev/null || echo 80)
    [ "$cols" -lt 40 ] && cols=80
    [ "$cols" -gt 120 ] && cols=100
    echo "$cols"
}

# Safe UTF-8 character repeater
_repeat_char() {
    local char="$1"
    local count="$2"
    local out=""
    for (( i=0; i<count; i++ )); do
        out="${out}${char}"
    done
    printf "%s" "$out"
}

# Print a boxed header banner
ui_header() {
    local title="$1"
    local width
    width=$(ui_term_width)
    local inner_width=$(( width - 4 ))
    local pad_len=$(( inner_width - ${#title} ))
    [ "$pad_len" -lt 0 ] && pad_len=0
    local border
    border=$(_repeat_char "$BOX_H" "$(( width - 2 ))")

    printf "\n%b%s%s%s%b\n" "$BOLD$C_SOVEREIGN_CYAN" "$BOX_RND_TL" "$border" "$BOX_RND_TR" "$RESET"
    printf "%b%s  %b%s%b%*s%b%s%b\n" \
        "$BOLD$C_SOVEREIGN_CYAN" "$BOX_V" "$BOLD$C_SOVEREIGN_WHITE" "$title" "$RESET" "$pad_len" "" "$BOLD$C_SOVEREIGN_CYAN" "$BOX_V" "$RESET"
    printf "%b%s%s%s%b\n\n" "$BOLD$C_SOVEREIGN_CYAN" "$BOX_RND_BL" "$border" "$BOX_RND_BR" "$RESET"
}

# Print a sub-section divider
ui_section() {
    local title="$1"
    local width
    width=$(ui_term_width)
    local line_len=$(( width - ${#title} - 6 ))
    [ "$line_len" -lt 4 ] && line_len=4
    local border
    border=$(_repeat_char "$BOX_H" "$line_len")

    printf "\n%b%s %b%s%b %s%b\n" \
        "$C_SOVEREIGN_PURPLE" "$BOX_H$BOX_H" "$BOLD$C_SOVEREIGN_GOLD" "$title" "$C_SOVEREIGN_PURPLE" "$border" "$RESET"
}

# Print a formatted key-value line
ui_kv() {
    local key="$1"
    local val="$2"
    local key_width="${3:-24}"
    printf "  %b%-*s%b %b%s%b\n" "$DIM$FG_WHITE" "$key_width" "$key:" "$RESET" "$BOLD$C_SOVEREIGN_ICE" "$val" "$RESET"
}

# Print status indicator lines
ui_info() {
    printf "  %b%s%b  %s\n" "$BOLD$FG_BRIGHT_CYAN" "$GLYPH_ARROW" "$RESET" "$*"
}

ui_success() {
    printf "  %b%s%b  %b%s%b\n" "$BOLD$FG_BRIGHT_GREEN" "$GLYPH_CHECK" "$RESET" "$FG_BRIGHT_GREEN" "$*" "$RESET"
}

ui_warn() {
    printf "  %b%s%b  %b%s%b\n" "$BOLD$FG_BRIGHT_YELLOW" "$GLYPH_WARN" "$RESET" "$FG_BRIGHT_YELLOW" "$*" "$RESET"
}

ui_error() {
    printf "  %b%s%b  %b%s%b\n" "$BOLD$FG_BRIGHT_RED" "$GLYPH_CROSS" "$RESET" "$FG_BRIGHT_RED" "$*" "$RESET" >&2
}

ui_pass() {
    printf "  %b[%s PASS]%b %s\n" "$BOLD$FG_BRIGHT_GREEN" "$GLYPH_CHECK" "$RESET" "$*"
}

ui_fail() {
    printf "  %b[%s FAIL]%b %b%s%b\n" "$BOLD$FG_BRIGHT_RED" "$GLYPH_CROSS" "$RESET" "$BOLD$FG_BRIGHT_RED" "$*" "$RESET" >&2
}

# Render an animated or static progress bar: ui_progress <current> <total> [label]
ui_progress() {
    local current="$1"
    local total="$2"
    local label="${3:-Progress}"
    local bar_width=30
    local pct=0
    [ "$total" -gt 0 ] && pct=$(( current * 100 / total ))
    local filled=$(( pct * bar_width / 100 ))
    local empty=$(( bar_width - filled ))

    local fill_str=""
    local empty_str=""
    [ "$filled" -gt 0 ] && fill_str=$(_repeat_char "█" "$filled")
    [ "$empty" -gt 0 ] && empty_str=$(_repeat_char "░" "$empty")

    printf "\r  %-16s %b[%s%s]%b %3d%% (%d/%d)" \
        "$label" "$C_SOVEREIGN_CYAN" "$fill_str" "$empty_str" "$RESET" "$pct" "$current" "$total"
    if [ "$current" -ge "$total" ]; then
        printf "\n"
    fi
}

# Interactive confirmation prompt: ui_confirm "Are you sure?" [default_yes]
ui_confirm() {
    local prompt="$1"
    local def_yes="${2:-1}"
    local hint="[Y/n]"
    [ "$def_yes" -eq 0 ] && hint="[y/N]"

    printf "  %b%s%b %s %b%s%b " "$BOLD$C_SOVEREIGN_GOLD" "$GLYPH_RADIO_ON" "$RESET" "$prompt" "$DIM" "$hint" "$RESET"
    read -r resp
    resp=$(echo "$resp" | tr '[:upper:]' '[:lower:]')

    if [ -z "$resp" ]; then
        return "$(( 1 - def_yes ))"
    fi
    if [ "$resp" = "y" ] || [ "$resp" = "yes" ]; then
        return 0
    fi
    return 1
}

# Self-demo when run directly
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    ui_header "METAFORGE SOVEREIGN CONSOLE UI TEST HARNESS"
    ui_section "Ecosystem Metadata"
    ui_kv "Binary Target" "metaforge"
    ui_kv "Architecture" "x86_64-unknown-linux-musl"
    ui_kv "Security Standard" "AGY-RULE-SOVEREIGN-FLAGSHIP-01"
    ui_kv "Crate Count" "5 Decoupled Workspaces"

    ui_section "Status Verification Indicators"
    ui_info "Initializing sandbox validation pipeline..."
    ui_success "Radix trie memory allocation verified."
    ui_warn "High entropy segment detected in test sample."
    ui_error "Corrupted ISOBMFF box offset intercepted."
    ui_pass "adversarial_bombs_tests (100% Green)"
    ui_fail "Legacy JNI bridge eliminated"

    ui_section "Progress Simulation"
    for i in {1..10}; do
        ui_progress "$i" 10 "Container Audit"
        sleep 0.05
    done
    printf "\n"
fi
