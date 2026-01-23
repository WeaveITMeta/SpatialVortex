//! Tokenization for ONNX Inference
//!
//! This module provides text tokenization using the HuggingFace tokenizers library,
//! preparing text for ONNX model inference.

#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;
use std::error::Error;
use std::path::Path;

/// Wrapper for HuggingFace tokenizer with BERT-style encoding
///
/// Handles tokenization, padding, and attention mask generation for
/// sentence-transformers models.
pub struct TokenizerWrapper {
    #[cfg(feature = "onnx")]
    tokenizer: Tokenizer,
    max_length: usize,
    #[cfg(not(feature = "onnx"))]
    _phantom: std::marker::PhantomData<()>,
}

/// Tokenized output ready for ONNX inference
#[derive(Debug, Clone)]
pub struct TokenizedInput {
    /// Token IDs (input_ids)
    pub token_ids: Vec<i64>,
    /// Attention mask (1 for real tokens, 0 for padding)
    pub attention_mask: Vec<i64>,
    /// Token type IDs (0 for single sentence)
    pub token_type_ids: Vec<i64>,
}

impl TokenizerWrapper {
    /// Create a new tokenizer from a file
    ///
    /// # Arguments
    /// * `tokenizer_path` - Path to tokenizer.json file
    ///
    /// # Returns
    /// * `Result<Self>` - The tokenizer wrapper or an error
    ///
    /// # Example
    /// ```no_run
    /// use spatial_vortex::inference_engine::tokenizer::TokenizerWrapper;
    /// let tokenizer = TokenizerWrapper::new("models/tokenizer.json")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn new<P: AsRef<Path>>(tokenizer_path: P) -> Result<Self, Box<dyn Error>> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;
        
        Ok(Self {
            tokenizer,
            max_length: 384, // Default for sentence-transformers
        })
    }

    #[cfg(not(feature = "onnx"))]
    pub fn new<P: AsRef<Path>>(_tokenizer_path: P) -> Result<Self, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Set maximum sequence length
    ///
    /// # Arguments
    /// * `max_length` - Maximum number of tokens (default: 384)
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    /// Tokenize text into model inputs
    ///
    /// # Arguments
    /// * `text` - Input text to tokenize
    ///
    /// # Returns
    /// * `Result<TokenizedInput>` - Tokenized input ready for inference
    ///
    /// # Example
    /// ```no_run
    /// # use spatial_vortex::inference_engine::tokenizer::TokenizerWrapper;
    /// let tokenizer = TokenizerWrapper::new("models/tokenizer.json")?;
    /// let tokens = tokenizer.tokenize("Hello world")?;
    /// assert!(!tokens.token_ids.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn tokenize(&self, text: &str) -> Result<TokenizedInput, Box<dyn Error>> {
        // Encode the text
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        // Get token IDs
        let mut token_ids: Vec<i64> = encoding.get_ids()
            .iter()
            .map(|&id| id as i64)
            .collect();

        // Get attention mask (1 for real tokens, 0 for padding)
        let mut attention_mask: Vec<i64> = encoding.get_attention_mask()
            .iter()
            .map(|&mask| mask as i64)
            .collect();

        // Pad or truncate to max_length
        let current_len = token_ids.len();
        if current_len < self.max_length {
            // Pad with zeros
            token_ids.resize(self.max_length, 0);
            attention_mask.resize(self.max_length, 0);
        } else if current_len > self.max_length {
            // Truncate
            token_ids.truncate(self.max_length);
            attention_mask.truncate(self.max_length);
        }

        // Token type IDs (all zeros for single sentence)
        let token_type_ids = vec![0i64; self.max_length];

        Ok(TokenizedInput {
            token_ids,
            attention_mask,
            token_type_ids,
        })
    }

    #[cfg(not(feature = "onnx"))]
    pub fn tokenize(&self, _text: &str) -> Result<TokenizedInput, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Tokenize multiple texts in batch
    ///
    /// # Arguments
    /// * `texts` - Slice of text strings to tokenize
    ///
    /// # Returns
    /// * `Result<Vec<TokenizedInput>>` - Vector of tokenized inputs
    #[cfg(feature = "onnx")]
    pub fn tokenize_batch(&self, texts: &[String]) -> Result<Vec<TokenizedInput>, Box<dyn Error>> {
        texts.iter()
            .map(|text| self.tokenize(text))
            .collect()
    }

    #[cfg(not(feature = "onnx"))]
    pub fn tokenize_batch(&self, _texts: &[String]) -> Result<Vec<TokenizedInput>, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Get the vocabulary size
    #[cfg(feature = "onnx")]
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    #[cfg(not(feature = "onnx"))]
    pub fn vocab_size(&self) -> usize {
        0
    }

    /// Get maximum sequence length
    pub fn max_length(&self) -> usize {
        self.max_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "onnx")]
    fn test_tokenized_input_creation() {
        let input = TokenizedInput {
            token_ids: vec![101, 7592, 2088, 102],
            attention_mask: vec![1, 1, 1, 1],
            token_type_ids: vec![0, 0, 0, 0],
        };
        
        assert_eq!(input.token_ids.len(), 4);
        assert_eq!(input.attention_mask.len(), 4);
        assert_eq!(input.token_type_ids.len(), 4);
    }

    #[test]
    #[cfg(not(feature = "onnx"))]
    fn test_tokenizer_feature_disabled() {
        let result = TokenizerWrapper::new("dummy.json");
        assert!(result.is_err());
        let err_msg = format!("{}", result.err().unwrap());
        assert!(err_msg.contains("ONNX feature not enabled"));
    }
}
