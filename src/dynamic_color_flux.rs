/// Dynamic Color-Based Flux Matrix Generation
/// 
/// Creates flux matrices in real-time from text/voice input where colors
/// represent averaged Ethos-Logos-Pathos (ELP) scores, inspired by Roblox BrickColors.
/// Each subject matter gets a color that is the composite of all aspect-oriented
/// dimensions analyzed from the request.

use crate::models::FluxMatrix;
use crate::flux_matrix::FluxMatrixEngine;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Roblox-inspired color palette for flux matrices
/// Each color maps to specific ELP channel combinations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrickColor {
    /// BrickColor ID (1-1032 in Roblox)
    pub id: u16,
    /// Display name
    pub name: &'static str,
    /// RGB values (0.0 - 1.0)
    pub rgb: (f32, f32, f32),
    /// Dominant ELP channel
    pub dominant_channel: ELPChannel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ELPChannel {
    Ethos,   // Character/Ethics - Blue dominant
    Logos,   // Logic/Reason - Green dominant
    Pathos,  // Emotion/Passion - Red dominant
    Balanced, // Equal channels
}

/// Aspect-oriented analysis result
/// Multiple aspects combine to form the final subject color
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct AspectAnalysis {
    /// Individual aspect scores
    pub aspects: Vec<AspectScore>,
    /// Averaged ELP scores across all aspects
    pub averaged_elp: ELPScore,
    /// Derived BrickColor from averaged scores
    pub brick_color: BrickColor,
}

/// Single aspect of the input (e.g., tone, semantics, structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectScore {
    /// Aspect name (e.g., "emotional_tone", "logical_structure", "moral_stance")
    pub aspect_name: String,
    /// ELP scores for this aspect
    pub elp: ELPScore,
    /// Confidence in this aspect analysis (0.0 - 1.0)
    pub confidence: f32,
}

/// Ethos-Logos-Pathos scores (normalized 0.0 - 1.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ELPScore {
    pub ethos: f32,   // Character/Ethics dimension
    pub logos: f32,   // Logic/Reason dimension
    pub pathos: f32,  // Emotion/Passion dimension
}

impl ELPScore {
    /// Create from raw scores and normalize
    pub fn new(ethos: f32, logos: f32, pathos: f32) -> Self {
        let total = ethos + logos + pathos;
        if total > 0.0 {
            Self {
                ethos: ethos / total,
                logos: logos / total,
                pathos: pathos / total,
            }
        } else {
            Self { ethos: 0.33, logos: 0.33, pathos: 0.33 }
        }
    }

    /// Convert to RGB color (E=Blue, L=Green, P=Red)
    pub fn to_rgb(&self) -> (f32, f32, f32) {
        (
            self.pathos, // Red channel
            self.logos,  // Green channel
            self.ethos,  // Blue channel
        )
    }

    /// Get dominant channel
    pub fn dominant_channel(&self) -> ELPChannel {
        if self.ethos > self.logos && self.ethos > self.pathos {
            ELPChannel::Ethos
        } else if self.logos > self.ethos && self.logos > self.pathos {
            ELPChannel::Logos
        } else if self.pathos > self.ethos && self.pathos > self.logos {
            ELPChannel::Pathos
        } else {
            ELPChannel::Balanced
        }
    }

    /// Calculate 13-weighted scale values (per sacred geometry principle)
    pub fn to_13_scale(&self) -> (i8, i8, i8) {
        (
            ((self.ethos * 13.0) as i8).clamp(-13, 13),
            ((self.logos * 13.0) as i8).clamp(-13, 13),
            ((self.pathos * 13.0) as i8).clamp(-13, 13),
        )
    }
}

/// Dynamic flux matrix generator using color-based Machine Learning
pub struct DynamicColorFluxGenerator {
    flux_engine: FluxMatrixEngine,
    brick_palette: Vec<BrickColor>,
}

impl DynamicColorFluxGenerator {
    pub fn new() -> Self {
        Self {
            flux_engine: FluxMatrixEngine::new(),
            brick_palette: Self::initialize_brick_palette(),
        }
    }

    /// Initialize Roblox-inspired BrickColor palette
    fn initialize_brick_palette() -> Vec<BrickColor> {
        vec![
            // Ethos-dominant colors (Blue tones)
            BrickColor {
                id: 23,
                name: "Bright blue",
                rgb: (0.13, 0.50, 0.95),
                dominant_channel: ELPChannel::Ethos,
            },
            BrickColor {
                id: 11,
                name: "Really blue",
                rgb: (0.0, 0.0, 1.0),
                dominant_channel: ELPChannel::Ethos,
            },
            BrickColor {
                id: 102,
                name: "Medium blue",
                rgb: (0.43, 0.67, 0.85),
                dominant_channel: ELPChannel::Ethos,
            },
            
            // Logos-dominant colors (Green tones)
            BrickColor {
                id: 28,
                name: "Dark green",
                rgb: (0.13, 0.54, 0.13),
                dominant_channel: ELPChannel::Logos,
            },
            BrickColor {
                id: 37,
                name: "Bright green",
                rgb: (0.29, 0.59, 0.29),
                dominant_channel: ELPChannel::Logos,
            },
            BrickColor {
                id: 119,
                name: "Br. yellowish green",
                rgb: (0.74, 0.85, 0.51),
                dominant_channel: ELPChannel::Logos,
            },
            
            // Pathos-dominant colors (Red tones)
            BrickColor {
                id: 21,
                name: "Bright red",
                rgb: (0.77, 0.16, 0.16),
                dominant_channel: ELPChannel::Pathos,
            },
            BrickColor {
                id: 192,
                name: "Really red",
                rgb: (1.0, 0.0, 0.0),
                dominant_channel: ELPChannel::Pathos,
            },
            BrickColor {
                id: 330,
                name: "Crimson",
                rgb: (0.82, 0.0, 0.31),
                dominant_channel: ELPChannel::Pathos,
            },
            
            // Balanced colors
            BrickColor {
                id: 1,
                name: "White",
                rgb: (0.95, 0.95, 0.95),
                dominant_channel: ELPChannel::Balanced,
            },
            BrickColor {
                id: 194,
                name: "Medium stone grey",
                rgb: (0.64, 0.64, 0.64),
                dominant_channel: ELPChannel::Balanced,
            },
        ]
    }

    /// Analyze text/voice input across multiple aspects
    pub async fn analyze_aspects(&self, input: &str) -> Result<AspectAnalysis> {
        let mut aspects = Vec::new();

        // Aspect 1: Emotional Tone Analysis
        aspects.push(self.analyze_emotional_tone(input));

        // Aspect 2: Logical Structure Analysis
        aspects.push(self.analyze_logical_structure(input));

        // Aspect 3: Moral/Ethical Stance Analysis
        aspects.push(self.analyze_moral_stance(input));

        // Aspect 4: Semantic Complexity Analysis
        aspects.push(self.analyze_semantic_complexity(input));

        // Average all aspects to get composite ELP score
        let averaged_elp = self.average_aspects(&aspects);

        // Find best matching BrickColor
        let brick_color = self.find_closest_brick_color(&averaged_elp);

        Ok(AspectAnalysis {
            aspects,
            averaged_elp,
            brick_color,
        })
    }

    /// Analyze emotional tone of input
    fn analyze_emotional_tone(&self, input: &str) -> AspectScore {
        // Detect emotional keywords
        let emotional_words = ["love", "hate", "happy", "sad", "angry", "fear", "joy"];
        let logical_words = ["because", "therefore", "thus", "prove", "analyze"];
        let ethical_words = ["should", "must", "right", "wrong", "duty", "virtue"];

        let input_lower = input.to_lowercase();
        
        let emotion_score = emotional_words.iter()
            .filter(|w| input_lower.contains(*w))
            .count() as f32;
        
        let logic_score = logical_words.iter()
            .filter(|w| input_lower.contains(*w))
            .count() as f32;
        
        let ethics_score = ethical_words.iter()
            .filter(|w| input_lower.contains(*w))
            .count() as f32;

        AspectScore {
            aspect_name: "emotional_tone".to_string(),
            elp: ELPScore::new(ethics_score, logic_score, emotion_score),
            confidence: 0.75,
        }
    }

    /// Analyze logical structure
    fn analyze_logical_structure(&self, input: &str) -> AspectScore {
        let has_reasoning = input.contains("because") || input.contains("therefore");
        let has_questions = input.contains("?");
        let has_statements = input.contains(".");
        
        let logos = if has_reasoning { 1.0 } else { 0.3 };
        let pathos = if has_questions { 0.7 } else { 0.3 };
        let ethos = if has_statements { 0.5 } else { 0.3 };

        AspectScore {
            aspect_name: "logical_structure".to_string(),
            elp: ELPScore::new(ethos, logos, pathos),
            confidence: 0.8,
        }
    }

    /// Analyze moral/ethical stance
    fn analyze_moral_stance(&self, input: &str) -> AspectScore {
        let imperative_words = ["should", "must", "ought", "need to"];
        let input_lower = input.to_lowercase();
        
        let moral_intensity = imperative_words.iter()
            .filter(|w| input_lower.contains(*w))
            .count() as f32;

        AspectScore {
            aspect_name: "moral_stance".to_string(),
            elp: ELPScore::new(
                0.8 + moral_intensity * 0.2, // High ethos
                0.4,
                0.3,
            ),
            confidence: 0.7,
        }
    }

    /// Analyze semantic complexity
    fn analyze_semantic_complexity(&self, input: &str) -> AspectScore {
        let word_count = input.split_whitespace().count();
        let avg_word_length = input.split_whitespace()
            .map(|w| w.len())
            .sum::<usize>() as f32 / word_count.max(1) as f32;

        // Complex language = higher logos
        let complexity = (avg_word_length / 10.0).min(1.0);

        AspectScore {
            aspect_name: "semantic_complexity".to_string(),
            elp: ELPScore::new(0.3, complexity, 1.0 - complexity),
            confidence: 0.85,
        }
    }

    /// Average multiple aspect scores with confidence weighting
    fn average_aspects(&self, aspects: &[AspectScore]) -> ELPScore {
        let total_confidence: f32 = aspects.iter().map(|a| a.confidence).sum();
        
        if total_confidence == 0.0 {
            return ELPScore::new(1.0, 1.0, 1.0);
        }

        let weighted_ethos: f32 = aspects.iter()
            .map(|a| a.elp.ethos * a.confidence)
            .sum();
        let weighted_logos: f32 = aspects.iter()
            .map(|a| a.elp.logos * a.confidence)
            .sum();
        let weighted_pathos: f32 = aspects.iter()
            .map(|a| a.elp.pathos * a.confidence)
            .sum();

        ELPScore::new(
            weighted_ethos / total_confidence,
            weighted_logos / total_confidence,
            weighted_pathos / total_confidence,
        )
    }

    /// Find closest BrickColor match to ELP score
    fn find_closest_brick_color(&self, elp: &ELPScore) -> BrickColor {
        let target_rgb = elp.to_rgb();
        
        self.brick_palette.iter()
            .min_by(|a, b| {
                let dist_a = Self::color_distance(a.rgb, target_rgb);
                let dist_b = Self::color_distance(b.rgb, target_rgb);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
            .unwrap_or(self.brick_palette[0])
    }

    /// Calculate Euclidean distance between two RGB colors
    fn color_distance(rgb1: (f32, f32, f32), rgb2: (f32, f32, f32)) -> f32 {
        let dr = rgb1.0 - rgb2.0;
        let dg = rgb1.1 - rgb2.1;
        let db = rgb1.2 - rgb2.2;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    /// Generate flux matrix from text/voice input with dynamic coloring
    pub async fn generate_from_input(
        &self,
        subject: String,
        input: &str,
    ) -> Result<(FluxMatrix, AspectAnalysis)> {
        // Step 1: Analyze all aspects to get averaged color
        let analysis = self.analyze_aspects(input).await?;

        // Step 2: Create flux matrix
        let mut matrix = self.flux_engine.create_matrix(subject.clone())?;

        // Step 3: Apply color-based ELP scores to all nodes
        for node in matrix.nodes.values_mut() {
            // Use 13-scale for sacred geometry compliance
            let (e_13, l_13, p_13) = analysis.averaged_elp.to_13_scale();
            
            // Store in node attributes
            node.attributes.properties.insert(
                "elp_ethos_13".to_string(),
                (e_13 as f32).to_string(),
            );
            node.attributes.properties.insert(
                "elp_logos_13".to_string(),
                (l_13 as f32).to_string(),
            );
            node.attributes.properties.insert(
                "elp_pathos_13".to_string(),
                (p_13 as f32).to_string(),
            );
            
            // Store BrickColor info
            node.attributes.properties.insert(
                "brick_color_id".to_string(),
                (analysis.brick_color.id as f32).to_string(),
            );
            node.attributes.properties.insert(
                "brick_color_r".to_string(),
                analysis.brick_color.rgb.0.to_string(),
            );
            node.attributes.properties.insert(
                "brick_color_g".to_string(),
                analysis.brick_color.rgb.1.to_string(),
            );
            node.attributes.properties.insert(
                "brick_color_b".to_string(),
                analysis.brick_color.rgb.2.to_string(),
            );
        }

        Ok((matrix, analysis))
    }
}

impl Default for DynamicColorFluxGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aspect_analysis() {
        let gen = DynamicColorFluxGenerator::new();
        let analysis = gen.analyze_aspects("I love this logical proof!").await.unwrap();
        
        // Should have emotional (love) and logical (proof) aspects
        assert!(analysis.aspects.len() >= 2);
        assert!(analysis.averaged_elp.pathos > 0.0); // Love = pathos
        assert!(analysis.averaged_elp.logos > 0.0);  // Proof = logos
    }

    #[test]
    fn test_elp_to_rgb() {
        let elp = ELPScore::new(0.5, 0.3, 0.7); // Mixed
        let rgb = elp.to_rgb();
        
        assert_eq!(rgb.0, elp.pathos); // Red = pathos
        assert_eq!(rgb.1, elp.logos);  // Green = logos
        assert_eq!(rgb.2, elp.ethos);  // Blue = ethos
    }

    #[test]
    fn test_13_scale() {
        let elp = ELPScore::new(1.0, 0.5, 0.0);
        let (e, l, p) = elp.to_13_scale();
        
        assert!(e >= 0 && e <= 13);
        assert!(l >= 0 && l <= 13);
        assert!(p >= -13 && p <= 13);
    }
}
