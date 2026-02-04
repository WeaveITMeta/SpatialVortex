/// Aspect-Based Color System (Semantic Meaning via Hexagonal Color Wheel)
/// 
/// **Core Principle**: Colors represent subject matter MEANING, not geometric position
/// 
/// Architecture:
/// - **Ascendency**: Colors reach unity in divine light (white at apex)
/// - **Hexagonal Descent**: Major colors with layered variations
/// - **Intention**: Key that binds similarities/differences between subjects
/// - **Variance**: Allows inference engine aspect/subject selection
/// 
/// Relations:
/// - Subjects → Aspects = Flux matrices with intention
/// - Aspects → Subjects = Colors with intention
/// - Nodes inherit object colors during processing (no static colors)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGBA color (0.0 - 1.0 range) in hexagonal Color Wheel space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AspectColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    
    /// Height in hexagonal space (0.0=pure color, 1.0=white/divine light)
    pub luminance: f32,
    
    /// Angle on color wheel (0-360 degrees)
    pub hue: f32,
    
    /// Color saturation (0.0=gray, 1.0=pure color)
    pub saturation: f32,
}

impl AspectColor {
    /// Blue color constant
    pub const Blue: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
        luminance: 0.5,
        hue: 240.0,
        saturation: 1.0,
    };

    /// Create new aspect color from RGB
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        let (h, s, l) = Self::rgb_to_hsl(r, g, b);
        
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
            luminance: l,
            hue: h,
            saturation: s,
        }
    }
    
    /// Create from semantic meaning (maps meaning to color wheel position)
    pub fn from_meaning(meaning: &str) -> Self {
        // Hash meaning to hue (0-360)
        let hue = Self::semantic_hash(meaning);
        
        // Default saturation and luminance
        let saturation = 0.8;  // Vibrant colors
        let luminance = 0.5;   // Mid-range brightness
        
        Self::from_hsl(hue, saturation, luminance)
    }
    
    /// Create from HSL (Hue, Saturation, Luminance)
    pub fn from_hsl(hue: f32, saturation: f32, luminance: f32) -> Self {
        let (r, g, b) = Self::hsl_to_rgb(hue, saturation, luminance);
        
        Self {
            r,
            g,
            b,
            a: 1.0,
            luminance,
            hue: hue % 360.0,
            saturation: saturation.clamp(0.0, 1.0),
        }
    }
    
    /// Create from BrickColor ID
    pub fn from_brick_color_id(id: u16) -> Self {
        // Map common BrickColor IDs to colors
        let (r, g, b) = match id {
            1 => (0.95, 0.95, 0.95),     // White (divine light)
            21 => (0.77, 0.16, 0.16),    // Bright red
            23 => (0.13, 0.50, 0.95),    // Bright blue
            28 => (0.13, 0.54, 0.13),    // Dark green
            24 => (1.0, 0.84, 0.0),      // Bright yellow
            104 => (0.43, 0.18, 0.78),   // Bright violet
            106 => (0.98, 0.50, 0.20),   // Bright orange
            330 => (0.82, 0.0, 0.31),    // Crimson
            _ => {
                // Generate from ID hash
                let hue = (id as f32 * 137.508) % 360.0;  // Golden angle
                return Self::from_hsl(hue, 0.7, 0.5);
            }
        };
        
        Self::new(r, g, b, 1.0)
    }
    
    /// Hash semantic meaning to hue angle (0-360)
    fn semantic_hash(meaning: &str) -> f32 {
        let mut hash: u64 = 5381;
        for byte in meaning.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        // Map to 0-360 range using golden angle for good distribution
        (hash as f32 * 137.508) % 360.0
    }
    
    /// Convert RGB to HSL
    fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        
        // Luminance
        let l = (max + min) / 2.0;
        
        // Saturation
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };
        
        // Hue
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        
        let h = if h < 0.0 { h + 360.0 } else { h };
        
        (h, s, l)
    }
    
    /// Convert HSL to RGB
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = l - c / 2.0;
        
        let (r1, g1, b1) = if h_prime < 1.0 {
            (c, x, 0.0)
        } else if h_prime < 2.0 {
            (x, c, 0.0)
        } else if h_prime < 3.0 {
            (0.0, c, x)
        } else if h_prime < 4.0 {
            (0.0, x, c)
        } else if h_prime < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        (r1 + m, g1 + m, b1 + m)
    }
    
    /// Calculate color distance (semantic proximity)
    pub fn distance(&self, other: &Self) -> f32 {
        // Distance in HSL space weighted by components
        let hue_diff = (self.hue - other.hue).abs();
        let hue_dist = hue_diff.min(360.0 - hue_diff) / 180.0;  // Circular distance
        
        let sat_dist = (self.saturation - other.saturation).abs();
        let lum_dist = (self.luminance - other.luminance).abs();
        
        // Weighted distance (hue most important for semantic meaning)
        (hue_dist * 0.6 + sat_dist * 0.2 + lum_dist * 0.2).sqrt()
    }
    
    /// Blend with another color (for variance)
    pub fn blend(&self, other: &Self, ratio: f32) -> Self {
        let t = ratio.clamp(0.0, 1.0);
        Self::new(
            self.r * (1.0 - t) + other.r * t,
            self.g * (1.0 - t) + other.g * t,
            self.b * (1.0 - t) + other.b * t,
            self.a * (1.0 - t) + other.a * t,
        )
    }
    
    /// Ascend toward divine light (increase luminance toward white)
    pub fn ascend(&self, amount: f32) -> Self {
        let new_lum = (self.luminance + amount).clamp(0.0, 1.0);
        Self::from_hsl(self.hue, self.saturation, new_lum)
    }
    
    /// Descend into pure color (decrease luminance)
    pub fn descend(&self, amount: f32) -> Self {
        let new_lum = (self.luminance - amount).clamp(0.0, 1.0);
        Self::from_hsl(self.hue, self.saturation, new_lum)
    }
    
    /// Get nearby colors (for similar semantic meanings)
    pub fn nearby(&self, variance: f32) -> Vec<Self> {
        let hue_variance = variance * 60.0;  // ±60° max
        
        vec![
            Self::from_hsl((self.hue - hue_variance + 360.0) % 360.0, self.saturation, self.luminance),
            Self::from_hsl((self.hue + hue_variance) % 360.0, self.saturation, self.luminance),
            Self::from_hsl(self.hue, (self.saturation - variance * 0.2).max(0.0), self.luminance),
            Self::from_hsl(self.hue, (self.saturation + variance * 0.2).min(1.0), self.luminance),
        ]
    }
    
    /// Convert to RGB tuple (0-255 range)
    pub fn to_rgb_u8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }
    
    /// Convert to RGBA tuple (0-255 range)
    pub fn to_rgba_u8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
    
    /// Get hex color string (#RRGGBB)
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", 
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8)
    }
    
    /// Get hex color string with alpha (#RRGGBBAA)
    pub fn to_hex_rgba(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", 
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8)
    }
    
    // ========================================================================
    // ML Feature Extraction (Week 1-2 Foundation)
    // ========================================================================
    
    /// Convert to ML feature vector for training/inference
    /// 
    /// Returns 6D vector: [hue_norm, saturation, luminance, r, g, b]
    /// All values normalized to [0, 1] range for ML compatibility
    /// 
    /// # Example
    /// ```
    /// use spatial_vortex::data::AspectColor;
    /// 
    /// let color = AspectColor::from_meaning("love");
    /// let features = color.to_feature_vector();
    /// assert_eq!(features.len(), 6);
    /// ```
    pub fn to_feature_vector(&self) -> Vec<f32> {
        vec![
            self.hue / 360.0,        // Normalize hue to [0, 1]
            self.saturation,         // Already [0, 1]
            self.luminance,          // Already [0, 1]
            self.r,                  // Already [0, 1]
            self.g,                  // Already [0, 1]
            self.b,                  // Already [0, 1]
        ]
    }
    
    /// Create AspectColor from ML feature vector
    /// 
    /// Reconstructs color from 6D feature vector output by ML model.
    /// Uses HSL values (first 3 components) for reconstruction.
    /// 
    /// # Arguments
    /// * `features` - 6D vector [hue_norm, sat, lum, r, g, b]
    /// 
    /// # Panics
    /// Panics if feature vector length != 6
    /// 
    /// # Example
    /// ```
    /// use spatial_vortex::data::AspectColor;
    /// 
    /// let features = vec![0.5, 0.8, 0.5, 0.0, 1.0, 1.0];
    /// let color = AspectColor::from_feature_vector(&features);
    /// assert_eq!(color.hue, 180.0);  // 0.5 * 360
    /// ```
    pub fn from_feature_vector(features: &[f32]) -> Self {
        assert_eq!(
            features.len(), 
            6, 
            "Feature vector must be 6D [hue_norm, sat, lum, r, g, b]"
        );
        
        // Reconstruct from HSL (more semantically meaningful than RGB)
        let hue = features[0] * 360.0;
        let saturation = features[1].clamp(0.0, 1.0);
        let luminance = features[2].clamp(0.0, 1.0);
        
        Self::from_hsl(hue, saturation, luminance)
    }
    
    /// Convert to normalized feature vector with additional semantic features
    /// 
    /// Returns 10D extended vector for advanced ML models:
    /// [hue_norm, sat, lum, r, g, b, hue_sin, hue_cos, chroma, perceived_brightness]
    /// 
    /// Extra features:
    /// - hue_sin/cos: Circular encoding of hue (better for ML)
    /// - chroma: Color purity
    /// - perceived_brightness: Human perception-weighted luminance
    pub fn to_extended_feature_vector(&self) -> Vec<f32> {
        let hue_rad = self.hue.to_radians();
        let chroma = self.saturation * (1.0 - (2.0 * self.luminance - 1.0).abs());
        
        // Perceived brightness using standard weights
        let perceived = 0.299 * self.r + 0.587 * self.g + 0.114 * self.b;
        
        vec![
            self.hue / 360.0,
            self.saturation,
            self.luminance,
            self.r,
            self.g,
            self.b,
            hue_rad.sin(),           // Circular hue encoding
            hue_rad.cos(),           // Circular hue encoding
            chroma,                  // Color purity
            perceived,               // Perceived brightness
        ]
    }
}

/// Aspect orientation - the semantic meaning of a subject with intentional color
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AspectOrientation {
    /// Primary semantic meaning (e.g., "love", "logic", "courage", "mystery")
    pub meaning: String,
    
    /// Color representing this meaning in hexagonal color wheel
    pub color: AspectColor,
    
    /// Semantic variance (0.0-1.0, how flexible the meaning is)
    pub variance: f32,
    
    /// Related aspects (semantically similar, nearby in color space)
    pub related_aspects: Vec<String>,
    
    /// Intention strength (how strongly this aspect defines the subject)
    pub intention: f32,
}

impl AspectOrientation {
    /// Create aspect from semantic meaning
    pub fn from_meaning(meaning: &str, variance: f32) -> Self {
        let color = AspectColor::from_meaning(meaning);
        
        Self {
            meaning: meaning.to_string(),
            color,
            variance: variance.clamp(0.0, 1.0),
            related_aspects: Vec::new(),
            intention: 1.0,
        }
    }
    
    /// Create with specific color (for explicit color assignments)
    pub fn with_color(meaning: &str, color: AspectColor, variance: f32) -> Self {
        Self {
            meaning: meaning.to_string(),
            color,
            variance: variance.clamp(0.0, 1.0),
            related_aspects: Vec::new(),
            intention: 1.0,
        }
    }
    
    /// Find nearby aspects by color similarity (inference engine helper)
    pub fn find_similar(&self, candidates: &[AspectOrientation], max_distance: f32) -> Vec<String> {
        candidates
            .iter()
            .filter(|a| {
                let dist = self.color.distance(&a.color);
                dist <= max_distance && a.meaning != self.meaning
            })
            .map(|a| a.meaning.clone())
            .collect()
    }
    
    /// Set related aspects (similar meanings)
    pub fn with_related(mut self, related: Vec<String>) -> Self {
        self.related_aspects = related;
        self
    }
    
    /// Set intention strength
    pub fn with_intention(mut self, intention: f32) -> Self {
        self.intention = intention.clamp(0.0, 1.0);
        self
    }
    
    /// Check if this aspect is similar to another (within variance)
    pub fn is_similar_to(&self, other: &AspectOrientation) -> bool {
        let distance = self.color.distance(&other.color);
        distance <= (self.variance + other.variance) / 2.0
    }
}

/// Semantic color space manager (Hexagonal Color Wheel based)
pub struct SemanticColorSpace {
    /// Known aspects by meaning
    aspects: HashMap<String, AspectOrientation>,
    
    /// Divine light (white, unity at apex)
    divine_light: AspectColor,
}

impl SemanticColorSpace {
    /// Create new semantic color space
    pub fn new() -> Self {
        Self {
            aspects: HashMap::new(),
            divine_light: AspectColor::new(1.0, 1.0, 1.0, 1.0),
        }
    }
    
    /// Register an aspect with its semantic meaning
    pub fn register_aspect(&mut self, aspect: AspectOrientation) {
        self.aspects.insert(aspect.meaning.clone(), aspect);
    }
    
    /// Get or create aspect from meaning
    pub fn get_or_create_aspect(&mut self, meaning: &str, variance: f32) -> AspectOrientation {
        if let Some(aspect) = self.aspects.get(meaning) {
            aspect.clone()
        } else {
            let aspect = AspectOrientation::from_meaning(meaning, variance);
            self.aspects.insert(meaning.to_string(), aspect.clone());
            aspect
        }
    }
    
    /// Find aspects by color proximity (inference engine query)
    pub fn find_by_color(&self, target_color: &AspectColor, max_distance: f32) -> Vec<AspectOrientation> {
        self.aspects
            .values()
            .filter(|a| a.color.distance(target_color) <= max_distance)
            .cloned()
            .collect()
    }
    
    /// Find aspects by semantic similarity (related aspects)
    pub fn find_related(&self, meaning: &str) -> Vec<AspectOrientation> {
        if let Some(aspect) = self.aspects.get(meaning) {
            aspect.related_aspects
                .iter()
                .filter_map(|m| self.aspects.get(m).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get divine light color (unity at apex)
    pub fn divine_light(&self) -> AspectColor {
        self.divine_light
    }
    
    /// List all registered aspects
    pub fn list_aspects(&self) -> Vec<&AspectOrientation> {
        self.aspects.values().collect()
    }
}

impl Default for SemanticColorSpace {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ML Training Data (Week 1-2 Foundation)
// ============================================================================

/// Training sample for aspect color ML model
/// 
/// Associates semantic meanings with colors and their relationships
/// for supervised learning of color-meaning mappings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectTrainingData {
    /// Primary semantic meaning
    pub meaning: String,
    
    /// Color associated with this meaning
    pub color: AspectColor,
    
    /// Related meanings (semantically similar)
    pub related_meanings: Vec<String>,
    
    /// Semantic distance to related meanings (0.0-1.0)
    /// Lower = more similar
    pub semantic_distances: HashMap<String, f32>,
    
    /// Context where this meaning appears (for contextual learning)
    pub context: Option<String>,
    
    /// Training weight (importance of this sample)
    pub weight: f32,
}

impl AspectTrainingData {
    /// Create new training sample
    pub fn new(meaning: String, color: AspectColor) -> Self {
        Self {
            meaning,
            color,
            related_meanings: Vec::new(),
            semantic_distances: HashMap::new(),
            context: None,
            weight: 1.0,
        }
    }
    
    /// Add related meaning with distance
    pub fn add_related(mut self, meaning: String, distance: f32) -> Self {
        self.related_meanings.push(meaning.clone());
        self.semantic_distances.insert(meaning, distance.clamp(0.0, 1.0));
        self
    }
    
    /// Set context
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Set training weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 10.0);
        self
    }
    
    /// Convert to feature vector pair (input, target)
    /// Returns (meaning_features, color_features)
    pub fn to_training_pair(&self) -> (Vec<f32>, Vec<f32>) {
        // For now, color features as both input and target
        // In advanced models, meaning would be embedded separately
        let color_features = self.color.to_feature_vector();
        (color_features.clone(), color_features)
    }
    
    /// Create training batch from multiple samples
    pub fn create_batch(samples: &[AspectTrainingData]) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
        let mut inputs = Vec::new();
        let mut targets = Vec::new();
        
        for sample in samples {
            let (input, target) = sample.to_training_pair();
            inputs.push(input);
            targets.push(target);
        }
        
        (inputs, targets)
    }
}

/// Training dataset builder for aspect colors
pub struct AspectColorDataset {
    samples: Vec<AspectTrainingData>,
    #[allow(dead_code)]
    color_space: SemanticColorSpace,
}

impl AspectColorDataset {
    /// Create new dataset
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            color_space: SemanticColorSpace::new(),
        }
    }
    
    /// Add training sample
    pub fn add_sample(&mut self, sample: AspectTrainingData) {
        self.samples.push(sample);
    }
    
    /// Generate synthetic training data from color space
    pub fn generate_from_space(&mut self, space: &SemanticColorSpace, count: usize) {
        let aspects = space.list_aspects();
        
        for _ in 0..count {
            if let Some(aspect) = aspects.get(rand::random::<usize>() % aspects.len()) {
                let sample = AspectTrainingData::new(
                    aspect.meaning.clone(),
                    aspect.color,
                )
                .with_weight(aspect.intention);
                
                // Add related aspects
                let sample = aspect.related_aspects.iter().fold(sample, |s, related| {
                    s.add_related(related.clone(), 0.3)
                });
                
                self.add_sample(sample);
            }
        }
    }
    
    /// Get all samples
    pub fn samples(&self) -> &[AspectTrainingData] {
        &self.samples
    }
    
    /// Split into train/validation sets
    pub fn train_val_split(&self, train_ratio: f32) -> (Vec<AspectTrainingData>, Vec<AspectTrainingData>) {
        let split_idx = (self.samples.len() as f32 * train_ratio) as usize;
        let mut samples = self.samples.clone();
        
        // Shuffle samples
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        samples.shuffle(&mut rng);
        
        let (train, val) = samples.split_at(split_idx);
        (train.to_vec(), val.to_vec())
    }
}

impl Default for AspectColorDataset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_from_meaning() {
        let color1 = AspectColor::from_meaning("love");
        let color2 = AspectColor::from_meaning("love");
        
        // Same meaning = same color
        assert_eq!(color1.hue, color2.hue);
        
        // Different meanings = different colors
        let color3 = AspectColor::from_meaning("hate");
        assert_ne!(color1.hue, color3.hue);
    }
    
    #[test]
    fn test_aspect_similarity() {
        let love = AspectOrientation::from_meaning("love", 0.2);
        let affection = AspectOrientation::from_meaning("affection", 0.2);
        
        // Similar concepts should have nearby colors
        let distance = love.color.distance(&affection.color);
        assert!(distance < 2.0, "Similar concepts should have nearby colors");
    }
    
    #[test]
    fn test_divine_ascendency() {
        let color = AspectColor::from_meaning("hope");
        let ascended = color.ascend(0.3);
        
        // Ascending increases luminance (toward white/divine)
        assert!(ascended.luminance > color.luminance);
    }
    
    #[test]
    fn test_semantic_space() {
        let mut space = SemanticColorSpace::new();
        
        let love = AspectOrientation::from_meaning("love", 0.15);
        space.register_aspect(love.clone());
        
        let found = space.get_or_create_aspect("love", 0.15);
        assert_eq!(found.meaning, "love");
    }
    
    #[test]
    fn test_nearby_colors() {
        let color = AspectColor::from_meaning("joy");
        let nearby = color.nearby(0.2);
        
        // Should return multiple nearby colors
        assert!(nearby.len() > 0);
        
        // All should be relatively close
        for n in nearby {
            assert!(color.distance(&n) < 0.5);
        }
    }
    
    // ========================================================================
    // ML Feature Tests (Week 1-2)
    // ========================================================================
    
    #[test]
    fn test_to_feature_vector() {
        let color = AspectColor::from_meaning("love");
        let features = color.to_feature_vector();
        
        // Should be 6D vector
        assert_eq!(features.len(), 6);
        
        // All values should be in [0, 1] range
        for f in &features {
            assert!(*f >= 0.0 && *f <= 1.0, "Feature {} out of range", f);
        }
    }
    
    #[test]
    fn test_from_feature_vector() {
        let original = AspectColor::from_meaning("courage");
        let features = original.to_feature_vector();
        let reconstructed = AspectColor::from_feature_vector(&features);
        
        // Reconstructed should match original (within tolerance)
        assert!((original.hue - reconstructed.hue).abs() < 1.0);
        assert!((original.saturation - reconstructed.saturation).abs() < 0.01);
        assert!((original.luminance - reconstructed.luminance).abs() < 0.01);
    }
    
    #[test]
    fn test_extended_feature_vector() {
        let color = AspectColor::from_meaning("wisdom");
        let features = color.to_extended_feature_vector();
        
        // Should be 10D vector
        assert_eq!(features.len(), 10);
        
        // Check circular hue encoding
        let hue_sin = features[6];
        let hue_cos = features[7];
        let magnitude = (hue_sin.powi(2) + hue_cos.powi(2)).sqrt();
        assert!((magnitude - 1.0).abs() < 0.01, "Circular encoding should have unit magnitude");
    }
    
    #[test]
    fn test_aspect_training_data() {
        let color = AspectColor::from_meaning("love");
        let data = AspectTrainingData::new("love".to_string(), color)
            .add_related("affection".to_string(), 0.2)
            .add_related("passion".to_string(), 0.3)
            .with_weight(1.5);
        
        assert_eq!(data.meaning, "love");
        assert_eq!(data.related_meanings.len(), 2);
        assert_eq!(data.weight, 1.5);
        
        // Check semantic distances
        assert_eq!(data.semantic_distances.get("affection"), Some(&0.2));
        assert_eq!(data.semantic_distances.get("passion"), Some(&0.3));
    }
    
    #[test]
    fn test_training_batch_creation() {
        let samples = vec![
            AspectTrainingData::new("love".to_string(), AspectColor::from_meaning("love")),
            AspectTrainingData::new("joy".to_string(), AspectColor::from_meaning("joy")),
            AspectTrainingData::new("peace".to_string(), AspectColor::from_meaning("peace")),
        ];
        
        let (inputs, targets) = AspectTrainingData::create_batch(&samples);
        
        assert_eq!(inputs.len(), 3);
        assert_eq!(targets.len(), 3);
        
        // Each should be 6D feature vector
        for input in &inputs {
            assert_eq!(input.len(), 6);
        }
        for target in &targets {
            assert_eq!(target.len(), 6);
        }
    }
    
    #[test]
    fn test_aspect_color_dataset() {
        let mut dataset = AspectColorDataset::new();
        
        // Add samples
        dataset.add_sample(AspectTrainingData::new(
            "love".to_string(),
            AspectColor::from_meaning("love"),
        ));
        dataset.add_sample(AspectTrainingData::new(
            "joy".to_string(),
            AspectColor::from_meaning("joy"),
        ));
        
        assert_eq!(dataset.samples().len(), 2);
    }
    
    #[test]
    fn test_train_val_split() {
        let mut dataset = AspectColorDataset::new();
        
        // Add multiple samples
        for i in 0..10 {
            let meaning = format!("meaning_{}", i);
            dataset.add_sample(AspectTrainingData::new(
                meaning.clone(),
                AspectColor::from_meaning(&meaning),
            ));
        }
        
        let (train, val) = dataset.train_val_split(0.8);
        
        // Should split 80/20
        assert_eq!(train.len(), 8);
        assert_eq!(val.len(), 2);
        
        // Total should equal original
        assert_eq!(train.len() + val.len(), 10);
    }
    
    #[test]
    fn test_feature_vector_roundtrip() {
        // Test multiple colors
        let meanings = vec!["love", "hate", "joy", "sorrow", "courage", "fear"];
        
        for meaning in meanings {
            let original = AspectColor::from_meaning(meaning);
            let features = original.to_feature_vector();
            let reconstructed = AspectColor::from_feature_vector(&features);
            
            // HSL should roundtrip accurately
            let hsl_error = (original.hue - reconstructed.hue).abs() +
                           (original.saturation - reconstructed.saturation).abs() +
                           (original.luminance - reconstructed.luminance).abs();
            
            assert!(hsl_error < 2.0, "Roundtrip error too large for {}: {}", meaning, hsl_error);
        }
    }
}
