//! Confidence scoring for high-value pattern identification.
//!
//! Computes confidence scores based on multiple factors including
//! tensor magnitude, sacred position proximity, and voice energy.

/// Computes confidence score for pattern preservation decisions.
///
/// # Arguments
///
/// * `ethos, logos, pathos` - ELP tensor components
/// * `sacred_distance` - Distance to nearest sacred position (3, 6, or 9)
/// * `voice_energy` - Voice intensity (0-100 range)
///
/// # Returns
///
/// * `f64` - Confidence score (higher = more valuable)
///
/// # Formula
///
/// ```text
/// score = magnitude × sacred_bonus × energy_factor
/// where:
///   magnitude = sqrt(E² + L² + P²)
///   sacred_bonus = 2.0 if distance < 1.0, else 1.0
///   energy_factor = voice_energy / 50.0 (clamped to 0.5-2.0)
/// ```
///
/// # Examples
///
/// ```
/// use spatial_vortex::confidence_scoring::compute_confidence;
///
/// let score = compute_confidence(10.0, 11.0, 9.0, 0.8, 85.0);
/// assert!(score > 25.0); // High-value pattern
/// ```
pub fn compute_confidence(
    ethos: f64,
    logos: f64,
    pathos: f64,
    sacred_distance: f64,
    voice_energy: f64,
) -> f64 {
    let magnitude = (ethos.powi(2) + logos.powi(2) + pathos.powi(2)).sqrt();
    let sacred_bonus = if sacred_distance < 1.0 { 2.0 } else { 1.0 };
    let energy_factor = (voice_energy / 50.0).clamp(0.5, 2.0);
    
    magnitude * sacred_bonus * energy_factor
}

/// Checks if score qualifies as high-value (worthy of preservation).
///
/// # Arguments
///
/// * `score` - Confidence score
/// * `threshold` - Minimum score for high-value classification
pub fn is_high_value(score: f64, threshold: f64) -> bool {
    score >= threshold
}

/// Computes decay factor for aging patterns.
///
/// Older patterns decay in importance unless reinforced.
/// Uses exponential decay: e^(-λt) where λ = 0.01/hour
///
/// # Arguments
///
/// * `age_hours` - Pattern age in hours since creation
///
/// # Returns
///
/// * `f64` - Decay multiplier (1.0 = fresh, 0.0 = fully decayed)
///
/// # Examples
///
/// ```
/// use spatial_vortex::confidence_scoring::decay_factor;
///
/// assert!((decay_factor(0.0) - 1.0).abs() < 0.001); // Fresh
/// assert!(decay_factor(100.0) < 0.5); // Old, significantly decayed
/// ```
pub fn decay_factor(age_hours: f64) -> f64 {
    (-0.01 * age_hours).exp()
}

/// Computes adjusted confidence with decay applied.
pub fn confidence_with_decay(
    base_confidence: f64,
    age_hours: f64,
) -> f64 {
    base_confidence * decay_factor(age_hours)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_high_value_near_sacred() {
        let score = compute_confidence(10.0, 11.0, 9.0, 0.5, 90.0);
        assert!(is_high_value(score, 8.0));
        assert!(score > 25.0);
    }
    
    #[test]
    fn test_low_value_far_from_sacred() {
        let score = compute_confidence(5.0, 5.0, 5.0, 10.0, 20.0);
        assert!(!is_high_value(score, 8.0));
    }
    
    #[test]
    fn test_sacred_bonus() {
        let near = compute_confidence(10.0, 10.0, 10.0, 0.5, 50.0);
        let far = compute_confidence(10.0, 10.0, 10.0, 5.0, 50.0);
        assert!(near > far * 1.9); // Should be ~2x
    }
    
    #[test]
    fn test_decay() {
        assert!((decay_factor(0.0) - 1.0).abs() < 0.001);
        assert!(decay_factor(100.0) < 0.5);
        assert!(decay_factor(50.0) > decay_factor(100.0));
    }
    
    #[test]
    fn test_confidence_with_decay() {
        let base = 10.0;
        let fresh = confidence_with_decay(base, 0.0);
        let old = confidence_with_decay(base, 100.0);
        assert!((fresh - base).abs() < 0.001);
        assert!(old < base / 2.0);
    }
}
