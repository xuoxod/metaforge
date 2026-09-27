/// Computes the Shannon Entropy of a byte slice.
/// Returns a value between 0.0 (completely predictable) and 8.0 (completely random / encrypted / compressed).
pub fn calculate_shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;
    for count in counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Evaluates if an entropy level indicates likely encryption or compressed steganography.
pub fn is_suspicious_entropy(entropy: f64) -> bool {
    entropy >= 7.95
}
