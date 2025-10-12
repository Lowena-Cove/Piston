//! Configuration management for the SteamOS emulator

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::{Result, EmulatorError};

/// Main configuration structure for the emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    /// Memory size in bytes
    pub memory_size: u64,
    
    /// Number of CPU cores to emulate
    pub cpu_cores: u32,
    
    /// CPU configuration
    pub cpu: CpuConfig,
    
    /// Memory configuration
    pub memory: MemoryConfig,
    
    /// File system configuration
    pub filesystem: FileSystemConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Device configuration
    pub devices: DeviceConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    /// CPU architecture (currently only x86_64 supported)
    pub architecture: String,
    
    /// Enable CPU debugging features
    pub debug: bool,
    
    /// CPU frequency in MHz (for timing simulation)
    pub frequency_mhz: u32,
    
    /// Enable hardware virtualization features if available
    pub enable_virtualization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable memory debugging
    pub debug: bool,
    
    /// Page size in bytes
    pub page_size: u64,
    
    /// Enable memory protection
    pub enable_protection: bool,
    
    /// Enable memory mapping
    pub enable_mmap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemConfig {
    /// Root directory for the emulated file system
    pub root_path: String,
    
    /// Enable file system debugging
    pub debug: bool,
    
    /// Enable case-sensitive file names
    pub case_sensitive: bool,
    
    /// Maximum number of open files
    pub max_open_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable network emulation
    pub enabled: bool,
    
    /// Network interface name
    pub interface: String,
    
    /// Enable network debugging
    pub debug: bool,
    
    /// Network mode (NAT, bridge, etc.)
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Enable graphics device emulation
    pub graphics: bool,
    
    /// Enable audio device emulation
    pub audio: bool,
    
    /// Enable input device emulation
    pub input: bool,
    
    /// Enable storage device emulation
    pub storage: bool,
    
    /// Enable device debugging
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    
    /// Log to file
    pub file: Option<String>,
    
    /// Enable colored output
    pub colored: bool,
    
    /// Enable timestamps
    pub timestamps: bool,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            memory_size: 4 * 1024 * 1024 * 1024, // 4GB
            cpu_cores: 4,
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            filesystem: FileSystemConfig::default(),
            network: NetworkConfig::default(),
            devices: DeviceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            architecture: "x86_64".to_string(),
            debug: false,
            frequency_mhz: 3000,
            enable_virtualization: true,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            debug: false,
            page_size: 4096,
            enable_protection: true,
            enable_mmap: true,
        }
    }
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            root_path: "/".to_string(),
            debug: false,
            case_sensitive: true,
            max_open_files: 1024,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: "eth0".to_string(),
            debug: false,
            mode: "NAT".to_string(),
        }
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            graphics: true,
            audio: true,
            input: true,
            storage: true,
            debug: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            colored: true,
            timestamps: true,
        }
    }
}

impl EmulatorConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EmulatorError::config(format!("Failed to read config file: {}", e)))?;
        
        let config: EmulatorConfig = toml::from_str(&content)
            .map_err(|e| EmulatorError::config(format!("Failed to parse config file: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| EmulatorError::config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| EmulatorError::config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.memory_size < 64 * 1024 * 1024 {
            return Err(EmulatorError::config("Memory size must be at least 64MB"));
        }
        
        if self.cpu_cores == 0 {
            return Err(EmulatorError::config("CPU cores must be at least 1"));
        }
        
        if self.cpu.architecture != "x86_64" {
            return Err(EmulatorError::config("Only x86_64 architecture is currently supported"));
        }
        
        if self.memory.page_size == 0 || (self.memory.page_size & (self.memory.page_size - 1)) != 0 {
            return Err(EmulatorError::config("Page size must be a power of 2"));
        }
        
        Ok(())
    }
    
    /// Create a configuration optimized for SteamOS
    pub fn steamos_optimized() -> Self {
        let mut config = Self::default();
        
        // SteamOS typically needs more memory
        config.memory_size = 8 * 1024 * 1024 * 1024; // 8GB
        
        // Enable all devices for gaming
        config.devices.graphics = true;
        config.devices.audio = true;
        config.devices.input = true;
        
        // Enable network for Steam connectivity
        config.network.enabled = true;
        
        // Optimize for performance
        config.cpu.enable_virtualization = true;
        config.memory.enable_mmap = true;
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = EmulatorConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.cpu.architecture, "x86_64");
        assert!(config.memory_size >= 64 * 1024 * 1024);
    }
    
    #[test]
    fn test_steamos_optimized_config() {
        let config = EmulatorConfig::steamos_optimized();
        assert!(config.validate().is_ok());
        assert_eq!(config.memory_size, 8 * 1024 * 1024 * 1024);
        assert!(config.devices.graphics);
        assert!(config.network.enabled);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = EmulatorConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: EmulatorConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.memory_size, deserialized.memory_size);
    }
    
    #[test]
    fn test_config_file_operations() {
        let config = EmulatorConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Save config
        config.to_file(temp_file.path()).unwrap();
        
        // Load config
        let loaded_config = EmulatorConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.memory_size, loaded_config.memory_size);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = EmulatorConfig::default();
        
        // Test invalid memory size
        config.memory_size = 1024; // Too small
        assert!(config.validate().is_err());
        
        // Test invalid CPU cores
        config.memory_size = 1024 * 1024 * 1024; // Reset to valid
        config.cpu_cores = 0;
        assert!(config.validate().is_err());
        
        // Test invalid architecture
        config.cpu_cores = 1; // Reset to valid
        config.cpu.architecture = "arm64".to_string();
        assert!(config.validate().is_err());
    }
}