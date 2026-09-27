#!/usr/bin/env bash
# ==============================================================================
# 🎨 METAFORGE ENTERPRISE COLOR & ANSI STYLE MATRIX
# File: scripts/lib/colors.sh
# Purpose: High-fidelity 256-color, Truecolor, and ANSI text styling singleton
# Standards: Pure POSIX-compliant escape sequences with Bash parameter guards
# ==============================================================================

# Guard against duplicate inclusion
if [ -n "${_METAFORGE_COLORS_LOADED:-}" ]; then
    return 0 2>/dev/null || exit 0
fi
_METAFORGE_COLORS_LOADED=1

# ------------------------------------------------------------------------------
# 1. Terminal Capability Detection
# ------------------------------------------------------------------------------
_check_color_support() {
    if [ -t 1 ] && [ -n "${TERM:-}" ] && [ "${TERM}" != "dumb" ]; then
        return 0
    fi
    # Force color if requested
    if [ "${METAFORGE_FORCE_COLOR:-0}" = "1" ]; then
        return 0
    fi
    return 1
}

# ------------------------------------------------------------------------------
# 2. ANSI Reset & Typography Styles
# ------------------------------------------------------------------------------
if _check_color_support; then
    RESET="\033[0m"
    BOLD="\033[1m"
    DIM="\033[2m"
    ITALIC="\033[3m"
    UNDERLINE="\033[4m"
    BLINK="\033[5m"
    INVERSE="\033[7m"
    HIDDEN="\033[8m"
    STRIKETHROUGH="\033[9m"

    # Standard 16 Foreground Colors
    FG_BLACK="\033[30m"
    FG_RED="\033[31m"
    FG_GREEN="\033[32m"
    FG_YELLOW="\033[33m"
    FG_BLUE="\033[34m"
    FG_MAGENTA="\033[35m"
    FG_CYAN="\033[36m"
    FG_WHITE="\033[37m"

    # Standard 16 Bright Foreground Colors
    FG_BRIGHT_BLACK="\033[90m"
    FG_BRIGHT_RED="\033[91m"
    FG_BRIGHT_GREEN="\033[92m"
    FG_BRIGHT_YELLOW="\033[93m"
    FG_BRIGHT_BLUE="\033[94m"
    FG_BRIGHT_MAGENTA="\033[95m"
    FG_BRIGHT_CYAN="\033[96m"
    FG_BRIGHT_WHITE="\033[97m"

    # Standard 16 Background Colors
    BG_BLACK="\033[40m"
    BG_RED="\033[41m"
    BG_GREEN="\033[42m"
    BG_YELLOW="\033[43m"
    BG_BLUE="\033[44m"
    BG_MAGENTA="\033[45m"
    BG_CYAN="\033[46m"
    BG_WHITE="\033[47m"

    # Standard 16 Bright Background Colors
    BG_BRIGHT_BLACK="\033[100m"
    BG_BRIGHT_RED="\033[101m"
    BG_BRIGHT_GREEN="\033[102m"
    BG_BRIGHT_YELLOW="\033[103m"
    BG_BRIGHT_BLUE="\033[104m"
    BG_BRIGHT_MAGENTA="\033[105m"
    BG_BRIGHT_CYAN="\033[106m"
    BG_BRIGHT_WHITE="\033[107m"

    # Sovereign Semantic 256-Color Palette
    C_SOVEREIGN_CYAN="\033[38;5;51m"
    C_SOVEREIGN_GOLD="\033[38;5;220m"
    C_SOVEREIGN_PURPLE="\033[38;5;141m"
    C_SOVEREIGN_EMERALD="\033[38;5;48m"
    C_SOVEREIGN_CRIMSON="\033[38;5;196m"
    C_SOVEREIGN_ORANGE="\033[38;5;208m"
    C_SOVEREIGN_ICE="\033[38;5;159m"
    C_SOVEREIGN_SLATE="\033[38;5;244m"
    C_SOVEREIGN_MUTED="\033[38;5;240m"
    C_SOVEREIGN_WHITE="\033[38;5;255m"

    # Semantic Status Indicators
    C_SUCCESS="${FG_BRIGHT_GREEN}"
    C_INFO="${FG_BRIGHT_CYAN}"
    C_WARN="${FG_BRIGHT_YELLOW}"
    C_ERROR="${FG_BRIGHT_RED}"
    C_DEBUG="${FG_BRIGHT_BLACK}"
    C_ACCENT="${C_SOVEREIGN_CYAN}"
else
    RESET=""
    BOLD=""
    DIM=""
    ITALIC=""
    UNDERLINE=""
    BLINK=""
    INVERSE=""
    HIDDEN=""
    STRIKETHROUGH=""

    FG_BLACK=""
    FG_RED=""
    FG_GREEN=""
    FG_YELLOW=""
    FG_BLUE=""
    FG_MAGENTA=""
    FG_CYAN=""
    FG_WHITE=""

    FG_BRIGHT_BLACK=""
    FG_BRIGHT_RED=""
    FG_BRIGHT_GREEN=""
    FG_BRIGHT_YELLOW=""
    FG_BRIGHT_BLUE=""
    FG_BRIGHT_MAGENTA=""
    FG_BRIGHT_CYAN=""
    FG_BRIGHT_WHITE=""

    BG_BLACK=""
    BG_RED=""
    BG_GREEN=""
    BG_YELLOW=""
    BG_BLUE=""
    BG_MAGENTA=""
    BG_CYAN=""
    BG_WHITE=""

    BG_BRIGHT_BLACK=""
    BG_BRIGHT_RED=""
    BG_BRIGHT_GREEN=""
    BG_BRIGHT_YELLOW=""
    BG_BRIGHT_BLUE=""
    BG_BRIGHT_MAGENTA=""
    BG_BRIGHT_CYAN=""
    BG_BRIGHT_WHITE=""

    C_SOVEREIGN_CYAN=""
    C_SOVEREIGN_GOLD=""
    C_SOVEREIGN_PURPLE=""
    C_SOVEREIGN_EMERALD=""
    C_SOVEREIGN_CRIMSON=""
    C_SOVEREIGN_ORANGE=""
    C_SOVEREIGN_ICE=""
    C_SOVEREIGN_SLATE=""
    C_SOVEREIGN_MUTED=""
    C_SOVEREIGN_WHITE=""

    C_SUCCESS=""
    C_INFO=""
    C_WARN=""
    C_ERROR=""
    C_DEBUG=""
    C_ACCENT=""
fi

# ------------------------------------------------------------------------------
# 3. Dynamic Color Generator Functions
# ------------------------------------------------------------------------------

# Generate 256-color foreground escape: color_256 <0-255>
color_256_fg() {
    local code="${1:-0}"
    if _check_color_support; then
        printf "\033[38;5;%dm" "$code"
    fi
}

# Generate 256-color background escape: color_256_bg <0-255>
color_256_bg() {
    local code="${1:-0}"
    if _check_color_support; then
        printf "\033[48;5;%dm" "$code"
    fi
}

# Generate 24-bit Truecolor (RGB) foreground escape: color_rgb_fg <r> <g> <b>
color_rgb_fg() {
    local r="${1:-0}" g="${2:-0}" b="${3:-0}"
    if _check_color_support; then
        printf "\033[38;2;%d;%d;%dm" "$r" "$g" "$b"
    fi
}

# Generate 24-bit Truecolor (RGB) background escape: color_rgb_bg <r> <g> <b>
color_rgb_bg() {
    local r="${1:-0}" g="${2:-0}" b="${3:-0}"
    if _check_color_support; then
        printf "\033[48;2;%d;%d;%dm" "$r" "$g" "$b"
    fi
}

# ------------------------------------------------------------------------------
# 4. Unicode Monospace Box Drawing Glyphs
# ------------------------------------------------------------------------------
BOX_TL="┌"
BOX_TR="┐"
BOX_BL="└"
BOX_BR="┘"
BOX_H="─"
BOX_V="│"
BOX_T_DOWN="┬"
BOX_T_UP="┴"
BOX_T_RIGHT="├"
BOX_T_LEFT="┤"
BOX_CROSS="┼"

# Double Line Glyphs
BOX_DBL_TL="╔"
BOX_DBL_TR="╗"
BOX_DBL_BL="╚"
BOX_DBL_BR="╝"
BOX_DBL_H="═"
BOX_DBL_V="║"

# Rounded Corner Glyphs
BOX_RND_TL="╭"
BOX_RND_TR="╮"
BOX_RND_BL="╰"
BOX_RND_BR="╯"

# Sovereign Status Glyphs
GLYPH_CHECK="✔"
GLYPH_CROSS="✘"
GLYPH_WARN="⚠"
GLYPH_ARROW="➜"
GLYPH_STAR="★"
GLYPH_DOT="•"
GLYPH_RADIO_ON="◉"
GLYPH_RADIO_OFF="○"

# ------------------------------------------------------------------------------
# 5. Interactive Palette Viewer (Invoked when run directly)
# ------------------------------------------------------------------------------
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    printf "\n%b╔════════════════════════════════════════════════════════════════════╗%b\n" "$BOLD$C_SOVEREIGN_CYAN" "$RESET"
    printf "%b║            METAFORGE 256-COLOR PALETTE & GLYPH MATRIX              ║%b\n" "$BOLD$C_SOVEREIGN_CYAN" "$RESET"
    printf "%b╚════════════════════════════════════════════════════════════════════╝%b\n\n" "$BOLD$C_SOVEREIGN_CYAN" "$RESET"

    printf "%bStandard 16 Colors:%b\n" "$BOLD" "$RESET"
    for i in {0..15}; do
        printf "%b %02d %b" "$(color_256_bg "$i")" "$i" "$RESET"
        [ "$(( (i + 1) % 8 ))" -eq 0 ] && printf "\n"
    done

    printf "\n%b216 Color Cube (6x6x6):%b\n" "$BOLD" "$RESET"
    for i in {16..231}; do
        printf "%b %03d %b" "$(color_256_bg "$i")" "$i" "$RESET"
        [ "$(( (i - 15) % 18 ))" -eq 0 ] && printf "\n"
    done

    printf "\n%b24 Grayscale Levels:%b\n" "$BOLD" "$RESET"
    for i in {232..255}; do
        printf "%b %03d %b" "$(color_256_bg "$i")" "$i" "$RESET"
    done
    printf "\n\n"

    printf "%bBox Drawing Showcase:%b\n" "$BOLD" "$RESET"
    printf "%b%s%s%s%b\n" "$C_SOVEREIGN_CYAN" "$BOX_RND_TL" "$(printf '%*s' 40 '' | tr ' ' "$BOX_H")" "$BOX_RND_TR" "$RESET"
    printf "%b%s  %b%s Verified Sovereign System Matrix%b%*s%b%s%b\n" \
        "$C_SOVEREIGN_CYAN" "$BOX_V" "$BOLD$C_SOVEREIGN_GOLD" "$GLYPH_CHECK" "$RESET" 7 "" "$C_SOVEREIGN_CYAN" "$BOX_V" "$RESET"
    printf "%b%s%s%s%b\n\n" "$C_SOVEREIGN_CYAN" "$BOX_RND_BL" "$(printf '%*s' 40 '' | tr ' ' "$BOX_H")" "$BOX_RND_BR" "$RESET"
fi
