//! Voice feature to ELP tensor mapping
//!
//! Maps voice spectral features to ELP (Ethos-Logos-Pathos) tensor coordinates
//! using heuristic and ML-based approaches.

use crate::voice_pipeline::SpectralFeatures;
use crate::normalization::normalize_to_13_scale;
use crate::models::ELPTensor;

/// Maps voice spectral features to ELP tensor coordinates
///
/// Uses heuristic mapping based on voice characteristics:
/// - **Ethos** (Character): Derived from loudness and voice stability
/// - **Logos** (Logic): Derived from pitch height and clarity
/// - **Pathos** (Emotion): Derived from spectral complexity and expressiveness
///
/// # Examples
///
/// ```
/// use spatial_vortex::voice_pipeline::{VoiceToELPMapper, SpectralFeatures};
///
/// let mapper = VoiceToELPMapper::new();
/// let features = SpectralFeatures {
///     pitch: 200.0,
///     loudness: -20.0,
///     spectral_complexity: 0.5,
///     spectral_centroid: 1500.0,
///     spectral_flux: 0.3,
/// };
///
/// let elp = mapper.map(&features);
/// println!("ELP: E={:.2}, L={:.2}, P={:.2}", elp.ethos, elp.logos, elp.pathos);
/// ```
pub struct VoiceToELPMapper {
    // Future: ML model for learned mapping
    // model: Option<Box<dyn VoiceModel>>,
}

impl VoiceToELPMapper {
    /// Creates a new voice to ELP mapper
    pub fn new() -> Self {
        Self {}
    }
    
    /// Maps spectral features to ELP tensor in 13-scale coordinates
    ///
    /// # Mapping Strategy
    ///
    /// **Ethos (Character/Authority)**:
    /// - High loudness → High authority → High Ethos
    /// - Voice stability (low flux) → Confidence → Positive Ethos
    /// - Range: Loud/stable voice = +13, Quiet/unstable = -13
    ///
    /// **Logos (Logic/Analytical)**:
    /// - High pitch → More analytical/intellectual → High Logos
    /// - Clear voice (low complexity) → Rational → Positive Logos
    /// - Range: High pitch/clear = +13, Low pitch/unclear = -13
    ///
    /// **Pathos (Emotion/Expression)**:
    /// - High complexity → More emotional/expressive → High Pathos
    /// - Dynamic voice (high flux) → Passionate → Positive Pathos
    /// - Range: Complex/dynamic = +13, Simple/static = -13
    ///
    /// All values are normalized to [-13, 13] using the sacred scale.
    pub fn map(&self, features: &SpectralFeatures) -> ELPTensor {
        // Raw mappings (before normalization)
        let raw_ethos = self.compute_ethos(features);
        let raw_logos = self.compute_logos(features);
        let raw_pathos = self.compute_pathos(features);
        
        // Normalize to 13-scale
        let (ethos, logos, pathos) = normalize_to_13_scale(raw_ethos, raw_logos, raw_pathos);
        
        ELPTensor { ethos, logos, pathos }
    }
    
    /// Maps with confidence score
    ///
    /// Returns both the ELP tensor and a confidence score indicating
    /// how reliable the mapping is based on feature quality.
    pub fn map_with_confidence(&self, features: &SpectralFeatures) -> (ELPTensor, f64) {
        let elp = self.map(features);
        let confidence = self.compute_confidence(features);
        (elp, confidence)
    }
    
    /// Computes Ethos from loudness and stability
    ///
    /// Ethos represents authority and character:
    /// - Louder voice (controlled) → More authoritative
    /// - Lower pitch range → More grounded/authoritative
    /// - Stable voice (low flux) → More confident
    fn compute_ethos(&self, features: &SpectralFeatures) -> f64 {
        // Loudness contribution (normalize -60 to 0 dB to 0-100)
        let loudness_factor = ((features.loudness + 60.0) / 60.0 * 100.0).clamp(0.0, 100.0);
        
        // Pitch grounding: Lower pitch often perceived as more authoritative
        // Normalize 80Hz-200Hz as peak Ethos range
        let pitch_grounding = if features.pitch > 0.0 {
            let target = 120.0; // Hz
            let dist = (features.pitch - target).abs();
            (1.0 - (dist / 200.0)).clamp(0.0, 1.0) * 100.0
        } else {
            0.0
        };

        // Stability contribution (inverse of flux)
        let stability_factor = (1.0 - features.spectral_flux.min(1.0)) * 50.0;
        
        // Combine: 50% loudness, 30% pitch, 20% stability
        0.5 * loudness_factor + 0.3 * pitch_grounding + 0.2 * stability_factor
    }
    
    /// Computes Logos from pitch clarity and articulation
    ///
    /// Logos represents analytical and logical qualities:
    /// - Mid-High pitch → More analytical/intellectual
    /// - Lower complexity → More rational/clear
    fn compute_logos(&self, features: &SpectralFeatures) -> f64 {
        // Pitch contribution (normalize 150-300 Hz as peak Logos range)
        let pitch_factor = if features.pitch > 0.0 {
             let target = 220.0;
             let dist = (features.pitch - target).abs();
             (1.0 - (dist / 250.0)).clamp(0.0, 1.0) * 100.0
        } else {
            0.0
        };
        
        // Clarity contribution (inverse of complexity)
        let clarity_factor = (1.0 - features.spectral_complexity) * 100.0;
        
        // Combine: 50% pitch, 50% clarity
        0.5 * pitch_factor + 0.5 * clarity_factor
    }
    
    /// Computes Pathos from complexity and dynamics
    ///
    /// Pathos represents emotional and expressive qualities:
    /// - High variation/flux → Emotional modulation
    /// - Extreme pitch (High or Low) → Emotional intensity
    fn compute_pathos(&self, features: &SpectralFeatures) -> f64 {
        // Complexity contribution
        let complexity_factor = features.spectral_complexity * 100.0;
        
        // Dynamics contribution (flux as expressiveness)
        let dynamics_factor = features.spectral_flux.min(1.0) * 100.0;
        
        // Pitch extremity: deviation from neutral 180Hz
        let pitch_intensity = if features.pitch > 0.0 {
            let dist = (features.pitch - 180.0).abs();
            (dist / 100.0).clamp(0.0, 1.0) * 100.0
        } else {
            0.0
        };

        // Combine: 40% complexity, 30% dynamics, 30% pitch intensity
        0.4 * complexity_factor + 0.3 * dynamics_factor + 0.3 * pitch_intensity
    }
    
    /// Computes confidence in the mapping
    ///
    /// Based on feature quality indicators:
    /// - Reasonable pitch range
    /// - Adequate loudness
    /// - Consistent features
    fn compute_confidence(&self, features: &SpectralFeatures) -> f64 {
        let mut confidence: f64 = 1.0;
        
        // Reduce confidence if pitch is outside human voice range
        if features.pitch < 80.0 || features.pitch > 400.0 {
            confidence *= 0.5;
        }
        
        // Reduce confidence if too quiet
        if features.loudness < -50.0 {
            confidence *= 0.7;
        }
        
        // Reduce confidence if features are extreme
        if features.spectral_complexity > 0.95 || features.spectral_complexity < 0.05 {
            confidence *= 0.8;
        }
        
        confidence.clamp(0.0, 1.0)
    }
}

impl Default for VoiceToELPMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_features(pitch: f64, loudness: f64, complexity: f64) -> SpectralFeatures {
        SpectralFeatures {
            pitch,
            loudness,
            spectral_complexity: complexity,
            spectral_centroid: 1500.0,
            spectral_flux: 0.3,
        }
    }
    
    #[test]
    fn test_mapper_creation() {
        let mapper = VoiceToELPMapper::new();
        let features = create_test_features(200.0, -20.0, 0.5);
        let elp = mapper.map(&features);
        
        // All values should be in 13-scale range
        assert!(elp.ethos.abs() <= 13.0);
        assert!(elp.logos.abs() <= 13.0);
        assert!(elp.pathos.abs() <= 13.0);
    }
    
    #[test]
    fn test_loud_voice_high_ethos() {
        let mapper = VoiceToELPMapper::new();
        
        // Quiet voice at neutral pitch
        let quiet = create_test_features(150.0, -50.0, 0.5);
        // Loud voice at grounded pitch (120Hz) - optimal for Ethos
        let loud = create_test_features(120.0, -10.0, 0.5);
        
        let quiet_elp = mapper.map(&quiet);
        let loud_elp = mapper.map(&loud);
        
        // Louder, grounded voice should have higher Ethos
        assert!(loud_elp.ethos > quiet_elp.ethos);
    }
    
    #[test]
    fn test_high_pitch_high_logos() {
        let mapper = VoiceToELPMapper::new();
        
        // Low pitch (outside Logos peak)
        let low_pitch = create_test_features(100.0, -20.0, 0.5);
        // Mid-High pitch (220Hz - optimal Logos range)
        let high_pitch = create_test_features(220.0, -20.0, 0.5);
        
        let low_elp = mapper.map(&low_pitch);
        let high_elp = mapper.map(&high_pitch);
        
        // Optimal Logos pitch should score higher
        assert!(high_elp.logos > low_elp.logos);
    }
    
    #[test]
    fn test_complex_spectrum_high_pathos() {
        let mapper = VoiceToELPMapper::new();
        
        let simple = create_test_features(200.0, -20.0, 0.1);
        let complex = create_test_features(200.0, -20.0, 0.9);
        
        let simple_elp = mapper.map(&simple);
        let complex_elp = mapper.map(&complex);
        
        // More complex spectrum should have higher Pathos
        assert!(complex_elp.pathos > simple_elp.pathos);
    }
    
    #[test]
    fn test_tensor_magnitude() {
        let tensor = ELPTensor::new(3.0, 4.0, 0.0);
        assert!((tensor.magnitude() - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_dominant_channel() {
        let ethos_dominant = ELPTensor::new(10.0, 5.0, 3.0);
        assert_eq!(ethos_dominant.dominant_channel(), "Ethos");
        
        let logos_dominant = ELPTensor::new(3.0, 10.0, 5.0);
        assert_eq!(logos_dominant.dominant_channel(), "Logos");
        
        let pathos_dominant = ELPTensor::new(3.0, 5.0, 10.0);
        assert_eq!(pathos_dominant.dominant_channel(), "Pathos");
    }
    
    #[test]
    fn test_confidence_computation() {
        let mapper = VoiceToELPMapper::new();
        
        // Good features should have high confidence
        let good = create_test_features(200.0, -20.0, 0.5);
        let (_, conf_good) = mapper.map_with_confidence(&good);
        assert!(conf_good > 0.9);
        
        // Bad features should have lower confidence
        let bad = create_test_features(50.0, -70.0, 0.99); // Too low pitch, too quiet, extreme complexity
        let (_, conf_bad) = mapper.map_with_confidence(&bad);
        assert!(conf_bad < 0.5);
    }
}
