use serde::{Deserialize, Serialize};

/// Severity level of a steganalysis finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Category of the steganalysis finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StegFindingType {
    SuspiciousScanData,
    SuspiciousComment,
    SuspiciousAppSegment,
    EmbeddedAsciiText,
    AnomalousPoVDistribution,
    GenericAnomaly,
}

/// Individual anomaly or finding detected during steganalysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteganalysisFinding {
    pub finding_type: StegFindingType,
    pub description: String,
    pub severity: FindingSeverity,
    pub metric_value: Option<f64>,
}

/// Comprehensive report of statistical steganalysis performed on JPEG scan data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteganalysisReport {
    pub scan_data_bytes: usize,
    pub chi_square_stat: f64,
    pub degrees_of_freedom: usize,
    pub p_value_approx: f64,
    pub pov_chi_square_stat: Option<f64>,
    pub ascii_ratio: f64,
    pub findings: Vec<SteganalysisFinding>,
    pub is_suspicious: bool,
}

/// Critical value for Chi-Square distribution with df=255 at alpha=0.01 (99% confidence).
pub const CHI_SQUARE_CRITICAL_DF255_A01: f64 = 310.457;

/// Critical value for Chi-Square distribution with df=255 at alpha=0.05 (95% confidence).
pub const CHI_SQUARE_CRITICAL_DF255_A05: f64 = 293.248;

/// Extracts the raw entropy-coded scan data stream following the Start of Scan (SOS, 0xFFDA) marker.
///
/// Un-stuffs `0xFF00` sequences (converting them back to a single `0xFF` byte) and consumes
/// restart markers (`0xFFD0` - `0xFFD7`) without adding them to the scan payload.
/// Parsing stops upon encountering the next legitimate structural marker (such as `0xFFD9` EOI).
pub fn extract_entropy_continuum(bytes: &[u8]) -> Vec<u8> {
    let mut scan_data = Vec::new();
    let mut pos = 0;

    // 1. Locate the SOS marker (0xFF, 0xDA)
    while pos + 1 < bytes.len() {
        if bytes[pos] == 0xFF && bytes[pos + 1] == 0xDA {
            pos += 2;
            break;
        }
        pos += 1;
    }

    if pos >= bytes.len() {
        return scan_data; // No SOS marker found
    }

    // 2. Skip SOS header (length is 2-byte big-endian)
    if pos + 2 > bytes.len() {
        return scan_data;
    }
    let header_len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
    pos += header_len;

    // 3. Read entropy-coded data
    while pos < bytes.len() {
        let b1 = bytes[pos];
        pos += 1;

        if b1 == 0xFF {
            if pos >= bytes.len() {
                scan_data.push(0xFF);
                break;
            }
            let b2 = bytes[pos];
            pos += 1;

            if b2 == 0x00 {
                // Stuffed byte: 0xFF 0x00 represents literal 0xFF
                scan_data.push(0xFF);
            } else if (0xD0..=0xD7).contains(&b2) {
                // RSTn restart marker (0xFFD0 - 0xFFD7): skip without adding to scan data
                continue;
            } else {
                // Structural marker encountered (e.g. 0xFFD9 EOI or DNL/another SOS) -> scan end
                break;
            }
        } else {
            scan_data.push(b1);
        }
    }

    scan_data
}

/// Calculates the Chi-Square statistic and approximated p-value against a uniform distribution.
///
/// Degree of freedom = 255 (256 byte values - 1).
/// Uses the Wilson-Hilferty normal approximation of the Chi-Square distribution for high df.
pub fn calculate_chi_square_uniformity(data: &[u8]) -> (f64, usize, f64) {
    if data.is_empty() {
        return (0.0, 255, 1.0);
    }

    let mut observed = [0u64; 256];
    for &b in data {
        observed[b as usize] += 1;
    }

    let total = data.len() as f64;
    let expected = (total / 256.0).max(1.0);

    let mut chi_square = 0.0;
    for &count in &observed {
        let diff = count as f64 - expected;
        chi_square += (diff * diff) / expected;
    }

    let df = 255;
    let p_val = approximate_chi_square_p_value(chi_square, df);
    (chi_square, df, p_val)
}

/// Westfeld-Pfitzmann Pairs-of-Values (PoVs) Chi-Square test for LSB embedding.
///
/// Under natural image compression, frequencies within adjacent pairs (2k, 2k+1) differ.
/// Steganographic LSB replacement equalizes their frequencies, making chi-square drop towards 0.
pub fn calculate_pov_chi_square(data: &[u8]) -> (f64, usize) {
    if data.is_empty() {
        return (0.0, 0);
    }

    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }

    let mut chi_square = 0.0;
    let mut active_pairs: usize = 0;

    for k in 0..128 {
        let n0 = counts[2 * k] as f64;
        let n1 = counts[2 * k + 1] as f64;
        let sum = n0 + n1;
        if sum > 0.0 {
            active_pairs += 1;
            let diff = n0 - n1;
            chi_square += (diff * diff) / (2.0 * sum);
        }
    }

    let df = active_pairs.saturating_sub(1);
    (chi_square, df)
}

/// Computes the ratio of printable graphic/whitespace ASCII characters.
pub fn calculate_ascii_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let printable = data
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\r' || b == b'\t')
        .count();
    printable as f64 / data.len() as f64
}

/// Approximate p-value for large degrees of freedom using the Wilson-Hilferty transformation.
pub fn approximate_chi_square_p_value(chi_sq: f64, df: usize) -> f64 {
    if df == 0 || chi_sq <= 0.0 {
        return 1.0;
    }

    let k = df as f64;
    // Wilson-Hilferty transformation into standard normal Z
    let term1 = (chi_sq / k).powf(1.0 / 3.0);
    let term2 = 1.0 - (2.0 / (9.0 * k));
    let denominator = (2.0 / (9.0 * k)).sqrt();
    let z = (term1 - term2) / denominator;

    // Normal complementary CDF approximation (erfc)
    normal_ccdf(z)
}

/// Standard normal complementary CDF Q(z) = 1 - Phi(z).
fn normal_ccdf(z: f64) -> f64 {
    if z < -8.0 {
        return 1.0;
    }
    if z > 8.0 {
        return 0.0;
    }
    // High-precision rational Chebyshev approximation for erfc(x / sqrt(2)) / 2
    let x = z / std::f64::consts::SQRT_2;
    0.5 * erfc_approx(x)
}

/// Rational approximation for complementary error function erfc(x).
fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let poly = t
        * (0.254829592
            + t * (-0.284496736
                + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    let ans = poly * (-x * x).exp();
    if x >= 0.0 {
        ans
    } else {
        2.0 - ans
    }
}

/// Analyzes JPEG comment (0xFFFE) payload for hidden text or anomalies.
pub fn analyze_comment_segment(comment_data: &[u8]) -> Vec<SteganalysisFinding> {
    let mut findings = Vec::new();
    if comment_data.is_empty() {
        return findings;
    }

    // 1. Length check (> 256 bytes)
    if comment_data.len() > 256 {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::SuspiciousComment,
            description: format!(
                "Unusually large comment payload detected ({} bytes > 256 threshold)",
                comment_data.len()
            ),
            severity: FindingSeverity::Low,
            metric_value: Some(comment_data.len() as f64),
        });
    }

    // 2. Non-printable ratio (> 15%)
    let non_printable = comment_data
        .iter()
        .filter(|&&b| !(32..=126).contains(&b) && !(9..=13).contains(&b))
        .count();
    let ratio = non_printable as f64 / comment_data.len() as f64;
    if ratio > 0.15 {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::SuspiciousComment,
            description: format!(
                "High proportion of non-printable bytes in COM segment ({:.1}% > 15.0% threshold)",
                ratio * 100.0
            ),
            severity: FindingSeverity::Medium,
            metric_value: Some(ratio),
        });
    }

    findings
}

/// Analyzes an APPn segment for anomalies and excessive binary footprints.
pub fn analyze_app_segment(app_data: &[u8], marker: u8) -> Vec<SteganalysisFinding> {
    let mut findings = Vec::new();
    if app_data.is_empty() {
        return findings;
    }

    let app_n = marker.saturating_sub(0xE0);

    // 1. Max reasonable size check (> 32 KB)
    if app_data.len() > 32_768 {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::SuspiciousAppSegment,
            description: format!(
                "Unusually large APP{} segment detected ({} bytes > 32768 threshold)",
                app_n,
                app_data.len()
            ),
            severity: FindingSeverity::Low,
            metric_value: Some(app_data.len() as f64),
        });
    }

    findings
}

/// Analyzes extracted entropy-coded scan data for statistical steganography anomalies.
pub fn analyze_scan_data(scan_data: &[u8]) -> SteganalysisReport {
    let mut findings = Vec::new();

    if scan_data.len() < 256 {
        return SteganalysisReport {
            scan_data_bytes: scan_data.len(),
            chi_square_stat: 0.0,
            degrees_of_freedom: 255,
            p_value_approx: 1.0,
            pov_chi_square_stat: None,
            ascii_ratio: calculate_ascii_ratio(scan_data),
            findings,
            is_suspicious: false,
        };
    }

    // 1. Uniform distribution Chi-Square test
    let (chi_stat, df, p_val) = calculate_chi_square_uniformity(scan_data);

    if chi_stat > CHI_SQUARE_CRITICAL_DF255_A01 || p_val < 0.01 {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::SuspiciousScanData,
            description: format!(
                "Chi-Square test indicates significant statistical deviation from expected distribution (Stat={:.2}, p={:.4} < alpha=0.01)",
                chi_stat, p_val
            ),
            severity: FindingSeverity::Medium,
            metric_value: Some(chi_stat),
        });
    }

    // 2. Pairs of Values (PoVs) check
    let (pov_stat, pov_df) = calculate_pov_chi_square(scan_data);
    // If active pairs exist and variance is abnormally suppressed
    if pov_df > 64 && pov_stat < (pov_df as f64 * 0.2) {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::AnomalousPoVDistribution,
            description: format!(
                "Pairs-of-Values distribution is artificially equalized (PoV Stat={:.2} vs df={}), indicative of LSB steganography",
                pov_stat, pov_df
            ),
            severity: FindingSeverity::High,
            metric_value: Some(pov_stat),
        });
    }

    // 3. ASCII content check in scan data (> 12% printable graphic)
    let ascii_ratio = calculate_ascii_ratio(scan_data);
    if ascii_ratio > 0.12 {
        findings.push(SteganalysisFinding {
            finding_type: StegFindingType::EmbeddedAsciiText,
            description: format!(
                "Suspicious ASCII concentration detected in compressed scan stream ({:.1}% > 12.0% threshold)",
                ascii_ratio * 100.0
            ),
            severity: FindingSeverity::High,
            metric_value: Some(ascii_ratio),
        });
    }

    let is_suspicious = findings.iter().any(|f| {
        f.severity == FindingSeverity::Medium
            || f.severity == FindingSeverity::High
            || f.severity == FindingSeverity::Critical
    });

    SteganalysisReport {
        scan_data_bytes: scan_data.len(),
        chi_square_stat: chi_stat,
        degrees_of_freedom: df,
        p_value_approx: p_val,
        pov_chi_square_stat: Some(pov_stat),
        ascii_ratio,
        findings,
        is_suspicious,
    }
}

/// End-to-end steganalysis on raw JPEG file bytes.
pub fn analyze_jpeg_steganography(bytes: &[u8]) -> SteganalysisReport {
    let scan_data = extract_entropy_continuum(bytes);
    let mut report = analyze_scan_data(&scan_data);

    // Also scan for comment and APP markers
    let mut pos = 2;
    while pos + 4 <= bytes.len() {
        if bytes[pos] == 0xFF {
            let marker = bytes[pos + 1];
            if marker == 0xFE {
                // COM
                let len = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
                if pos + 2 + len <= bytes.len() && len >= 2 {
                    let com_data = &bytes[pos + 4..pos + 2 + len];
                    let com_findings = analyze_comment_segment(com_data);
                    report.findings.extend(com_findings);
                }
            } else if (0xE0..=0xEF).contains(&marker) {
                // APPn
                let len = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
                if pos + 2 + len <= bytes.len() && len >= 2 {
                    let app_data = &bytes[pos + 4..pos + 2 + len];
                    let app_findings = analyze_app_segment(app_data, marker);
                    report.findings.extend(app_findings);
                }
            }
        }
        pos += 1;
    }

    report.is_suspicious = report.findings.iter().any(|f| {
        f.severity == FindingSeverity::Medium
            || f.severity == FindingSeverity::High
            || f.severity == FindingSeverity::Critical
    });

    report
}
