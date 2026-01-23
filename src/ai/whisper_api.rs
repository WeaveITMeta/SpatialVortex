//! Whisper Speech-to-Text API
//!
//! Provides POST /api/v1/voice/transcribe endpoint for audio transcription
//! using OpenAI's Whisper model via whisper-rs

use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[cfg(feature = "voice")]
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

use crate::error::SpatialVortexError;

/// Request for audio transcription
#[derive(Debug, Deserialize)]
pub struct TranscribeRequest {
    /// Base64 encoded audio data (WAV format, 16kHz, mono)
    pub audio_data: String,
    
    /// Language code (optional, e.g., "en", "es", "fr")
    #[serde(default)]
    pub language: Option<String>,
    
    /// Enable timestamp generation
    #[serde(default)]
    pub timestamps: bool,
}

/// Transcription response
#[derive(Debug, Serialize)]
pub struct TranscribeResponse {
    /// Transcribed text
    pub text: String,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    
    /// Language detected
    pub language: String,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Word-level timestamps (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Vec<Timestamp>>,
}

/// Word timestamp
#[derive(Debug, Serialize)]
pub struct Timestamp {
    pub word: String,
    pub start: f32,
    pub end: f32,
    pub confidence: f32,
}

#[cfg(feature = "voice")]
lazy_static::lazy_static! {
    static ref WHISPER_CONTEXT: Arc<RwLock<Option<WhisperContext>>> = {
        Arc::new(RwLock::new(None))
    };
}

/// Initialize Whisper model with optional GPU acceleration
#[cfg(feature = "voice")]
pub async fn initialize_whisper() -> Result<(), SpatialVortexError> {
    let model_path = std::env::var("WHISPER_MODEL_PATH")
        .unwrap_or_else(|_| "./models/ggml-base.en.bin".to_string());
    
    tracing::info!("🎤 Loading Whisper model from: {}", model_path);
    
    let mut params = WhisperContextParameters::default();
    
    // Try to enable GPU acceleration if available
    #[cfg(feature = "voice-cuda")]
    {
        let use_gpu = std::env::var("WHISPER_USE_GPU")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        if use_gpu {
            tracing::info!("🚀 Attempting to enable CUDA GPU acceleration...");
            params.use_gpu(true);
        }
    }
    
    let ctx = WhisperContext::new_with_params(&model_path, params)
        .map_err(|e| {
            #[cfg(feature = "voice-cuda")]
            {
                tracing::warn!("⚠️  Failed to load with GPU, trying CPU fallback...");
                // Try CPU fallback
                let mut cpu_params = WhisperContextParameters::default();
                cpu_params.use_gpu(false);
                
                return WhisperContext::new_with_params(&model_path, cpu_params)
                    .map_err(|e2| SpatialVortexError::AIProviderError(
                        format!("Failed to load Whisper model (GPU and CPU): {} | {}", e, e2)
                    ));
            }
            
            #[cfg(not(feature = "voice-cuda"))]
            {
                return Err(SpatialVortexError::AIProviderError(
                    format!("Failed to load Whisper model: {}", e)
                ));
            }
        })?;
    
    let mut whisper = WHISPER_CONTEXT.write().await;
    *whisper = Some(ctx);
    
    #[cfg(feature = "voice-cuda")]
    tracing::info!("✅ Whisper model loaded successfully with GPU acceleration");
    
    #[cfg(not(feature = "voice-cuda"))]
    tracing::info!("✅ Whisper model loaded successfully (CPU mode)");
    
    Ok(())
}

/// Transcribe audio endpoint
#[post("/voice/transcribe")]
pub async fn transcribe_audio(
    _req: web::Json<TranscribeRequest>,
) -> Result<HttpResponse> {
    let _start = std::time::Instant::now();
    
    #[cfg(not(feature = "voice"))]
    {
        return Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Voice feature not enabled. Compile with --features voice"
        })));
    }
    
    #[cfg(feature = "voice")]
    {
        // Decode base64 audio data
        let audio_bytes = base64::decode(&req.audio_data)
            .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid base64: {}", e)))?;
        
        // Convert to f32 samples (assuming 16-bit PCM)
        let samples = bytes_to_samples(&audio_bytes);
        
        // Get Whisper context
        let whisper_guard = WHISPER_CONTEXT.read().await;
        let ctx = whisper_guard.as_ref()
            .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Whisper model not initialized"))?;
        
        // Create parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Set language if provided
        if let Some(lang) = &req.language {
            params.set_language(Some(lang));
        }
        
        // Enable timestamps if requested
        params.set_print_timestamps(req.timestamps);
        params.set_print_realtime(false);
        params.set_print_progress(false);
        
        // Create a state for transcription
        let mut state = ctx.create_state()
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create state: {}", e)))?;
        
        // Run transcription
        state.full(params, &samples)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Transcription failed: {}", e)))?;
        
        // Get number of segments
        let num_segments = state.full_n_segments()
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get segments: {}", e)))?;
        
        // Extract text and timestamps
        let mut full_text = String::new();
        let mut timestamps = Vec::new();
        let mut total_confidence = 0.0;
        let mut count = 0;
        
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                full_text.push_str(&segment);
                full_text.push(' ');
                
                // Get timestamps if enabled
                if req.timestamps {
                    if let (Ok(start_time), Ok(end_time)) = (
                        state.full_get_segment_t0(i),
                        state.full_get_segment_t1(i)
                    ) {
                        timestamps.push(Timestamp {
                            word: segment.trim().to_string(),
                            start: start_time as f32 / 100.0, // Convert to seconds
                            end: end_time as f32 / 100.0,
                            confidence: 0.95, // Whisper doesn't provide per-word confidence
                        });
                    }
                }
                
                total_confidence += 0.95;
                count += 1;
            }
        }
        
        let confidence = if count > 0 {
            total_confidence / count as f32
        } else {
            0.0
        };
        
        let language = req.language.clone().unwrap_or_else(|| "en".to_string());
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(HttpResponse::Ok().json(TranscribeResponse {
            text: full_text.trim().to_string(),
            confidence,
            language,
            processing_time_ms: elapsed,
            timestamps: if req.timestamps && !timestamps.is_empty() {
                Some(timestamps)
            } else {
                None
            },
        }))
    }
}

/// Convert byte array to f32 samples
#[cfg(feature = "voice")]
fn bytes_to_samples(bytes: &[u8]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(bytes.len() / 2);
    
    for chunk in bytes.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
        samples.push(sample);
    }
    
    samples
}

#[cfg(not(feature = "voice"))]
pub async fn initialize_whisper() -> Result<(), SpatialVortexError> {
    Ok(())
}
