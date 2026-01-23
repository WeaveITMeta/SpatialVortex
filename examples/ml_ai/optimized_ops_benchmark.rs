//! Optimized Operations Benchmark
//!
//! Compares performance of manual loops vs SIMD/vectorized operations.
//! Demonstrates 10-100x speedups from proper optimization.
//!
//! ## Run
//!
//! ```bash
//! cargo run --example optimized_ops_benchmark --release
//! ```

use spatial_vortex::ml::inference::{
    matmul,
    softmax,
    softmax_2d,
    layer_norm,
    gelu,
    dot_product,
    scaled_dot_product_attention,
    has_avx2,
    has_avx512,
    QuantizedTensor,
};
use ndarray::{Array1, Array2};
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Optimized Operations Benchmark                         ║");
    println!("║              Manual vs SIMD/Vectorized                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // System info
    println!("🖥️  SYSTEM CAPABILITIES\n");
    println!("   AVX2 available: {}", has_avx2());
    println!("   AVX-512 available: {}", has_avx512());
    println!();

    // ========== 1. Matrix Multiplication ==========
    println!("📊 1. MATRIX MULTIPLICATION\n");
    
    let sizes = [(64, 64), (128, 128), (256, 256), (512, 512)];
    
    for (m, n) in sizes {
        let a = Array2::from_shape_fn((m, n), |(i, j)| (i * j) as f32 / 1000.0);
        let b = Array2::from_shape_fn((n, m), |(i, j)| (i + j) as f32 / 1000.0);
        
        // Warmup
        let _ = matmul(&a, &b);
        
        // Benchmark optimized (ndarray.dot - uses BLAS if available)
        let start = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let _ = matmul(&a, &b);
        }
        let optimized_time = start.elapsed() / iterations;
        
        // Benchmark manual loop
        let start = Instant::now();
        for _ in 0..iterations.min(10) {
            let _ = manual_matmul(&a, &b);
        }
        let manual_time = start.elapsed() / iterations.min(10);
        
        let speedup = manual_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        
        println!("   {}x{}: Optimized={:?}, Manual={:?}, Speedup={:.1}x",
            m, n, optimized_time, manual_time, speedup);
    }
    println!();

    // ========== 2. Softmax ==========
    println!("📊 2. SOFTMAX\n");
    
    let sizes = [1000, 10000, 32000, 100000];
    
    for size in sizes {
        let logits = Array1::from_shape_fn(size, |i| (i as f32 / 100.0).sin());
        
        // Warmup
        let _ = softmax(&logits);
        
        // Benchmark optimized
        let start = Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let _ = softmax(&logits);
        }
        let optimized_time = start.elapsed() / iterations;
        
        // Benchmark manual
        let start = Instant::now();
        for _ in 0..iterations.min(100) {
            let _ = manual_softmax(&logits);
        }
        let manual_time = start.elapsed() / iterations.min(100);
        
        let speedup = manual_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        
        println!("   size={}: Optimized={:?}, Manual={:?}, Speedup={:.1}x",
            size, optimized_time, manual_time, speedup);
    }
    println!();

    // ========== 3. Layer Normalization ==========
    println!("📊 3. LAYER NORMALIZATION\n");
    
    let configs = [(32, 256), (64, 512), (128, 768), (256, 1024)];
    
    for (seq_len, d_model) in configs {
        let x = Array2::from_shape_fn((seq_len, d_model), |(i, j)| ((i * j) as f32).sin());
        
        // Warmup
        let _ = layer_norm(&x, 1e-5);
        
        // Benchmark optimized
        let start = Instant::now();
        let iterations = 500;
        for _ in 0..iterations {
            let _ = layer_norm(&x, 1e-5);
        }
        let optimized_time = start.elapsed() / iterations;
        
        // Benchmark manual
        let start = Instant::now();
        for _ in 0..iterations.min(50) {
            let _ = manual_layer_norm(&x, 1e-5);
        }
        let manual_time = start.elapsed() / iterations.min(50);
        
        let speedup = manual_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        
        println!("   {}x{}: Optimized={:?}, Manual={:?}, Speedup={:.1}x",
            seq_len, d_model, optimized_time, manual_time, speedup);
    }
    println!();

    // ========== 4. Dot Product ==========
    println!("📊 4. DOT PRODUCT (SIMD vs Scalar)\n");
    
    let sizes = [64, 256, 1024, 4096, 16384];
    
    for size in sizes {
        let a: Vec<f32> = (0..size).map(|i| (i as f32 / 100.0).sin()).collect();
        let b: Vec<f32> = (0..size).map(|i| (i as f32 / 100.0).cos()).collect();
        
        // Warmup
        let _ = dot_product(&a, &b);
        
        // Benchmark optimized (SIMD when available)
        let start = Instant::now();
        let iterations = 10000;
        for _ in 0..iterations {
            let _ = dot_product(&a, &b);
        }
        let optimized_time = start.elapsed() / iterations;
        
        // Benchmark manual scalar
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = manual_dot_product(&a, &b);
        }
        let manual_time = start.elapsed() / iterations;
        
        let speedup = manual_time.as_nanos() as f64 / optimized_time.as_nanos().max(1) as f64;
        
        println!("   size={}: Optimized={:?}, Manual={:?}, Speedup={:.1}x",
            size, optimized_time, manual_time, speedup);
    }
    println!();

    // ========== 5. GELU Activation ==========
    println!("📊 5. GELU ACTIVATION\n");
    
    let configs = [(64, 256), (128, 512), (256, 1024)];
    
    for (seq_len, d_model) in configs {
        let x = Array2::from_shape_fn((seq_len, d_model), |(i, j)| ((i * j) as f32 / 1000.0) - 0.5);
        
        // Warmup
        let _ = gelu(&x);
        
        // Benchmark optimized
        let start = Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let _ = gelu(&x);
        }
        let optimized_time = start.elapsed() / iterations;
        
        // Benchmark manual
        let start = Instant::now();
        for _ in 0..iterations.min(100) {
            let _ = manual_gelu(&x);
        }
        let manual_time = start.elapsed() / iterations.min(100);
        
        let speedup = manual_time.as_nanos() as f64 / optimized_time.as_nanos() as f64;
        
        println!("   {}x{}: Optimized={:?}, Manual={:?}, Speedup={:.1}x",
            seq_len, d_model, optimized_time, manual_time, speedup);
    }
    println!();

    // ========== 6. INT8 Quantization ==========
    println!("📊 6. INT8 QUANTIZATION\n");
    
    let sizes = [1000, 10000, 100000, 1000000];
    
    for size in sizes {
        let data: Vec<f32> = (0..size).map(|i| (i as f32 / 1000.0).sin()).collect();
        
        // Benchmark quantization
        let start = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let _ = QuantizedTensor::from_f32(&data);
        }
        let quant_time = start.elapsed() / iterations;
        
        // Benchmark INT8 dot product
        let q1 = QuantizedTensor::from_f32(&data);
        let q2 = QuantizedTensor::from_f32(&data);
        
        let start = Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let _ = q1.dot(&q2);
        }
        let int8_dot_time = start.elapsed() / iterations;
        
        // Compare to FP32 dot product
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = dot_product(&data, &data);
        }
        let fp32_dot_time = start.elapsed() / iterations;
        
        let speedup = fp32_dot_time.as_nanos() as f64 / int8_dot_time.as_nanos().max(1) as f64;
        
        println!("   size={}: Quant={:?}, INT8 dot={:?}, FP32 dot={:?}, Speedup={:.1}x",
            size, quant_time, int8_dot_time, fp32_dot_time, speedup);
    }
    println!();

    // ========== 7. Attention ==========
    println!("📊 7. SCALED DOT-PRODUCT ATTENTION\n");
    
    let configs = [(8, 64), (16, 128), (32, 256), (64, 512)];
    
    for (seq_len, d_model) in configs {
        let q = Array2::from_shape_fn((seq_len, d_model), |(i, j)| ((i * j) as f32).sin());
        let k = Array2::from_shape_fn((seq_len, d_model), |(i, j)| ((i + j) as f32).cos());
        let v = Array2::from_shape_fn((seq_len, d_model), |(i, j)| (i as f32 / j.max(1) as f32));
        let scale = 1.0 / (d_model as f32).sqrt();
        
        // Warmup
        let _ = scaled_dot_product_attention(&q, &k, &v, scale);
        
        // Benchmark
        let start = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let _ = scaled_dot_product_attention(&q, &k, &v, scale);
        }
        let time = start.elapsed() / iterations;
        
        let flops = (seq_len * seq_len * d_model * 2 + seq_len * seq_len * d_model) as f64;
        let gflops = flops / time.as_secs_f64() / 1e9;
        
        println!("   {}x{}: Time={:?}, GFLOPS={:.2}",
            seq_len, d_model, time, gflops);
    }
    println!();

    // ========== Summary ==========
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SUMMARY                                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ✅ ndarray.dot(): Uses BLAS when available (10-100x faster)  ║");
    println!("║ ✅ Broadcast ops: LLVM auto-vectorizes (5-20x faster)        ║");
    println!("║ ✅ SIMD intrinsics: AVX2/AVX-512 for custom ops (2-8x)       ║");
    println!("║ ✅ INT8 quantization: 4x memory, 2-4x compute speedup        ║");
    println!("║ ✅ Fused operations: Reduce memory bandwidth                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("💡 TIP: Enable BLAS for maximum performance:");
    println!("   cargo add ndarray --features blas");
    println!("   cargo add blas-src --features openblas");
}

// ========== Manual (slow) implementations for comparison ==========

fn manual_matmul(a: &Array2<f32>, b: &Array2<f32>) -> Array2<f32> {
    let (m, k) = (a.nrows(), a.ncols());
    let n = b.ncols();
    
    let mut c = Array2::zeros((m, n));
    
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for l in 0..k {
                sum += a[[i, l]] * b[[l, j]];
            }
            c[[i, j]] = sum;
        }
    }
    
    c
}

fn manual_softmax(logits: &Array1<f32>) -> Array1<f32> {
    let mut max = f32::NEG_INFINITY;
    for &v in logits.iter() {
        if v > max { max = v; }
    }
    
    let mut exp_sum = 0.0;
    let mut exp_vals = Vec::with_capacity(logits.len());
    for &v in logits.iter() {
        let e = (v - max).exp();
        exp_vals.push(e);
        exp_sum += e;
    }
    
    let mut result = Array1::zeros(logits.len());
    for (i, e) in exp_vals.iter().enumerate() {
        result[i] = e / exp_sum;
    }
    
    result
}

fn manual_layer_norm(x: &Array2<f32>, eps: f32) -> Array2<f32> {
    let (rows, cols) = (x.nrows(), x.ncols());
    let mut result = Array2::zeros((rows, cols));
    
    for i in 0..rows {
        // Calculate mean
        let mut sum = 0.0;
        for j in 0..cols {
            sum += x[[i, j]];
        }
        let mean = sum / cols as f32;
        
        // Calculate variance
        let mut var_sum = 0.0;
        for j in 0..cols {
            let diff = x[[i, j]] - mean;
            var_sum += diff * diff;
        }
        let std = (var_sum / cols as f32 + eps).sqrt();
        
        // Normalize
        for j in 0..cols {
            result[[i, j]] = (x[[i, j]] - mean) / std;
        }
    }
    
    result
}

fn manual_dot_product(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

fn manual_gelu(x: &Array2<f32>) -> Array2<f32> {
    let (rows, cols) = (x.nrows(), x.ncols());
    let mut result = Array2::zeros((rows, cols));
    
    const SQRT_2_PI: f32 = 0.7978845608;
    const COEFF: f32 = 0.044715;
    
    for i in 0..rows {
        for j in 0..cols {
            let v = x[[i, j]];
            let inner = SQRT_2_PI * (v + COEFF * v * v * v);
            result[[i, j]] = 0.5 * v * (1.0 + inner.tanh());
        }
    }
    
    result
}
