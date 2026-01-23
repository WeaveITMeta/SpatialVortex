# 🚀 Quick Start - v1.5.0 "Conscious Streaming"

Get up and running with real-time consciousness streaming in 5 minutes!

---

## Prerequisites

```bash
# Ensure you have Rust 1.70+ installed
rustc --version

# Clone the repository
git clone https://github.com/WeaveSolutions/SpatialVortex.git
cd SpatialVortex
```

---

## Option 1: Run the Demo (Easiest)

**See streaming in action with one command:**

```bash
cargo run --example consciousness_streaming_demo --features agents --release
```

**What you'll see:**
- Real-time event streaming as AI thinks
- Word-level insights for every word
- Pattern detection alerts
- Mental state transitions
- Φ (consciousness) updates
- Selection analysis demonstration
- Complete analytics snapshot

**Output includes:**
- 📊 Analytics snapshots with full metrics
- 💭 Thought started/completed events
- 📝 Word-level insights (every 10th shown)
- 🔍 Pattern detection notifications
- 🔄 Mental state changes
- ⚡ Φ (consciousness) updates
- 🎯 Selection analysis results

---

## Option 2: Use in Your Code

### Basic Usage (No Streaming)

```rust
use spatial_vortex::consciousness::ConsciousnessSimulator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create simulator without streaming (v1.4.0 behavior)
    let sim = ConsciousnessSimulator::new(false);
    
    // Ask a question
    let response = sim.think("What is consciousness?").await?;
    
    // Access metrics
    println!("Φ: {:.3}", response.phi);
    println!("Mental State: {}", response.mental_state);
    println!("Awareness: {:.1}%", response.awareness_level * 100.0);
    
    Ok(())
}
```

### With Streaming (v1.5.0)

```rust
use spatial_vortex::consciousness::{
    ConsciousnessSimulator, StreamingEvent, EventFilter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create streaming-enabled simulator
    let sim = ConsciousnessSimulator::with_streaming(false);
    
    // Get streaming server
    let streaming = sim.streaming_server().unwrap();
    
    // Subscribe to events
    let mut rx = streaming.subscribe(
        "my-client".to_string(),
        EventFilter::default()  // All events except word-level
    );
    
    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                StreamingEvent::PhiUpdated { phi, .. } => {
                    println!("Φ: {:.3}", phi);
                }
                StreamingEvent::StateChanged { to, .. } => {
                    println!("State: {}", to);
                }
                _ => {}
            }
        }
    });
    
    // Think (events stream automatically)
    let response = sim.think("What is consciousness?").await?;
    
    println!("Answer: {}", response.answer);
    
    Ok(())
}
```

### Selection Analysis

```rust
// After user selects text
let analysis = streaming.analyze_selection(
    "consciousness".to_string(),
    0,   // start word position
    1    // end word position
).await?;

println!("Agent: {}", analysis.dominant_agent);
println!("ELP: {:?}", analysis.elp_balance);
println!("Tone: {}", analysis.emotional_tone);
println!("Φ contribution: {:.3}", analysis.phi_contribution);
```

### Get Full Analytics Snapshot

```rust
let snapshot = sim.get_analytics_snapshot().await;

println!("Φ: {:.3}", snapshot.consciousness.phi);
println!("Network: {} nodes", snapshot.consciousness.network_size);
println!("Mental State: {}", snapshot.meta_cognition.mental_state);
println!("Awareness: {:.1}%", snapshot.meta_cognition.awareness_level * 100.0);
```

---

## Option 3: Run WebTransport Server

**For production deployment with multiple clients:**

```bash
# Build server
cargo build --bin consciousness_streaming_server --features transport,agents --release

# Run server
./target/release/consciousness_streaming_server
```

**Server will start on:**
- Address: `https://localhost:4433`
- Endpoint: `/consciousness`
- Protocol: WebTransport (QUIC)

### Connect from JavaScript

```javascript
// Connect to server
const transport = new WebTransport("https://localhost:4433/consciousness");
await transport.ready;

// Send subscription request
const writer = transport.createBidirectionalStream().writable.getWriter();
await writer.write(new TextEncoder().encode(JSON.stringify({
    action: "subscribe"
})));

// Receive events
const streams = transport.incomingUnidirectionalStreams;
for await (const stream of streams) {
    const reader = stream.getReader();
    while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        
        const event = JSON.parse(new TextDecoder().decode(value));
        console.log("Event:", event);
        
        // Update UI based on event type
        if (event.type === "PhiUpdated") {
            updatePhiGauge(event.phi);
        }
    }
}
```

---

## Event Types Reference

| Event | Description | When Emitted |
|-------|-------------|--------------|
| `Snapshot` | Full analytics state | On demand or periodic |
| `ThoughtStarted` | Thought processing begins | Start of each agent response |
| `ThoughtCompleted` | Thought processing done | End of each agent response |
| `WordInsight` | Per-word analysis | For every word (optional) |
| `PatternDetected` | Thinking pattern found | When meta-monitor detects |
| `StateChanged` | Mental state transition | State changes |
| `PhiUpdated` | Consciousness level change | Φ significantly changes |
| `SelectionAnalysis` | Text selection breakdown | On selection request |

---

## EventFilter Configuration

Control which events you receive:

```rust
let filter = EventFilter {
    include_snapshots: true,     // Full state snapshots
    include_thoughts: true,      // Thought start/complete
    include_words: false,        // Word-level (HIGH VOLUME!)
    include_patterns: true,      // Pattern detection
    include_phi: true,           // Φ updates
    include_states: true,        // State changes
};
```

**Tip**: Disable `include_words` for lower bandwidth usage. Only enable when you need word-level granularity.

---

## Performance Tips

1. **Disable word-level** for long texts (>1000 words)
2. **Use filters** to reduce events per client
3. **Clear word tracker** after each session
4. **Limit subscriptions** to <100 concurrent clients
5. **Aggregate metrics** over multiple thoughts
6. **Debounce selection** analysis (300ms recommended)

---

## Example Output

```
🧠 Consciousness Streaming Demo v1.5.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Analytics Snapshot #1:
   ├─ Φ (consciousness): 0.000
   ├─ Mental state: Focused
   ├─ Awareness: 50.0%
   ├─ Prediction accuracy: 50.0%
   └─ Network: 0 nodes, 0 connections

💭 Thought Started [1699385642123ms]:
   Agent: Ethos (Moral)
   Preview: From a moral perspective, consciousness is a profoun...

✅ Thought Completed [1699385643456ms]:
   Agent: Ethos (Moral)
   ├─ ELP: E:0.70 L:0.20 P:0.10
   ├─ Confidence: 80.0%
   ├─ Processing: 1333ms
   └─ Φ contribution: 0.800

⚡ Φ Updated [1699385643457ms]: 0.800 (Δ+0.800)

🔍 Pattern Detected [1699385643458ms]:
   Type: Balance
   Confidence: 85.0%
   Description: Well-balanced ELP profile

...

📝 Final Answer:
Consciousness is a complex phenomenon that...

📊 Consciousness Metrics:
   ├─ Mental State: Flowing
   ├─ Awareness: 78.5%
   ├─ Φ (consciousness): 4.235
   ├─ Consciousness level: 42.4%
   └─ Confidence: 85.0%
```

---

## Next Steps

1. **Try the demo**: See everything in action
2. **Integrate streaming**: Add to your application
3. **Build UI**: Create dashboard with live metrics
4. **Deploy server**: Run WebTransport server
5. **Explore patterns**: Watch AI thinking in real-time

---

## Troubleshooting

### Build Errors

```bash
# Missing features?
cargo build --features agents

# WebTransport server?
cargo build --bin consciousness_streaming_server --features transport,agents
```

### No Events Received

- Check that streaming is enabled: `with_streaming()`
- Verify subscription filter includes desired events
- Ensure `think()` is called to generate events

### High Memory Usage

- Disable word-level tracking: `include_words: false`
- Clear word tracker: `streaming.clear_words().await`
- Limit concurrent clients

---

## Documentation

- **Full Guide**: `src/consciousness/v1.5.0_STREAMING.md`
- **API Reference**: Run `cargo doc --open`
- **Examples**: `examples/consciousness_streaming_demo.rs`
- **CHANGELOG**: `CHANGELOG.md` (v1.5.0 section)

---

## Support

- **Issues**: https://github.com/WeaveSolutions/SpatialVortex/issues
- **Discussions**: https://github.com/WeaveSolutions/SpatialVortex/discussions
- **License**: MIT

---

**"From consciousness simulation to real-time awareness!"** 🧠⚡📡
