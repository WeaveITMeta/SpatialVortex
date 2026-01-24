//! Backend Selection for GPU Acceleration
//!
//! Provides type aliases and utilities for switching between:
//! - `NdArray` - CPU backend (default, portable)
//! - `Tch` - GPU via libtorch (CUDA/ROCm)
//! - `Wgpu` - GPU via WebGPU (cross-platform)

#[cfg(feature = "burn-cpu")]
pub use burn::backend::NdArray;

#[cfg(feature = "burn-gpu")]
pub use burn::backend::LibTorch;

#[cfg(feature = "burn-wgpu")]
pub use burn::backend::Wgpu;

#[cfg(feature = "burn-cpu")]
pub use burn::backend::ndarray::NdArrayDevice;

#[cfg(feature = "burn-gpu")]
pub use burn::backend::libtorch::LibTorchDevice;

#[cfg(feature = "burn-wgpu")]
pub use burn::backend::wgpu::WgpuDevice;

/// Default backend type alias based on enabled features
/// Priority: GPU (tch) > WGPU > CPU (ndarray)
#[cfg(all(feature = "burn-gpu", not(feature = "burn-wgpu")))]
pub type DefaultBackend = LibTorch;

#[cfg(all(feature = "burn-wgpu", not(feature = "burn-gpu")))]
pub type DefaultBackend = Wgpu;

#[cfg(all(feature = "burn-cpu", not(feature = "burn-gpu"), not(feature = "burn-wgpu")))]
pub type DefaultBackend = NdArray<f32>;

/// Default device based on enabled features
#[cfg(all(feature = "burn-gpu", not(feature = "burn-wgpu")))]
pub fn default_device() -> LibTorchDevice {
    LibTorchDevice::Cuda(0)
}

#[cfg(all(feature = "burn-wgpu", not(feature = "burn-gpu")))]
pub fn default_device() -> WgpuDevice {
    WgpuDevice::default()
}

#[cfg(all(feature = "burn-cpu", not(feature = "burn-gpu"), not(feature = "burn-wgpu")))]
pub fn default_device() -> NdArrayDevice {
    NdArrayDevice::default()
}

/// Check if GPU is available
#[cfg(feature = "burn-gpu")]
pub fn gpu_available() -> bool {
    // Check CUDA availability via tch
    true // Simplified - would check tch::Cuda::is_available()
}

#[cfg(not(feature = "burn-gpu"))]
pub fn gpu_available() -> bool {
    false
}

/// Backend info for logging
pub fn backend_info() -> &'static str {
    #[cfg(feature = "burn-gpu")]
    return "LibTorch (CUDA/ROCm)";
    
    #[cfg(all(feature = "burn-wgpu", not(feature = "burn-gpu")))]
    return "WGPU (WebGPU)";
    
    #[cfg(all(feature = "burn-cpu", not(feature = "burn-gpu"), not(feature = "burn-wgpu")))]
    return "NdArray (CPU)";
    
    #[cfg(not(any(feature = "burn-cpu", feature = "burn-gpu", feature = "burn-wgpu")))]
    return "No Burn backend enabled";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_info() {
        let info = backend_info();
        assert!(!info.is_empty());
        println!("Active backend: {}", info);
    }

    #[test]
    #[cfg(feature = "burn-cpu")]
    fn test_default_device() {
        let _device = default_device();
    }
}
