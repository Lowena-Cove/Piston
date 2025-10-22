//! System capability detection for Android devices
//! 
//! This module detects hardware and software capabilities of the Android device
//! to determine the best emulation strategy for SteamOS.

use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;
use log::{info, warn, debug};

/// System capabilities detected on the Android device
#[derive(Debug, Clone)]
pub struct SystemCapabilities {
    /// CPU architecture (arm64, x86_64, etc.)
    pub architecture: String,
    
    /// Number of CPU cores
    pub cpu_cores: u32,
    
    /// Available system memory in MB
    pub available_memory_mb: u64,
    
    /// Whether hardware virtualization is available
    pub has_virtualization: bool,
    
    /// Whether GPU acceleration is available
    pub has_gpu_acceleration: bool,
    
    /// OpenGL ES version string
    pub opengl_version: Option<String>,
    
    /// Whether Vulkan is supported
    pub has_vulkan: bool,
    
    /// Whether x86_64 emulation is feasible
    pub can_emulate_x86_64: bool,
    
    /// Whether hardware acceleration can be used
    pub has_hardware_acceleration: bool,
    
    /// Whether container/namespace support is available
    pub has_container_support: bool,
    
    /// Recommended memory allocation in MB
    pub recommended_memory_mb: u64,
    
    /// Recommended number of CPU cores to use
    pub recommended_cpu_cores: u32,
    
    /// Recommended graphics mode
    pub recommended_graphics_mode: String,
}

impl SystemCapabilities {
    /// Detect system capabilities
    pub fn detect() -> Result<Self> {
        info!("Detecting system capabilities...");
        
        let architecture = Self::detect_architecture()?;
        let cpu_cores = Self::detect_cpu_cores()?;
        let available_memory_mb = Self::detect_memory()?;
        let has_virtualization = Self::detect_virtualization();
        let (has_gpu_acceleration, opengl_version, has_vulkan) = Self::detect_graphics();
        let can_emulate_x86_64 = Self::can_emulate_x86_64(&architecture);
        let has_hardware_acceleration = Self::detect_hardware_acceleration();
        let has_container_support = Self::detect_container_support();
        
        // Calculate recommendations
        let recommended_memory_mb = Self::calculate_recommended_memory(available_memory_mb);
        let recommended_cpu_cores = Self::calculate_recommended_cores(cpu_cores);
        let recommended_graphics_mode = Self::determine_graphics_mode(
            has_gpu_acceleration, 
            has_vulkan, 
            &opengl_version
        );
        
        Ok(SystemCapabilities {
            architecture,
            cpu_cores,
            available_memory_mb,
            has_virtualization,
            has_gpu_acceleration,
            opengl_version,
            has_vulkan,
            can_emulate_x86_64,
            has_hardware_acceleration,
            has_container_support,
            recommended_memory_mb,
            recommended_cpu_cores,
            recommended_graphics_mode,
        })
    }
    
    /// Check if the system is compatible with SteamOS emulation
    pub fn is_compatible(&self) -> bool {
        // Minimum requirements:
        // - ARM64 or x86_64 architecture
        // - At least 2GB RAM
        // - At least 2 CPU cores
        // - Some form of graphics acceleration
        
        let arch_compatible = matches!(
            self.architecture.as_str(), 
            "arm64" | "aarch64" | "x86_64" | "x64"
        );
        
        let memory_sufficient = self.available_memory_mb >= 2048;
        let cores_sufficient = self.cpu_cores >= 2;
        
        arch_compatible && memory_sufficient && cores_sufficient
    }
    
    /// Detect CPU architecture
    fn detect_architecture() -> Result<String> {
        // Try multiple methods to detect architecture
        
        // Method 1: Check /proc/cpuinfo
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            if cpuinfo.contains("AArch64") || cpuinfo.contains("arm64") {
                return Ok("arm64".to_string());
            }
            if cpuinfo.contains("x86_64") {
                return Ok("x86_64".to_string());
            }
        }
        
        // Method 2: Use uname command
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            let arch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !arch.is_empty() {
                return Ok(arch);
            }
        }
        
        // Method 3: Check system properties (Android specific)
        if let Ok(output) = Command::new("getprop").arg("ro.product.cpu.abi").output() {
            let abi = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match abi.as_str() {
                "arm64-v8a" => return Ok("arm64".to_string()),
                "x86_64" => return Ok("x86_64".to_string()),
                "armeabi-v7a" => return Ok("arm".to_string()),
                "x86" => return Ok("x86".to_string()),
                _ => {}
            }
        }
        
        warn!("Could not detect architecture, assuming arm64");
        Ok("arm64".to_string())
    }
    
    /// Detect number of CPU cores
    fn detect_cpu_cores() -> Result<u32> {
        // Method 1: Count online CPUs
        if let Ok(online) = fs::read_to_string("/sys/devices/system/cpu/online") {
            if let Some(count) = Self::parse_cpu_range(&online.trim()) {
                return Ok(count);
            }
        }
        
        // Method 2: Count CPU directories
        if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
            let count = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_name()
                        .to_string_lossy()
                        .starts_with("cpu")
                        && entry.file_name()
                            .to_string_lossy()
                            .chars()
                            .skip(3)
                            .all(|c| c.is_ascii_digit())
                })
                .count() as u32;
            
            if count > 0 {
                return Ok(count);
            }
        }
        
        // Method 3: Use nproc command
        if let Ok(output) = Command::new("nproc").output() {
            if let Ok(count_str) = String::from_utf8(output.stdout) {
                if let Ok(count) = count_str.trim().parse::<u32>() {
                    return Ok(count);
                }
            }
        }
        
        warn!("Could not detect CPU cores, assuming 4");
        Ok(4)
    }
    
    /// Parse CPU range string (e.g., "0-7" or "0,2-5,7")
    fn parse_cpu_range(range_str: &str) -> Option<u32> {
        let mut count = 0;
        
        for part in range_str.split(',') {
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
                    count += e - s + 1;
                }
            } else if part.parse::<u32>().is_ok() {
                count += 1;
            }
        }
        
        if count > 0 { Some(count) } else { None }
    }
    
    /// Detect available memory
    fn detect_memory() -> Result<u64> {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Ok(kb / 1024); // Convert KB to MB
                        }
                    }
                }
            }
        }
        
        warn!("Could not detect memory, assuming 4GB");
        Ok(4096)
    }
    
    /// Detect virtualization support
    fn detect_virtualization() -> bool {
        // Check for KVM support
        if fs::metadata("/dev/kvm").is_ok() {
            return true;
        }
        
        // Check CPU features for virtualization
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            // ARM virtualization extensions
            if cpuinfo.contains("vfp") && cpuinfo.contains("neon") {
                return true;
            }
            
            // x86 virtualization
            if cpuinfo.contains("vmx") || cpuinfo.contains("svm") {
                return true;
            }
        }
        
        false
    }
    
    /// Detect graphics capabilities
    fn detect_graphics() -> (bool, Option<String>, bool) {
        let mut has_gpu = false;
        let mut opengl_version = None;
        let mut has_vulkan = false;
        
        // Check for GPU devices
        if let Ok(entries) = fs::read_dir("/dev/dri") {
            has_gpu = entries.count() > 0;
        }
        
        // Try to get OpenGL version (this would typically require EGL context)
        // For now, assume OpenGL ES 3.0+ on modern Android devices
        opengl_version = Some("OpenGL ES 3.0+".to_string());
        
        // Check for Vulkan support
        // This is a simplified check - real implementation would query Vulkan loader
        has_vulkan = std::path::Path::new("/vendor/lib64/hw/vulkan.adreno.so").exists() ||
                    std::path::Path::new("/vendor/lib64/hw/vulkan.mali.so").exists() ||
                    std::path::Path::new("/system/lib64/hw/vulkan.adreno.so").exists();
        
        (has_gpu, opengl_version, has_vulkan)
    }
    
    /// Check if x86_64 emulation is feasible
    fn can_emulate_x86_64(architecture: &str) -> bool {
        match architecture {
            "x86_64" | "x64" => true, // Native x86_64
            "arm64" | "aarch64" => true, // ARM64 can emulate x86_64 with reasonable performance
            _ => false, // 32-bit architectures not suitable
        }
    }
    
    /// Detect hardware acceleration support
    fn detect_hardware_acceleration() -> bool {
        // Check for various acceleration methods
        
        // KVM acceleration
        if fs::metadata("/dev/kvm").is_ok() {
            return true;
        }
        
        // Check for ARM hardware features
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            if cpuinfo.contains("neon") || cpuinfo.contains("asimd") {
                return true;
            }
        }
        
        false
    }
    
    /// Detect container/namespace support
    fn detect_container_support() -> bool {
        // Check if we can create user namespaces
        std::path::Path::new("/proc/sys/user/max_user_namespaces").exists()
    }
    
    /// Calculate recommended memory allocation
    fn calculate_recommended_memory(available_mb: u64) -> u64 {
        // Use 50-75% of available memory, with reasonable bounds
        let recommended = (available_mb * 3) / 4;
        
        // Clamp between 2GB and 8GB
        recommended.max(2048).min(8192)
    }
    
    /// Calculate recommended CPU cores
    fn calculate_recommended_cores(available_cores: u32) -> u32 {
        // Use 75% of available cores, minimum 2, maximum 8
        let recommended = (available_cores * 3) / 4;
        recommended.max(2).min(8)
    }
    
    /// Determine best graphics mode
    fn determine_graphics_mode(
        has_gpu: bool, 
        has_vulkan: bool, 
        opengl_version: &Option<String>
    ) -> String {
        if has_vulkan {
            "Vulkan".to_string()
        } else if has_gpu && opengl_version.is_some() {
            "OpenGL ES".to_string()
        } else {
            "Software".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_range_parsing() {
        assert_eq!(SystemCapabilities::parse_cpu_range("0-3"), Some(4));
        assert_eq!(SystemCapabilities::parse_cpu_range("0,2-5,7"), Some(6));
        assert_eq!(SystemCapabilities::parse_cpu_range("0"), Some(1));
        assert_eq!(SystemCapabilities::parse_cpu_range("invalid"), None);
    }
    
    #[test]
    fn test_memory_calculation() {
        assert_eq!(SystemCapabilities::calculate_recommended_memory(8192), 6144);
        assert_eq!(SystemCapabilities::calculate_recommended_memory(1024), 2048); // Minimum
        assert_eq!(SystemCapabilities::calculate_recommended_memory(16384), 8192); // Maximum
    }
    
    #[test]
    fn test_core_calculation() {
        assert_eq!(SystemCapabilities::calculate_recommended_cores(8), 6);
        assert_eq!(SystemCapabilities::calculate_recommended_cores(2), 2); // Minimum
        assert_eq!(SystemCapabilities::calculate_recommended_cores(16), 8); // Maximum
    }
}