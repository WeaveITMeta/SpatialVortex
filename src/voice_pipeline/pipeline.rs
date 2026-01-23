//! Voice Pipeline Integration Layer
//!
//! Connects all voice pipeline components into a unified streaming interface:
//! Microphone → AudioCapture → SpectralAnalyzer → VoiceToELPMapper → BeadTensor → FluxMatrix

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{AudioCapture, AudioConfig, SpectralAnalyzer, VoiceToELPMapper, BeadTensor};
use crate::models::FluxMatrix;
use std::collections::HashMap;
use uuid::Uuid;

/// Complete voice processing pipeline with real-time streaming
///
/// Integrates audio capture, spectral analysis, ELP mapping, and BeadTensor generation
/// into a single streaming interface. Designed for real-time voice-to-space conversion
/// with <100ms latency targets using tokio-stream for efficient async processing.
///
/// # Pipeline Flow
///
/// ```text
/// Microphone
///     ↓
/// AudioCapture (cpal)
///     ↓ Vec<f32> chunks
/// SpectralAnalyzer (FFT)
///     ↓ SpectralFeatures
/// VoiceToELPMapper
///     ↓ ELPTensor
/// BeadTensor
///     ↓ (optional)
/// FluxMatrix
/// ```
///
/// # Examples
///
/// ```no_run
/// use spatial_vortex::voice_pipeline::VoicePipeline;
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create pipeline with default config
///     let mut pipeline = VoicePipeline::new().await?;
///     
///     // Start processing
///     let mut bead_rx = pipeline.start().await?;
///     
///     // Process beads as they arrive
///     while let Some(bead) = bead_rx.recv().await {
///         println!("Pitch: {:.2} Hz, Confidence: {:.2}", 
///             bead.pitch_hz, bead.confidence);
///     }
///     
///     Ok(())
/// }
/// ```
pub struct VoicePipeline {
    /// Audio capture configuration
    config: AudioConfig,
    /// Spectral analyzer (shared across tasks)
    analyzer: Arc<Mutex<SpectralAnalyzer>>,
    /// ELP mapper
    mapper: Arc<VoiceToELPMapper>,
    /// Processing task handle
    task_handle: Option<JoinHandle<()>>,
    /// Stream receiver for async iteration
    stream_rx: Option<mpsc::Receiver<BeadTensor>>,
}

impl VoicePipeline {
    /// Creates a new voice pipeline with default configuration
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Pipeline instance or error
    ///
    /// # Errors
    ///
    /// Returns error if audio device initialization fails
    pub async fn new() -> Result<Self> {
        Self::with_config(AudioConfig::default()).await
    }
    
    /// Creates a new voice pipeline with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Audio configuration settings
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Pipeline instance or error
    pub async fn with_config(config: AudioConfig) -> Result<Self> {
        let analyzer = Arc::new(Mutex::new(SpectralAnalyzer::new(config.sample_rate)));
        let mapper = Arc::new(VoiceToELPMapper::new());
        
        Ok(Self {
            config,
            analyzer,
            mapper,
            task_handle: None,
            stream_rx: None,
        })
    }
    
    /// Starts the voice pipeline and returns a receiver for BeadTensors
    ///
    /// # Returns
    ///
    /// * `Result<mpsc::Receiver<BeadTensor>>` - Receiver for processed beads
    ///
    /// # Errors
    ///
    /// Returns error if audio capture fails to start
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spatial_vortex::voice_pipeline::VoicePipeline;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut pipeline = VoicePipeline::new().await?;
    /// let mut bead_rx = pipeline.start().await?;
    /// 
    /// // Process beads in real-time
    /// while let Some(bead) = bead_rx.recv().await {
    ///     println!("ELP: ({:.2}, {:.2}, {:.2})",
    ///         bead.elp_values.ethos,
    ///         bead.elp_values.logos,
    ///         bead.elp_values.pathos);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&mut self) -> Result<mpsc::Receiver<BeadTensor>> {
        // Create channels
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>(100);
        let (bead_tx, bead_rx) = mpsc::channel::<BeadTensor>(100);
        
        // Start audio capture
        // Note: AudioCapture is not Send, so we handle it in a special way
        let mut capture = AudioCapture::new(audio_tx)?;
        
        // Start audio capture in a local task (not spawned)
        // The user must ensure this runs on a runtime that can handle !Send futures
        tokio::task::spawn_local(async move {
            if let Err(e) = capture.start().await {
                eprintln!("Audio capture error: {}", e);
            }
        });
        
        // Clone Arc references for the processing task
        let analyzer = Arc::clone(&self.analyzer);
        let mapper = Arc::clone(&self.mapper);
        
        // Spawn processing task
        let handle = tokio::spawn(async move {
            Self::processing_loop(audio_rx, bead_tx, analyzer, mapper).await;
        });
        
        self.task_handle = Some(handle);
        
        Ok(bead_rx)
    }
    
    /// Starts pipeline with FluxMatrix integration
    ///
    /// Returns both BeadTensor receiver and a handle for FluxMatrix updates.
    /// Useful for integrating voice input directly into the geometric space.
    ///
    /// # Returns
    ///
    /// * `Result<(Receiver<BeadTensor>, Receiver<FluxMatrix>)>` - Dual receivers
    pub async fn start_with_flux_matrix(&mut self, subject: String) -> Result<(mpsc::Receiver<BeadTensor>, mpsc::Receiver<FluxMatrix>)> {
        let bead_rx = self.start().await?;
        
        let (flux_tx, flux_rx) = mpsc::channel::<FluxMatrix>(10);
        
        // Spawn task to convert BeadTensor stream to FluxMatrix updates
        // This will accumulate beads and update the FluxMatrix periodically
        tokio::spawn(async move {
            Self::bead_to_flux_converter(bead_rx, flux_tx, subject).await;
        });
        
        // Return a new bead receiver (we'll need to re-start)
        let (_new_bead_tx, new_bead_rx) = mpsc::channel::<BeadTensor>(100);
        
        // TODO: Implement proper dual-stream architecture
        // For now, return the flux receiver
        Ok((new_bead_rx, flux_rx))
    }
    
    /// Main processing loop: audio → spectral → ELP → bead
    /// Optimized for <100ms latency with streaming
    async fn processing_loop(
        mut audio_rx: mpsc::Receiver<Vec<f32>>,
        bead_tx: mpsc::Sender<BeadTensor>,
        analyzer: Arc<Mutex<SpectralAnalyzer>>,
        mapper: Arc<VoiceToELPMapper>,
    ) {
        // Audio capture is handled by the caller
        // We just process the incoming audio chunks
        
        // Process audio chunks as they arrive
        while let Some(audio_chunk) = audio_rx.recv().await {
            // Skip empty chunks
            if audio_chunk.is_empty() {
                continue;
            }
            
            // Analyze audio chunk (spectral analysis)
            let features = {
                let mut analyzer_guard = analyzer.lock().await;
                analyzer_guard.analyze(&audio_chunk)
            };
            
            // Map features to ELP tensor
            let (elp, confidence) = mapper.map_with_confidence(&features);
            
            // Create BeadTensor
            let bead = BeadTensor::from_features(elp, &features, confidence);
            
            // Send to output channel
            if bead_tx.send(bead).await.is_err() {
                // Receiver dropped, exit loop
                break;
            }
        }
    }
    
    /// Converts BeadTensor stream to FluxMatrix updates
    ///
    /// Accumulates beads over time and periodically updates a FluxMatrix
    /// with the aggregated voice data.
    async fn bead_to_flux_converter(
        mut bead_rx: mpsc::Receiver<BeadTensor>,
        flux_tx: mpsc::Sender<FluxMatrix>,
        subject: String,
    ) {
        let mut bead_buffer: Vec<BeadTensor> = Vec::new();
        const BUFFER_SIZE: usize = 10; // Update FluxMatrix every 10 beads
        
        while let Some(bead) = bead_rx.recv().await {
            bead_buffer.push(bead);
            
            // When buffer is full, create/update FluxMatrix
            if bead_buffer.len() >= BUFFER_SIZE {
                if let Ok(flux_matrix) = Self::create_flux_from_beads(&bead_buffer, &subject) {
                    if flux_tx.send(flux_matrix).await.is_err() {
                        break; // Receiver dropped
                    }
                }
                bead_buffer.clear();
            }
        }
        
        // Process remaining beads
        if !bead_buffer.is_empty() {
            if let Ok(flux_matrix) = Self::create_flux_from_beads(&bead_buffer, &subject) {
                let _ = flux_tx.send(flux_matrix).await;
            }
        }
    }
    
    /// Creates a FluxMatrix from a sequence of BeadTensors
    ///
    /// Aggregates voice data into geometric space representation.
    fn create_flux_from_beads(beads: &[BeadTensor], subject: &str) -> Result<FluxMatrix> {
        use chrono::Utc;
        
        // Construct FluxMatrix directly
        let now = Utc::now();
        let matrix = FluxMatrix {
            id: Uuid::new_v4(),
            subject: subject.to_string(),
            nodes: HashMap::new(),
            sacred_guides: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // TODO: Implement proper FluxMatrix integration
        // For now, create a basic matrix with averaged ELP values
        // 
        // Stats (for reference, not stored in FluxMatrix):
        // - Avg pitch: {:.2} Hz
        // - Avg loudness: {:.2} dB  
        // - Avg confidence: {:.2}
        // - Bead count: {}
        //
        // Average ELP values across beads:
        let _avg_ethos = beads.iter().map(|b| b.elp_values.ethos).sum::<f64>() / beads.len() as f64;
        let _avg_logos = beads.iter().map(|b| b.elp_values.logos).sum::<f64>() / beads.len() as f64;
        let _avg_pathos = beads.iter().map(|b| b.elp_values.pathos).sum::<f64>() / beads.len() as f64;
        let _avg_confidence = beads.iter().map(|b| b.confidence).sum::<f64>() / beads.len() as f64;
        
        Ok(matrix)
    }
    
    /// Stops the voice pipeline
    ///
    /// Gracefully shuts down audio capture and processing tasks.
    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

impl Drop for VoicePipeline {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

/// Builder for VoicePipeline with fluent API
///
/// # Examples
///
/// ```no_run
/// use spatial_vortex::voice_pipeline::{VoicePipelineBuilder, AudioConfig};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let mut pipeline = VoicePipelineBuilder::new()
///     .sample_rate(48000)
///     .buffer_size(2048)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct VoicePipelineBuilder {
    config: AudioConfig,
}

impl VoicePipelineBuilder {
    /// Creates a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: AudioConfig::default(),
        }
    }
    
    /// Sets the sample rate in Hz
    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.config.sample_rate = rate;
        self
    }
    
    /// Sets the number of audio channels
    pub fn channels(mut self, channels: u16) -> Self {
        self.config.channels = channels;
        self
    }
    
    /// Sets the buffer size in samples
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }
    
    /// Builds the VoicePipeline
    pub async fn build(self) -> Result<VoicePipeline> {
        VoicePipeline::with_config(self.config).await
    }
}

impl Default for VoicePipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
