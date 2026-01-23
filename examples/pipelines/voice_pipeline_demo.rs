//! Voice Pipeline Demo
//!
//! Demonstrates the complete voice-to-space pipeline:
//! Microphone → Audio Capture → Spectral Analysis → ELP Mapping → BeadTensor → FluxMatrix
//!
//! Run with:
//! ```bash
//! cargo run --example voice_pipeline_demo --features voice
//! ```
//!
//! Make sure your microphone is connected and accessible!

use spatial_vortex::voice_pipeline::{VoicePipeline, VoicePipelineBuilder};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎤 Voice Pipeline Demo");
    println!("====================\n");
    println!("Note: Voice pipeline requires LocalSet for audio capture");
    println!("      (audio APIs are platform-specific and not Send)\n");
    
    // Use LocalSet for !Send futures (required by cpal audio capture)
    let local = tokio::task::LocalSet::new();
    
    local.run_until(async {
        // Demo 1: Basic pipeline
        if let Err(e) = demo_basic_pipeline().await {
            eprintln!("Demo 1 error: {}", e);
        }
        
        // Demo 2: Custom configuration
        if let Err(e) = demo_custom_config().await {
            eprintln!("Demo 2 error: {}", e);
        }
        
        // Demo 3: Real-time monitoring
        if let Err(e) = demo_realtime_monitoring().await {
            eprintln!("Demo 3 error: {}", e);
        }
        
        println!("\n✅ All demos completed!");
    }).await;
    
    Ok(())
}

/// Demo 1: Basic voice pipeline with default configuration
async fn demo_basic_pipeline() -> anyhow::Result<()> {
    println!("═══ Demo 1: Basic Voice Pipeline ═══\n");
    
    // Create pipeline with defaults
    let mut pipeline = VoicePipeline::new().await?;
    println!("✓ Pipeline created (44.1kHz, mono, 1024 samples/buffer)");
    
    // Start capturing
    let mut bead_rx = pipeline.start().await?;
    println!("✓ Audio capture started");
    println!("\nListening for 3 seconds...");
    println!("Speak into your microphone!\n");
    
    // Process beads for 3 seconds
    let duration = Duration::from_secs(3);
    let result = timeout(duration, async {
        let mut count = 0;
        while let Some(bead) = bead_rx.recv().await {
            count += 1;
            if count <= 5 {
                // Show first 5 beads
                println!(
                    "Bead #{}: Pitch={:.1} Hz, Loudness={:.1} dB, Confidence={:.2}",
                    count,
                    bead.pitch_hz,
                    bead.loudness_db,
                    bead.confidence
                );
                println!(
                    "  ELP: Ethos={:.2}, Logos={:.2}, Pathos={:.2}",
                    bead.elp_values.ethos,
                    bead.elp_values.logos,
                    bead.elp_values.pathos
                );
            }
        }
        count
    }).await;
    
    match result {
        Ok(count) => println!("\n✓ Processed {} beads in 3 seconds", count),
        Err(_) => println!("\n✓ Timeout reached (3 seconds)"),
    }
    
    pipeline.stop().await;
    println!("✓ Pipeline stopped\n");
    
    Ok(())
}

/// Demo 2: Custom configuration
async fn demo_custom_config() -> anyhow::Result<()> {
    println!("═══ Demo 2: Custom Configuration ═══\n");
    
    // Build pipeline with custom settings
    let mut pipeline = VoicePipelineBuilder::new()
        .sample_rate(48000)      // Higher quality
        .buffer_size(2048)       // Larger buffer
        .channels(1)             // Mono
        .build()
        .await?;
    
    println!("✓ Pipeline created with custom config:");
    println!("  - Sample rate: 48kHz");
    println!("  - Buffer size: 2048 samples");
    println!("  - Channels: Mono");
    
    let mut bead_rx = pipeline.start().await?;
    println!("✓ Audio capture started");
    println!("\nCapturing 2 seconds with high-quality settings...\n");
    
    // Capture for 2 seconds
    let duration = Duration::from_secs(2);
    let result = timeout(duration, async {
        let mut beads = Vec::new();
        while let Some(bead) = bead_rx.recv().await {
            beads.push(bead);
        }
        beads
    }).await;
    
    if let Ok(beads) = result {
        if !beads.is_empty() {
            // Analyze captured beads
            let avg_pitch: f64 = beads.iter().map(|b| b.pitch_hz).sum::<f64>() / beads.len() as f64;
            let avg_loudness: f64 = beads.iter().map(|b| b.loudness_db).sum::<f64>() / beads.len() as f64;
            let avg_confidence: f64 = beads.iter().map(|b| b.confidence).sum::<f64>() / beads.len() as f64;
            
            println!("Analysis of {} beads:", beads.len());
            println!("  Average pitch: {:.1} Hz", avg_pitch);
            println!("  Average loudness: {:.1} dB", avg_loudness);
            println!("  Average confidence: {:.2}", avg_confidence);
            
            // Find pitch range
            let min_pitch = beads.iter().map(|b| b.pitch_hz).fold(f64::INFINITY, f64::min);
            let max_pitch = beads.iter().map(|b| b.pitch_hz).fold(f64::NEG_INFINITY, f64::max);
            println!("  Pitch range: {:.1} Hz - {:.1} Hz", min_pitch, max_pitch);
        }
    }
    
    pipeline.stop().await;
    println!("\n✓ Pipeline stopped\n");
    
    Ok(())
}

/// Demo 3: Real-time monitoring with ELP visualization
async fn demo_realtime_monitoring() -> anyhow::Result<()> {
    println!("═══ Demo 3: Real-time ELP Monitoring ═══\n");
    
    let mut pipeline = VoicePipeline::new().await?;
    let mut bead_rx = pipeline.start().await?;
    
    println!("✓ Real-time monitoring started");
    println!("\nSpeak into your microphone - watch the ELP values change!");
    println!("Monitoring for 5 seconds...\n");
    
    // Visual scale for ELP values
    let scale = |value: f64| -> String {
        let normalized = ((value + 13.0) / 26.0 * 20.0) as usize;
        let bars = "█".repeat(normalized.clamp(0, 20));
        format!("{:3.0} {:<20}", value, bars)
    };
    
    let duration = Duration::from_secs(5);
    let result = timeout(duration, async {
        let mut last_print = std::time::Instant::now();
        let print_interval = Duration::from_millis(500); // Update every 500ms
        
        let mut latest_bead = None;
        
        while let Some(bead) = bead_rx.recv().await {
            latest_bead = Some(bead);
            
            // Print update every 500ms
            if last_print.elapsed() >= print_interval {
                if let Some(ref b) = latest_bead {
                    println!("┌─────────────────────────────────────┐");
                    println!("│ Pitch: {:6.1} Hz | Confidence: {:.2} │", b.pitch_hz, b.confidence);
                    println!("├─────────────────────────────────────┤");
                    println!("│ Ethos  (Authority): {}│", scale(b.elp_values.ethos));
                    println!("│ Logos  (Logic):     {}│", scale(b.elp_values.logos));
                    println!("│ Pathos (Emotion):   {}│", scale(b.elp_values.pathos));
                    println!("└─────────────────────────────────────┘\n");
                }
                last_print = std::time::Instant::now();
            }
        }
    }).await;
    
    match result {
        Ok(_) => println!("✓ Monitoring complete"),
        Err(_) => println!("✓ Monitoring timeout (5 seconds)"),
    }
    
    pipeline.stop().await;
    println!("✓ Pipeline stopped\n");
    
    Ok(())
}

/// Helper: Interpret ELP values
#[allow(dead_code)]
fn interpret_elp(ethos: f64, logos: f64, pathos: f64) -> String {
    let mut interpretation = String::new();
    
    // Ethos interpretation
    if ethos > 8.0 {
        interpretation.push_str("Strong, authoritative voice");
    } else if ethos > 0.0 {
        interpretation.push_str("Moderate authority");
    } else {
        interpretation.push_str("Gentle, soft voice");
    }
    
    interpretation.push_str(" | ");
    
    // Logos interpretation
    if logos > 8.0 {
        interpretation.push_str("High clarity, analytical");
    } else if logos > 0.0 {
        interpretation.push_str("Balanced expression");
    } else {
        interpretation.push_str("Casual, conversational");
    }
    
    interpretation.push_str(" | ");
    
    // Pathos interpretation
    if pathos > 8.0 {
        interpretation.push_str("Highly emotional/expressive");
    } else if pathos > 0.0 {
        interpretation.push_str("Moderate emotion");
    } else {
        interpretation.push_str("Calm, neutral tone");
    }
    
    interpretation
}
