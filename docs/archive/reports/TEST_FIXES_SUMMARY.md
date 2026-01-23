# ASI Compression Test Fixes

## ✅ Fixed: test_sacred_anchor_finding

### Issue
Test expected position 0 to be closest to sacred anchor 3, but in circular 0-9 space, position 0 is actually closest to anchor 9.

### Root Cause
In a circular space with positions 0-9 and sacred anchors at [3, 6, 9]:
- Distance from 0 to 3: min(3, 7) = **3**
- Distance from 0 to 6: min(6, 4) = **4**  
- Distance from 0 to 9: min(9, 1) = **1** ← Closest!

The wrap-around logic `diff.min(10 - diff)` correctly calculates circular distance.

### Fix Applied
```rust
// ❌ OLD:
assert_eq!(find_nearest_sacred_anchor(0), 3);

// ✅ NEW:
assert_eq!(find_nearest_sacred_anchor(0), 9);  // Closest to 9 (distance 1 via wrap)
```

**File**: `src/compression/asi_12byte.rs:382`

---

## ⚠️ Investigating: test_similarity_in_compressed_space

### Test Logic
```rust
let love = engine.compress("Love", 3, ELPTensor::new(0.7, 0.5, 0.95), 1.0);
let joy = engine.compress("Joy", 1, ELPTensor::new(0.6, 0.4, 0.9), 1.0);
let truth = engine.compress("Truth", 6, ELPTensor::new(0.85, 0.95, 0.5), 1.0);

// Expects: love_joy_sim > love_truth_sim
```

### Analysis

**Semantic Profiles:**
- **Love**: High pathos (0.95), medium ethos (0.7), low logos (0.5) → Emotional
- **Joy**: High pathos (0.9), medium ethos (0.6), low logos (0.4) → Emotional  
- **Truth**: High logos (0.95), high ethos (0.85), low pathos (0.5) → Logical

**Expected**: Love and Joy should be more similar (both emotional).

**Compression Details:**

Love at position 3:
- Nearest anchor: 3 (Ethos = 1.0, Logos = 0.0, Pathos = 0.0)
- Deltas stored: (0.7-1.0, 0.5-0.0, 0.95-0.0) = (-0.3, 0.5, 0.95)

Joy at position 1:
- Nearest anchor: 3 (due to our fix!)
- Deltas stored: (0.6-1.0, 0.4-0.0, 0.9-0.0) = (-0.4, 0.4, 0.9)

Truth at position 6:
- Nearest anchor: 6 (Ethos = 0.0, Logos = 0.0, Pathos = 1.0)
- Deltas stored: (0.85-0.0, 0.95-0.0, 0.5-1.0) = (0.85, 0.95, -0.5)

**Similarity Calculation:**

```rust
pub fn compressed_similarity(&self, other: &Self) -> f32 {
    // Position distance
    let pos_dist = {
        let diff = (self.position_0_9 as i32 - other.position_0_9 as i32).abs();
        diff.min(10 - diff)
    } as f32;
    
    // ELP delta distance (in quantized space)
    let elp_dist = (
        (self.ethos_delta_i16 - other.ethos_delta_i16).pow(2) +
        (self.logos_delta_i16 - other.logos_delta_i16).pow(2) +
        (self.pathos_delta_i16 - other.pathos_delta_i16).pow(2)
    ) as f32 / 1_000_000.0;
    
    // Combined similarity
    let total_dist = pos_dist / 5.0 + elp_dist.sqrt() / 13.0;
    (1.0 - total_dist.min(1.0)).max(0.0)
}
```

**Love vs Joy:**
- pos_dist: |3-1| = 2 → 2/5 = 0.4
- Deltas: Love(-0.3, 0.5, 0.95) vs Joy(-0.4, 0.4, 0.9)
  - Quantized (×13000): Love(-3900, 6500, 12350) vs Joy(-5200, 5200, 11700)
  - Differences: (1300, 1300, 650)
  - Squared: (1.69M, 1.69M, 0.42M) = 3.8M
  - sqrt(3.8M / 1M) = sqrt(3.8) = 1.95
  - 1.95 / 13 = 0.15
- total_dist = 0.4 + 0.15 = 0.55
- **similarity = 1.0 - 0.55 = 0.45**

**Love vs Truth:**
- pos_dist: |3-6| = 3 → 3/5 = 0.6
- Deltas: Love(-0.3, 0.5, 0.95) vs Truth(0.85, 0.95, -0.5)
  - Quantized: Love(-3900, 6500, 12350) vs Truth(11050, 12350, -6500)
  - Differences: (14950, 5850, 18850)
  - Squared: (223.5M, 34.2M, 355.3M) = 613M
  - sqrt(613M / 1M) = sqrt(613) = 24.8
  - 24.8 / 13 = 1.91 → clamped to 1.0
- total_dist = 0.6 + 1.0 = 1.6 → clamped to 1.0
- **similarity = 1.0 - 1.0 = 0.0**

**Result**: Love-Joy (0.45) > Love-Truth (0.0) ✅ Should pass!

### Potential Issues

1. **Anchor Reference Frame**: If Love and Joy both compress against anchor 3, but Truth compresses against anchor 6, the delta values are in different reference frames, making direct comparison problematic.

2. **Position Distance Weight**: The formula weights position distance (pos_dist/5.0) equally with semantic distance (elp_dist.sqrt()/13.0). For very different positions, this may dominate.

3. **Test May Actually Pass**: The math suggests this test should pass. Need to see actual error message.

---

## Testing Status

### Run Tests
```bash
# Test sacred anchor finding (should pass now)
cargo test --lib compression::asi_12byte::tests::test_sacred_anchor_finding

# Test similarity (investigating)
cargo test --lib compression::asi_12byte::tests::test_similarity_in_compressed_space

# All compression tests
cargo test --lib compression::asi_12byte::tests
```

---

## Summary

| Test | Status | Issue | Fix |
|------|--------|-------|-----|
| `test_sacred_anchor_finding` | ✅ Fixed | Incorrect test expectation for position 0 | Changed expected anchor from 3 to 9 |
| `test_similarity_in_compressed_space` | ⏳ Investigating | Unknown (waiting for error message) | TBD |

---

## Next Steps

1. ✅ **test_sacred_anchor_finding**: Fixed
2. ⏳ **test_similarity_in_compressed_space**: 
   - Wait for test output to see actual error
   - May need to adjust similarity calculation
   - May need to fix test expectations
   - May need to add debug output to see intermediate values

---

## Files Modified

- `src/compression/asi_12byte.rs` (line 382)
  - Changed: `assert_eq!(find_nearest_sacred_anchor(0), 3);`
  - To: `assert_eq!(find_nearest_sacred_anchor(0), 9);`
  - Reason: Circular distance calculation is correct, test expectation was wrong

---

*Waiting for test execution to complete...*
