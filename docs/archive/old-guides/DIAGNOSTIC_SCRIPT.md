# 🔍 Diagnostic Script for 0% Accuracy

## Step 1: Add Debug Output to Benchmark

Add this code to your benchmark file **before** the main evaluation loop:

```rust
// === DIAGNOSTIC: Task Data Inspection ===
println!("\n🔍 DIAGNOSTIC: Inspecting Task Data");
println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

// Check task loading
println!("Total tasks loaded: {}", tasks.len());

let tasks_with_gold = tasks.iter()
    .filter(|t| t.gold_position.is_some())
    .count();
    
println!("Tasks with gold position: {}/{}", tasks_with_gold, tasks.len());

if tasks_with_gold == 0 {
    eprintln!("❌ CRITICAL ERROR: No tasks have gold positions!");
    eprintln!("   This means accuracy will always be 0%");
    eprintln!("   Check task data file format");
}

// Inspect first 3 tasks
println!("\n📋 Sample Tasks:");
for (i, task) in tasks.iter().take(3).enumerate() {
    println!("\nTask {}:", i + 1);
    println!("  ID: {}", task.id);
    println!("  Type: {}", task.task_type);
    println!("  Angle: {:.1}°", task.angle);
    println!("  Distance: {:.2}", task.distance);
    println!("  Complexity: {:.2}", task.complexity);
    println!("  Gold Position: {:?}", task.gold_position);
}

println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
```

---

## Step 2: Add Per-Task Debug Output

In the evaluation loop, add detailed logging:

```rust
// === DIAGNOSTIC: Per-Task Evaluation ===
for (i, task) in tasks.iter().enumerate() {
    println!("\n{'='*60}");
    println!("TASK {}/{}: {}", i + 1, tasks.len(), task.id);
    println!("{'='*60}");
    
    // Show input
    println!("📥 INPUT:");
    println!("   Type: {}", task.task_type);
    println!("   Angle: {:.1}°", task.angle);
    println!("   Distance: {:.2}", task.distance);
    println!("   Complexity: {:.2}", task.complexity);
    
    // Make prediction
    let predicted = infer_position(&task);
    println!("\n🤖 PREDICTION: {}", predicted);
    
    // Check correctness
    if let Some(gold) = task.gold_position {
        println!("🎯 GOLD: {}", gold);
        
        let correct = predicted == gold;
        let error = (predicted as i16 - gold as i16).abs();
        
        println!("\n📊 RESULT:");
        println!("   Correct: {}", if correct { "✅ YES" } else { "❌ NO" });
        println!("   Error: {} positions", error);
        
        // Show why it might be wrong
        if !correct {
            println!("\n🔍 ANALYSIS:");
            println!("   Predicted {} but gold was {}", predicted, gold);
            println!("   Off by {} positions", error);
            
            // Show what angle/distance should map to
            let angle_pos = (task.angle / 36.0).round() as u8 % 10;
            let dist_pos = (task.distance * 1.5).round() as u8 % 10;
            
            println!("   Angle alone would give: {}", angle_pos);
            println!("   Distance alone would give: {}", dist_pos);
            println!("   Gold position is: {}", gold);
        }
    } else {
        println!("⚠️  NO GOLD POSITION - Cannot evaluate!");
    }
    
    // Pause after first 5 for inspection
    if i < 5 {
        println!("\nPress Enter to continue...");
        let mut _input = String::new();
        std::io::stdin().read_line(&mut _input).ok();
    }
}
```

---

## Step 3: Check Inference Function

Add this diagnostic to your inference function:

```rust
pub fn infer_position(task: &GeometricReasoningTask) -> u8 {
    // === DIAGNOSTIC: Show what inference is doing ===
    println!("  🧠 Inference called");
    println!("     Task type: {}", task.task_type);
    
    let position = match task.task_type.as_str() {
        "SacredRecognition" => {
            let pos = /* your logic */;
            println!("     Sacred logic returned: {}", pos);
            pos
        },
        "PositionMapping" => {
            let pos = ((task.angle / 36.0).round() as u8) % 10;
            println!("     Position mapping: {}° → pos {}", task.angle, pos);
            pos
        },
        _ => {
            let pos = 5; // Default
            println!("     Default fallback returned: {}", pos);
            pos
        }
    };
    
    println!("     FINAL PREDICTION: {}", position);
    position
}
```

---

## Step 4: Validate Comparison Logic

Add this before accuracy calculation:

```rust
// === DIAGNOSTIC: Accuracy Calculation ===
println!("\n🧮 ACCURACY CALCULATION:");
println!("   Total tasks: {}", total);
println!("   Correct predictions: {}", correct);
println!("   Wrong predictions: {}", total - correct);

if total > 0 {
    let accuracy = (correct as f64 / total as f64) * 100.0;
    println!("   Accuracy: {:.2}%", accuracy);
    
    // Double-check the math
    assert_eq!(
        accuracy,
        (correct as f64 / total as f64) * 100.0,
        "Accuracy calculation error!"
    );
} else {
    println!("   ⚠️  No tasks to evaluate!");
}

// Show which tasks were correct
println!("\n✅ CORRECT TASKS:");
for (i, task) in tasks.iter().enumerate() {
    let predicted = infer_position(task);
    if Some(predicted) == task.gold_position {
        println!("   Task {}: {} (predicted=gold={})", 
            i + 1, task.id, predicted);
    }
}

println!("\n❌ WRONG TASKS:");
for (i, task) in tasks.iter().enumerate() {
    let predicted = infer_position(task);
    if Some(predicted) != task.gold_position {
        println!("   Task {}: {} (predicted={}, gold={:?})", 
            i + 1, task.id, predicted, task.gold_position);
    }
}
```

---

## Step 5: Run with Diagnostics

```bash
# Build with diagnostics
cargo build --release --bin geometric_reasoning_benchmark

# Run and save output
cargo run --release --bin geometric_reasoning_benchmark > diagnostic_output.txt 2>&1

# View output
cat diagnostic_output.txt | less
```

---

## What to Look For

### ✅ GOOD Signs:
- "Tasks with gold position: 22/22" - All tasks have targets
- Predictions varying (not all the same number)
- Some tasks showing as correct
- Logical angle→position mappings

### ❌ BAD Signs:
- "Tasks with gold position: 0/22" - **No targets set** ← Most likely cause
- All predictions are same number (e.g., always 5)
- Predicted values outside 0-9 range
- "Gold Position: None" for all tasks

---

## Common Patterns and Fixes

### Pattern 1: All Predictions Same
```
🤖 PREDICTION: 5
🤖 PREDICTION: 5
🤖 PREDICTION: 5
```

**Cause**: Inference function returns constant
**Fix**: Implement actual inference logic

### Pattern 2: No Gold Positions
```
Tasks with gold position: 0/22
Gold Position: None
Gold Position: None
```

**Cause**: Task data file doesn't have gold positions
**Fix**: Add gold positions to task data

### Pattern 3: Predictions Out of Range
```
🤖 PREDICTION: 12
🤖 PREDICTION: 15
```

**Cause**: Missing modulo operation
**Fix**: Add `% 10` to all position calculations

### Pattern 4: Inverted Logic
```
Correct predictions: 19
Accuracy: 0%
```

**Cause**: Comparison logic is inverted
**Fix**: Check `if predicted == gold` not `if predicted != gold`

---

## Next Steps After Diagnostics

1. **Share the diagnostic output** - Post the first 5 task results
2. **Identify the pattern** - Which bad sign are we seeing?
3. **Apply targeted fix** - Use the appropriate fix from EMERGENCY_FIX document
4. **Rerun and verify** - Should see immediate improvement

---

## Expected Output (Healthy System)

```
🔍 DIAGNOSTIC: Inspecting Task Data
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total tasks loaded: 22
Tasks with gold position: 22/22  ← Good!

📋 Sample Tasks:
Task 1:
  ID: task_001
  Type: PositionMapping
  Angle: 72.0°
  Distance: 5.50
  Complexity: 0.60
  Gold Position: Some(2)  ← Gold position exists!

==========================================================
TASK 1/22: task_001
==========================================================
📥 INPUT:
   Type: PositionMapping
   Angle: 72.0°
   Distance: 5.50
   Complexity: 0.60

🤖 PREDICTION: 2

🎯 GOLD: 2

📊 RESULT:
   Correct: ✅ YES
   Error: 0 positions
```

---

Run diagnostics and share output to get targeted fix!
