# Runtime Weakness Detector - Autonomous Monitoring

**Status**: ✅ OPERATIONAL  
**Date**: December 30, 2025  
**Version**: 1.0.0

## Overview

The Runtime Weakness Detector provides **continuous autonomous monitoring** of system performance with automatic RSI triggering. It monitors metrics in real-time, detects degradation patterns, and auto-triggers self-improvement proposals without manual intervention.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   RUNTIME DETECTOR ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐                                                       │
│  │ ASI Runs     │  Every inference records metrics:                     │
│  │ Inferences   │  • Latency (ms)                                       │
│  │              │  • Confidence (0-1)                                   │
│  │              │  • Prediction error                                   │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Rolling      │  Maintains window of recent samples:                 │
│  │ Window       │  • Default: 60 samples (60 seconds)                  │
│  │              │  • Calculates baselines from first half              │
│  │              │  • Compares recent samples to baseline               │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Background   │  Runs every N milliseconds:                          │
│  │ Monitor      │  • Default: 1000ms (1 second)                        │
│  │              │  • Analyzes window for patterns                      │
│  │              │  • Detects 6 types of weaknesses                     │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Weakness     │  Detects:                                             │
│  │ Detection    │  1. Latency spikes (N spikes above threshold)        │
│  │              │  2. Confidence drop (% decrease from baseline)       │
│  │              │  3. Low confidence (below absolute threshold)        │
│  │              │  4. Prediction error spike (% increase)              │
│  │              │  5. Memory pressure (% usage)                        │
│  │              │  6. Throughput drop (% decrease)                     │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ Auto-Trigger │  If weaknesses detected:                              │
│  │ RSI          │  • Check cooldown period                             │
│  │              │  • Send trigger event via channel                    │
│  │              │  • Log all weaknesses                                │
│  └──────┬───────┘                                                       │
│         │                                                                │
│         ▼                                                                │
│  ┌──────────────┐                                                       │
│  │ RSI Handler  │  Receives trigger events:                             │
│  │              │  • Proposes improvements                              │
│  │              │  • Tests proposals                                    │
│  │              │  • Auto-applies if safe                               │
│  │              │  • Queues risky changes                               │
│  └──────────────┘                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Key Features

### 1. Continuous Monitoring
- **Background task** runs independently
- **Non-blocking** - doesn't affect inference performance
- **Configurable interval** (default: 1 second)
- **Rolling window** for trend analysis

### 2. Six Detection Types

**Latency Spikes**
- Detects when latency exceeds baseline + threshold
- Requires N spikes in window (default: 3)
- Example: 500ms above 200ms baseline

**Confidence Drop**
- Detects sudden confidence decrease
- Compares recent average to baseline
- Example: Drop from 0.85 to 0.65 (20% drop)

**Low Confidence**
- Detects absolute low confidence
- Independent of baseline
- Example: Below 0.6 threshold

**Prediction Error Spike**
- Detects increase in prediction errors
- Percentage increase from baseline
- Example: 50% increase in error rate

**Memory Pressure**
- Detects high memory usage
- Percentage of maximum
- Example: Above 85% threshold

**Throughput Drop**
- Detects request rate decrease
- Percentage drop from baseline
- Example: 30% throughput reduction

### 3. Auto-Trigger RSI
- **Automatic proposal generation** when weaknesses detected
- **Cooldown period** prevents trigger spam (default: 5 minutes)
- **Channel-based communication** between detector and orchestrator
- **Async processing** doesn't block monitoring

### 4. Baseline Calculation
- Uses **first half of window** as baseline
- Adapts to changing conditions over time
- Requires minimum samples (10) before detection
- Prevents false positives during startup

## Configuration

### RuntimeDetectorConfig

```rust
pub struct RuntimeDetectorConfig {
    /// Enable runtime detection
    pub enabled: bool,  // Default: true
    
    /// Monitoring interval in milliseconds
    pub monitor_interval_ms: u64,  // Default: 1000
    
    /// Window size for rolling statistics
    pub window_size: usize,  // Default: 60
    
    /// Latency spike threshold (ms above baseline)
    pub latency_spike_threshold_ms: f32,  // Default: 500.0
    
    /// Latency spike count before trigger
    pub latency_spike_count: usize,  // Default: 3
    
    /// Confidence drop threshold (absolute drop)
    pub confidence_drop_threshold: f32,  // Default: 0.15 (15%)
    
    /// Confidence low threshold (absolute value)
    pub confidence_low_threshold: f32,  // Default: 0.6
    
    /// Prediction error spike threshold (% increase)
    pub prediction_error_spike_threshold: f32,  // Default: 0.5 (50%)
    
    /// Memory pressure threshold (% of max)
    pub memory_pressure_threshold: f32,  // Default: 0.85 (85%)
    
    /// Throughput drop threshold (% decrease)
    pub throughput_drop_threshold: f32,  // Default: 0.3 (30%)
    
    /// Auto-trigger RSI on detection
    pub auto_trigger_rsi: bool,  // Default: true
    
    /// Cooldown period after trigger (seconds)
    pub trigger_cooldown_secs: u64,  // Default: 300 (5 min)
}
```

## Usage

### Basic Setup

```rust
use spatial_vortex::ai::orchestrator::{ASIOrchestrator, RSIConfig};
use spatial_vortex::asi::runtime_detector::RuntimeDetectorConfig;

// Configure detector
let detector_config = RuntimeDetectorConfig {
    enabled: true,
    monitor_interval_ms: 1000,
    confidence_low_threshold: 0.7,
    auto_trigger_rsi: true,
    ..Default::default()
};

// Create orchestrator with detector
let orchestrator = ASIOrchestrator::new()
    .await?
    .with_self_modification(source_path)
    .with_runtime_detector(detector_config);

// Configure RSI
let rsi_config = RSIConfig {
    enabled: true,
    auto_apply_low_risk: true,
    ..Default::default()
};
orchestrator.enable_rsi(rsi_config).await;

// Start autonomous monitoring
let monitor_handle = orchestrator.start_runtime_monitoring().await?;

// System now monitors itself continuously
// Auto-triggers RSI when issues detected
```

### Manual Metric Recording

```rust
// Metrics are auto-recorded during process()
orchestrator.process(input, ExecutionMode::Fast).await?;

// Or record manually
if let Some(ref detector) = orchestrator.runtime_detector {
    detector.record_sample(
        latency_ms,
        confidence,
        prediction_error,
    );
}
```

### Monitoring Statistics

```rust
// Get runtime statistics
if let Some(stats) = orchestrator.runtime_stats() {
    println!("Samples collected: {}", stats.samples_collected);
    println!("Weaknesses detected: {}", stats.weaknesses_detected);
    println!("RSI triggers: {}", stats.rsi_triggers);
    println!("Avg latency: {:.0}ms", stats.avg_latency_ms);
    println!("Avg confidence: {:.2}", stats.avg_confidence);
}

// Get recent weaknesses
let weaknesses = orchestrator.get_runtime_weaknesses(10);
for weakness in weaknesses {
    println!("{:?}: {}", weakness.weakness_type, weakness.description);
    println!("Severity: {:.2}", weakness.severity);
}
```

### Stopping Monitoring

```rust
// Stop background monitoring
orchestrator.stop_runtime_monitoring();
```

## Integration with RSI Loop

The runtime detector integrates seamlessly with the RSI loop:

1. **Detector** monitors metrics continuously
2. **Weaknesses** detected based on thresholds
3. **Trigger event** sent via channel when cooldown elapsed
4. **RSI handler** receives event and processes weaknesses
5. **Proposals** generated for each weakness type
6. **Testing** validates proposals
7. **Auto-apply** if risk level permits
8. **Manual approval** queued for risky changes

### Weakness Type Mapping

| Detector Type | RSI Weakness Type |
|---------------|-------------------|
| LatencySpike | slow_reasoning |
| ConfidenceDrop | confidence_drop |
| ConfidenceLow | low_confidence |
| PredictionErrorSpike | high_error_rate |
| MemoryPressure | memory_leak |
| ThroughputDrop | slow_reasoning |

## Performance Impact

### Monitoring Overhead
- **CPU**: <0.5% (background task)
- **Memory**: ~10KB per 60 samples
- **Latency**: 0ms (async, non-blocking)

### Detection Latency
- **Minimum**: 10 samples required
- **Typical**: 10-20 seconds to detect
- **Maximum**: Window size (60 seconds default)

### Trigger Frequency
- **Cooldown**: 5 minutes default
- **Rate limit**: Max 1 trigger per cooldown period
- **Prevents**: Trigger spam and proposal flooding

## Testing

### Unit Tests
**File**: `tests/runtime_detector_test.rs`

Tests cover:
- Detector creation and configuration
- Monitoring start/stop
- Metric recording
- Confidence drop detection
- Latency spike detection
- Auto-trigger with self-mod
- Statistics tracking

### Integration Demo
**File**: `examples/runtime_detector_demo.rs`

Demonstrates:
1. Orchestrator setup with detector
2. Autonomous monitoring start
3. Workload simulation (normal → degraded → recovery)
4. Real-time statistics display
5. Weakness detection
6. Auto-trigger RSI
7. Proposal generation

**Run demo**:
```bash
cargo run --example runtime_detector_demo
```

## Comparison: Manual vs Autonomous

### Manual RSI Cycle
```rust
// User must explicitly call
let result = orchestrator.rsi_cycle().await?;
```

**Pros**: Full control, predictable timing  
**Cons**: Requires manual intervention, delayed response

### Autonomous Runtime Detector
```rust
// Runs continuously in background
orchestrator.start_runtime_monitoring().await?;
```

**Pros**: Immediate response, no manual intervention, continuous monitoring  
**Cons**: Requires tuning thresholds, potential false positives

### Best Practice: Use Both
- **Runtime detector**: Immediate response to acute issues
- **Manual RSI cycle**: Scheduled comprehensive analysis
- **Combined**: Fast response + thorough periodic review

## Tuning Guidelines

### Conservative (Production)
```rust
RuntimeDetectorConfig {
    confidence_low_threshold: 0.5,  // Very low
    latency_spike_threshold_ms: 1000.0,  // High
    latency_spike_count: 5,  // Many spikes
    trigger_cooldown_secs: 600,  // 10 minutes
    ..Default::default()
}
```

### Aggressive (Development)
```rust
RuntimeDetectorConfig {
    confidence_low_threshold: 0.8,  // High
    latency_spike_threshold_ms: 200.0,  // Low
    latency_spike_count: 2,  // Few spikes
    trigger_cooldown_secs: 60,  // 1 minute
    ..Default::default()
}
```

### Balanced (Recommended)
```rust
RuntimeDetectorConfig::default()
```

## Future Enhancements

### Phase 2: Advanced Detection
- Trend analysis (moving averages)
- Anomaly detection (statistical outliers)
- Pattern recognition (recurring issues)
- Predictive alerts (before failure)

### Phase 3: Adaptive Thresholds
- Learn optimal thresholds from history
- Adjust based on workload patterns
- Per-mode thresholds (Fast vs Thorough)
- Time-of-day adjustments

### Phase 4: Multi-Metric Correlation
- Detect complex failure patterns
- Cross-metric analysis
- Root cause identification
- Cascading failure prevention

## Conclusion

The Runtime Weakness Detector enables **true autonomous operation** by:

✅ **Monitoring** system performance continuously  
✅ **Detecting** degradation in real-time  
✅ **Triggering** RSI automatically  
✅ **Responding** to issues immediately  
✅ **Preventing** performance degradation  
✅ **Maintaining** system health autonomously  

Combined with the RSI loop, this creates a **fully autonomous self-improving system** that monitors itself, detects issues, and applies fixes without human intervention.

## References

- `src/asi/runtime_detector.rs` - Core implementation
- `src/ai/orchestrator.rs` - Integration with ASI
- `tests/runtime_detector_test.rs` - Test suite
- `examples/runtime_detector_demo.rs` - Demo application
- `docs/RSI_LOOP_IMPLEMENTATION.md` - RSI loop documentation
