//! Statistics for honest reporting.

/// Wilson score interval for a proportion (95% by default, z = 1.96).
pub fn wilson(p: f64, n: usize, z: f64) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let n = n as f64;
    let denom = 1.0 + z * z / n;
    let centre = (p + z * z / (2.0 * n)) / denom;
    let half = z * ((p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt()) / denom;
    ((centre - half).max(0.0), (centre + half).min(1.0))
}

pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_brackets_p() {
        let (lo, hi) = wilson(0.5, 100, 1.96);
        assert!(lo < 0.5 && hi > 0.5);
        assert!((lo - 0.4038).abs() < 1e-3 && (hi - 0.5962).abs() < 1e-3);
    }
}
