//! Voice-to-Space Pipeline
//!
//! Transforms real-time audio input into geometric ELP (Ethos-Logos-Pathos)
//! tensor representations within the Vortex Math coordinate system.
//!
//! # Pipeline Flow
//!
//! ```text
//! Microphone → AudioCapture → SpectralAnalyzer → VoiceToELPMapper → BeadTensor → FluxMatrix
//! ```
//!
//! # Components
//!
//! - **AudioCapture**: Real-time audio capture using cpal
//! - **SpectralAnalyzer**: FFT-based pitch and feature extraction
//! - **VoiceToELPMapper**: Maps voice features to ELP tensor coordinates
//! - **BeadTensor**: Time-stamped tensor with voice metadata
//!
//! # Example
//!
//! ```no_run
//! use spatial_vortex::voice_pipeline::AudioCapture;
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let mut capture = AudioCapture::new(tx)?;
//!     
//!     // Start capturing in background
//!     tokio::spawn(async move {
//!         capture.start().await
//!     });
//!     
//!     // Process audio chunks
//!     while let Some(chunk) = rx.recv().await {
//!         println!("Received {} samples", chunk.len());
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod capture;
pub mod spectral;
pub mod mapper;
pub mod bead_tensor;
pub mod pipeline;
pub mod streaming;

pub use capture::{AudioCapture, AudioConfig};
pub use spectral::{SpectralAnalyzer, SpectralFeatures};
pub use mapper::VoiceToELPMapper;
pub use crate::models::ELPTensor;
pub use bead_tensor::{BeadTensor, BeadSequence};
pub use pipeline::{VoicePipeline, VoicePipelineBuilder};
pub use streaming::{StreamingVoiceProcessor, VoiceToASIStream, BufferedAudioStream};
