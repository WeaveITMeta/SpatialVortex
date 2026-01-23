//! FFT-based spectral analysis for voice feature extraction
//!
//! Extracts pitch, spectral features, and characteristics from audio
//! using Fast Fourier Transform.

use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

/// Spectral features extracted from audio
#[derive(Debug, Clone)]
pub struct SpectralFeatures {
    /// Fundamental frequency (pitch) in Hz
    pub pitch: f64,
    /// Spectral centroid (brightness)
    pub spectral_centroid: f64,
    /// Spectral flux (change over time)
    pub spectral_flux: f64,
    /// Loudness in dB
    pub loudness: f64,
    /// Spectral complexity (0=tonal, 1=noisy)
    pub spectral_complexity: f64,
}

/// FFT-based spectral analyzer for voice
///
/// Performs Fast Fourier Transform on audio signals to extract
/// fundamental frequency (pitch), spectral features, and voice characteristics.
///
/// # Examples
///
/// ```
/// use spatial_vortex::voice_pipeline::SpectralAnalyzer;
///
/// let mut analyzer = SpectralAnalyzer::new(44100);
/// 
/// // Analyze audio chunk
/// let audio: Vec<f32> = vec![0.0; 1024]; // Mock audio data
/// let features = analyzer.analyze(&audio);
/// 
/// println!("Pitch: {} Hz", features.pitch);
/// println!("Loudness: {} dB", features.loudness);
/// ```
pub struct SpectralAnalyzer {
    planner: FftPlanner<f32>,
    sample_rate: u32,
    window: Vec<f32>,
    previous_spectrum: Vec<f64>,
}

impl SpectralAnalyzer {
    /// Creates a new spectral analyzer
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100)
    pub fn new(sample_rate: u32) -> Self {
        Self {
            planner: FftPlanner::new(),
            sample_rate,
            window: Vec::new(),
            previous_spectrum: Vec::new(),
        }
    }
    
    /// Analyzes audio chunk and extracts spectral features
    ///
    /// # Arguments
    ///
    /// * `audio` - Audio samples (mono, f32)
    ///
    /// # Returns
    ///
    /// * `SpectralFeatures` - Extracted features
    ///
    /// # Panics
    ///
    /// Panics if audio buffer is empty
    pub fn analyze(&mut self, audio: &[f32]) -> SpectralFeatures {
        assert!(!audio.is_empty(), "Audio buffer cannot be empty");
        
        // Apply Hann window to reduce spectral leakage
        let windowed = self.apply_hann_window(audio);
        
        // Convert to complex
        let mut buffer: Vec<Complex<f32>> = windowed
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        
        // Perform FFT
        let fft = self.planner.plan_fft_forward(buffer.len());
        fft.process(&mut buffer);
        
        // Compute magnitude spectrum
        let magnitudes: Vec<f64> = buffer
            .iter()
            .take(buffer.len() / 2) // Only need first half (Nyquist)
            .map(|c| (c.re * c.re + c.im * c.im).sqrt() as f64)
            .collect();
        
        // Extract features
        let pitch = self.extract_fundamental(&magnitudes);
        let spectral_centroid = self.compute_centroid(&magnitudes);
        let spectral_flux = self.compute_flux(&magnitudes);
        let loudness = self.compute_loudness(&magnitudes);
        let spectral_complexity = self.compute_complexity(&magnitudes);
        
        // Store for next flux calculation
        self.previous_spectrum = magnitudes;
        
        SpectralFeatures {
            pitch,
            spectral_centroid,
            spectral_flux,
            loudness,
            spectral_complexity,
        }
    }
    
    /// Applies Hann window to reduce spectral leakage
    ///
    /// Hann window: w(n) = 0.5 * (1 - cos(2πn/N))
    fn apply_hann_window(&mut self, audio: &[f32]) -> Vec<f32> {
        // Generate window if needed or size changed
        if self.window.len() != audio.len() {
            self.window = (0..audio.len())
                .map(|n| {
                    0.5 * (1.0 - (2.0 * PI * n as f32 / audio.len() as f32).cos())
                })
                .collect();
        }
        
        audio
            .iter()
            .zip(&self.window)
            .map(|(a, w)| a * w)
            .collect()
    }
    
    /// Extracts fundamental frequency (pitch) using peak detection
    ///
    /// Searches for the peak magnitude in the 80-400 Hz range,
    /// which covers typical human voice fundamental.
    fn extract_fundamental(&self, magnitudes: &[f64]) -> f64 {
        let start_hz = 80.0;
        let end_hz = 400.0;
        
        let start_bin = (start_hz * magnitudes.len() as f64 
            / self.sample_rate as f64) as usize;
        let end_bin = (end_hz * magnitudes.len() as f64 
            / self.sample_rate as f64) as usize;
        
        let peak_bin = magnitudes[start_bin..end_bin.min(magnitudes.len())]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i + start_bin)
            .unwrap_or(start_bin);
        
        peak_bin as f64 * self.sample_rate as f64 / (magnitudes.len() * 2) as f64
    }
    
    /// Computes spectral centroid (center of mass of spectrum)
    fn compute_centroid(&self, magnitudes: &[f64]) -> f64 {
        let weighted_sum: f64 = magnitudes
            .iter()
            .enumerate()
            .map(|(i, &mag)| i as f64 * mag)
            .sum();
        
        let sum: f64 = magnitudes.iter().sum();
        
        if sum > 0.0 {
            weighted_sum / sum
        } else {
            0.0
        }
    }
    
    /// Computes spectral flux (change in spectrum over time)
    fn compute_flux(&self, magnitudes: &[f64]) -> f64 {
        if self.previous_spectrum.is_empty() {
            return 0.0;
        }
        
        let min_len = magnitudes.len().min(self.previous_spectrum.len());
        
        magnitudes[..min_len]
            .iter()
            .zip(&self.previous_spectrum[..min_len])
            .map(|(curr, prev)| (curr - prev).powi(2))
            .sum::<f64>()
            .sqrt()
    }
    
    /// Computes loudness in dB scale
    fn compute_loudness(&self, magnitudes: &[f64]) -> f64 {
        let rms: f64 = magnitudes
            .iter()
            .map(|m| m * m)
            .sum::<f64>()
            .sqrt();
        
        if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            -100.0 // Very quiet
        }
    }
    
    /// Computes spectral complexity (flatness)
    ///
    /// Spectral flatness = geometric mean / arithmetic mean
    /// 0 = tonal (single frequency), 1 = noisy (white noise)
    fn compute_complexity(&self, magnitudes: &[f64]) -> f64 {
        let geometric_mean = magnitudes
            .iter()
            .filter(|&&m| m > 0.0)
            .map(|&m| m.ln())
            .sum::<f64>() / magnitudes.len() as f64;
        
        let arithmetic_mean = magnitudes.iter().sum::<f64>() 
            / magnitudes.len() as f64;
        
        if arithmetic_mean > 0.0 {
            (geometric_mean.exp() / arithmetic_mean).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    
    /// Generate a sine wave at specific frequency
    fn generate_sine_wave(freq: f32, sample_rate: u32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| {
                (2.0 * PI * freq * i as f32 / sample_rate as f32).sin()
            })
            .collect()
    }
    
    #[test]
    fn test_spectral_analyzer_creation() {
        let analyzer = SpectralAnalyzer::new(44100);
        assert_eq!(analyzer.sample_rate, 44100);
    }
    
    #[test]
    fn test_440hz_detection() {
        let mut analyzer = SpectralAnalyzer::new(44100);
        
        // Generate 440 Hz sine wave (A4 note)
        let audio = generate_sine_wave(440.0, 44100, 1024);
        
        let features = analyzer.analyze(&audio);
        
        // Should detect ~440 Hz pitch (allow 10 Hz tolerance)
        assert!(
            (features.pitch - 440.0).abs() < 20.0,
            "Expected ~440 Hz, got {} Hz",
            features.pitch
        );
    }
    
    #[test]
    fn test_spectral_features_range() {
        let mut analyzer = SpectralAnalyzer::new(44100);
        let audio = generate_sine_wave(200.0, 44100, 1024);
        
        let features = analyzer.analyze(&audio);
        
        // All features should be in reasonable ranges
        assert!(features.pitch >= 0.0);
        assert!(features.spectral_centroid >= 0.0);
        assert!(features.spectral_flux >= 0.0);
        assert!(features.spectral_complexity >= 0.0 && features.spectral_complexity <= 1.0);
    }
    
    #[test]
    fn test_hann_window() {
        let mut analyzer = SpectralAnalyzer::new(44100);
        let audio = vec![1.0; 100];
        
        let windowed = analyzer.apply_hann_window(&audio);
        
        // Window should have reduced edges
        assert!(windowed[0] < 0.1); // Near zero at edges
        assert!(windowed[50] > 0.9); // Near 1.0 at center
        assert!(windowed[99] < 0.1); // Near zero at edges
    }
}
