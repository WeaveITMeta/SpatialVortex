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

/// One-sided exact McNemar test: probability, under no difference, of at
/// least `b` wins out of `b + c` discordant pairs. `b` = items only the
/// candidate got right, `c` = items only the incumbent got right.
pub fn mcnemar_one_sided(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    // P(X >= b), X ~ Binomial(n, 0.5), via log-space terms.
    let ln_half_n = -(n as f64) * std::f64::consts::LN_2;
    let mut ln_c = 0.0f64; // ln C(n, 0)
    let mut total = 0.0;
    for k in 0..=n {
        if k > 0 {
            ln_c += ((n - k + 1) as f64).ln() - (k as f64).ln();
        }
        if k >= b {
            total += (ln_c + ln_half_n).exp();
        }
    }
    total.min(1.0)
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

    #[test]
    fn mcnemar_matches_binomial_tail() {
        // 10 discordant, all for the candidate: p = 0.5^10.
        assert!((mcnemar_one_sided(10, 0) - 0.5f64.powi(10)).abs() < 1e-12);
        assert!((mcnemar_one_sided(0, 0) - 1.0).abs() < 1e-12);
        assert!(mcnemar_one_sided(5, 5) > 0.5);
    }
}
