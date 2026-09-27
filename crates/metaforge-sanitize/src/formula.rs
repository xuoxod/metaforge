//! Formula Injection Protection
//!
//! Neutralizes CSV/spreadsheet formula payloads (`=`, `+`, `-`, `@`, `\t`, `\r`)
//! in user-facing text/metadata exports by prefixing with a single quote (`'`).

/// Checks if a text string begins with an active spreadsheet formula trigger.
pub fn is_formula_injection(text: &str) -> bool {
    let trimmed = text.trim_start();
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r')
}

/// Sanitizes a string for spreadsheet/CSV safety by prefixing formula triggers with `'`.
pub fn escape_formula_injection(text: &str) -> String {
    let trimmed = text.trim_start();
    if is_formula_injection(trimmed) {
        format!("'{}", text)
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_detection() {
        assert!(is_formula_injection("=cmd|'/C calc'!A0"));
        assert!(is_formula_injection("+12345"));
        assert!(is_formula_injection("-5+5"));
        assert!(is_formula_injection("@SUM(A1:A10)"));
        assert!(is_formula_injection("\t=1+1"));
        assert!(!is_formula_injection("Normal Photo Description"));
    }

    #[test]
    fn test_formula_escape() {
        assert_eq!(escape_formula_injection("=cmd|'/C calc'!A0"), "'=cmd|'/C calc'!A0");
        assert_eq!(escape_formula_injection("Normal Title"), "Normal Title");
    }
}
