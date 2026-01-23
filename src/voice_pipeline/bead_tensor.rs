//! BeadTensor: Time-stamped ELP tensors with voice metadata
//!
//! Represents a single "bead" on the time-series string of voice analysis,
//! containing ELP tensor coordinates plus voice characteristics.

use chrono::{DateTime, Utc};
use crate::voice_pipeline::{ELPTensor, SpectralFeatures};

/// A time-stamped tensor representing a moment of voice input
///
/// BeadTensor captures a single point in time with:
/// - ELP tensor coordinates in geometric space
/// - Voice characteristics (pitch, loudness, etc.)
/// - Confidence metrics
/// - Temporal metadata
///
/// # Examples
///
/// ```
/// use spatial_vortex::voice_pipeline::{BeadTensor, ELPTensor};
/// use chrono::Utc;
///
/// let elp = ELPTensor::new(8.0, 10.0, 6.0);
/// let bead = BeadTensor::new(elp, 200.0, -20.0, 0.85);
///
/// println!("Pitch: {} Hz", bead.pitch_hz);
/// println!("Confidence: {:.2}", bead.confidence);
/// ```
#[derive(Debug, Clone)]
pub struct BeadTensor {
    /// Timestamp when this bead was created
    pub timestamp: DateTime<Utc>,
    
    /// ELP tensor coordinates in 13-scale geometric space
    pub elp_values: ELPTensor,
    
    /// Fundamental frequency (pitch) in Hz
    pub pitch_hz: f64,
    
    /// Loudness in dB
    pub loudness_db: f64,
    
    /// Confidence in the ELP mapping (0.0 to 1.0)
    pub confidence: f64,
    
    /// Confidence width (uncertainty range)
    pub confidence_width: f64,
    
    /// Curviness (signed change rate in pitch)
    pub curviness_signed: f64,
}

impl BeadTensor {
    /// Creates a new BeadTensor with current timestamp
    ///
    /// # Arguments
    ///
    /// * `elp_values` - ELP tensor coordinates
    /// * `pitch_hz` - Fundamental frequency in Hz
    /// * `loudness_db` - Loudness in dB
    /// * `confidence` - Mapping confidence (0.0-1.0)
    pub fn new(
        elp_values: ELPTensor,
        pitch_hz: f64,
        loudness_db: f64,
        confidence: f64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            elp_values,
            pitch_hz,
            loudness_db,
            confidence,
            confidence_width: Self::compute_confidence_width(confidence),
            curviness_signed: 0.0, // Will be computed relative to previous bead
        }
    }
    
    /// Creates a BeadTensor from spectral features
    ///
    /// This is the primary constructor that converts raw voice features
    /// into a complete BeadTensor.
    pub fn from_features(
        elp_values: ELPTensor,
        features: &SpectralFeatures,
        confidence: f64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            elp_values,
            pitch_hz: features.pitch,
            loudness_db: features.loudness,
            confidence,
            confidence_width: Self::compute_confidence_width(confidence),
            curviness_signed: 0.0,
        }
    }
    
    /// Computes confidence width from confidence score
    ///
    /// Lower confidence = wider uncertainty range
    fn compute_confidence_width(confidence: f64) -> f64 {
        // Inverse relationship: high confidence = narrow width
        (1.0 - confidence.clamp(0.0, 1.0)) * 5.0
    }
    
    /// Updates curviness based on previous bead
    ///
    /// Curviness represents the rate of pitch change over time:
    /// - Positive: Rising pitch
    /// - Negative: Falling pitch
    /// - Near zero: Stable pitch
    pub fn update_curviness(&mut self, previous: &BeadTensor) {
        let time_delta = (self.timestamp - previous.timestamp)
            .num_milliseconds() as f64 / 1000.0; // Convert to seconds
        
        if time_delta > 0.0 {
            let pitch_delta = self.pitch_hz - previous.pitch_hz;
            self.curviness_signed = pitch_delta / time_delta; // Hz per second
        }
    }
    
    /// Returns true if this bead represents a high-value moment
    ///
    /// High-value moments have:
    /// - High confidence (>0.8)
    /// - Strong ELP magnitude
    /// - Clear voice characteristics
    pub fn is_high_value(&self) -> bool {
        self.confidence > 0.8 && self.elp_values.magnitude() > 10.0
    }
    
    /// Computes distance to another bead in ELP space
    pub fn distance_to(&self, other: &BeadTensor) -> f64 {
        let dx = self.elp_values.ethos - other.elp_values.ethos;
        let dy = self.elp_values.logos - other.elp_values.logos;
        let dz = self.elp_values.pathos - other.elp_values.pathos;
        
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// A sequence of BeadTensors representing a time series of voice input
#[derive(Debug, Clone)]
pub struct BeadSequence {
    beads: Vec<BeadTensor>,
    max_length: usize,
}

impl BeadSequence {
    /// Creates a new bead sequence with maximum length
    pub fn new(max_length: usize) -> Self {
        Self {
            beads: Vec::with_capacity(max_length),
            max_length,
        }
    }
    
    /// Adds a bead to the sequence
    ///
    /// Automatically updates curviness and maintains max length by
    /// removing oldest beads if needed.
    pub fn push(&mut self, mut bead: BeadTensor) {
        // Update curviness if we have a previous bead
        if let Some(previous) = self.beads.last() {
            bead.update_curviness(previous);
        }
        
        self.beads.push(bead);
        
        // Maintain max length
        if self.beads.len() > self.max_length {
            self.beads.remove(0);
        }
    }
    
    /// Returns all beads in the sequence
    pub fn beads(&self) -> &[BeadTensor] {
        &self.beads
    }
    
    /// Returns the most recent bead
    pub fn latest(&self) -> Option<&BeadTensor> {
        self.beads.last()
    }
    
    /// Returns high-value beads only
    pub fn high_value_beads(&self) -> Vec<&BeadTensor> {
        self.beads.iter().filter(|b| b.is_high_value()).collect()
    }
    
    /// Computes average ELP over the sequence
    pub fn average_elp(&self) -> Option<ELPTensor> {
        if self.beads.is_empty() {
            return None;
        }
        
        let sum_e: f64 = self.beads.iter().map(|b| b.elp_values.ethos).sum();
        let sum_l: f64 = self.beads.iter().map(|b| b.elp_values.logos).sum();
        let sum_p: f64 = self.beads.iter().map(|b| b.elp_values.pathos).sum();
        
        let count = self.beads.len() as f64;
        
        Some(ELPTensor::new(
            sum_e / count,
            sum_l / count,
            sum_p / count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    
    #[test]
    fn test_bead_tensor_creation() {
        let elp = ELPTensor::new(8.0, 10.0, 6.0);
        let bead = BeadTensor::new(elp, 200.0, -20.0, 0.85);
        
        assert_eq!(bead.pitch_hz, 200.0);
        assert_eq!(bead.loudness_db, -20.0);
        assert_eq!(bead.confidence, 0.85);
        assert!(bead.confidence_width > 0.0);
    }
    
    #[test]
    fn test_confidence_width() {
        // High confidence = narrow width
        let high_conf = BeadTensor::new(ELPTensor::new(1.0, 2.0, 3.0), 200.0, -20.0, 0.95);
        assert!(high_conf.confidence_width < 1.0);
        
        // Low confidence = wide width
        let low_conf = BeadTensor::new(ELPTensor::new(1.0, 2.0, 3.0), 200.0, -20.0, 0.3);
        assert!(low_conf.confidence_width > 3.0);
    }
    
    #[test]
    fn test_curviness_calculation() {
        let elp1 = ELPTensor::new(1.0, 2.0, 3.0);
        let elp2 = ELPTensor::new(1.0, 2.0, 3.0);
        
        let bead1 = BeadTensor::new(elp1, 200.0, -20.0, 0.8);
        
        sleep(Duration::from_millis(100));
        
        let mut bead2 = BeadTensor::new(elp2, 250.0, -20.0, 0.8);
        bead2.update_curviness(&bead1);
        
        // Pitch increased, so curviness should be positive
        assert!(bead2.curviness_signed > 0.0);
    }
    
    #[test]
    fn test_high_value_detection() {
        let high_mag = ELPTensor::new(12.0, 11.0, 10.0);
        let low_mag = ELPTensor::new(2.0, 3.0, 1.0);
        
        let high_value = BeadTensor::new(high_mag, 200.0, -20.0, 0.9);
        let low_value = BeadTensor::new(low_mag, 200.0, -20.0, 0.9);
        
        assert!(high_value.is_high_value());
        assert!(!low_value.is_high_value());
    }
    
    #[test]
    fn test_distance_calculation() {
        let bead1 = BeadTensor::new(ELPTensor::new(0.0, 0.0, 0.0), 200.0, -20.0, 0.8);
        let bead2 = BeadTensor::new(ELPTensor::new(3.0, 4.0, 0.0), 200.0, -20.0, 0.8);
        
        let distance = bead1.distance_to(&bead2);
        assert!((distance - 5.0).abs() < 0.001); // 3-4-5 triangle
    }
    
    #[test]
    fn test_bead_sequence() {
        let mut sequence = BeadSequence::new(10);
        
        for i in 0..5 {
            let elp = ELPTensor::new(i as f64, i as f64, i as f64);
            let bead = BeadTensor::new(elp, 200.0 + i as f64, -20.0, 0.8);
            sequence.push(bead);
        }
        
        assert_eq!(sequence.beads().len(), 5);
        assert!(sequence.latest().is_some());
    }
    
    #[test]
    fn test_sequence_max_length() {
        let mut sequence = BeadSequence::new(3);
        
        for i in 0..5 {
            let elp = ELPTensor::new(i as f64, 0.0, 0.0);
            let bead = BeadTensor::new(elp, 200.0, -20.0, 0.8);
            sequence.push(bead);
        }
        
        // Should only keep last 3
        assert_eq!(sequence.beads().len(), 3);
    }
    
    #[test]
    fn test_average_elp() {
        let mut sequence = BeadSequence::new(10);
        
        sequence.push(BeadTensor::new(ELPTensor::new(10.0, 0.0, 0.0), 200.0, -20.0, 0.8));
        sequence.push(BeadTensor::new(ELPTensor::new(0.0, 10.0, 0.0), 200.0, -20.0, 0.8));
        sequence.push(BeadTensor::new(ELPTensor::new(0.0, 0.0, 10.0), 200.0, -20.0, 0.8));
        
        let avg = sequence.average_elp().unwrap();
        assert!((avg.ethos - 10.0/3.0).abs() < 0.001);
        assert!((avg.logos - 10.0/3.0).abs() < 0.001);
        assert!((avg.pathos - 10.0/3.0).abs() < 0.001);
    }
}
