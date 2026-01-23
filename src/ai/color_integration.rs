//! Color ML Integration for ASI Orchestrator
//!
//! Provides color-aware methods for mood detection and generation

use crate::ai::orchestrator::{ASIOrchestrator, ASIOutput, ExecutionMode};
use crate::error::Result;

impl ASIOrchestrator {
    /// Detect semantic color from generated output
    ///
    /// Analyzes the result text and ELP to determine the semantic color/mood
    #[cfg(feature = "color_ml")]
    pub fn detect_output_color(&self, output: &mut ASIOutput) {
        if let Some(ref color_engine) = self.color_engine {
            // Map ELP to approximate color
            use crate::data::AspectColor;
            
            // Convert ELP to hue (ethos dominates hue)
            let hue = ((output.elp.ethos / 9.0) * 360.0) as f32;
            // Saturation from logos
            let sat = ((output.elp.logos / 9.0).clamp(0.3, 1.0)) as f32;
            // Luminance from pathos
            let lum = ((output.elp.pathos / 9.0 * 0.4 + 0.3).clamp(0.2, 0.8)) as f32;
            
            let color = AspectColor::from_hsl(hue, sat, lum);
            
            // Predict meanings from color
            let predictions = color_engine.color_to_meaning(&color);
            
            if let Some(pred) = predictions.first() {
                output.semantic_color = Some(color);
                output.primary_meaning = Some(pred.meaning.clone());
                output.color_confidence = Some(pred.confidence);
                
                // Get top 3 related meanings
                let related: Vec<String> = predictions.iter()
                    .skip(1)
                    .take(3)
                    .map(|p| p.meaning.clone())
                    .collect();
                output.related_meanings = Some(related);
            }
        }
    }
    
    /// Generate content with specific mood/color guidance
    ///
    /// Uses color inference to guide generation toward desired mood
    ///
    /// # Arguments
    ///
    /// * `prompt` - Base prompt text
    /// * `mood` - Desired mood/emotion (e.g., "peaceful", "energetic")
    /// * `mode` - Execution mode
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut asi = ASIOrchestrator::new()?;
    /// let result = asi.generate_with_mood(
    ///     "Write a story",
    ///     "peaceful",
    ///     ExecutionMode::Balanced
    /// ).await?;
    /// println!("Mood: {}", result.primary_meaning.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "color_ml")]
    pub async fn generate_with_mood(
        &mut self,
        prompt: &str,
        mood: &str,
        mode: ExecutionMode,
    ) -> Result<ASIOutput> {
        if let Some(color_engine) = self.color_engine.as_ref() {
            // Get color for mood
            let mood_color = color_engine.meaning_to_color(mood)?;
            
            // Create color context
            let context = color_engine.create_color_context(&mood_color)?;
            
            // Enhance prompt with color context
            let enhanced_prompt = format!(
                "{} [Mood: {} ({}). Related: {}]",
                prompt,
                context.primary_meaning,
                mood_color.to_hex(),
                context.related_meanings.join(", ")
            );
            
            // Process with enhanced prompt
            let mut output = self.process(&enhanced_prompt, mode).await?;
            
            // Inject the target color into output
            output.semantic_color = Some(mood_color);
            output.primary_meaning = Some(context.primary_meaning.clone());
            output.related_meanings = Some(context.related_meanings.clone());
            output.color_confidence = Some(context.average_confidence);
            
            return Ok(output);
        }
        
        // Fallback without color ML
        self.process(prompt, mode).await
    }
    
    /// Find semantically similar concepts via color proximity
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::ai::orchestrator::ASIOrchestrator;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let asi = ASIOrchestrator::new()?;
    /// let similar = asi.find_similar_concepts("love", 5);
    /// // Returns: ["affection", "compassion", "devotion", ...]
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "color_ml")]
    pub fn find_similar_concepts(&self, concept: &str, max_count: usize) -> Vec<String> {
        if let Some(color_engine) = self.color_engine.as_ref() {
            return color_engine.find_similar_meanings(concept, max_count);
        }
        Vec::new()
    }
    
    /// Detect mood/sentiment of any text
    ///
    /// Analyzes text and returns predicted mood with confidence
    #[cfg(feature = "color_ml")]
    pub fn detect_text_mood(&self, _text: &str) -> Result<(String, f32)> {
        use crate::models::ELPTensor;
        use crate::data::AspectColor;
        
        // Simple ELP estimation from text (placeholder)
        // TODO: Actually analyze text to determine ELP
        let elp = ELPTensor {
            ethos: 5.0,
            logos: 6.0,
            pathos: 5.0,
        };
        
        if let Some(color_engine) = self.color_engine.as_ref() {
            // Map ELP to color
            let hue = ((elp.ethos / 9.0) * 360.0) as f32;
            let sat = ((elp.logos / 9.0).clamp(0.4, 0.9)) as f32;
            let lum = 0.5;
            
            let color = AspectColor::from_hsl(hue, sat, lum);
            
            // Predict mood
            let predictions = color_engine.color_to_meaning(&color);
            if let Some(pred) = predictions.first() {
                return Ok((pred.meaning.clone(), pred.confidence));
            }
        }
        
        Ok(("neutral".to_string(), 0.5))
    }
}
