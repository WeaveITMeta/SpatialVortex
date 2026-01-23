# SpatialVortex Test Suite

Comprehensive test coverage for SpatialVortex including unit tests, integration tests, performance tests, and API tests.

---

## 📂 Test Organization

### **unit/** - Unit Tests
Isolated tests for individual components and functions.

**Core Components**:
- **angle_tests.rs** - Angle calculations and transformations
- **flux_matrix_tests.rs** - Flux matrix operations
- **flux_reducer_tests.rs** - Flux reduction algorithms

**Geometry & Mathematics**:
- **alphabet_flux_tests.rs** - Alphabet mapping to flux
- **change_dot_events_tests.rs** - ChangeDot iterator (3-6-9 checkpoints)
- **linear_ladder_raycast_tests.rs** - Ladder indexing and raycasting

**Generators**:
- **grammar_graph_tests.rs** - Grammar graph construction
- **physics_seed_test.rs** - Physics domain seed generation

---

### **integration/** - Integration Tests
Tests for interactions between multiple components and systems.

**Vortex Integration**:
- **agglomerated_vortex_integration.rs** - Complete vortex system integration
- **compression_inference_tests.rs** - Compression + inference pipeline
- **integration_tests.rs** - General integration scenarios

**Inference Engine**:
- **inference_engine_tests.rs** - ML inference engine
- **inference_engine_onnx_tests.rs** - ONNX runtime integration
- **detailed_inference_test.rs** - Detailed inference scenarios

**AI Systems**:
- **ai_router_tests.rs** - AI routing and consensus
- **vector_search_integration.rs** - Vector search operations

---

### **api/** - API Tests
REST API endpoint and integration tests.

- **api_integration_test.rs** - Full API integration testing
- **quick_api_test.rs** - Quick API smoke tests

---

### **performance/** - Performance Tests
Load testing, stress testing, and performance benchmarks.

- **concurrent_stress_test.rs** - Concurrent load and stress testing

---

### **common/** - Shared Test Utilities
Common test fixtures, helpers, and utilities used across test suites.

- **mod.rs** - Shared test utilities and helpers

---

## 🚀 Running Tests

### Run All Tests

```bash
cargo test
```

### Run Tests by Category

**Unit Tests**:
```bash
cargo test --test "unit/*"
# Or specific file
cargo test --test unit/flux_matrix_tests
```

**Integration Tests**:
```bash
cargo test --test "integration/*"
# Or specific file
cargo test --test integration/inference_engine_tests
```

**API Tests**:
```bash
cargo test --test "api/*"
```

**Performance Tests**:
```bash
cargo test --test "performance/*" --release
```

### Run Specific Test

```bash
# Run specific test function
cargo test test_function_name

# Run tests matching pattern
cargo test flux

# Run with output
cargo test -- --nocapture
```

---

## 📊 Test Coverage

### Current Coverage

| Category | Files | Coverage | Status |
|----------|-------|----------|--------|
| **Unit Tests** | 8 | ~70% | ✅ Good |
| **Integration Tests** | 8 | ~60% | ⚠️ Needs improvement |
| **API Tests** | 2 | ~40% | ⚠️ Partial |
| **Performance Tests** | 1 | Basic | ⚠️ Needs expansion |

**Overall Coverage**: ~65%  
**Target Coverage**: 80%+

---

## 🎯 Testing Standards

### Unit Test Requirements

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_behavior() {
        // Arrange - Set up test data
        let input = create_test_input();
        
        // Act - Execute the function
        let result = component_function(input);
        
        // Assert - Verify the result
        assert_eq!(result.expected_field, expected_value);
    }
}
```

### Integration Test Requirements

```rust
#[test]
fn test_system_integration() {
    // Setup multiple components
    let component_a = ComponentA::new();
    let component_b = ComponentB::new();
    
    // Test interaction
    let result = component_a.interact_with(component_b);
    
    // Verify integration
    assert!(result.is_ok());
}
```

### Test Naming Convention

- **Unit tests**: `test_<function>_<scenario>`
- **Integration tests**: `test_<feature>_integration`
- **API tests**: `test_api_<endpoint>_<method>`
- **Performance tests**: `test_<feature>_performance`

---

## 🔧 Test Utilities

### Common Test Helpers

Located in `common/mod.rs`:

```rust
// Example utilities
pub fn create_test_beam() -> BeamTensor { ... }
pub fn create_test_flux_matrix() -> FluxMatrix { ... }
pub fn assert_confidence(beam: &BeamTensor, expected: f32) { ... }
```

### Using Common Utilities

```rust
mod common;

#[test]
fn my_test() {
    let beam = common::create_test_beam();
    // ... test with beam
}
```

---

## 📝 Adding New Tests

### 1. Choose Category

- **Unit**: Testing single function/component?
- **Integration**: Testing component interactions?
- **API**: Testing REST endpoints?
- **Performance**: Testing load/stress?

### 2. Create Test File

```bash
# For unit test
touch tests/unit/my_component_tests.rs

# For integration test
touch tests/integration/my_feature_integration.rs
```

### 3. Write Tests

Follow the standards above and existing test patterns.

### 4. Run & Verify

```bash
cargo test --test <category>/<your_test_file>
```

---

## 🐛 Debugging Tests

### View Test Output

```bash
# Show println! output
cargo test -- --nocapture

# Show all output including passed tests
cargo test -- --nocapture --show-output
```

### Run Single Test

```bash
cargo test specific_test_name -- --exact
```

### Run with Backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

---

## ⚡ Performance Testing

### Running Performance Tests

Always run performance tests in release mode:

```bash
cargo test --test performance/concurrent_stress_test --release
```

### Writing Performance Tests

```rust
use std::time::Instant;

#[test]
fn test_performance_benchmark() {
    let start = Instant::now();
    
    // Run operation
    perform_operation();
    
    let duration = start.elapsed();
    
    // Assert performance requirement
    assert!(duration.as_millis() < 100, "Operation too slow: {:?}", duration);
}
```

---

## 📈 Coverage Reports

### Generate Coverage Report

```bash
# Using tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

### View Coverage

Open `coverage/index.html` in browser.

---

## ✅ CI/CD Integration

### GitHub Actions

Tests run automatically on:
- Pull requests
- Commits to main
- Release tags

### Required Checks

- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Code coverage ≥ 65%
- ✅ No compiler warnings in tests

---

## 🔍 Test Organization Principles

### Why This Structure?

1. **Unit Tests**: Fast, isolated, focused
   - Test individual functions
   - No external dependencies
   - Run in milliseconds

2. **Integration Tests**: Comprehensive, realistic
   - Test component interactions
   - May use external resources
   - Run in seconds

3. **API Tests**: Endpoint validation
   - Test REST API behavior
   - Verify request/response
   - E2E scenarios

4. **Performance Tests**: Load & stress
   - Measure performance
   - Identify bottlenecks
   - Ensure scalability

---

## 📚 Testing Best Practices

### Do's ✅

✅ Test edge cases and error conditions  
✅ Use descriptive test names  
✅ Keep tests independent and isolated  
✅ Use setup/teardown appropriately  
✅ Assert one concept per test  
✅ Use test fixtures for common data  
✅ Document complex test scenarios  

### Don'ts ❌

❌ Test implementation details  
❌ Create interdependent tests  
❌ Use hard-coded values without explanation  
❌ Skip cleanup in tests  
❌ Write overly complex tests  
❌ Ignore test warnings  
❌ Commit failing tests  

---

## 🎓 Learning Resources

### Test Examples

- See existing tests for patterns
- Check `common/mod.rs` for utilities
- Review test documentation in code

### Rust Testing

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)

---

## 🆘 Common Issues

### Test Not Found

**Problem**: `cargo test` doesn't find test

**Solution**: 
- Ensure file is in correct directory
- Check file naming (must end with `.rs`)
- Verify test function has `#[test]` attribute

### Test Timeout

**Problem**: Test hangs or times out

**Solution**:
- Check for infinite loops
- Verify async/await usage
- Add timeout attribute: `#[test(timeout = 5000)]`

### Flaky Tests

**Problem**: Test passes/fails intermittently

**Solution**:
- Remove race conditions
- Avoid time-dependent assertions
- Use proper synchronization

---

## 📊 Test Statistics

| Metric | Count |
|--------|-------|
| **Total Test Files** | 19 |
| **Unit Tests** | 8 files |
| **Integration Tests** | 8 files |
| **API Tests** | 2 files |
| **Performance Tests** | 1 file |
| **Test Coverage** | 65% |

---

## 🎯 Testing Roadmap

### Short Term (Week 1-2)
- ⚠️ Increase unit test coverage to 80%
- ⚠️ Add more API endpoint tests
- ⚠️ Create performance benchmarks

### Medium Term (Month 1)
- Add property-based testing
- Implement fuzzing tests
- Add mutation testing

### Long Term (Quarter 1)
- Achieve 90% coverage
- Complete E2E test suite
- Automated regression testing

---

**Test Quality**: Good  
**Organization**: ✅ Complete  
**Maintainability**: High  
**Coverage Target**: 80%+

**Start testing**: `cargo test` | **View coverage**: `cargo tarpaulin`
