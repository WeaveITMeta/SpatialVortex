//! Real-time streaming support for voice pipeline
//!
//! Provides efficient async streaming using tokio-stream for <100ms latency targets.

use anyhow::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{Stream, StreamExt};

use super::{BeadTensor, SpectralAnalyzer, SpectralFeatures};
use crate::ai::orchestrator::ASIOrchestrator;
use crate::models::ELPTensor;

/// Streaming voice processor with real-time FFT
pub struct StreamingVoiceProcessor {
    analyzer: Arc<Mutex<SpectralAnalyzer>>,
    audio_rx: mpsc::Receiver<Vec<f32>>,
    buffer: Vec<f32>,
    window_size: usize,
    hop_size: usize,
}

impl StreamingVoiceProcessor {
    /// Creates a new streaming voice processor
    pub fn new(
        analyzer: Arc<Mutex<SpectralAnalyzer>>,
        audio_rx: mpsc::Receiver<Vec<f32>>,
    ) -> Self {
        Self {
            analyzer,
            audio_rx,
            buffer: Vec::with_capacity(4096),
            window_size: 2048,
            hop_size: 512, // 75% overlap for smooth processing
        }
    }

    /// Process audio buffer with sliding window for real-time FFT
    async fn process_buffer(&mut self) -> Option<SpectralFeatures> {
        if self.buffer.len() >= self.window_size {
            // Extract window
            let window: Vec<f32> = self.buffer[..self.window_size].to_vec();
            
            // Slide buffer forward by hop_size
            self.buffer.drain(..self.hop_size);
            
            // Perform FFT analysis
            let mut analyzer = self.analyzer.lock().await;
            Some(analyzer.analyze(&window))
        } else {
            None
        }
    }
}

impl Stream for StreamingVoiceProcessor {
    type Item = SpectralFeatures;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check for new audio data
        match self.audio_rx.poll_recv(cx) {
            Poll::Ready(Some(chunk)) => {
                // Add to buffer
                self.buffer.extend(chunk);
                
                // Process if we have enough data
                let fut = self.process_buffer();
                let mut pinned_fut = Box::pin(fut);
                
                match pinned_fut.as_mut().poll(cx) {
                    Poll::Ready(Some(features)) => Poll::Ready(Some(features)),
                    Poll::Ready(None) => {
                        // Not enough data yet, wake up and try again
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
            Poll::Ready(None) => Poll::Ready(None), // Stream ended
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Voice stream adapter for ASI Orchestrator integration
pub struct VoiceToASIStream {
    voice_stream: Pin<Box<dyn Stream<Item = BeadTensor> + Send>>,
    orchestrator: Arc<ASIOrchestrator>,
}

impl VoiceToASIStream {
    /// Creates a new voice to ASI stream adapter
    pub fn new(
        voice_stream: Pin<Box<dyn Stream<Item = BeadTensor> + Send>>,
        orchestrator: Arc<ASIOrchestrator>,
    ) -> Self {
        Self {
            voice_stream,
            orchestrator,
        }
    }

    /// Processes voice-derived BeadTensor through ASI Orchestrator
    pub async fn process_with_orchestrator(&mut self) -> Result<()> {
        while let Some(bead) = self.voice_stream.next().await {
            // Extract voice characteristics
            let voice_input = format!(
                "Voice: pitch={:.1}Hz, loudness={:.1}dB, confidence={:.2}",
                bead.pitch_hz, bead.loudness_db, bead.confidence
            );
            
            // Convert ELP to orchestrator-compatible format
            let elp = ELPTensor {
                ethos: bead.elp_values.ethos,
                logos: bead.elp_values.logos,
                pathos: bead.elp_values.pathos,
            };
            
            // Process through ASI orchestrator with voice mode
            match self.orchestrator.process_voice(&voice_input, Some(elp)).await {
                Ok(result) => {
                    // Check for hallucination in voice-derived content (consolidated metric)
                    if result.confidence < 0.5 {
                        tracing::warn!(
                            "Low confidence in voice input: {:.2} (possible hallucination)",
                            result.confidence
                        );
                    }
                    
                    // Store in Confidence Lake if high-quality
                    if result.confidence >= 0.6 {
                        tracing::info!(
                            "High-quality voice moment captured: confidence={:.2}",
                            result.confidence
                        );
                        // Lake storage happens inside orchestrator
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process voice through orchestrator: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

/// Real-time audio stream with buffering for consistent FFT windows
pub struct BufferedAudioStream {
    rx: mpsc::Receiver<Vec<f32>>,
    buffer: Vec<f32>,
    target_size: usize,
}

impl BufferedAudioStream {
    /// Creates a new buffered audio stream
    pub fn new(rx: mpsc::Receiver<Vec<f32>>, target_size: usize) -> Self {
        Self {
            rx,
            buffer: Vec::with_capacity(target_size * 2),
            target_size,
        }
    }
}

impl Stream for BufferedAudioStream {
    type Item = Vec<f32>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Try to fill buffer to target size
        loop {
            match self.rx.poll_recv(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.buffer.extend(chunk);
                    
                    if self.buffer.len() >= self.target_size {
                        // Extract target_size samples
                        let output: Vec<f32> = self.buffer.drain(..self.target_size).collect();
                        return Poll::Ready(Some(output));
                    }
                    // Continue filling buffer
                }
                Poll::Ready(None) => {
                    // Channel closed
                    if !self.buffer.is_empty() {
                        // Return remaining data
                        let output = self.buffer.drain(..).collect();
                        return Poll::Ready(Some(output));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    // No more data available right now
                    if self.buffer.len() >= self.target_size {
                        // We have enough buffered data
                        let output: Vec<f32> = self.buffer.drain(..self.target_size).collect();
                        return Poll::Ready(Some(output));
                    }
                    return Poll::Pending;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffered_audio_stream() {
        let (tx, rx) = mpsc::channel(10);
        let mut stream = BufferedAudioStream::new(rx, 100);
        
        // Send partial data
        tx.send(vec![1.0; 50]).await.unwrap();
        tx.send(vec![2.0; 60]).await.unwrap();
        
        // Should get exactly 100 samples
        if let Some(chunk) = stream.next().await {
            assert_eq!(chunk.len(), 100);
            assert_eq!(chunk[0], 1.0);
            assert_eq!(chunk[50], 2.0);
        }
        
        // Should have 10 samples remaining in buffer
    }
}
