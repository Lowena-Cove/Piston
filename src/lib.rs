//! SteamOS Emulator - A Rust-based Linux emulation framework
//! 
//! This library provides the core functionality for emulating Linux environments,
//! specifically optimized for running SteamOS and Steam games.

pub mod cpu;
pub mod memory;
pub mod syscalls;
pub mod filesystem;
pub mod process;
pub mod devices;
pub mod config;
pub mod error;

use std::sync::Arc;
use tokio::sync::RwLock;

pub use error::{EmulatorError, Result};
pub use config::EmulatorConfig;

/// Main emulator instance that coordinates all subsystems
pub struct SteamOSEmulator {
    config: EmulatorConfig,
    cpu: Arc<RwLock<cpu::CpuState>>,
    memory: Arc<RwLock<memory::MemoryManager>>,
    filesystem: Arc<RwLock<filesystem::VirtualFileSystem>>,
    process_manager: Arc<RwLock<process::ProcessManager>>,
}

impl SteamOSEmulator {
    /// Create a new emulator instance with the given configuration
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        let cpu = Arc::new(RwLock::new(cpu::CpuState::new()?));
        let memory = Arc::new(RwLock::new(memory::MemoryManager::new(config.memory_size)?));
        let filesystem = Arc::new(RwLock::new(filesystem::VirtualFileSystem::new()?));
        let process_manager = Arc::new(RwLock::new(process::ProcessManager::new()));

        Ok(Self {
            config,
            cpu,
            memory,
            filesystem,
            process_manager,
        })
    }

    /// Load a SteamOS disk image and prepare it for execution
    pub async fn load_steamos_image(&mut self, image_path: &str) -> Result<()> {
        log::info!("Loading SteamOS image: {}", image_path);
        
        // Mount the disk image in the virtual filesystem
        let mut fs = self.filesystem.write().await;
        fs.mount_disk_image(image_path, "/").await?;
        
        log::info!("SteamOS image loaded successfully");
        Ok(())
    }

    /// Start the emulated Linux environment
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting SteamOS emulator");
        
        // Initialize the kernel environment
        self.init_kernel_environment().await?;
        
        // Start the init process
        self.start_init_process().await?;
        
        log::info!("SteamOS emulator started successfully");
        Ok(())
    }

    /// Initialize the basic kernel environment
    async fn init_kernel_environment(&mut self) -> Result<()> {
        // Set up basic memory layout
        let mut memory = self.memory.write().await;
        memory.setup_kernel_space()?;
        
        // Initialize basic devices
        let mut fs = self.filesystem.write().await;
        fs.create_device_nodes().await?;
        
        Ok(())
    }

    /// Start the init process (typically /sbin/init or systemd)
    async fn start_init_process(&mut self) -> Result<()> {
        let mut proc_mgr = self.process_manager.write().await;
        
        // Look for init binary in common locations
        let init_paths = ["/sbin/init", "/usr/lib/systemd/systemd", "/bin/init"];
        
        for init_path in &init_paths {
            if self.filesystem.read().await.file_exists(init_path).await? {
                log::info!("Starting init process: {}", init_path);
                proc_mgr.create_process(init_path, &[], &[]).await?;
                return Ok(());
            }
        }
        
        Err(EmulatorError::InitNotFound)
    }

    /// Execute a single instruction cycle
    pub async fn step(&mut self) -> Result<bool> {
        let mut cpu = self.cpu.write().await;
        let memory = self.memory.read().await;
        
        cpu.execute_instruction(&*memory).await
    }

    /// Run the emulator until shutdown
    pub async fn run(&mut self) -> Result<()> {
        loop {
            if !self.step().await? {
                break;
            }
            
            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }
        
        log::info!("Emulator shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_emulator_creation() {
        let config = EmulatorConfig::default();
        let emulator = SteamOSEmulator::new(config);
        assert!(emulator.is_ok());
    }
}