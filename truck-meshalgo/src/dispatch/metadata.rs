//! Hardware metadata detection for CPU and GPU devices.

use serde::{Deserialize, Serialize};

/// Metadata describing the host CPU.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuMetadata {
    /// Processor model name (e.g. "AMD Ryzen 9 5950X 16-Core Processor").
    pub model_name: String,
    /// System architecture (e.g. "x86_64").
    pub architecture: String,
    /// Number of available logical cores / threads.
    pub logical_cores: usize,
}

impl CpuMetadata {
    /// Detects current host CPU metadata.
    pub fn current() -> Self {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let architecture = std::env::consts::ARCH.to_string();

        let model_name = Self::detect_model_name()
            .unwrap_or_else(|| format!("Generic {} ({} threads)", architecture, logical_cores));

        Self {
            model_name,
            architecture,
            logical_cores,
        }
    }

    fn detect_model_name() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("model name") || line.starts_with("Hardware") || line.starts_with("Processor") {
                        if let Some((_, val)) = line.split_once(':') {
                            let name = val.trim().to_string();
                            if !name.is_empty() {
                                return Some(name);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// Metadata describing available GPU accelerators (CUDA and Vulkan).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuMetadata {
    /// CUDA device details, if available.
    #[cfg(feature = "cuda")]
    pub cuda: Option<crate::gpu::cuda::CudaDeviceInfo>,
    /// Placeholder when CUDA feature is disabled.
    #[cfg(not(feature = "cuda"))]
    pub cuda: Option<()>,

    /// Vulkan device details, if available.
    #[cfg(feature = "gpu")]
    pub vulkan: Option<crate::gpu::VulkanDeviceInfo>,
    /// Placeholder when Vulkan/GPU feature is disabled.
    #[cfg(not(feature = "gpu"))]
    pub vulkan: Option<()>,
}

impl GpuMetadata {
    /// Returns true if at least one GPU accelerator (CUDA or Vulkan) is detected and available.
    pub fn has_gpu(&self) -> bool {
        self.cuda.is_some() || self.vulkan.is_some()
    }

    /// Detects current GPU devices on the system.
    pub fn current() -> Self {
        #[cfg(feature = "cuda")]
        let cuda = crate::gpu::cuda::CudaMeshAnalyzer::get_device_info();
        #[cfg(not(feature = "cuda"))]
        let cuda = None;

        #[cfg(feature = "gpu")]
        let vulkan = crate::gpu::GpuMeshAnalyzer::try_create_default()
            .map(|analyzer| analyzer.device_info());
        #[cfg(not(feature = "gpu"))]
        let vulkan = None;

        Self { cuda, vulkan }
    }
}

/// Combined system hardware fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemMetadata {
    /// Host CPU metadata.
    pub cpu: CpuMetadata,
    /// GPU accelerators metadata.
    pub gpu: GpuMetadata,
}

static CURRENT_SYSTEM_METADATA: std::sync::OnceLock<SystemMetadata> = std::sync::OnceLock::new();

impl SystemMetadata {
    /// Captures the current system hardware environment, memoized for process lifetime.
    pub fn current() -> Self {
        CURRENT_SYSTEM_METADATA
            .get_or_init(|| Self {
                cpu: CpuMetadata::current(),
                gpu: GpuMetadata::current(),
            })
            .clone()
    }

    /// Returns true if at least one GPU accelerator is available on the system.
    pub fn has_gpu(&self) -> bool {
        self.gpu.has_gpu()
    }

    /// Checks if this metadata matches the target metadata.
    pub fn matches(&self, other: &Self) -> bool {
        self.cpu == other.cpu && self.gpu == other.gpu
    }
}
