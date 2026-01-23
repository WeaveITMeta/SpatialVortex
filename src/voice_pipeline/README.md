# Voice Pipeline Module

Complete voice-to-space transformation pipeline for SpatialVortex.

## Overview

Transforms real-time audio input into geometric Ethos-Logos-Pathos (ELP) tensor representations within the Vortex Math coordinate system.

## Pipeline Architecture

```
┌─────────────┐
│ Microphone  │
└──────┬──────┘
       │ Raw Audio
       ▼
┌─────────────────┐
│ AudioCapture    │ ← cpal (Cross-Platform Audio Library)
│ (capture.rs)    │
└────────┬────────┘
         │ Vec<f32> chunks (1024 samples)
         ▼
┌──────────────────┐
│ SpectralAnalyzer │ ← rustfft (Fast Fourier Transform)
│ (spectral.rs)    │
└────────┬─────────┘
         │ SpectralFeatures { pitch, loudness, complexity, ... }
         ▼
┌──────────────────┐
│ VoiceToELPMapper │ ← Heuristic mapping
│ (mapper.rs)      │
└────────┬─────────┘
         │ ELPTensor { ethos, logos, pathos }
         ▼
┌─────────────────┐
│ BeadTensor      │ ← Time-stamped tensor + metadata
│ (bead_tensor.rs)│
└────────┬────────┘
         │ Stream of BeadTensors
         ▼
┌─────────────────┐
│ FluxMatrix      │ ← Geometric space integration
│ (optional)      │
└─────────────────┘
```

## Components

### 1. AudioCapture (`capture.rs`)

**Purpose**: Real-time audio input from microphone

**Key Features**:
- Cross-platform (Windows/Mac/Linux) via cpal
- Configurable sample rate (default: 44.1kHz)
- Mono/stereo support
- Async channel streaming

**Configuration**:
```rust
AudioConfig {
    sample_rate: 44100,  // Hz
    channels: 1,         // Mono for voice
    buffer_size: 1024,   // Samples per chunk
}
```

**Thread Safety**: ⚠️ Not `Send` (platform audio APIs are thread-local)
- Must use `tokio::task::LocalSet`
- Audio capture runs in local task context

### 2. SpectralAnalyzer (`spectral.rs`)

**Purpose**: Extract voice features via Fast Fourier Transform (FFT)

**Extracted Features**:
- **Pitch**: Fundamental frequency (Hz)
- **Loudness**: Volume in dB
- **Spectral Centroid**: Brightness/timbral quality
- **Spectral Flux**: Change over time (expressiveness)
- **Spectral Complexity**: Tonal vs noisy (0=pure tone, 1=noise)

**Algorithm**:
1. Apply Hann window (reduce spectral leakage)
2. Perform FFT on windowed audio
3. Compute magnitude spectrum
4. Extract pitch via autocorrelation
5. Compute spectral statistics

**Performance**: 
- FFT size: 1024-2048 samples
- Processing time: ~1-2ms per chunk
- Real-time capable

### 3. VoiceToELPMapper (`mapper.rs`)

**Purpose**: Map voice features to ELP tensor coordinates

**Mapping Strategy**:

#### Ethos (Character/Authority)
- **Input**: Loudness + Voice Stability
- **Formula**: `0.7 × loudness + 0.3 × stability`
- **Range**: -13 to +13
- **Interpretation**:
  - High (+13): Strong, authoritative voice
  - Low (-13): Gentle, soft voice

#### Logos (Logic/Analytical)
- **Input**: Pitch Height + Clarity
- **Formula**: `0.6 × pitch + 0.4 × clarity`
- **Range**: -13 to +13
- **Interpretation**:
  - High (+13): Analytical, high-pitched
  - Low (-13): Conversational, low-pitched

#### Pathos (Emotion/Expression)
- **Input**: Spectral Complexity + Dynamics
- **Formula**: `0.5 × complexity + 0.5 × flux`
- **Range**: -13 to +13
- **Interpretation**:
  - High (+13): Expressive, emotional
  - Low (-13): Calm, neutral

**Confidence Score**:
- Computed based on feature quality
- Range: 0.0 (unreliable) to 1.0 (high confidence)
- Factors: pitch clarity, loudness sufficiency, stability

### 4. BeadTensor (`bead_tensor.rs`)

**Purpose**: Time-stamped tensor with voice metadata

**Structure**:
```rust
BeadTensor {
    timestamp: DateTime<Utc>,      // When captured
    elp_values: ELPTensor,          // E/L/P coordinates
    pitch_hz: f64,                  // Fundamental frequency
    loudness_db: f64,               // Volume level
    confidence: f64,                // Mapping quality
    confidence_width: f64,          // Uncertainty range
    curviness_signed: f64,          // Pitch change rate
}
```

**Sequence Aggregation**:
- `BeadSequence` accumulates beads over time
- Computes statistics (avg, stddev, trends)
- Identifies patterns in pitch/loudness

### 5. VoicePipeline (`pipeline.rs`)

**Purpose**: Unified integration layer

**Usage**:
```rust
use spatial_vortex::voice_pipeline::VoicePipeline;

// Create pipeline
let mut pipeline = VoicePipeline::new().await?;

// Start processing
let mut bead_rx = pipeline.start().await?;

// Process beads
while let Some(bead) = bead_rx.recv().await {
    println!("ELP: ({:.2}, {:.2}, {:.2})",
        bead.elp_values.ethos,
        bead.elp_values.logos,
        bead.elp_values.pathos);
}
```

**Builder Pattern**:
```rust
let mut pipeline = VoicePipelineBuilder::new()
    .sample_rate(48000)
    .buffer_size(2048)
    .channels(1)
    .build()
    .await?;
```

## Usage Examples

### Example 1: Basic Voice Processing

```rust
use spatial_vortex::voice_pipeline::VoicePipeline;
use tokio::task::LocalSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use LocalSet for audio capture
    let local = LocalSet::new();
    
    local.run_until(async {
        let mut pipeline = VoicePipeline::new().await?;
        let mut bead_rx = pipeline.start().await?;
        
        while let Some(bead) = bead_rx.recv().await {
            println!("Voice tensor: {:?}", bead.elp_values);
        }
        Ok(())
    }).await
}
```

### Example 2: Custom Configuration

```rust
use spatial_vortex::voice_pipeline::VoicePipelineBuilder;

let mut pipeline = VoicePipelineBuilder::new()
    .sample_rate(48000)      // High quality
    .buffer_size(2048)       // Larger buffer
    .build()
    .await?;
```

### Example 3: FluxMatrix Integration

```rust
let (bead_rx, flux_rx) = pipeline
    .start_with_flux_matrix("voice_session".to_string())
    .await?;

// Process both streams
tokio::select! {
    Some(bead) = bead_rx.recv() => {
        // Real-time bead processing
    }
    Some(matrix) = flux_rx.recv() => {
        // Aggregated FluxMatrix updates
    }
}
```

## Running the Demo

```bash
# Run voice pipeline demonstration
cargo run --example voice_pipeline_demo --features voice

# Expected output:
# - Basic pipeline demo (3 seconds)
# - Custom configuration demo (2 seconds)
# - Real-time monitoring with ELP visualization (5 seconds)
```

## Requirements

### Dependencies

- **cpal** 0.15: Cross-platform audio I/O
- **rustfft** 6.4: Fast Fourier Transform
- **tokio**: Async runtime with LocalSet support

### System Requirements

**Audio Input**:
- Microphone or audio input device
- Proper permissions (system may prompt)

**Operating Systems**:
- ✅ Windows (WASAPI)
- ✅ macOS (CoreAudio)
- ✅ Linux (ALSA/PulseAudio)

**Performance**:
- CPU: Any modern processor
- Memory: ~10MB for audio buffers
- Latency: ~50-100ms (capture → ELP)

## Technical Details

### Sample Rate Selection

| Rate | Quality | Use Case | CPU Load |
|------|---------|----------|----------|
| 16kHz | Low | Speech recognition | Low |
| 44.1kHz | Standard | Voice analysis | Medium |
| 48kHz | High | Professional audio | High |

**Recommendation**: 44.1kHz for voice (default)

### Buffer Size Trade-offs

| Size | Latency | Stability | Frequency Resolution |
|------|---------|-----------|---------------------|
| 512 | ~10ms | Lower | Poor |
| 1024 | ~20ms | Good | Good |
| 2048 | ~40ms | Better | Better |
| 4096 | ~80ms | Best | Best |

**Recommendation**: 1024 (default) balances latency and quality

### Frequency Response

**Voice Range**:
- Male: 85-180 Hz (fundamental)
- Female: 165-255 Hz (fundamental)
- Harmonics: Up to ~8kHz

**Nyquist Theorem**:
- Sample rate ≥ 2× max frequency
- 44.1kHz captures up to 22.05kHz (sufficient for voice)

## Threading Model

```
Main Thread (Tokio Runtime)
  │
  ├─► LocalSet
  │   └─► AudioCapture (local task)
  │       └─► Audio callback (cpal thread)
  │
  ├─► Processing Task (spawned)
  │   └─► Spectral analysis + ELP mapping
  │
  └─► User Code (async)
      └─► Receives BeadTensors
```

**Critical**: AudioCapture uses `spawn_local` due to platform audio API limitations.

## Error Handling

### Common Errors

1. **No Audio Device**
   ```
   Error: No input device available
   ```
   **Solution**: Connect microphone, check system audio settings

2. **Permission Denied**
   ```
   Error: Failed to build input stream
   ```
   **Solution**: Grant microphone permissions to application

3. **Sample Rate Not Supported**
   ```
   Error: Device doesn't support required sample rate
   ```
   **Solution**: Use device's default sample rate or try 44100/48000

### Graceful Degradation

```rust
match pipeline.start().await {
    Ok(rx) => { /* Process */ },
    Err(e) if e.to_string().contains("No input") => {
        // Fallback to test data
        use_synthetic_audio();
    },
    Err(e) => return Err(e),
}
```

## Performance Optimization

### Reduce Latency
- Decrease buffer size (512 samples)
- Use dedicated audio thread priority
- Minimize processing in audio callback

### Improve Quality
- Increase buffer size (2048+ samples)
- Use higher sample rate (48kHz)
- Apply noise reduction filters

### Save CPU
- Lower sample rate (16kHz)
- Skip spectral analysis for silent periods
- Batch process multiple chunks

## Future Enhancements

### Planned Features
- [ ] Machine learning ELP mapper (trained on voice data)
- [ ] Voice activity detection (VAD)
- [ ] Noise reduction filters
- [ ] Multi-speaker support
- [ ] Real-time pitch correction
- [ ] Emotion detection via prosody
- [ ] Integration with speech-to-text (STT)

### Research Directions
- [ ] Optimal ELP mapping validation (user studies)
- [ ] Correlation between voice features and sacred geometry
- [ ] Real-time FluxMatrix updates with streaming inference
- [ ] Voice-based hallucination detection

## Testing

```bash
# Unit tests
cargo test voice_pipeline --features voice

# Integration test (requires microphone)
cargo test --test voice_integration --features voice

# Benchmarks
cargo bench --features voice voice_pipeline
```

## Troubleshooting

### Audio Not Capturing

1. Check device availability:
   ```rust
   let host = cpal::default_host();
   let devices = host.input_devices()?;
   for device in devices {
       println!("Device: {:?}", device.name());
   }
   ```

2. Verify permissions (especially macOS/Windows)

3. Test with system audio settings

### Low Confidence Scores

- Speak closer to microphone
- Reduce background noise
- Check loudness levels (should be > -40 dB)
- Verify microphone quality

### High Latency

- Reduce buffer size
- Use lower sample rate
- Check system audio settings
- Close other audio applications

## API Reference

See individual module documentation:
- [`capture`](capture.rs) - Audio input
- [`spectral`](spectral.rs) - FFT analysis
- [`mapper`](mapper.rs) - ELP mapping
- [`bead_tensor`](bead_tensor.rs) - Tensor representation
- [`pipeline`](pipeline.rs) - Integration layer

## License

Part of SpatialVortex project (same license as main crate).
