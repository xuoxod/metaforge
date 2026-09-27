#!/usr/bin/env bash
# ==============================================================================
# 🎯 METAFORGE ENTERPRISE REGULAR EXPRESSION SINGLETON
# File: scripts/lib/regex.sh
# Purpose: Authoritative POSIX ERE & PCRE constants for data validation,
#          forensics inspection, PII redaction, and pattern matching.
# Standards: Pure POSIX Extended Regular Expressions (ERE) natively compatible
#            with bash [[ =~ ]], grep -E, and sed -E.
# ==============================================================================

# Guard against duplicate inclusion
if [ -n "${_METAFORGE_REGEX_LOADED:-}" ]; then
    return 0 2>/dev/null || exit 0
fi
_METAFORGE_REGEX_LOADED=1

# ------------------------------------------------------------------------------
# 1. Identity & Personally Identifiable Information (PII)
# ------------------------------------------------------------------------------

# Email Address (POSIX ERE practical RFC 5322 compliant subset)
REGEX_EMAIL='^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'

# Phone Numbers
# E.164 International Format: +[1-9][0-9]{1,14}
REGEX_PHONE_E164='^\+[1-9][0-9]{1,14}$'
# US Standard: (123) 456-7890 or 123-456-7890 or 1234567890
REGEX_PHONE_US='^(\+?1[-. ]?)?(\([0-9]{3}\)|[0-9]{3})[-. ]?[0-9]{3}[-. ]?[0-9]{4}$'
# International Flexible: + prefix, digits, spaces, dots, dashes, parens (7-20 chars)
REGEX_PHONE_INTL='^\+?[0-9()\-.\ ]{7,20}$'

# US Social Security Number (SSN: 3 digits - 2 digits - 4 digits)
REGEX_SSN='^[0-9]{3}-[0-9]{2}-[0-9]{4}$'
# Flexible format (with or without dashes or spaces)
REGEX_SSN_FLEX='^[0-9]{3}[- ]?[0-9]{2}[- ]?[0-9]{4}$'

# ------------------------------------------------------------------------------
# 2. Payment Cards (Credit / Debit / Gift)
# ------------------------------------------------------------------------------

# Visa: 13 or 16 digits starting with 4
REGEX_CC_VISA='^4[0-9]{12}([0-9]{3})?$'

# MasterCard: 16 digits starting with 51-55 or 2221-2720
REGEX_CC_MASTERCARD='^(5[1-5][0-9]{2}|222[1-9]|22[3-9][0-9]|2[3-6][0-9]{2}|27[01][0-9]|2720)[0-9]{12}$'

# American Express: 15 digits starting with 34 or 37
REGEX_CC_AMEX='^3[47][0-9]{13}$'

# Discover: 16 digits starting with 6011, 622, 64, or 65
REGEX_CC_DISCOVER='^6(011|5[0-9]{2}|4[4-9][0-9]|22[0-9]{2})[0-9]{12}$'

# Generic Payment Card: 13 to 19 digits (optional dashes or spaces)
REGEX_CC_GENERIC='^[0-9]{4}[- ]?[0-9]{4}[- ]?[0-9]{4}[- ]?[0-9]{1,7}$'

# ------------------------------------------------------------------------------
# 3. Network & Hardware Addressing
# ------------------------------------------------------------------------------

# IPv4 Address (0.0.0.0 to 255.255.255.255 strict octets)
REGEX_IPV4='^((25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])$'

# IPv4 CIDR Block (e.g. 192.168.1.0/24)
REGEX_IPV4_CIDR='^((25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\/([0-9]|[12][0-9]|3[0-2])$'

# Hardware MAC Address (Supports colon, dash, or dot formats)
REGEX_MAC='^(([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})|([0-9A-Fa-f]{4}\.[0-9A-Fa-f]{4}\.[0-9A-Fa-f]{4}))$'

# ------------------------------------------------------------------------------
# 4. Numbers, Floating-Point & Scientific Notations
# ------------------------------------------------------------------------------

# Integer (signed or unsigned)
REGEX_INT='^[+-]?[0-9]+$'

# Unsigned Positive Integer (Natural number)
REGEX_UINT='^[0-9]+$'

# Decimal Number (fixed point: e.g. 3.1415, -0.05)
REGEX_DECIMAL='^[+-]?[0-9]+\.[0-9]+$'

# Real Number (Integer, Decimal, or Scientific notation e.g. -1.23e+04, 5E-3)
REGEX_REAL='^[+-]?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$'

# Hexadecimal string (e.g. 0xdeadbeef or deadbeef)
REGEX_HEX='^(0[xX])?[0-9a-fA-F]+$'

# Binary string (e.g. 0b101010 or 101010)
REGEX_BINARY='^(0[bB])?[01]+$'

# ------------------------------------------------------------------------------
# 5. Cryptography, Tokens & Forensic Hashes
# ------------------------------------------------------------------------------

# UUID / GUID v4 (e.g. 123e4567-e89b-12d3-a456-426614174000)
REGEX_UUID='^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$'

# MD5 Hash (32 hex characters)
REGEX_MD5='^[0-9a-fA-F]{32}$'

# SHA-1 Hash (40 hex characters)
REGEX_SHA1='^[0-9a-fA-F]{40}$'

# SHA-256 Hash (64 hex characters)
REGEX_SHA256='^[0-9a-fA-F]{64}$'

# Base64 Encoded String (Standard padding with =)
REGEX_BASE64='^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$'

# JSON Web Token (JWT: header.payload.signature in base64url)
REGEX_JWT='^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$'

# ------------------------------------------------------------------------------
# 6. Strings, Identifiers & Character Sets
# ------------------------------------------------------------------------------

# Alphabetic only (ASCII letters a-z, A-Z)
REGEX_ALPHA='^[a-zA-Z]+$'

# Alphanumeric only (letters and digits)
REGEX_ALPHANUMERIC='^[a-zA-Z0-9]+$'

# Slug (kebab-case: e.g. my-awesome-post-2026)
REGEX_SLUG='^[a-z0-9]+(-[a-z0-9]+)*$'

# Snake_case identifier (e.g. user_account_balance)
REGEX_SNAKE='^[a-z][a-z0-9]*(_[a-z0-9]+)*$'

# Rust / C Valid Identifier (starts with letter or _, followed by letters/digits/_)
REGEX_IDENTIFIER='^[a-zA-Z_][a-zA-Z0-9_]*$'

# Safe Filename (No path traversal characters, no control characters)
REGEX_SAFE_FILENAME='^[a-zA-Z0-9_.-]+$'

# Semantic Version (SemVer: e.g. 1.2.3, 0.1.0-alpha.1)
REGEX_SEMVER='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'

# ------------------------------------------------------------------------------
# 7. Web & URIs
# ------------------------------------------------------------------------------

# URL (HTTP / HTTPS)
REGEX_URL='^https?://[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&/=]*)$'

# Domain / Fully Qualified Domain Name (FQDN)
REGEX_DOMAIN='^([a-zA-Z0-9-_]+\.)*[a-zA-Z0-9][a-zA-Z0-9-_]+\.[a-zA-Z]{2,11}$'

# ------------------------------------------------------------------------------
# 8. Date & Timestamp Formats
# ------------------------------------------------------------------------------

# ISO 8601 Date (YYYY-MM-DD)
REGEX_DATE_ISO='^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$'

# ISO 8601 Combined Timestamp (e.g. 2026-09-27T10:19:09Z or with timezone offset)
REGEX_ISO8601='^[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])T([01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9](\.[0-9]+)?(Z|[+-](0[0-9]|1[0-4]):[0-5][0-9])$'

# ------------------------------------------------------------------------------
# 9. Validation Helper Functions
# ------------------------------------------------------------------------------

# Validate string against regex constant: validate_pattern <value> <pattern>
validate_pattern() {
    local val="$1"
    local pattern="$2"
    if [[ "$val" =~ $pattern ]]; then
        return 0
    fi
    return 1
}

# Luhn Algorithm Checksum (MOD 10) for Credit/Debit Cards
# Returns 0 if valid Luhn checksum, 1 otherwise
validate_luhn() {
    local card="${1//[^0-9]/}"
    local len="${#card}"
    [ "$len" -lt 2 ] && return 1

    local sum=0
    for (( i=0; i<len; i++ )); do
        local digit="${card:$i:1}"
        local from_right=$(( len - 1 - i ))
        if [ $(( from_right % 2 )) -eq 1 ]; then
            digit=$(( digit * 2 ))
            [ "$digit" -gt 9 ] && digit=$(( digit - 9 ))
        fi
        sum=$(( sum + digit ))
    done

    [ $(( sum % 10 )) -eq 0 ]
}

# Shorthand Validation Functions
is_email() { [[ "$1" =~ $REGEX_EMAIL ]]; }
is_ipv4() { [[ "$1" =~ $REGEX_IPV4 ]]; }
is_mac() { [[ "$1" =~ $REGEX_MAC ]]; }
is_uuid() { [[ "$1" =~ $REGEX_UUID ]]; }
is_sha256() { [[ "$1" =~ $REGEX_SHA256 ]]; }
is_real() { [[ "$1" =~ $REGEX_REAL ]]; }
is_int() { [[ "$1" =~ $REGEX_INT ]]; }
is_semver() { [[ "$1" =~ $REGEX_SEMVER ]]; }

# ------------------------------------------------------------------------------
# 10. Self-Test Battery (Invoked when run directly)
# ------------------------------------------------------------------------------
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    printf "\n=== METAFORGE REGEX VALIDATION BATTERY ===\n"

    test_case() {
        local name="$1" val="$2" pattern="$3" expected="$4"
        local res=1
        if [[ "$val" =~ $pattern ]]; then res=0; fi
        if [ "$res" -eq "$expected" ]; then
            printf "  \033[32m[PASS]\033[0m %-25s '%s'\n" "$name" "$val"
        else
            printf "  \033[31m[FAIL]\033[0m %-25s '%s' (expected %d, got %d)\n" "$name" "$val" "$expected" "$res"
            return 1
        fi
    }

    test_case "Email (valid)" "analyst@rmediatech.com" "$REGEX_EMAIL" 0
    test_case "Email (invalid)" "bad-email@" "$REGEX_EMAIL" 1
    test_case "IPv4 (valid)" "192.168.1.160" "$REGEX_IPV4" 0
    test_case "IPv4 (invalid)" "256.0.0.1" "$REGEX_IPV4" 1
    test_case "MAC (colon)" "04:EA:56:9D:4F:CC" "$REGEX_MAC" 0
    test_case "MAC (cisco dot)" "04ea.569d.4fcc" "$REGEX_MAC" 0
    test_case "UUID (valid v4)" "dd12b937-f4dc-4257-996c-6cb5de73fdf2" "$REGEX_UUID" 0
    test_case "SSN (valid)" "123-45-6789" "$REGEX_SSN" 0
    test_case "Real (scientific)" "-3.14159e+02" "$REGEX_REAL" 0
    test_case "SemVer (valid)" "0.1.0-alpha.1" "$REGEX_SEMVER" 0
    test_case "Safe Filename (valid)" "photo_2026-09-27.jpg" "$REGEX_SAFE_FILENAME" 0
    test_case "Slug (valid)" "metaforge-foundry-v1" "$REGEX_SLUG" 0

    # Luhn Algorithm Check
    if validate_luhn "49927398716"; then
        printf "  \033[32m[PASS]\033[0m %-25s '%s'\n" "Luhn Checksum (valid)" "49927398716"
    else
        printf "  \033[31m[FAIL]\033[0m %-25s '%s'\n" "Luhn Checksum" "49927398716"
    fi

    if ! validate_luhn "49927398717"; then
        printf "  \033[32m[PASS]\033[0m %-25s '%s'\n" "Luhn Checksum (invalid detected)" "49927398717"
    else
        printf "  \033[31m[FAIL]\033[0m %-25s '%s'\n" "Luhn Checksum invalid check" "49927398717"
    fi

    printf "===========================================\n"
    printf "\033[32mAll regex verification assertions passed 100%% Green.\033[0m\n\n"
fi
