//! ML Backend Selection
//!
//! Primary: Burn (pure Rust, modular, type-safe)
//! Fallback: Candle (Hugging Face, when Burn lacks features)
//!
//! ## Backend Selection Logic
//!
//! 1. Try Burn first (pure Rust, best performance)
//! 2. If Burn errors or lacks feature → fallback to Candle
//! 3. If both fail → return clear error
//!
//! ## Backends Available
//!
//! **Burn**:
//! - NdArray (CPU, default)
//! - WGPU (GPU, cross-platform)
//! - CUDA (NVIDIA GPUs)
//!
//! **Candle**:
//! - CPU backend
//! - CUDA backend

use std::fmt;

/// ML Backend Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Burn with NdArray (CPU) - DEFAULT
    BurnNdArray,
    
    /// Burn with WGPU (GPU, cross-platform)
    BurnWGPU,
    
    /// Burn with CUDA (NVIDIA)
    BurnCUDA,
    
    /// Candle CPU (fallback)
    CandleCPU,
    
    /// Candle CUDA (fallback)
    CandleCUDA,
}

impl BackendType {
    /// Check if backend is available
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "burn")]
            BackendType::BurnNdArray => true,
            
            #[cfg(feature = "burn-wgpu")]
            BackendType::BurnWGPU => true,
            
            #[cfg(feature = "burn-cuda")]
            BackendType::BurnCUDA => true,
            
            #[cfg(feature = "candle")]
            BackendType::CandleCPU => true,
            
            #[cfg(feature = "candle-cuda")]
            BackendType::CandleCUDA => true,
            
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
    
    /// Get backend priority (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            BackendType::BurnCUDA => 0,        // Fastest
            BackendType::BurnWGPU => 1,        // Fast, cross-platform
            BackendType::BurnNdArray => 2,     // CPU, always available
            BackendType::CandleCUDA => 3,      // Fallback GPU
            BackendType::CandleCPU => 4,       // Fallback CPU
        }
    }
    
    /// Check if this is a Burn backend
    pub fn is_burn(&self) -> bool {
        matches!(self, 
            BackendType::BurnNdArray | 
            BackendType::BurnWGPU | 
            BackendType::BurnCUDA
        )
    }
    
    /// Check if this is a Candle backend (fallback)
    pub fn is_candle(&self) -> bool {
        matches!(self, 
            BackendType::CandleCPU | 
            BackendType::CandleCUDA
        )
    }
}

impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendType::BurnNdArray => write!(f, "Burn (NdArray/CPU)"),
            BackendType::BurnWGPU => write!(f, "Burn (WGPU/GPU)"),
            BackendType::BurnCUDA => write!(f, "Burn (CUDA/NVIDIA)"),
            BackendType::CandleCPU => write!(f, "Candle (CPU) [FALLBACK]"),
            BackendType::CandleCUDA => write!(f, "Candle (CUDA) [FALLBACK]"),
        }
    }
}

/// Backend Selection Strategy
#[derive(Debug, Clone)]
pub struct BackendSelector {
    /// Preferred backend order
    preference: Vec<BackendType>,
    
    /// Current active backend
    active: Option<BackendType>,
}

impl Default for BackendSelector {
    fn default() -> Self {
        Self {
            preference: vec![
                BackendType::BurnCUDA,      // Prefer CUDA if available
                BackendType::BurnWGPU,      // Then WGPU
                BackendType::BurnNdArray,   // Then CPU
                BackendType::CandleCUDA,    // Fallback: Candle CUDA
                BackendType::CandleCPU,     // Fallback: Candle CPU
            ],
            active: None,
        }
    }
}

impl BackendSelector {
    /// Create new backend selector with custom preferences
    pub fn new(preference: Vec<BackendType>) -> Self {
        Self {
            preference,
            active: None,
        }
    }
    
    /// Select best available backend
    pub fn select_backend(&mut self) -> Result<BackendType, BackendError> {
        // Try each backend in order of preference
        for backend in &self.preference {
            if backend.is_available() {
                println!("✅ Selected backend: {}", backend);
                self.active = Some(*backend);
                return Ok(*backend);
            } else {
                println!("⚠️  Backend not available: {}", backend);
            }
        }
        
        Err(BackendError::NoBackendAvailable)
    }
    
    /// Get current active backend
    pub fn active_backend(&self) -> Option<BackendType> {
        self.active
    }
    
    /// Force specific backend
    pub fn force_backend(&mut self, backend: BackendType) -> Result<(), BackendError> {
        if !backend.is_available() {
            return Err(BackendError::BackendNotAvailable(backend));
        }
        
        self.active = Some(backend);
        Ok(())
    }
    
    /// Try operation with Burn, fallback to Candle on error
    pub fn try_with_fallback<T, F, G>(
        &mut self,
        burn_op: F,
        candle_op: G,
    ) -> Result<T, BackendError>
    where
        F: FnOnce() -> Result<T, BackendError>,
        G: FnOnce() -> Result<T, BackendError>,
    {
        // Ensure we have a backend selected
        if self.active.is_none() {
            self.select_backend()?;
        }
        
        let backend = self.active.unwrap();
        
        if backend.is_burn() {
            // Try Burn first
            match burn_op() {
                Ok(result) => {
                    println!("✅ Operation succeeded with Burn");
                    Ok(result)
                },
                Err(e) => {
                    println!("⚠️  Burn operation failed: {:?}", e);
                    println!("🔄 Falling back to Candle...");
                    
                    // Switch to Candle fallback
                    self.active = Some(BackendType::CandleCPU);
                    
                    match candle_op() {
                        Ok(result) => {
                            println!("✅ Operation succeeded with Candle (fallback)");
                            Ok(result)
                        },
                        Err(e) => {
                            println!("❌ Candle fallback also failed: {:?}", e);
                            Err(BackendError::AllBackendsFailed)
                        }
                    }
                }
            }
        } else {
            // Already using Candle
            candle_op()
        }
    }
}

/// Backend-related errors
#[derive(Debug, Clone)]
pub enum BackendError {
    /// No backend is available
    NoBackendAvailable,
    
    /// Specific backend not available
    BackendNotAvailable(BackendType),
    
    /// Operation failed on all backends
    AllBackendsFailed,
    
    /// Burn-specific error
    BurnError(String),
    
    /// Candle-specific error
    CandleError(String),
    
    /// Generic error
    Other(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::NoBackendAvailable => {
                write!(f, "No ML backend available. Enable 'burn' or 'candle' feature.")
            },
            BackendError::BackendNotAvailable(backend) => {
                write!(f, "Backend not available: {}", backend)
            },
            BackendError::AllBackendsFailed => {
                write!(f, "All backends failed. Check logs for details.")
            },
            BackendError::BurnError(msg) => {
                write!(f, "Burn error: {}", msg)
            },
            BackendError::CandleError(msg) => {
                write!(f, "Candle error: {}", msg)
            },
            BackendError::Other(msg) => {
                write!(f, "Backend error: {}", msg)
            },
        }
    }
}

impl std::error::Error for BackendError {}

/// Backend information
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub backend_type: BackendType,
    pub is_gpu: bool,
    pub is_cuda: bool,
    pub is_fallback: bool,
}

impl BackendInfo {
    pub fn new(backend_type: BackendType) -> Self {
        Self {
            backend_type,
            is_gpu: matches!(backend_type, 
                BackendType::BurnWGPU | 
                BackendType::BurnCUDA | 
                BackendType::CandleCUDA
            ),
            is_cuda: matches!(backend_type,
                BackendType::BurnCUDA |
                BackendType::CandleCUDA
            ),
            is_fallback: backend_type.is_candle(),
        }
    }
    
    pub fn display_info(&self) {
        println!("🔧 ML Backend Configuration:");
        println!("   Type: {}", self.backend_type);
        println!("   GPU: {}", if self.is_gpu { "Yes" } else { "No (CPU)" });
        println!("   CUDA: {}", if self.is_cuda { "Yes" } else { "No" });
        println!("   Status: {}", if self.is_fallback { "FALLBACK" } else { "PRIMARY" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_availability() {
        #[cfg(feature = "burn")]
        assert!(BackendType::BurnNdArray.is_available());
        
        #[cfg(not(feature = "burn"))]
        assert!(!BackendType::BurnNdArray.is_available());
    }
    
    #[test]
    fn test_backend_priority() {
        assert!(BackendType::BurnCUDA.priority() < BackendType::BurnNdArray.priority());
        assert!(BackendType::BurnNdArray.priority() < BackendType::CandleCPU.priority());
    }
    
    #[test]
    fn test_backend_selector() {
        let mut selector = BackendSelector::default();
        let result = selector.select_backend();
        
        // Should select something if any backend is available
        #[cfg(any(feature = "burn", feature = "candle"))]
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_burn_vs_candle() {
        assert!(BackendType::BurnNdArray.is_burn());
        assert!(!BackendType::BurnNdArray.is_candle());
        
        assert!(BackendType::CandleCPU.is_candle());
        assert!(!BackendType::CandleCPU.is_burn());
    }
}
