//! ONNX Session Pooling for Improved Performance
//!
//! Provides session pooling for ONNX Runtime to improve throughput
//! and enable concurrent inference requests.

use std::sync::{Arc, OnceLock};
use parking_lot::Mutex;
use anyhow::Result;

#[cfg(feature = "onnx")]
use std::collections::VecDeque;
#[cfg(feature = "onnx")]
use super::onnx_runtime::OnnxInferenceEngine;

/// Pooled ONNX session for concurrent inference
pub struct OnnxSessionPool {
    /// Pool of available sessions
    #[cfg(feature = "onnx")]
    sessions: Arc<Mutex<VecDeque<OnnxInferenceEngine>>>,
    
    /// Model path for creating new sessions
    #[allow(dead_code)]
    model_path: String,
    
    /// Tokenizer path
    #[allow(dead_code)]
    tokenizer_path: String,
    
    /// Maximum pool size
    max_size: usize,
    
    /// Current pool size
    current_size: Arc<Mutex<usize>>,
}

impl OnnxSessionPool {
    /// Create a new ONNX session pool
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to ONNX model file
    /// * `tokenizer_path` - Path to tokenizer file
    /// * `initial_size` - Initial number of sessions
    /// * `max_size` - Maximum number of sessions
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Session pool or error
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::ml::inference::onnx_pool::OnnxSessionPool;
    /// let pool = OnnxSessionPool::new(
    ///     "models/model.onnx",
    ///     "models/tokenizer.json",
    ///     4,  // initial size
    ///     8   // max size
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn new(
        model_path: impl Into<String>,
        tokenizer_path: impl Into<String>,
        initial_size: usize,
        max_size: usize,
    ) -> Result<Self> {
        let model_path = model_path.into();
        let tokenizer_path = tokenizer_path.into();
        
        let mut sessions = VecDeque::with_capacity(max_size);
        
        // Create initial sessions
        for _ in 0..initial_size {
            let session = OnnxInferenceEngine::new(&model_path, &tokenizer_path)
                .context("Failed to create ONNX session")?;
            sessions.push_back(session);
        }
        
        Ok(Self {
            sessions: Arc::new(Mutex::new(sessions)),
            model_path,
            tokenizer_path,
            max_size,
            current_size: Arc::new(Mutex::new(initial_size)),
        })
    }
    
    #[cfg(not(feature = "onnx"))]
    pub fn new(
        model_path: impl Into<String>,
        tokenizer_path: impl Into<String>,
        _initial_size: usize,
        max_size: usize,
    ) -> Result<Self> {
        Ok(Self {
            model_path: model_path.into(),
            tokenizer_path: tokenizer_path.into(),
            max_size,
            current_size: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Get a session from the pool
    #[cfg(feature = "onnx")]
    async fn get_session(&self) -> Result<OnnxInferenceEngine> {
        // Try to get existing session
        {
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.pop_front() {
                return Ok(session);
            }
        }
        
        // Check if we can create a new session
        {
            let mut current_size = self.current_size.lock();
            if *current_size < self.max_size {
                *current_size += 1;
                drop(current_size);
                
                // Create new session
                let session = OnnxInferenceEngine::new(&self.model_path, &self.tokenizer_path)
                    .context("Failed to create new ONNX session")?;
                return Ok(session);
            }
        }
        
        // Wait for a session to become available
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let mut sessions = self.sessions.lock();
            if let Some(session) = sessions.pop_front() {
                return Ok(session);
            }
        }
    }
    
    /// Return a session to the pool
    #[cfg(feature = "onnx")]
    fn return_session(&self, session: OnnxInferenceEngine) {
        let mut sessions = self.sessions.lock();
        if sessions.len() < self.max_size {
            sessions.push_back(session);
        }
        // If pool is full, just drop the session
    }
    
    /// Execute embedding with pooled session
    ///
    /// # Arguments
    ///
    /// * `text` - Text to embed
    ///
    /// # Returns
    ///
    /// * `Result<Vec<f32>>` - Embedding vector
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::ml::inference::onnx_pool::OnnxSessionPool;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = OnnxSessionPool::new("model.onnx", "tokenizer.json", 4, 8)?;
    /// let embedding = pool.embed("Hello world").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "onnx")]
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut session = self.get_session().await?;
        let result = session.embed(text)
            .map_err(|e| anyhow::anyhow!("Embedding failed: {}", e))?;
        self.return_session(session);
        Ok(result)
    }
    
    #[cfg(not(feature = "onnx"))]
    pub async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("ONNX feature not enabled"))
    }
    
    /// Execute batch embedding with pooled sessions
    ///
    /// Processes multiple texts concurrently using available sessions
    ///
    /// # Arguments
    ///
    /// * `texts` - Texts to embed
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Vec<f32>>>` - Embedding vectors
    #[cfg(feature = "onnx")]
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        use futures::future::join_all;
        
        let futures: Vec<_> = texts
            .iter()
            .map(|text| self.embed(text))
            .collect();
        
        let results = join_all(futures).await;
        
        // Collect results, propagating any errors
        results.into_iter().collect()
    }
    
    #[cfg(not(feature = "onnx"))]
    pub async fn embed_batch(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Err(anyhow::anyhow!("ONNX feature not enabled"))
    }
    
    /// Execute embedding with sacred geometry transformation
    #[cfg(feature = "onnx")]
    pub async fn embed_with_sacred_geometry(&self, text: &str) -> Result<(Vec<f32>, f32, f32, f32, f32)> {
        let mut session = self.get_session().await?;
        let result = session.embed_with_sacred_geometry(text)
            .map_err(|e| anyhow::anyhow!("Sacred embedding failed: {}", e))?;
        self.return_session(session);
        Ok(result)
    }
    
    #[cfg(not(feature = "onnx"))]
    pub async fn embed_with_sacred_geometry(&self, _text: &str) -> Result<(Vec<f32>, f32, f32, f32, f32)> {
        Err(anyhow::anyhow!("ONNX feature not enabled"))
    }
    
    /// Get current pool size
    pub fn current_size(&self) -> usize {
        *self.current_size.lock()
    }
    
    /// Get maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }
    
    /// Get number of available sessions
    #[cfg(feature = "onnx")]
    pub fn available_sessions(&self) -> usize {
        self.sessions.lock().len()
    }
    
    #[cfg(not(feature = "onnx"))]
    pub fn available_sessions(&self) -> usize {
        0
    }
}

/// Global ONNX session pool for the application
static GLOBAL_POOL: OnceLock<Arc<OnnxSessionPool>> = OnceLock::new();

/// Initialize the global ONNX session pool
///
/// Should be called once at application startup
///
/// # Arguments
///
/// * `model_path` - Path to ONNX model file
/// * `tokenizer_path` - Path to tokenizer file
/// * `initial_size` - Initial number of sessions
/// * `max_size` - Maximum number of sessions
pub fn initialize_global_pool(
    model_path: impl Into<String>,
    tokenizer_path: impl Into<String>,
    initial_size: usize,
    max_size: usize,
) -> Result<()> {
    let pool = OnnxSessionPool::new(model_path, tokenizer_path, initial_size, max_size)?;
    GLOBAL_POOL.set(Arc::new(pool))
        .map_err(|_| anyhow::anyhow!("Global ONNX pool already initialized"))
}

/// Get the global ONNX session pool
///
/// Returns None if pool is not initialized
pub fn get_global_pool() -> Option<Arc<OnnxSessionPool>> {
    GLOBAL_POOL.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(not(feature = "onnx"))]
    fn test_pool_creation_without_onnx() {
        let pool = OnnxSessionPool::new("dummy.onnx", "dummy.json", 2, 4).unwrap();
        assert_eq!(pool.max_size(), 4);
        assert_eq!(pool.current_size(), 0); // No sessions when ONNX disabled
    }
    
    #[tokio::test]
    #[cfg(not(feature = "onnx"))]
    async fn test_embed_without_onnx() {
        let pool = OnnxSessionPool::new("dummy.onnx", "dummy.json", 2, 4).unwrap();
        let result = pool.embed("test").await;
        assert!(result.is_err());
    }
}
