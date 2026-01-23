//! 13-Scale tensor normalization for Vortex Math coordinate system.
//!
//! The 13-scale provides standardized measurements where:
//! - Range: [-13, 13]
//! - Sacred proportion: 13 → 1+3 = 4 (stability)
//! - Sufficient granularity without over-precision

/// Normalizes ELP tensor values to the 13-scale coordinate system.
///
/// # Examples
///
/// ```
/// use spatial_vortex::normalization::normalize_to_13_scale;
///
/// let mut tensor = (50.0, 100.0, 75.0); // (E, L, P)
/// let (e, l, p) = normalize_to_13_scale(tensor.0, tensor.1, tensor.2);
///
/// // All values now in [-13, 13] range
/// assert!(e.abs() <= 13.0);
/// assert!(l.abs() <= 13.0);
/// assert!(p.abs() <= 13.0);
/// ```
pub fn normalize_to_13_scale(ethos: f64, logos: f64, pathos: f64) -> (f64, f64, f64) {
    let max_val = ethos.abs().max(logos.abs()).max(pathos.abs());
    
    if max_val == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let scale_factor = 13.0 / max_val;
    
    (
        (ethos * scale_factor).clamp(-13.0, 13.0),
        (logos * scale_factor).clamp(-13.0, 13.0),
        (pathos * scale_factor).clamp(-13.0, 13.0),
    )
}

/// Denormalizes from 13-scale back to original scale.
///
/// # Arguments
///
/// * `ethos, logos, pathos` - Normalized tensor in [-13, 13] range
/// * `original_max` - Original maximum value before normalization
pub fn denormalize_from_13_scale(
    ethos: f64,
    logos: f64,
    pathos: f64,
    original_max: f64,
) -> (f64, f64, f64) {
    let scale_factor = original_max / 13.0;
    (ethos * scale_factor, logos * scale_factor, pathos * scale_factor)
}

/// Computes tensor magnitude.
pub fn tensor_magnitude(ethos: f64, logos: f64, pathos: f64) -> f64 {
    (ethos.powi(2) + logos.powi(2) + pathos.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalization() {
        let (e, l, p) = normalize_to_13_scale(50.0, 100.0, 75.0);
        
        // Logos was largest (100), should now be 13
        assert!((l - 13.0).abs() < 0.001);
        // Others proportionally scaled
        assert!(e > 0.0 && e < 13.0);
        assert!(p > 0.0 && p < 13.0);
    }
    
    #[test]
    fn test_roundtrip() {
        let original = (50.0, 100.0, 75.0);
        let (e, l, p) = normalize_to_13_scale(original.0, original.1, original.2);
        let (e2, l2, p2) = denormalize_from_13_scale(e, l, p, 100.0);
        
        assert!((e2 - original.0).abs() < 0.001);
        assert!((l2 - original.1).abs() < 0.001);
        assert!((p2 - original.2).abs() < 0.001);
    }
    
    #[test]
    fn test_zero_case() {
        let (e, l, p) = normalize_to_13_scale(0.0, 0.0, 0.0);
        assert_eq!(e, 0.0);
        assert_eq!(l, 0.0);
        assert_eq!(p, 0.0);
    }
}
