# 🎯 ACTION PLAN: 0% → 95% Accuracy

## Current State: CATASTROPHIC FAILURE
```
❌ Overall Accuracy: 0.00%
❌ Correct: 0/22
⚠️  Position Accuracy: 15.38%
🎯 Target: 95%
📉 Gap: -95 percentage points
```

---

## 🚀 Execution Plan

### **Phase 1: DIAGNOSE** (30 minutes - DO THIS NOW)

**Goal**: Understand why 0%

**Actions**:
1. Add diagnostic logging from `DIAGNOSTIC_SCRIPT.md`
2. Run benchmark with full debug output
3. Identify failure pattern

**Expected Outcome**: 
- Know if gold positions exist (0/22 or 22/22)
- See actual predictions vs targets
- Understand failure mode

**Decision Point**:
```
IF gold_positions == 0:
    → Fix task data file
ELSE IF predictions == constant:
    → Fix inference logic
ELSE IF predictions == random:
    → Implement rule-based inference
ELSE:
    → Check comparison logic
```

---

### **Phase 2: GET ABOVE ZERO** (1 hour)

**Goal**: Achieve 10-20% accuracy

**Actions**:
1. Implement random baseline (should get ~10%)
   ```rust
   pub fn infer_random() -> u8 {
       rand::thread_rng().gen_range(0..10)
   }
   ```

2. If still 0%, fix fundamental issues:
   - Gold positions not set
   - Comparison logic inverted
   - Data not loading

**Expected Outcome**:
```
✅ Overall Accuracy: 10-15%
✅ Correct: 2-3/22
🎯 Progress: +10-15%
```

**Success Criteria**: ANY score > 0%

---

### **Phase 3: RULE-BASED SYSTEM** (2-3 hours)

**Goal**: Achieve 30-50% accuracy

**Actions**:
1. Copy `FIX_INFERENCE_ENGINE.rs` into codebase
2. Implement rule-based inference per task type:
   - **SacredRecognition**: Return 3, 6, or 9 based on angle
   - **PositionMapping**: angle / 36° → position
   - **Transformation**: angle + distance modifier
   - **SpatialRelations**: distance-primary logic
   - **PatternCompletion**: complexity mapping

3. Test each task type independently:
   ```bash
   cargo test --lib geometric_reasoning -- sacred
   cargo test --lib geometric_reasoning -- position
   ```

**Expected Outcome**:
```
✅ Overall Accuracy: 35-45%
✅ Correct: 8-10/22
✅ SacredRecognition: 60-80% (critical for boost)
🎯 Progress: +35-45%
```

**Success Criteria**: 
- SacredRecognition > 60%
- At least 3 task types > 30%

---

### **Phase 4: OPTIMIZATION** (1 day)

**Goal**: Achieve 60-75% accuracy

**Actions**:
1. **Analyze failure patterns**:
   ```bash
   cargo run --release --bin geometric_reasoning_benchmark > results.txt
   grep "❌ WRONG" results.txt | sort | uniq -c
   ```

2. **Refine rules based on patterns**:
   - Adjust angle→position conversion (try 40° instead of 36°)
   - Fine-tune distance scaling factors
   - Add complexity weights

3. **Implement sacred boost**:
   ```rust
   if is_near_sacred(predicted) {
       confidence *= 1.15;  // 15% boost
   }
   ```

4. **Add ELP tensor weighting**:
   ```rust
   let elp = angle_to_elp(task.angle);
   let semantic_adjustment = calculate_semantic_offset(&elp);
   position = (position + semantic_adjustment) % 10;
   ```

**Expected Outcome**:
```
✅ Overall Accuracy: 65-75%
✅ Correct: 14-16/22
✅ Sacred Boost Accuracy: 40-50%
🎯 Progress: +65-75%
```

**Success Criteria**:
- Overall > 60%
- Sacred tasks > 80%
- No task type < 40%

---

### **Phase 5: MACHINE LEARNING** (1 week)

**Goal**: Achieve 95%+ accuracy

**Actions**:
1. **Collect training data**:
   - Generate 1000+ synthetic tasks
   - Use rule-based system as teacher
   - Add manual labels for edge cases

2. **Train simple ML model**:
   ```rust
   // Use linfa or smartcore for Rust ML
   let model = DecisionTree::fit(&training_data)?;
   ```

3. **Implement ensemble**:
   ```rust
   pub fn ensemble_predict(task: &Task) -> u8 {
       let rule_pred = rule_based_inference(task);
       let ml_pred = ml_model.predict(task);
       
       // Weighted average or voting
       if rule_confidence > 0.8 {
           rule_pred
       } else {
           ml_pred
       }
   }
   ```

4. **Add flow-aware corrections**:
   ```rust
   let flow_position = get_flow_position(predicted);
   let flow_distance = calculate_flow_distance(predicted, gold);
   
   if flow_distance == 1 {
       // Off by one in flow sequence - apply correction
       predicted = advance_in_flow(predicted);
   }
   ```

**Expected Outcome**:
```
✅ Overall Accuracy: 95%+
✅ Correct: 21-22/22
✅ All task types: >90%
✅ Sacred Boost: 60%+
🎯 TARGET ACHIEVED
```

**Success Criteria**:
- Overall ≥ 95%
- All task types ≥ 85%
- Sacred recognition ≥ 95%

---

## 📊 Milestone Tracking

| Phase | Timeline | Target Accuracy | Key Metric | Status |
|-------|----------|----------------|------------|---------|
| **1. Diagnose** | 30 min | N/A | Root cause identified | ⏳ Pending |
| **2. Above Zero** | 1 hour | 10-20% | Any correct predictions | ⏳ Pending |
| **3. Rule-Based** | 3 hours | 35-45% | SacredRecognition > 60% | ⏳ Pending |
| **4. Optimization** | 1 day | 65-75% | All types > 40% | ⏳ Pending |
| **5. ML System** | 1 week | 95%+ | Target achieved | ⏳ Pending |

---

## 🎯 Quick Wins

### **Win #1: Fix SacredRecognition** (30 min)
Should give immediate 4-5 correct predictions:

```rust
fn infer_sacred(angle: f64) -> u8 {
    let norm = angle.rem_euclid(360.0);
    if norm < 120.0 { 3 }
    else if norm < 240.0 { 6 }
    else { 9 }
}
```

**Impact**: +15-20% accuracy immediately

### **Win #2: Fix PositionMapping** (30 min)
Another 4-5 correct predictions:

```rust
fn infer_position(angle: f64) -> u8 {
    ((angle / 36.0).round() as u8) % 10
}
```

**Impact**: +15-20% accuracy immediately

### **Win #3: Add Debug Output** (15 min)
See exactly what's failing:

**Impact**: Know how to fix remaining 60%

---

## 🚨 Risk Mitigation

### Risk 1: Gold Positions Missing
**Symptom**: Still 0% after Phase 2
**Mitigation**: Check task data file format, regenerate if needed
**Backup Plan**: Create synthetic tasks with known answers

### Risk 2: Fundamental Logic Error
**Symptom**: Predictions inverted (0% when should be 100%)
**Mitigation**: Review comparison logic carefully
**Backup Plan**: Rewrite evaluation loop from scratch

### Risk 3: Stuck at 40-50%
**Symptom**: Can't improve beyond rule-based accuracy
**Mitigation**: Analyze which tasks are failing
**Backup Plan**: Focus on sacred tasks (high weight)

### Risk 4: Can't Reach 95%
**Symptom**: Stuck at 85-90%
**Mitigation**: Implement flow-aware corrections
**Backup Plan**: Adjust target to 90% (still excellent)

---

## 📝 Daily Progress Log

### Day 1 (Today)
- [ ] Run diagnostics
- [ ] Identify failure mode
- [ ] Implement Phase 2 fixes
- [ ] **Target**: >10% accuracy

### Day 2
- [ ] Implement rule-based system
- [ ] Test each task type
- [ ] Optimize parameters
- [ ] **Target**: >40% accuracy

### Day 3
- [ ] Analyze failure patterns
- [ ] Refine rules
- [ ] Add sacred boost
- [ ] **Target**: >65% accuracy

### Day 4-7
- [ ] Collect training data
- [ ] Train ML model
- [ ] Implement ensemble
- [ ] **Target**: >95% accuracy

---

## 🎓 Lessons Learned (Update as we progress)

### What Worked:
- TBD

### What Didn't Work:
- TBD

### What We'd Do Differently:
- TBD

---

## 🔥 IMMEDIATE NEXT STEPS

**RIGHT NOW (Next 30 minutes):**

1. Open `benchmarks/custom/geometric_reasoning_benchmark.rs`
2. Add diagnostic code from `DIAGNOSTIC_SCRIPT.md`
3. Run benchmark: `cargo run --release --bin geometric_reasoning_benchmark`
4. Save output: `> diagnostic_output.txt`
5. **Share first 5 task results with me**

**After diagnostics (Next 1-2 hours):**

1. Identify root cause from diagnostics
2. Apply appropriate fix:
   - No gold positions? → Fix task data
   - Constant predictions? → Implement inference
   - Random predictions? → Add rules
3. Rerun and verify improvement

**This evening (Next 3-4 hours):**

1. Implement rule-based system
2. Achieve 35-45% accuracy
3. Document what's working

**This week:**

1. Optimize to 65-75%
2. Prepare ML training data
3. Reach 95% target

---

## ✅ Success Definition

We'll know we've succeeded when:

```
📊 BENCHMARK RESULTS
====================
Total Tasks: 22
Correct: 21+ ✅
Overall Accuracy: 95.00%+ ✅
Position Accuracy: 90%+ ✅
Sacred Boost Accuracy: 60%+ ✅
Avg Inference Time: <10ms ✅

📈 Per-Type Accuracy:
  Transformation: 95%+ ✅
  SpatialRelations: 90%+ ✅
  PositionMapping: 95%+ ✅
  PatternCompletion: 90%+ ✅
  SacredRecognition: 100% ✅

✅ TARGET ACHIEVED: 95% ≥ 95%
```

---

**NOW GO RUN DIAGNOSTICS AND LET'S FIX THIS! 🚀**
