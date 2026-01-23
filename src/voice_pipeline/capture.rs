//! Real-time audio capture using cpal
//!
//! Captures audio from the default input device and streams it through
//! an async channel for processing.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use tokio::sync::mpsc;

/// Audio configuration settings
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sample rate in Hz (e.g., 44100)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono for voice
            buffer_size: 1024,
        }
    }
}

/// Real-time audio capture using cpal
///
/// Captures audio from the default input device in real-time and sends
/// chunks through an async channel for downstream processing.
///
/// # Examples
///
/// ```no_run
/// use spatial_vortex::voice_pipeline::AudioCapture;
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (tx, mut rx) = mpsc::channel(100);
///     let mut capture = AudioCapture::new(tx)?;
///     
///     // Start capturing in background task
///     tokio::spawn(async move {
///         capture.start().await
///     });
///     
///     // Process audio chunks
///     while let Some(chunk) = rx.recv().await {
///         println!("Received {} samples", chunk.len());
///         // Process audio...
///     }
///     
///     Ok(())
/// }
/// ```
pub struct AudioCapture {
    /// Channel to send captured audio chunks
    sender: mpsc::Sender<Vec<f32>>,
    /// Audio configuration
    config: AudioConfig,
    /// Active audio stream (Some when running)
    stream: Option<Stream>,
    /// Input device
    device: Device,
}

impl AudioCapture {
    /// Creates a new audio capture instance
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel to send captured audio chunks
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - AudioCapture instance or error
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No input device is available
    /// - Device configuration fails
    /// - Device doesn't support required sample rate
    pub fn new(sender: mpsc::Sender<Vec<f32>>) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;
        
        let default_config = device
            .default_input_config()
            .context("Failed to get default input config")?;
        
        let config = AudioConfig {
            sample_rate: default_config.sample_rate().0,
            channels: default_config.channels(),
            buffer_size: 1024,
        };
        
        Ok(Self {
            sender,
            config,
            stream: None,
            device,
        })
    }
    
    /// Creates a new audio capture with custom configuration
    pub fn with_config(
        sender: mpsc::Sender<Vec<f32>>,
        config: AudioConfig,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;
        
        Ok(Self {
            sender,
            config,
            stream: None,
            device,
        })
    }
    
    /// Starts the audio capture stream
    ///
    /// This is an async function that runs the capture loop.
    /// Call this in a separate tokio task for concurrent operation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spatial_vortex::voice_pipeline::AudioCapture;
    /// # use tokio::sync::mpsc;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let (tx, _rx) = mpsc::channel(100);
    /// # let mut capture = AudioCapture::new(tx)?;
    /// tokio::spawn(async move {
    ///     capture.start().await
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&mut self) -> Result<()> {
        let stream_config = StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let sender = self.sender.clone();
        let buffer_size = self.config.buffer_size;
        
        // Buffer to accumulate samples
        let mut sample_buffer: Vec<f32> = Vec::with_capacity(buffer_size);
        
        let stream = self.device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Accumulate samples
                sample_buffer.extend_from_slice(data);
                
                // Send when we have enough samples
                while sample_buffer.len() >= buffer_size {
                    let chunk: Vec<f32> = sample_buffer
                        .drain(..buffer_size)
                        .collect();
                    
                    // Send via channel (non-blocking)
                    if sender.blocking_send(chunk).is_err() {
                        // Channel closed, stop capturing
                        return;
                    }
                }
            },
            |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None, // No timeout
        ).context("Failed to build input stream")?;
        
        stream.play().context("Failed to start audio stream")?;
        self.stream = Some(stream);
        
        // Keep stream alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Check if stream is still active
            if self.stream.is_none() {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Stops the audio capture stream
    pub fn stop(&mut self) {
        self.stream = None;
    }
    
    /// Returns the current audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }
    
    /// Returns the device name
    pub fn device_name(&self) -> Result<String> {
        self.device
            .name()
            .context("Failed to get device name")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audio_capture_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let result = AudioCapture::new(tx);
        
        // May fail if no microphone is available, which is okay in CI
        if let Ok(capture) = result {
            assert!(capture.config().sample_rate > 0);
            assert!(capture.config().channels > 0);
        }
    }
    
    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 1);
        assert_eq!(config.buffer_size, 1024);
    }
    
    #[test]
    fn test_audio_config_custom() {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 2048,
        };
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.buffer_size, 2048);
    }
}
